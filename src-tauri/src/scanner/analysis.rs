use std::{fs, path::Path, path::PathBuf};

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        error::{CommandError, ErrorCode},
        finding::{Finding, FindingEvidence, FindingType},
        rule::{RecommendedAction, RiskLevel, RuleCategory},
        scan::CustomScanOptions,
    },
    duplicates::scanner::validate_roots_with_limit,
};

use super::{cancellation::CancellationToken, exclusions::ExclusionMatcher};

const MAX_PENDING_PATHS: usize = 250_000;

#[derive(Debug, Clone, Default)]
pub struct AnalysisProgress {
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub bytes_examined: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub errors: Vec<CommandError>,
}

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub roots: Vec<PathBuf>,
    pub enabled_categories: Vec<RuleCategory>,
    pub minimum_file_size_bytes: u64,
    pub maximum_depth: u16,
    pub include_hidden_files: bool,
    pub include_external_drives: bool,
    pub large_file_threshold_bytes: u64,
    pub very_large_file_threshold_bytes: u64,
    pub huge_file_threshold_bytes: u64,
    pub old_file_threshold_days: u32,
}

pub fn validate_custom_options(
    options: &CustomScanOptions,
    home: &Path,
    platform: &str,
    large_file_threshold_bytes: u64,
    very_large_file_threshold_bytes: u64,
    huge_file_threshold_bytes: u64,
    old_file_threshold_days: u32,
) -> Result<AnalysisConfig, CommandError> {
    if options.maximum_depth == 0 || options.maximum_depth > 64 {
        return Err(CommandError::new(
            ErrorCode::InvalidSettings,
            "Custom scan depth must be between 1 and 64.",
            true,
        ));
    }
    if options.minimum_file_size_bytes > 1_099_511_627_776 {
        return Err(CommandError::new(
            ErrorCode::InvalidSettings,
            "Custom minimum file size cannot exceed 1 TiB.",
            true,
        ));
    }
    if options.enabled_categories.is_empty()
        || options
            .enabled_categories
            .iter()
            .any(|category| !matches!(category, RuleCategory::LargeFile | RuleCategory::OldFile))
    {
        return Err(CommandError::new(
            ErrorCode::InvalidSettings,
            "Custom analysis currently supports the Large files and Old installers categories.",
            true,
        ));
    }
    let roots = validate_roots_with_limit(&options.roots, home, platform, 20)?;
    Ok(AnalysisConfig {
        roots,
        enabled_categories: options.enabled_categories.clone(),
        minimum_file_size_bytes: options.minimum_file_size_bytes,
        maximum_depth: options.maximum_depth,
        include_hidden_files: options.include_hidden_files,
        include_external_drives: options.include_external_drives,
        large_file_threshold_bytes,
        very_large_file_threshold_bytes,
        huge_file_threshold_bytes,
        old_file_threshold_days,
    })
}

pub fn analyze(
    scan_id: &str,
    home: &Path,
    config: &AnalysisConfig,
    exclusions: &ExclusionMatcher,
    cancellation: &CancellationToken,
    mut on_progress: impl FnMut(&Path, &AnalysisProgress),
    mut on_finding: impl FnMut(Finding) -> Result<(), CommandError>,
) -> Result<AnalysisProgress, CommandError> {
    let mut progress = AnalysisProgress::default();
    for root in &config.roots {
        let root_metadata = fs::metadata(root).map_err(|error| path_error(root, error))?;
        let root_device = device_id(&root_metadata);
        let mut pending = vec![(root.clone(), 0_u16)];
        while let Some((path, depth)) = pending.pop() {
            if cancellation.is_cancelled() {
                return Ok(progress);
            }
            if exclusions.is_excluded(&path)
                || (!config.include_hidden_files && is_hidden(&path, root))
            {
                progress.skipped_count += 1;
                continue;
            }
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    record_error(&mut progress, &path, error);
                    continue;
                }
            };
            if metadata.file_type().is_symlink() {
                progress.skipped_count += 1;
                continue;
            }
            if metadata.is_dir() {
                progress.directories_scanned += 1;
                if depth >= config.maximum_depth {
                    progress.skipped_count += 1;
                    continue;
                }
                if !config.include_external_drives && device_id(&metadata) != root_device {
                    progress.skipped_count += 1;
                    continue;
                }
                match fs::read_dir(&path) {
                    Ok(entries) => {
                        for entry in entries {
                            if cancellation.is_cancelled() {
                                break;
                            }
                            match entry {
                                Ok(entry) => pending.push((entry.path(), depth + 1)),
                                Err(error) => record_error(&mut progress, &path, error),
                            }
                        }
                        if pending.len() > MAX_PENDING_PATHS {
                            return Err(CommandError::new(
                                ErrorCode::CommandUnavailable,
                                "Custom analysis found too many pending paths. Choose a narrower folder or lower the maximum depth.",
                                true,
                            ));
                        }
                    }
                    Err(error) => record_error(&mut progress, &path, error),
                }
            } else if metadata.is_file() {
                progress.files_scanned += 1;
                progress.bytes_examined = progress.bytes_examined.saturating_add(metadata.len());
                if metadata.len() >= config.minimum_file_size_bytes {
                    if let Some(finding) = classify_file(scan_id, home, &path, &metadata, config) {
                        on_finding(finding)?;
                    }
                }
            } else {
                progress.skipped_count += 1;
            }
            on_progress(&path, &progress);
        }
    }
    Ok(progress)
}

