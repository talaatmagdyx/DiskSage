use std::{fs, path::Path};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        cleanup::CleanupPlanItem,
        error::{CommandError, ErrorCode},
        finding::{Finding, FindingType},
        rule::{RecommendedAction, RiskLevel},
    },
    rules::registry::RulesRegistry,
    safety::protected_paths::ProtectedPathPolicy,
    scanner::{
        cancellation::CancellationToken, exclusions::ExclusionMatcher, walker::measure_target,
    },
};

const MAX_SIZE_DRIFT_PERCENT: u64 = 10;
const MIN_SIZE_DRIFT_BYTES: u64 = 1024 * 1024;

pub fn snapshot_finding(
    finding: &Finding,
    home: &Path,
    platform: &str,
    rules: &RulesRegistry,
) -> Result<CleanupPlanItem, CommandError> {
    if !finding.cleanup_allowed
        || finding.risk != RiskLevel::Safe
        || finding.recommended_action != RecommendedAction::MoveToTrash
    {
        return Err(validation_error(
            &finding.path,
            "The finding is not approved for safe cleanup.",
        ));
    }
    let resolved = rules
        .resolve(&finding.rule_id, finding.rule_version, home, platform)
        .filter(|rule| rule.target == finding.path)
        .ok_or_else(|| {
            validation_error(
                &finding.path,
                "The finding no longer matches a known cleanup rule.",
            )
        })?;
    validate_path_boundary(&finding.path, &resolved.target, home, platform)?;
    let canonical_path = finding.path.canonicalize().map_err(|error| {
        filesystem_error(&finding.path, "The cleanup item is unavailable.", error)
    })?;
    let item_type = metadata_type(&finding.path)?;
    if item_type != finding.item_type || item_type == FindingType::Symlink {
        return Err(validation_error(
            &finding.path,
            "The finding type changed or is a symbolic link.",
        ));
    }
    let measurement = measure_target(
        &finding.path,
        &ExclusionMatcher::default(),
        &CancellationToken::default(),
        |_, _| {},
    );
    if !measurement.exists || !measurement.errors.is_empty() {
        return Err(validation_error(
            &finding.path,
            "The finding could not be measured safely.",
        ));
    }
    Ok(CleanupPlanItem {
        scan_id: finding.scan_id.clone(),
        finding_id: finding.id.clone(),
        rule_id: finding.rule_id.clone(),
        rule_version: finding.rule_version,
        path: finding.path.clone(),
        canonical_path,
        expected_type: item_type,
        expected_size: measurement.logical_size,
        expected_modified_at: measurement.modified_at.map(DateTime::<Utc>::from),
        risk: finding.risk,
        validation_token: Uuid::new_v4().to_string(),
    })
}

