use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        duplicate::{
            DuplicateCopy, DuplicateGroup, DuplicateProgress, DuplicateScanPhase, DuplicateSummary,
        },
        error::{CommandError, ErrorCode},
    },
    scanner::cancellation::CancellationToken,
};

use super::hashing;

const MAX_CANDIDATES: usize = 250_000;
const MAX_RECORDED_ERRORS: usize = 100;

#[derive(Debug, Clone)]
struct Candidate {
    path: PathBuf,
    size: u64,
    modified_at: Option<DateTime<Utc>>,
    owner: Option<String>,
}

pub struct ScanOutput {
    pub groups: Vec<DuplicateGroup>,
    pub summary: DuplicateSummary,
}

pub fn validate_roots(
    roots: &[String],
    home: &Path,
    platform: &str,
) -> Result<Vec<PathBuf>, CommandError> {
    if roots.is_empty() || roots.len() > 8 {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "Choose between 1 and 8 folders for duplicate analysis.",
            true,
        ));
    }
    let canonical_home = home.canonicalize().unwrap_or_else(|_| home.to_path_buf());
    let mut validated = Vec::with_capacity(roots.len());
    for value in roots {
        let path = PathBuf::from(value);
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "Duplicate scan folders must be absolute and cannot contain traversal.",
                true,
            )
            .with_path(value));
        }
        let link_metadata =
            fs::symlink_metadata(&path).map_err(|error| path_error(&path, error))?;
        if link_metadata.file_type().is_symlink() || !link_metadata.is_dir() {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "Duplicate scan roots must be real directories, not files or symbolic links.",
                true,
            )
            .with_path(value));
        }
        let canonical = path
            .canonicalize()
            .map_err(|error| path_error(&path, error))?;
        if canonical.parent().is_none()
            || canonical == canonical_home
            || is_sensitive_root(&canonical, &canonical_home, platform)
        {
            return Err(CommandError::new(
                ErrorCode::PathProtected,
                "Choose a narrower folder. Filesystem, home, credential, and system roots are protected.",
                true,
            )
            .with_path(value));
        }
        validated.push(canonical);
    }
    validated.sort();
    validated.dedup();
    let mut narrowed = Vec::new();
    for root in validated {
        if !narrowed
            .iter()
            .any(|parent: &PathBuf| root.starts_with(parent))
        {
            narrowed.push(root);
        }
    }
    Ok(narrowed)
}