fn classify_file(
    scan_id: &str,
    home: &Path,
    path: &Path,
    metadata: &fs::Metadata,
    config: &AnalysisConfig,
) -> Option<Finding> {
    let modified = metadata.modified().ok().map(DateTime::<Utc>::from);
    let large = config.enabled_categories.contains(&RuleCategory::LargeFile)
        && metadata.len() >= config.large_file_threshold_bytes;
    let old = config.enabled_categories.contains(&RuleCategory::OldFile)
        && is_installer(path)
        && modified.is_some_and(|value| {
            value <= Utc::now() - Duration::days(i64::from(config.old_file_threshold_days))
        });
    let (rule_id, category, display_name, description, evidence) = if large {
        (
            "analysis.large-file-v1",
            RuleCategory::LargeFile,
            size_label(metadata.len(), config),
            "A large file in an explicitly selected folder. DiskSage does not classify it as junk; review its ownership and contents before taking any action.".to_owned(),
            FindingEvidence::SizeThreshold {
                minimum_bytes: config.large_file_threshold_bytes,
            },
        )
    } else if old {
        (
            "analysis.old-installer-v1",
            RuleCategory::OldFile,
            "Old installer".to_owned(),
            format!(
                "An installer image unchanged for at least {} days. It remains a review suggestion and is never selected automatically.",
                config.old_file_threshold_days
            ),
            FindingEvidence::AgeThreshold {
                older_than_days: config.old_file_threshold_days,
            },
        )
    } else {
        return None;
    };
    Some(Finding {
        id: Uuid::new_v4().to_string(),
        scan_id: scan_id.to_owned(),
        rule_id: rule_id.to_owned(),
        rule_version: 1,
        category,
        display_name,
        description,
        path: path.to_path_buf(),
        display_path: display_path(path, home),
        item_type: FindingType::File,
        logical_size: metadata.len(),
        allocated_size: Some(allocated_size(metadata)),
        modified_at: modified,
        risk: RiskLevel::Careful,
        recommended_action: RecommendedAction::Review,
        evidence,
        cleanup_allowed: false,
        cleanup_block_reason: Some(
            "Analysis suggestions are review-only and are never cleanup-authorized.".to_owned(),
        ),
        guided_action: None,
    })
}

fn is_installer(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "dmg" | "pkg" | "appimage" | "deb" | "rpm" | "iso"
            )
        })
}

fn size_label(size: u64, config: &AnalysisConfig) -> String {
    if size >= config.huge_file_threshold_bytes {
        "Huge file"
    } else if size >= config.very_large_file_threshold_bytes {
        "Very large file"
    } else {
        "Large file"
    }
    .to_owned()
}

fn is_hidden(path: &Path, root: &Path) -> bool {
    path != root
        && path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| format!("~/{}", relative.to_string_lossy()))
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

fn record_error(progress: &mut AnalysisProgress, path: &Path, error: std::io::Error) {
    let code = if error.kind() == std::io::ErrorKind::PermissionDenied {
        progress.permission_denied_count += 1;
        ErrorCode::PermissionDenied
    } else {
        progress.skipped_count += 1;
        ErrorCode::FilesystemError
    };
    if progress.errors.len() < 50 {
        progress.errors.push(
            CommandError::new(code, "A custom analysis path could not be read.", true)
                .with_path(path.to_string_lossy())
                .with_details(error.to_string()),
        );
    }
}

fn path_error(path: &Path, error: std::io::Error) -> CommandError {
    let code = if error.kind() == std::io::ErrorKind::PermissionDenied {
        ErrorCode::PermissionDenied
    } else {
        ErrorCode::InvalidPath
    };
    CommandError::new(code, "A custom scan root could not be accessed.", true)
        .with_path(path.to_string_lossy())
        .with_details(error.to_string())
}

#[cfg(unix)]
fn device_id(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.dev()
}

#[cfg(not(unix))]
fn device_id(_: &fs::Metadata) -> u64 {
    0
}

#[cfg(unix)]
fn allocated_size(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn allocated_size(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(root: PathBuf) -> AnalysisConfig {
        AnalysisConfig {
            roots: vec![root],
            enabled_categories: vec![RuleCategory::LargeFile, RuleCategory::OldFile],
            minimum_file_size_bytes: 0,
            maximum_depth: 8,
            include_hidden_files: false,
            include_external_drives: false,
            large_file_threshold_bytes: 4,
            very_large_file_threshold_bytes: 8,
            huge_file_threshold_bytes: 16,
            old_file_threshold_days: 365,
        }
    }

    #[test]
    fn large_files_are_review_only() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("selected");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("large.bin"), b"large").unwrap();
        let mut findings = Vec::new();
        analyze(
            &Uuid::new_v4().to_string(),
            directory.path(),
            &config(root),
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
            |finding| {
                findings.push(finding);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, RuleCategory::LargeFile);
        assert!(!findings[0].cleanup_allowed);
        assert_eq!(findings[0].recommended_action, RecommendedAction::Review);
    }

    #[test]
    fn hidden_files_and_non_installer_old_files_are_excluded() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("selected");
        fs::create_dir(&root).unwrap();
        fs::write(root.join(".hidden"), b"x").unwrap();
        fs::write(root.join("document.txt"), b"x").unwrap();
        let mut findings = Vec::new();
        let mut settings = config(root);
        settings.large_file_threshold_bytes = 100;
        analyze(
            &Uuid::new_v4().to_string(),
            directory.path(),
            &settings,
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
            |finding| {
                findings.push(finding);
                Ok(())
            },
        )
        .unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn old_installer_is_a_review_suggestion() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("selected");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("installer.dmg"), b"x").unwrap();
        let mut settings = config(root);
        settings.large_file_threshold_bytes = 100;
        settings.old_file_threshold_days = 0;
        let mut findings = Vec::new();
        analyze(
            &Uuid::new_v4().to_string(),
            directory.path(),
            &settings,
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
            |finding| {
                findings.push(finding);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, RuleCategory::OldFile);
        assert!(!findings[0].cleanup_allowed);
    }
}
