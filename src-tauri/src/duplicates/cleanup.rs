use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{
    cleanup::trash_executor,
    domain::{
        cleanup::{
            CleanupAction, CleanupItemResult, CleanupItemStatus, CleanupProgress, CleanupSummary,
        },
        duplicate::{
            CreateDuplicateCleanupPlanRequest, DuplicateCleanupPlan, DuplicateCleanupPlanItem,
            ExecuteDuplicateCleanupRequest,
        },
        error::{CommandError, ErrorCode},
    },
    persistence::{duplicates::DuplicateRepository, history::HistoryRepository},
    platform::disk_info,
    safety::protected_paths::ProtectedPathPolicy,
    scanner::cancellation::CancellationToken,
};

use super::hashing;

const PLAN_LIFETIME_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
struct ActiveCleanup {
    operation_id: String,
    cancellation: CancellationToken,
}

pub struct DuplicateCleanupManager {
    active: Mutex<Option<ActiveCleanup>>,
    plans: Mutex<HashMap<String, DuplicateCleanupPlan>>,
    consumed_plans: Mutex<HashSet<String>>,
    repository: DuplicateRepository,
    history: HistoryRepository,
}

impl DuplicateCleanupManager {
    pub fn new(repository: DuplicateRepository, history: HistoryRepository) -> Self {
        Self {
            active: Mutex::new(None),
            plans: Mutex::new(HashMap::new()),
            consumed_plans: Mutex::new(HashSet::new()),
            repository,
            history,
        }
    }