pub fn run(
    scan_id: &str,
    roots: Vec<PathBuf>,
    minimum_size_bytes: u64,
    byte_for_byte_verification: bool,
    home: &Path,
    cancellation: &CancellationToken,
    mut emit_progress: impl FnMut(&DuplicateProgress),
) -> Result<ScanOutput, CommandError> {
    let started_at = Utc::now();
    let timer = Instant::now();
    let mut progress = DuplicateProgress {
        scan_id: scan_id.to_owned(),
        phase: DuplicateScanPhase::Discovering,
        current_path: None,
        files_scanned: 0,
        candidate_files: 0,
        bytes_hashed: 0,
        groups_found: 0,
        reclaimable_bytes: 0,
        skipped_count: 0,
        permission_denied_count: 0,
        elapsed_ms: 0,
    };
    emit_progress(&progress);
    let mut errors = Vec::new();
    let candidates = discover(
        &roots,
        minimum_size_bytes,
        cancellation,
        &mut progress,
        &mut errors,
        &timer,
        &mut emit_progress,
    )?;

    check_cancelled(cancellation)?;
    progress.phase = DuplicateScanPhase::Grouping;
    progress.current_path = None;
    update_elapsed(&mut progress, &timer);
    emit_progress(&progress);

    let mut size_groups: HashMap<u64, Vec<Candidate>> = HashMap::new();
    for candidate in candidates {
        size_groups
            .entry(candidate.size)
            .or_default()
            .push(candidate);
    }
    let mut sized: Vec<Candidate> = size_groups
        .into_values()
        .filter(|group| group.len() > 1)
        .flatten()
        .collect();
    sized.sort_by(|left, right| left.path.cmp(&right.path));
    progress.candidate_files = sized.len() as u64;

    progress.phase = DuplicateScanPhase::PartialHashing;
    emit_progress(&progress);
    let mut partial_groups: HashMap<(u64, String), Vec<Candidate>> = HashMap::new();
    for candidate in sized {
        check_cancelled(cancellation)?;
        progress.current_path = Some(display_path(&candidate.path, home));
        match hashing::partial_hash(&candidate.path, candidate.size, cancellation) {
            Ok((hash, bytes)) => {
                progress.bytes_hashed = progress.bytes_hashed.saturating_add(bytes);
                partial_groups
                    .entry((candidate.size, hash))
                    .or_default()
                    .push(candidate);
            }
            Err(error) if error.code == ErrorCode::ScanCancelled => return Err(error),
            Err(error) => record_error(error, &mut errors, &mut progress),
        }
        update_elapsed(&mut progress, &timer);
        emit_progress(&progress);
    }

    progress.phase = DuplicateScanPhase::FullHashing;
    progress.current_path = None;
    emit_progress(&progress);
    let mut full_groups: HashMap<(u64, String), Vec<Candidate>> = HashMap::new();
    let mut partial_values: Vec<Vec<Candidate>> = partial_groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect();
    partial_values.sort_by_key(|group| group.first().map(|item| item.path.clone()));
    for group in partial_values {
        for candidate in group {
            check_cancelled(cancellation)?;
            progress.current_path = Some(display_path(&candidate.path, home));
            if !metadata_still_matches(&candidate) {
                progress.skipped_count += 1;
                continue;
            }
            match hashing::full_hash(&candidate.path, cancellation) {
                Ok((hash, bytes)) if metadata_still_matches(&candidate) => {
                    progress.bytes_hashed = progress.bytes_hashed.saturating_add(bytes);
                    full_groups
                        .entry((candidate.size, hash))
                        .or_default()
                        .push(candidate);
                }
                Ok(_) => progress.skipped_count += 1,
                Err(error) if error.code == ErrorCode::ScanCancelled => return Err(error),
                Err(error) => record_error(error, &mut errors, &mut progress),
            }
            update_elapsed(&mut progress, &timer);
            emit_progress(&progress);
        }
    }

    let mut verified_groups = Vec::new();
    let mut full_values: Vec<((u64, String), Vec<Candidate>)> = full_groups
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .collect();
    full_values.sort_by(|left, right| left.0.cmp(&right.0));
    if byte_for_byte_verification {
        progress.phase = DuplicateScanPhase::Verifying;
        progress.current_path = None;
        emit_progress(&progress);
    }
    for ((size, full_hash), group) in full_values {
        let clusters = if byte_for_byte_verification {
            verify_clusters(
                group,
                cancellation,
                &mut progress,
                &timer,
                &mut emit_progress,
            )?
        } else {
            vec![group]
        };
        for cluster in clusters.into_iter().filter(|cluster| cluster.len() > 1) {
            let group = build_group(
                scan_id,
                size,
                full_hash.clone(),
                cluster,
                byte_for_byte_verification,
                home,
            );
            progress.groups_found += 1;
            progress.reclaimable_bytes = progress
                .reclaimable_bytes
                .saturating_add(group.reclaimable_bytes);
            verified_groups.push(group);
            emit_progress(&progress);
        }
    }

    progress.phase = DuplicateScanPhase::Finalizing;
    progress.current_path = None;
    update_elapsed(&mut progress, &timer);
    emit_progress(&progress);
    let duplicate_files = verified_groups
        .iter()
        .map(|group| group.copies.len() as u64)
        .sum();
    let summary = DuplicateSummary {
        scan_id: scan_id.to_owned(),
        phase: DuplicateScanPhase::Completed,
        roots,
        minimum_size_bytes,
        byte_for_byte_verification,
        started_at,
        completed_at: Some(Utc::now()),
        files_scanned: progress.files_scanned,
        candidate_files: progress.candidate_files,
        bytes_hashed: progress.bytes_hashed,
        groups_found: progress.groups_found,
        duplicate_files,
        reclaimable_bytes: progress.reclaimable_bytes,
        skipped_count: progress.skipped_count,
        permission_denied_count: progress.permission_denied_count,
        elapsed_ms: timer.elapsed().as_millis() as u64,
        errors,
    };
    Ok(ScanOutput {
        groups: verified_groups,
        summary,
    })
}