pub fn revalidate_item(
    item: &CleanupPlanItem,
    home: &Path,
    platform: &str,
    rules: &RulesRegistry,
    cancellation: &CancellationToken,
) -> Result<(), CommandError> {
    Uuid::parse_str(&item.validation_token).map_err(|_| {
        validation_error(&item.path, "The cleanup item validation token is invalid.")
    })?;
    let resolved = rules
        .resolve(&item.rule_id, item.rule_version, home, platform)
        .filter(|rule| rule.target == item.path)
        .ok_or_else(|| {
            validation_error(
                &item.path,
                "The cleanup item no longer matches its original rule.",
            )
        })?;
    validate_path_boundary(&item.path, &resolved.target, home, platform)?;
    let canonical_path = item
        .path
        .canonicalize()
        .map_err(|error| filesystem_error(&item.path, "The cleanup item is unavailable.", error))?;
    if canonical_path != item.canonical_path {
        return Err(validation_error(
            &item.path,
            "The cleanup path was redirected after the plan was created.",
        ));
    }
    let item_type = metadata_type(&item.path)?;
    if item_type != item.expected_type || item_type == FindingType::Symlink {
        return Err(validation_error(
            &item.path,
            "The cleanup item type changed after the plan was created.",
        ));
    }
    let measurement = measure_target(
        &item.path,
        &ExclusionMatcher::default(),
        cancellation,
        |_, _| {},
    );
    if cancellation.is_cancelled() {
        return Err(CommandError::new(
            ErrorCode::ScanCancelled,
            "Cleanup was cancelled before this item was moved.",
            true,
        ));
    }
    if !measurement.exists || !measurement.errors.is_empty() {
        return Err(validation_error(
            &item.path,
            "The cleanup item could not be revalidated safely.",
        ));
    }
    let allowed_drift =
        MIN_SIZE_DRIFT_BYTES.max(item.expected_size.saturating_mul(MAX_SIZE_DRIFT_PERCENT) / 100);
    if item.expected_size.abs_diff(measurement.logical_size) > allowed_drift {
        return Err(validation_error(
            &item.path,
            "The cleanup item size changed unexpectedly.",
        ));
    }
    let current_modified = measurement.modified_at.map(DateTime::<Utc>::from);
    if item.expected_modified_at != current_modified {
        return Err(validation_error(
            &item.path,
            "The cleanup item was modified after the plan was created.",
        ));
    }
    Ok(())
}

fn validate_path_boundary(
    candidate: &Path,
    approved_target: &Path,
    home: &Path,
    platform: &str,
) -> Result<(), CommandError> {
    if !candidate.is_absolute() || candidate.parent().is_none() || candidate == home {
        return Err(CommandError::new(
            ErrorCode::PathProtected,
            "A protected location cannot be cleaned.",
            false,
        )
        .with_path(candidate.to_string_lossy()));
    }
    let canonical_candidate = candidate.canonicalize().map_err(|error| {
        filesystem_error(candidate, "The cleanup item no longer exists.", error)
    })?;
    let canonical_target = approved_target.canonicalize().map_err(|error| {
        filesystem_error(
            approved_target,
            "The approved cleanup target is unavailable.",
            error,
        )
    })?;
    if canonical_candidate != canonical_target {
        return Err(validation_error(
            candidate,
            "The cleanup path no longer matches its approved root.",
        ));
    }
    let canonical_home = home.canonicalize().unwrap_or_else(|_| home.to_path_buf());
    let policy = ProtectedPathPolicy::for_platform(&canonical_home, platform);
    if let Some(reason) = policy.check_cleanup_candidate(&canonical_candidate, true) {
        return Err(CommandError::new(
            ErrorCode::PathProtected,
            "A protected location cannot be cleaned.",
            false,
        )
        .with_path(candidate.to_string_lossy())
        .with_details(format!("protected by {}", reason.reason)));
    }
    Ok(())
}

fn metadata_type(path: &Path) -> Result<FindingType, CommandError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| filesystem_error(path, "The cleanup item is unavailable.", error))?;
    if metadata.file_type().is_symlink() {
        Ok(FindingType::Symlink)
    } else if metadata.is_file() {
        Ok(FindingType::File)
    } else if metadata.is_dir() {
        Ok(FindingType::Directory)
    } else {
        Err(validation_error(
            path,
            "This filesystem item type is not supported for cleanup.",
        ))
    }
}

fn validation_error(path: &Path, message: &str) -> CommandError {
    CommandError::new(ErrorCode::PlanValidationFailed, message, true)
        .with_path(path.to_string_lossy())
}