    pub fn create_plan(
        &self,
        request: CreateDuplicateCleanupPlanRequest,
        home: &Path,
        platform: &str,
    ) -> Result<DuplicateCleanupPlan, CommandError> {
        if request.action != CleanupAction::MoveToTrash {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Duplicate cleanup is Trash-only. Permanent deletion is disabled.",
                true,
            ));
        }
        let summary = self.repository.load_summary(&request.scan_id)?;
        if summary.phase != crate::domain::duplicate::DuplicateScanPhase::Completed {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Duplicate cleanup plans require a completed duplicate scan.",
                true,
            ));
        }
        if request.selections.is_empty() || request.selections.len() > 250 {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Select between 1 and 250 duplicate groups.",
                true,
            ));
        }
        let unique_groups: HashSet<String> = request
            .selections
            .iter()
            .map(|selection| selection.group_id.clone())
            .collect();
        if unique_groups.len() != request.selections.len() {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Each duplicate group may appear only once in a cleanup plan.",
                true,
            ));
        }

        let policy = ProtectedPathPolicy::for_platform(home, platform);
        let mut items = Vec::new();
        let mut expected_reclaimable_bytes = 0_u64;
        for selection in request.selections {
            let group = self
                .repository
                .group(&request.scan_id, &selection.group_id)?;
            let keep = group
                .copies
                .iter()
                .find(|copy| copy.id == selection.keep_copy_id)
                .ok_or_else(|| {
                    invalid_plan("The selected keep copy is not in its duplicate group.")
                })?;
            let trash_ids: HashSet<_> = selection.trash_copy_ids.iter().collect();
            if selection.trash_copy_ids.is_empty()
                || trash_ids.len() != selection.trash_copy_ids.len()
                || trash_ids.contains(&selection.keep_copy_id)
                || selection.trash_copy_ids.len() >= group.copies.len()
            {
                return Err(invalid_plan(
                    "Every duplicate group must keep at least one copy, and the keep copy cannot be selected for Trash.",
                ));
            }
            let keep_snapshot = snapshot_file(&keep.path, group.file_size, keep.modified_at)?;
            if let Some(reason) = policy.check_cleanup_candidate(&keep_snapshot.0, true) {
                return Err(protected_error(&keep.path, reason.reason));
            }
            for copy_id in &selection.trash_copy_ids {
                let copy = group
                    .copies
                    .iter()
                    .find(|copy| &copy.id == copy_id)
                    .ok_or_else(|| {
                        invalid_plan("A selected Trash copy is not in its duplicate group.")
                    })?;
                let snapshot = snapshot_file(&copy.path, group.file_size, copy.modified_at)?;
                if let Some(reason) = policy.check_cleanup_candidate(&snapshot.0, true) {
                    return Err(protected_error(&copy.path, reason.reason));
                }
                if snapshot.0 == keep_snapshot.0 {
                    return Err(invalid_plan("The keep and Trash paths must be distinct."));
                }
                expected_reclaimable_bytes =
                    expected_reclaimable_bytes.saturating_add(group.file_size);
                items.push(DuplicateCleanupPlanItem {
                    group_id: group.id.clone(),
                    copy_id: copy.id.clone(),
                    path: copy.path.clone(),
                    canonical_path: snapshot.0,
                    expected_size: group.file_size,
                    expected_modified_at: snapshot.1,
                    full_hash: group.full_hash.clone(),
                    keep_copy_id: keep.id.clone(),
                    keep_path: keep.path.clone(),
                    keep_canonical_path: keep_snapshot.0.clone(),
                    keep_modified_at: keep_snapshot.1,
                    byte_for_byte_verified: group.byte_for_byte_verified,
                    validation_token: Uuid::new_v4().to_string(),
                });
            }
        }
        if items.len() > 500 {
            return Err(invalid_plan(
                "A duplicate cleanup plan may move at most 500 files at once.",
            ));
        }
        let created_at = Utc::now();
        let plan = DuplicateCleanupPlan {
            id: Uuid::new_v4().to_string(),
            scan_id: request.scan_id,
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_LIFETIME_MINUTES),
            action: CleanupAction::MoveToTrash,
            items,
            expected_reclaimable_bytes,
            kept_copy_count: unique_groups.len() as u64,
            confirmation_token: Uuid::new_v4().to_string(),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("duplicate plan lock poisoned"))?;
        plans.retain(|_, stored| Utc::now() < stored.expires_at);
        if plans.len() >= 128 {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Too many duplicate cleanup plans are awaiting review.",
                true,
            ));
        }
        plans.insert(plan.id.clone(), plan.clone());
        Ok(plan)
    }

    pub fn execute(
        self: &Arc<Self>,
        app: AppHandle,
        request: ExecuteDuplicateCleanupRequest,
        home: PathBuf,
        platform: &'static str,
    ) -> Result<String, CommandError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("duplicate cleanup lock poisoned"))?;
        if active.is_some() {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Another duplicate cleanup is already running.",
                true,
            ));
        }
        let plan = self.consume_plan(&request)?;
        let operation_id = Uuid::new_v4().to_string();
        let cancellation = CancellationToken::default();
        *active = Some(ActiveCleanup {
            operation_id: operation_id.clone(),
            cancellation: cancellation.clone(),
        });
        drop(active);
        let manager = Arc::clone(self);
        let worker_operation_id = operation_id.clone();
        tauri::async_runtime::spawn(async move {
            let blocking_operation_id = worker_operation_id.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                manager.run_cleanup(
                    app,
                    blocking_operation_id,
                    plan,
                    home,
                    platform,
                    cancellation,
                )
            })
            .await;
            if let Err(error) = result {
                tracing::error!(operation_id = %worker_operation_id, error = %error, "duplicate cleanup worker panicked");
            }
        });
        Ok(operation_id)
    }

    pub fn cancel(&self, operation_id: &str) -> Result<(), CommandError> {
        let active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("duplicate cleanup lock poisoned"))?;
        let cleanup = active
            .as_ref()
            .filter(|cleanup| cleanup.operation_id == operation_id)
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PathNotFound,
                    "That duplicate cleanup operation is not running.",
                    true,
                )
            })?;
        cleanup.cancellation.cancel();
        Ok(())
    }

    fn consume_plan(
        &self,
        request: &ExecuteDuplicateCleanupRequest,
    ) -> Result<DuplicateCleanupPlan, CommandError> {
        Uuid::parse_str(&request.plan_id)
            .map_err(|_| invalid_plan("The duplicate cleanup plan identifier is invalid."))?;
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("duplicate plan lock poisoned"))?;
        let plan = plans.get(&request.plan_id).cloned().ok_or_else(|| {
            let consumed = self
                .consumed_plans
                .lock()
                .map(|plans| plans.contains(&request.plan_id))
                .unwrap_or(false);
            if consumed {
                invalid_plan("This duplicate cleanup plan has already been used.")
            } else {
                CommandError::new(
                    ErrorCode::PlanExpired,
                    "This duplicate cleanup plan is unavailable or expired.",
                    true,
                )
            }
        })?;
        if Utc::now() >= plan.expires_at {
            plans.remove(&request.plan_id);
            return Err(CommandError::new(
                ErrorCode::PlanExpired,
                "This duplicate cleanup plan expired. Review the duplicate groups again.",
                true,
            ));
        }
        if request.confirmation_token != plan.confirmation_token {
            return Err(invalid_plan(
                "Duplicate cleanup confirmation did not match the reviewed plan.",
            ));
        }
        plans.remove(&request.plan_id);
        let mut consumed = self
            .consumed_plans
            .lock()
            .map_err(|_| CommandError::internal("consumed duplicate plan lock poisoned"))?;
        if consumed.len() >= 1024 {
            consumed.clear();
        }
        consumed.insert(request.plan_id.clone());
        Ok(plan)
    }

    fn run_cleanup(
        &self,
        app: AppHandle,
        operation_id: String,
        plan: DuplicateCleanupPlan,
        home: PathBuf,
        platform: &'static str,
        cancellation: CancellationToken,
    ) {
        let started_at = Utc::now();
        let disks_before = disk_info::list_disks().unwrap_or_default();
        let mut progress = CleanupProgress {
            operation_id: operation_id.clone(),
            total_items: plan.items.len() as u64,
            completed_items: 0,
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
            processed_bytes: 0,
            current_path: None,
        };
        let _ = app.emit("cleanup://started", &progress);
        let _ = app.emit("duplicates://cleanup-progress", &progress);
        let mut results = Vec::with_capacity(plan.items.len());
        let policy = ProtectedPathPolicy::for_platform(&home, platform);
        for item in &plan.items {
            progress.current_path = Some(display_path(&item.path, &home));
            let result = if cancellation.is_cancelled() {
                skipped(
                    item,
                    &home,
                    CommandError::new(
                        ErrorCode::ScanCancelled,
                        "Duplicate cleanup was cancelled before this item was moved.",
                        true,
                    ),
                )
            } else {
                match revalidate(item, &policy, &cancellation) {
                    Err(error) => skipped(item, &home, error),
                    Ok(()) => match trash_executor::move_to_trash(&item.path) {
                        Ok(()) => CleanupItemResult {
                            finding_id: item.copy_id.clone(),
                            rule_id: "duplicate.blake3-v1".to_owned(),
                            display_path: display_path(&item.path, &home),
                            expected_bytes: item.expected_size,
                            status: CleanupItemStatus::MovedToTrash,
                            error: None,
                        },
                        Err(error) => CleanupItemResult {
                            finding_id: item.copy_id.clone(),
                            rule_id: "duplicate.blake3-v1".to_owned(),
                            display_path: display_path(&item.path, &home),
                            expected_bytes: item.expected_size,
                            status: CleanupItemStatus::Failed,
                            error: Some(error),
                        },
                    },
                }
            };
            progress.completed_items += 1;
            progress.processed_bytes = progress.processed_bytes.saturating_add(item.expected_size);
            match result.status {
                CleanupItemStatus::MovedToTrash | CleanupItemStatus::PermanentlyDeleted => {
                    progress.success_count += 1
                }
                CleanupItemStatus::Skipped => progress.skipped_count += 1,
                CleanupItemStatus::Failed => progress.failure_count += 1,
            }
            let _ = app.emit("cleanup://item-completed", &result);
            let _ = app.emit("cleanup://progress", &progress);
            let _ = app.emit("duplicates://cleanup-progress", &progress);
            results.push(result);
        }
        progress.current_path = None;
        let disks = disk_info::list_disks().unwrap_or_default();
        let summary = CleanupSummary {
            operation_id: operation_id.clone(),
            plan_id: plan.id,
            started_at,
            completed_at: Utc::now(),
            action: CleanupAction::MoveToTrash,
            selected_count: plan.items.len() as u64,
            success_count: progress.success_count,
            failure_count: progress.failure_count,
            skipped_count: progress.skipped_count,
            expected_bytes: plan.expected_reclaimable_bytes,
            actual_free_space_change_bytes: free_space_change(&disks_before, &disks),
            cancelled: cancellation.is_cancelled(),
            items: results,
            disks,
        };
        if let Err(error) = self.history.append(&summary) {
            let _ = app.emit("cleanup://failed", &error);
            let _ = app.emit("duplicates://cleanup-failed", &error);
        }
        let _ = app.emit("cleanup://completed", &summary);
        let _ = app.emit("duplicates://cleanup-completed", &summary);
        self.clear_active(&operation_id);
    }

    fn clear_active(&self, operation_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            if active
                .as_ref()
                .is_some_and(|cleanup| cleanup.operation_id == operation_id)
            {
                *active = None;
            }
        }
    }
}