fn discover(
    roots: &[PathBuf],
    minimum_size_bytes: u64,
    cancellation: &CancellationToken,
    progress: &mut DuplicateProgress,
    errors: &mut Vec<CommandError>,
    timer: &Instant,
    emit_progress: &mut impl FnMut(&DuplicateProgress),
) -> Result<Vec<Candidate>, CommandError> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        let root_metadata = fs::metadata(root).map_err(|error| path_error(root, error))?;
        let root_device = device_id(&root_metadata);
        let mut stack = vec![root.clone()];
        while let Some(directory) = stack.pop() {
            check_cancelled(cancellation)?;
            let entries = match fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) => {
                    record_error(path_error(&directory, error), errors, progress);
                    continue;
                }
            };
            for entry in entries {
                check_cancelled(cancellation)?;
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        record_error(path_error(&directory, error), errors, progress);
                        continue;
                    }
                };
                let path = entry.path();
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        record_error(path_error(&path, error), errors, progress);
                        continue;
                    }
                };
                if metadata.file_type().is_symlink() {
                    progress.skipped_count += 1;
                    continue;
                }
                if metadata.is_dir() {
                    if device_id(&metadata) == root_device {
                        stack.push(path);
                    } else {
                        progress.skipped_count += 1;
                    }
                    continue;
                }
                if !metadata.is_file() {
                    progress.skipped_count += 1;
                    continue;
                }
                progress.files_scanned += 1;
                if metadata.len() < minimum_size_bytes || metadata.len() == 0 {
                    continue;
                }
                if candidates.len() >= MAX_CANDIDATES {
                    return Err(CommandError::new(
                        ErrorCode::CommandUnavailable,
                        "This folder contains too many duplicate candidates. Choose a narrower folder or increase the minimum size.",
                        true,
                    ));
                }
                if seen.insert(path.clone()) {
                    candidates.push(Candidate {
                        path,
                        size: metadata.len(),
                        modified_at: metadata.modified().ok().map(DateTime::<Utc>::from),
                        owner: owner(&metadata),
                    });
                }
                if progress.files_scanned % 128 == 0 {
                    progress.current_path = Some(directory.to_string_lossy().into_owned());
                    update_elapsed(progress, timer);
                    emit_progress(progress);
                }
            }
        }
    }
    Ok(candidates)
}

fn verify_clusters(
    group: Vec<Candidate>,
    cancellation: &CancellationToken,
    progress: &mut DuplicateProgress,
    timer: &Instant,
    emit_progress: &mut impl FnMut(&DuplicateProgress),
) -> Result<Vec<Vec<Candidate>>, CommandError> {
    let mut clusters: Vec<Vec<Candidate>> = Vec::new();
    for candidate in group {
        check_cancelled(cancellation)?;
        let mut candidate = Some(candidate);
        for cluster in &mut clusters {
            let reference = &cluster[0];
            let value = candidate.as_ref().expect("candidate is available");
            progress.current_path = Some(value.path.to_string_lossy().into_owned());
            let (equal, compared) =
                hashing::byte_for_byte_equal(&reference.path, &value.path, cancellation)?;
            progress.bytes_hashed = progress
                .bytes_hashed
                .saturating_add(compared.saturating_mul(2));
            update_elapsed(progress, timer);
            emit_progress(progress);
            if equal {
                cluster.push(candidate.take().expect("candidate is available"));
                break;
            }
        }
        if let Some(candidate) = candidate {
            clusters.push(vec![candidate]);
        }
    }
    Ok(clusters)
}

fn build_group(
    scan_id: &str,
    size: u64,
    full_hash: String,
    candidates: Vec<Candidate>,
    byte_for_byte_verified: bool,
    home: &Path,
) -> DuplicateGroup {
    let keep_index = candidates
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| compare_keep(left, right))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let copies: Vec<DuplicateCopy> = candidates
        .iter()
        .map(|candidate| DuplicateCopy {
            id: Uuid::new_v4().to_string(),
            path: candidate.path.clone(),
            display_path: display_path(&candidate.path, home),
            modified_at: candidate.modified_at,
            owner: candidate.owner.clone(),
        })
        .collect();
    let recommended_keep_id = copies[keep_index].id.clone();
    DuplicateGroup {
        id: Uuid::new_v4().to_string(),
        scan_id: scan_id.to_owned(),
        file_size: size,
        reclaimable_bytes: size.saturating_mul(copies.len().saturating_sub(1) as u64),
        copies,
        recommended_keep_id,
        keep_reason: keep_reason(&candidates, keep_index),
        full_hash,
        byte_for_byte_verified,
    }
}

fn compare_keep(left: &Candidate, right: &Candidate) -> Ordering {
    is_cache_path(&left.path)
        .cmp(&is_cache_path(&right.path))
        .then_with(|| left.modified_at.cmp(&right.modified_at))
        .then_with(|| {
            left.path
                .components()
                .count()
                .cmp(&right.path.components().count())
        })
        .then_with(|| left.path.cmp(&right.path))
}