fn filesystem_error(path: &Path, message: &str, error: std::io::Error) -> CommandError {
    let code = match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::FilesystemError,
    };
    CommandError::new(code, message, true)
        .with_path(path.to_string_lossy())
        .with_details(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        finding::{Finding, FindingEvidence},
        rule::{RecommendedAction, RiskLevel, RuleCategory},
    };
    use std::io::Write;

    fn finding(home: &Path) -> Finding {
        let path = home.join(".npm/_cacache");
        fs::create_dir_all(&path).unwrap();
        fs::File::create(path.join("fixture"))
            .unwrap()
            .write_all(b"cache")
            .unwrap();
        Finding {
            id: Uuid::new_v4().to_string(),
            scan_id: Uuid::new_v4().to_string(),
            rule_id: "cache.npm.content-v1".to_owned(),
            rule_version: 1,
            category: RuleCategory::PackageManagerCache,
            display_name: "npm cache".to_owned(),
            description: "fixture".to_owned(),
            path: path.clone(),
            display_path: path.to_string_lossy().into_owned(),
            item_type: FindingType::Directory,
            logical_size: 5,
            allocated_size: Some(4096),
            modified_at: None,
            risk: RiskLevel::Safe,
            recommended_action: RecommendedAction::MoveToTrash,
            evidence: FindingEvidence::KnownPath,
            cleanup_allowed: true,
            cleanup_block_reason: None,
        }
    }

    #[test]
    fn rejects_root_home_and_sensitive_paths() {
        let directory = tempfile::tempdir().unwrap();
        let policy = ProtectedPathPolicy::for_platform(directory.path(), "linux");
        assert_eq!(
            validate_path_boundary(Path::new("/"), Path::new("/"), directory.path(), "linux")
                .unwrap_err()
                .code,
            ErrorCode::PathProtected
        );
        assert_eq!(
            validate_path_boundary(
                directory.path(),
                directory.path(),
                directory.path(),
                "linux",
            )
            .unwrap_err()
            .code,
            ErrorCode::PathProtected
        );
        assert!(policy
            .check_cleanup_candidate(&directory.path().join(".ssh/key"), true)
            .is_some());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_symlink_finding_without_touching_its_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target");
        fs::create_dir(&target).unwrap();
        let item = finding(directory.path());
        fs::remove_dir_all(&item.path).unwrap();
        std::os::unix::fs::symlink(&target, &item.path).unwrap();
        assert!(snapshot_finding(&item, directory.path(), "linux", &RulesRegistry).is_err());
        assert!(target.exists());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_parent_symlink_redirect_after_plan_creation() {
        let directory = tempfile::tempdir().unwrap();
        let finding = finding(directory.path());
        let item = snapshot_finding(&finding, directory.path(), "linux", &RulesRegistry).unwrap();
        let original_parent = directory.path().join(".npm");
        fs::rename(&original_parent, directory.path().join(".npm-original")).unwrap();
        let redirected_parent = directory.path().join("redirected");
        fs::create_dir_all(redirected_parent.join("_cacache")).unwrap();
        std::os::unix::fs::symlink(&redirected_parent, &original_parent).unwrap();
        assert!(revalidate_item(
            &item,
            directory.path(),
            "linux",
            &RulesRegistry,
            &CancellationToken::default(),
        )
        .is_err());
        assert!(redirected_parent.exists());
    }

    #[test]
    fn refuses_an_item_modified_after_plan_creation() {
        let directory = tempfile::tempdir().unwrap();
        let finding = finding(directory.path());
        let item = snapshot_finding(&finding, directory.path(), "linux", &RulesRegistry).unwrap();
        fs::File::create(finding.path.join("changed"))
            .unwrap()
            .write_all(b"changed")
            .unwrap();
        assert!(revalidate_item(
            &item,
            directory.path(),
            "linux",
            &RulesRegistry,
            &CancellationToken::default(),
        )
        .is_err());
    }

    #[test]
    fn duplicate_findings_have_no_cleanup_rule_in_this_phase() {
        let directory = tempfile::tempdir().unwrap();
        let mut finding = finding(directory.path());
        finding.rule_id = "duplicate.content-v1".to_owned();
        finding.category = RuleCategory::Duplicate;
        finding.cleanup_allowed = false;
        assert!(snapshot_finding(&finding, directory.path(), "linux", &RulesRegistry).is_err());
    }
}