fn snapshot_file(
    path: &Path,
    expected_size: u64,
    expected_modified_at: Option<DateTime<Utc>>,
) -> Result<(PathBuf, Option<DateTime<Utc>>), CommandError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| file_error(path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != expected_size {
        return Err(invalid_plan(
            "A duplicate copy changed type or size after the scan. Scan again before cleanup.",
        ));
    }
    let modified = metadata.modified().ok().map(DateTime::<Utc>::from);
    if modified != expected_modified_at {
        return Err(invalid_plan(
            "A duplicate copy changed after the scan. Scan again before cleanup.",
        ));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| file_error(path, error))?;
    Ok((canonical, modified))
}

fn revalidate(
    item: &DuplicateCleanupPlanItem,
    policy: &ProtectedPathPolicy,
    cancellation: &CancellationToken,
) -> Result<(), CommandError> {
    let keep = snapshot_file(&item.keep_path, item.expected_size, item.keep_modified_at)?;
    let target = snapshot_file(&item.path, item.expected_size, item.expected_modified_at)?;
    if keep.0 != item.keep_canonical_path || target.0 != item.canonical_path {
        return Err(invalid_plan(
            "A duplicate path now resolves somewhere else. The item was skipped.",
        ));
    }
    if policy.check_cleanup_candidate(&keep.0, true).is_some()
        || policy.check_cleanup_candidate(&target.0, true).is_some()
    {
        return Err(CommandError::new(
            ErrorCode::PathProtected,
            "A duplicate path is protected and cannot be moved to Trash.",
            true,
        ));
    }
    let keep_hash = hashing::full_hash(&item.keep_path, cancellation)?.0;
    let target_hash = hashing::full_hash(&item.path, cancellation)?.0;
    if keep_hash != item.full_hash || target_hash != item.full_hash {
        return Err(invalid_plan(
            "The keep copy or selected duplicate no longer matches the reviewed content.",
        ));
    }
    if item.byte_for_byte_verified
        && !hashing::byte_for_byte_equal(&item.keep_path, &item.path, cancellation)?.0
    {
        return Err(invalid_plan(
            "Byte-for-byte verification no longer matches the reviewed duplicate group.",
        ));
    }
    let keep_after = snapshot_file(&item.keep_path, item.expected_size, item.keep_modified_at)?;
    let target_after = snapshot_file(&item.path, item.expected_size, item.expected_modified_at)?;
    if keep_after.0 != item.keep_canonical_path || target_after.0 != item.canonical_path {
        return Err(invalid_plan(
            "A duplicate path changed while it was being revalidated. The item was skipped.",
        ));
    }
    Ok(())
}