fn keep_reason(candidates: &[Candidate], keep_index: usize) -> String {
    let keep = &candidates[keep_index];
    if !is_cache_path(&keep.path)
        && candidates
            .iter()
            .any(|candidate| is_cache_path(&candidate.path))
    {
        "Preferred because it is outside a cache or generated-artifact folder.".to_owned()
    } else if keep.modified_at.is_some() {
        "Preferred as the oldest stable copy, then by the shortest path.".to_owned()
    } else {
        "Preferred by the shortest stable path.".to_owned()
    }
}

fn is_cache_path(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(
            value.as_str(),
            "cache"
                | "caches"
                | ".cache"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".gradle"
        )
    })
}

fn metadata_still_matches(candidate: &Candidate) -> bool {
    fs::symlink_metadata(&candidate.path).is_ok_and(|metadata| {
        metadata.is_file()
            && !metadata.file_type().is_symlink()
            && metadata.len() == candidate.size
            && metadata.modified().ok().map(DateTime::<Utc>::from) == candidate.modified_at
    })
}

fn record_error(
    error: CommandError,
    errors: &mut Vec<CommandError>,
    progress: &mut DuplicateProgress,
) {
    if error.code == ErrorCode::PermissionDenied {
        progress.permission_denied_count += 1;
    } else {
        progress.skipped_count += 1;
    }
    if errors.len() < MAX_RECORDED_ERRORS {
        errors.push(error);
    }
}

fn check_cancelled(cancellation: &CancellationToken) -> Result<(), CommandError> {
    if cancellation.is_cancelled() {
        Err(CommandError::new(
            ErrorCode::ScanCancelled,
            "Duplicate analysis was cancelled.",
            true,
        ))
    } else {
        Ok(())
    }
}

fn update_elapsed(progress: &mut DuplicateProgress, timer: &Instant) {
    progress.elapsed_ms = timer.elapsed().as_millis() as u64;
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| format!("~/{}", relative.to_string_lossy()))
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

fn is_sensitive_root(path: &Path, home: &Path, platform: &str) -> bool {
    let sensitive_home = [".ssh", ".gnupg", ".aws", ".kube", "Library/Keychains"];
    if sensitive_home
        .iter()
        .any(|relative| path.starts_with(home.join(relative)))
    {
        return true;
    }
    let system_roots: &[&str] = if platform == "macos" {
        &[
            "/System",
            "/usr",
            "/bin",
            "/sbin",
            "/etc",
            "/var",
            "/private",
            "/Applications",
        ]
    } else {
        &[
            "/usr", "/bin", "/sbin", "/etc", "/var", "/proc", "/sys", "/dev", "/run", "/root",
        ]
    };
    system_roots.iter().any(|root| path.starts_with(root))
}

fn path_error(path: &Path, error: std::io::Error) -> CommandError {
    let code = match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::FilesystemError,
    };
    CommandError::new(code, "A duplicate scan path could not be accessed.", true)
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
fn owner(metadata: &fs::Metadata) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    Some(format!("UID {}", metadata.uid()))
}

#[cfg(not(unix))]
fn owner(_: &fs::Metadata) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn staged_scan_never_groups_same_size_non_identical_files() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        fs::create_dir_all(&root).unwrap();
        fs::File::create(root.join("a.bin"))
            .unwrap()
            .write_all(b"abcd")
            .unwrap();
        fs::File::create(root.join("b.bin"))
            .unwrap()
            .write_all(b"abce")
            .unwrap();
        let output = run(
            &Uuid::new_v4().to_string(),
            vec![root],
            1,
            true,
            directory.path(),
            &CancellationToken::default(),
            |_| {},
        )
        .unwrap();
        assert!(output.groups.is_empty());
    }

    #[test]
    fn identical_files_form_a_group_with_one_recommended_keep() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.bin"), b"identical").unwrap();
        fs::write(root.join("b.bin"), b"identical").unwrap();
        let output = run(
            &Uuid::new_v4().to_string(),
            vec![root],
            1,
            false,
            directory.path(),
            &CancellationToken::default(),
            |_| {},
        )
        .unwrap();
        assert_eq!(output.groups.len(), 1);
        assert_eq!(output.groups[0].copies.len(), 2);
        assert!(output.groups[0]
            .copies
            .iter()
            .any(|copy| copy.id == output.groups[0].recommended_keep_id));
    }

    #[test]
    fn roots_must_be_narrower_than_home() {
        let directory = tempfile::tempdir().unwrap();
        let home = directory.path();
        assert_eq!(
            validate_roots(&[home.to_string_lossy().into_owned()], home, "linux")
                .unwrap_err()
                .code,
            ErrorCode::PathProtected
        );
    }
}