fn skipped(item: &DuplicateCleanupPlanItem, home: &Path, error: CommandError) -> CleanupItemResult {
    CleanupItemResult {
        finding_id: item.copy_id.clone(),
        rule_id: "duplicate.blake3-v1".to_owned(),
        display_path: display_path(&item.path, home),
        expected_bytes: item.expected_size,
        status: CleanupItemStatus::Skipped,
        error: Some(error),
    }
}

fn invalid_plan(message: &str) -> CommandError {
    CommandError::new(ErrorCode::PlanValidationFailed, message, true)
}

fn protected_error(path: &Path, reason: &str) -> CommandError {
    CommandError::new(
        ErrorCode::PathProtected,
        format!("This duplicate is inside a protected {reason} location."),
        true,
    )
    .with_path(path.to_string_lossy())
}

fn file_error(path: &Path, error: std::io::Error) -> CommandError {
    let code = match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::FilesystemError,
    };
    CommandError::new(
        code,
        "A duplicate cleanup path could not be accessed.",
        true,
    )
    .with_path(path.to_string_lossy())
    .with_details(error.to_string())
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| format!("~/{}", relative.to_string_lossy()))
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

fn free_space_change(
    before: &[crate::domain::disk::DiskInfo],
    after: &[crate::domain::disk::DiskInfo],
) -> Option<u64> {
    if before.is_empty() || before.len() != after.len() {
        return None;
    }
    let before_by_id: HashMap<_, _> = before
        .iter()
        .map(|disk| (&disk.id, disk.available_bytes))
        .collect();
    let mut before_total = 0_u64;
    let mut after_total = 0_u64;
    for disk in after {
        before_total = before_total.checked_add(*before_by_id.get(&disk.id)?)?;
        after_total = after_total.checked_add(disk.available_bytes)?;
    }
    Some(after_total.saturating_sub(before_total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::duplicate::{
        DuplicateCleanupSelection, DuplicateCopy, DuplicateGroup, DuplicateScanPhase,
        DuplicateSummary,
    };

    fn fixture_manager() -> (
        tempfile::TempDir,
        DuplicateCleanupManager,
        String,
        DuplicateGroup,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let repository = DuplicateRepository::new(directory.path().join("duplicates"));
        let scan_id = Uuid::new_v4().to_string();
        repository.initialize(&scan_id).unwrap();
        let root = directory.path().join("Downloads");
        fs::create_dir_all(&root).unwrap();
        let first = root.join("first");
        let second = root.join("second");
        fs::write(&first, b"same").unwrap();
        fs::write(&second, b"same").unwrap();
        let first_modified = fs::metadata(&first)
            .unwrap()
            .modified()
            .ok()
            .map(DateTime::<Utc>::from);
        let second_modified = fs::metadata(&second)
            .unwrap()
            .modified()
            .ok()
            .map(DateTime::<Utc>::from);
        let hash = hashing::full_hash(&first, &CancellationToken::default())
            .unwrap()
            .0;
        let group = DuplicateGroup {
            id: Uuid::new_v4().to_string(),
            scan_id: scan_id.clone(),
            file_size: 4,
            reclaimable_bytes: 4,
            copies: vec![
                DuplicateCopy {
                    id: Uuid::new_v4().to_string(),
                    path: first,
                    display_path: "first".to_owned(),
                    modified_at: first_modified,
                    owner: None,
                },
                DuplicateCopy {
                    id: Uuid::new_v4().to_string(),
                    path: second,
                    display_path: "second".to_owned(),
                    modified_at: second_modified,
                    owner: None,
                },
            ],
            recommended_keep_id: String::new(),
            keep_reason: "fixture".to_owned(),
            full_hash: hash,
            byte_for_byte_verified: true,
        };
        repository.append_group(&group).unwrap();
        repository
            .save_summary(&DuplicateSummary {
                scan_id: scan_id.clone(),
                phase: DuplicateScanPhase::Completed,
                roots: vec![root],
                minimum_size_bytes: 1,
                byte_for_byte_verification: true,
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                files_scanned: 2,
                candidate_files: 2,
                bytes_hashed: 8,
                groups_found: 1,
                duplicate_files: 2,
                reclaimable_bytes: 4,
                skipped_count: 0,
                permission_denied_count: 0,
                elapsed_ms: 1,
                errors: vec![],
            })
            .unwrap();
        let manager = DuplicateCleanupManager::new(
            repository,
            HistoryRepository::new(directory.path().join("history")),
        );
        (directory, manager, scan_id, group)
    }

    #[test]
    fn backend_rejects_a_plan_that_trashes_every_copy() {
        let (directory, manager, scan_id, group) = fixture_manager();
        let error = manager
            .create_plan(
                CreateDuplicateCleanupPlanRequest {
                    scan_id,
                    selections: vec![DuplicateCleanupSelection {
                        group_id: group.id,
                        keep_copy_id: group.copies[0].id.clone(),
                        trash_copy_ids: group.copies.iter().map(|copy| copy.id.clone()).collect(),
                    }],
                    action: CleanupAction::MoveToTrash,
                },
                directory.path(),
                "linux",
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PlanValidationFailed);
    }

    #[test]
    fn valid_plan_preserves_one_keep_copy() {
        let (directory, manager, scan_id, group) = fixture_manager();
        let plan = manager
            .create_plan(
                CreateDuplicateCleanupPlanRequest {
                    scan_id,
                    selections: vec![DuplicateCleanupSelection {
                        group_id: group.id,
                        keep_copy_id: group.copies[0].id.clone(),
                        trash_copy_ids: vec![group.copies[1].id.clone()],
                    }],
                    action: CleanupAction::MoveToTrash,
                },
                directory.path(),
                "linux",
            )
            .unwrap();
        assert_eq!(plan.kept_copy_count, 1);
        assert_eq!(plan.items.len(), 1);
        assert_ne!(plan.items[0].copy_id, plan.items[0].keep_copy_id);
    }
}
