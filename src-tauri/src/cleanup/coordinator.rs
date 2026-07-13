use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{
    domain::{
        cleanup::{
            CleanupAction, CleanupItemResult, CleanupItemStatus, CleanupPlan, CleanupProgress,
            CleanupSummary, CreateCleanupPlanRequest, ExecuteCleanupRequest, RiskSummary,
        },
        error::{CommandError, ErrorCode},
        rule::RiskLevel,
    },
    persistence::{history::HistoryRepository, scans::ScanRepository},
    platform::disk_info,
    rules::registry::RulesRegistry,
    scanner::cancellation::CancellationToken,
};

use super::{revalidation, trash_executor};

const PLAN_LIFETIME_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
struct ActiveCleanup {
    operation_id: String,
    cancellation: CancellationToken,
}

pub struct CleanupManager {
    active: Mutex<Option<ActiveCleanup>>,
    plans: Mutex<HashMap<String, CleanupPlan>>,
    consumed_plans: Mutex<HashSet<String>>,
    scan_repository: ScanRepository,
    history_repository: HistoryRepository,
    rules_registry: RulesRegistry,
}

impl CleanupManager {
    pub fn new(scan_repository: ScanRepository, history_repository: HistoryRepository) -> Self {
        Self {
            active: Mutex::new(None),
            plans: Mutex::new(HashMap::new()),
            consumed_plans: Mutex::new(HashSet::new()),
            scan_repository,
            history_repository,
            rules_registry: RulesRegistry,
        }
    }

    pub fn create_plan(
        &self,
        request: CreateCleanupPlanRequest,
        home: &Path,
        platform: &str,
    ) -> Result<CleanupPlan, CommandError> {
        if request.action != CleanupAction::MoveToTrash {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Permanent deletion is disabled. Move to Trash is the only cleanup action.",
                true,
            ));
        }
        let scan_summary = self.scan_repository.load_summary(&request.scan_id)?;
        if scan_summary.phase != crate::domain::scan::ScanPhase::Completed {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Cleanup plans can only be created from a completed scan.",
                true,
            ));
        }
        if request.finding_ids.is_empty() || request.finding_ids.len() > 500 {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Select between 1 and 500 findings for a cleanup plan.",
                true,
            ));
        }
        let unique: HashSet<_> = request.finding_ids.iter().collect();
        if unique.len() != request.finding_ids.len() {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "A cleanup plan cannot contain duplicate findings.",
                true,
            ));
        }

        let mut items = Vec::with_capacity(request.finding_ids.len());
        let mut risk_summary = RiskSummary::default();
        let mut expected_reclaimable_bytes = 0_u64;
        for finding_id in request.finding_ids {
            let finding = self
                .scan_repository
                .finding(&request.scan_id, &finding_id)?;
            let item =
                revalidation::snapshot_finding(&finding, home, platform, &self.rules_registry)?;
            match item.risk {
                RiskLevel::Safe => risk_summary.safe += 1,
                RiskLevel::Careful => risk_summary.careful += 1,
                RiskLevel::Expert => risk_summary.expert += 1,
            }
            expected_reclaimable_bytes =
                expected_reclaimable_bytes.saturating_add(item.expected_size);
            items.push(item);
        }

        let created_at = Utc::now();
        let plan = CleanupPlan {
            id: Uuid::new_v4().to_string(),
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_LIFETIME_MINUTES),
            action: CleanupAction::MoveToTrash,
            items,
            expected_reclaimable_bytes,
            risk_summary,
            confirmation_token: Uuid::new_v4().to_string(),
        };
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("cleanup plan lock poisoned"))?;
        plans.retain(|_, stored| Utc::now() < stored.expires_at);
        if plans.len() >= 128 {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Too many cleanup plans are awaiting review.",
                true,
            ));
        }
        plans.insert(plan.id.clone(), plan.clone());
        Ok(plan)
    }

    pub fn execute(
        self: &Arc<Self>,
        app: AppHandle,
        request: ExecuteCleanupRequest,
        home: PathBuf,
        platform: &'static str,
    ) -> Result<String, CommandError> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("cleanup lock poisoned"))?;
        if active.is_some() {
            return Err(CommandError::new(
                ErrorCode::CommandUnavailable,
                "Another cleanup operation is already running.",
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
            let worker_manager = Arc::clone(&manager);
            let blocking_operation_id = worker_operation_id.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                worker_manager.run_cleanup(
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
                tracing::error!(operation_id = %worker_operation_id, error = %error, "cleanup worker panicked");
                manager.clear_active(&worker_operation_id);
            }
        });
        Ok(operation_id)
    }

    pub fn cancel(&self, operation_id: &str) -> Result<(), CommandError> {
        let active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("cleanup lock poisoned"))?;
        let cleanup = active
            .as_ref()
            .filter(|cleanup| cleanup.operation_id == operation_id)
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PathNotFound,
                    "That cleanup operation is not running.",
                    true,
                )
            })?;
        cleanup.cancellation.cancel();
        Ok(())
    }

    pub fn history(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<CleanupSummary>, CommandError> {
        self.history_repository.list(offset, limit)
    }

    pub fn clear_history(&self) -> Result<(), CommandError> {
        self.history_repository.clear()
    }

    fn consume_plan(&self, request: &ExecuteCleanupRequest) -> Result<CleanupPlan, CommandError> {
        Uuid::parse_str(&request.plan_id).map_err(|_| {
            CommandError::new(
                ErrorCode::PlanValidationFailed,
                "The cleanup plan identifier is invalid.",
                true,
            )
        })?;
        let mut plans = self
            .plans
            .lock()
            .map_err(|_| CommandError::internal("cleanup plan lock poisoned"))?;
        let plan = plans.get(&request.plan_id).cloned().ok_or_else(|| {
            let consumed = self
                .consumed_plans
                .lock()
                .map(|plans| plans.contains(&request.plan_id))
                .unwrap_or(false);
            CommandError::new(
                if consumed {
                    ErrorCode::PlanValidationFailed
                } else {
                    ErrorCode::PlanExpired
                },
                if consumed {
                    "This cleanup plan has already been used."
                } else {
                    "This cleanup plan is unavailable or expired."
                },
                true,
            )
        })?;
        if Utc::now() >= plan.expires_at {
            plans.remove(&request.plan_id);
            return Err(CommandError::new(
                ErrorCode::PlanExpired,
                "This cleanup plan expired. Review the findings again.",
                true,
            ));
        }
        if request.confirmation_token != plan.confirmation_token {
            return Err(CommandError::new(
                ErrorCode::PlanValidationFailed,
                "Cleanup confirmation did not match the reviewed plan.",
                false,
            ));
        }
        plans.remove(&request.plan_id);
        let mut consumed = self
            .consumed_plans
            .lock()
            .map_err(|_| CommandError::internal("consumed plan lock poisoned"))?;
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
        plan: CleanupPlan,
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
        let mut results = Vec::with_capacity(plan.items.len());

        for item in &plan.items {
            progress.current_path = Some(display_path(&item.path, &home));
            let result = if cancellation.is_cancelled() {
                skipped_result(
                    item,
                    &home,
                    CommandError::new(
                        ErrorCode::ScanCancelled,
                        "Cleanup was cancelled before this item was moved.",
                        true,
                    ),
                )
            } else {
                match revalidation::revalidate_item(
                    item,
                    &home,
                    platform,
                    &self.rules_registry,
                    &cancellation,
                ) {
                    Err(error) => skipped_result(item, &home, error),
                    Ok(()) => match trash_executor::move_to_trash(&item.path) {
                        Ok(()) => CleanupItemResult {
                            finding_id: item.finding_id.clone(),
                            rule_id: item.rule_id.clone(),
                            display_path: display_path(&item.path, &home),
                            expected_bytes: item.expected_size,
                            status: CleanupItemStatus::MovedToTrash,
                            error: None,
                        },
                        Err(error) => CleanupItemResult {
                            finding_id: item.finding_id.clone(),
                            rule_id: item.rule_id.clone(),
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
                CleanupItemStatus::MovedToTrash => progress.success_count += 1,
                CleanupItemStatus::Failed => progress.failure_count += 1,
                CleanupItemStatus::Skipped => progress.skipped_count += 1,
            }
            let _ = app.emit("cleanup://item-completed", &result);
            results.push(result);
            let _ = app.emit("cleanup://progress", &progress);
        }

        progress.current_path = None;
        let disks = disk_info::list_disks().unwrap_or_default();
        let summary = CleanupSummary {
            operation_id: operation_id.clone(),
            plan_id: plan.id,
            started_at,
            completed_at: Utc::now(),
            action: plan.action,
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
        if let Err(error) = self.history_repository.append(&summary) {
            tracing::error!(operation_id = %operation_id, error = %error, "failed to persist cleanup history");
            let _ = app.emit("cleanup://failed", error);
        }
        let _ = app.emit("cleanup://completed", &summary);
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

fn skipped_result(
    item: &crate::domain::cleanup::CleanupPlanItem,
    home: &Path,
    error: CommandError,
) -> CleanupItemResult {
    CleanupItemResult {
        finding_id: item.finding_id.clone(),
        rule_id: item.rule_id.clone(),
        display_path: display_path(&item.path, home),
        expected_bytes: item.expected_size,
        status: CleanupItemStatus::Skipped,
        error: Some(error),
    }
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
    use crate::domain::{
        finding::{Finding, FindingEvidence, FindingType},
        rule::{RecommendedAction, RuleCategory},
        scan::{ScanPhase, ScanProfileId, ScanSummary},
    };
    use std::{fs, io::Write};

    fn manager() -> CleanupManager {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.keep();
        CleanupManager::new(
            ScanRepository::new(root.join("scans")),
            HistoryRepository::new(root.join("history.ndjson")),
        )
    }

    fn plan(expires_at: chrono::DateTime<Utc>) -> CleanupPlan {
        CleanupPlan {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            expires_at,
            action: CleanupAction::MoveToTrash,
            items: Vec::new(),
            expected_reclaimable_bytes: 0,
            risk_summary: RiskSummary::default(),
            confirmation_token: Uuid::new_v4().to_string(),
        }
    }

    #[test]
    fn creates_a_plan_only_from_persisted_safe_findings() {
        let directory = tempfile::tempdir().unwrap();
        let home = directory.path().join("home");
        let path = home.join(".npm/_cacache");
        fs::create_dir_all(&path).unwrap();
        fs::File::create(path.join("fixture"))
            .unwrap()
            .write_all(b"cache")
            .unwrap();
        let scan_repository = ScanRepository::new(directory.path().join("scans"));
        let scan_id = Uuid::new_v4().to_string();
        scan_repository.initialize(&scan_id).unwrap();
        scan_repository
            .save_summary(&ScanSummary {
                scan_id: scan_id.clone(),
                profile: ScanProfileId::Quick,
                phase: ScanPhase::Completed,
                started_at: Utc::now().to_rfc3339(),
                completed_at: Some(Utc::now().to_rfc3339()),
                files_scanned: 1,
                directories_scanned: 1,
                bytes_examined: 5,
                findings_count: 1,
                reclaimable_bytes: 5,
                skipped_count: 0,
                permission_denied_count: 0,
                elapsed_ms: 1,
                errors: Vec::new(),
            })
            .unwrap();
        let finding_id = Uuid::new_v4().to_string();
        scan_repository
            .append_finding(&Finding {
                id: finding_id.clone(),
                scan_id: scan_id.clone(),
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
            })
            .unwrap();
        let manager = CleanupManager::new(
            scan_repository,
            HistoryRepository::new(directory.path().join("history.ndjson")),
        );
        let plan = manager
            .create_plan(
                CreateCleanupPlanRequest {
                    scan_id,
                    finding_ids: vec![finding_id],
                    action: CleanupAction::MoveToTrash,
                },
                &home,
                "linux",
            )
            .unwrap();
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.risk_summary.safe, 1);
        assert_eq!(plan.items[0].canonical_path, path.canonicalize().unwrap());
    }

    #[test]
    fn cannot_execute_outside_a_backend_plan() {
        let manager = manager();
        let request = ExecuteCleanupRequest {
            plan_id: Uuid::new_v4().to_string(),
            confirmation_token: Uuid::new_v4().to_string(),
        };
        assert!(manager.consume_plan(&request).is_err());
    }

    #[test]
    fn permanent_delete_cannot_be_planned() {
        let manager = manager();
        let error = manager
            .create_plan(
                CreateCleanupPlanRequest {
                    scan_id: Uuid::new_v4().to_string(),
                    finding_ids: Vec::new(),
                    action: CleanupAction::PermanentDelete,
                },
                Path::new("/home/alex"),
                "linux",
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::CommandUnavailable);
    }

    #[test]
    fn cannot_reuse_a_consumed_plan() {
        let manager = manager();
        let plan = plan(Utc::now() + Duration::minutes(1));
        manager
            .plans
            .lock()
            .unwrap()
            .insert(plan.id.clone(), plan.clone());
        let request = ExecuteCleanupRequest {
            plan_id: plan.id,
            confirmation_token: plan.confirmation_token,
        };
        manager.consume_plan(&request).unwrap();
        assert_eq!(
            manager.consume_plan(&request).unwrap_err().code,
            ErrorCode::PlanValidationFailed
        );
    }

    #[test]
    fn cannot_reuse_an_expired_plan() {
        let manager = manager();
        let plan = plan(Utc::now() - Duration::seconds(1));
        manager
            .plans
            .lock()
            .unwrap()
            .insert(plan.id.clone(), plan.clone());
        let request = ExecuteCleanupRequest {
            plan_id: plan.id,
            confirmation_token: plan.confirmation_token,
        };
        assert_eq!(
            manager.consume_plan(&request).unwrap_err().code,
            ErrorCode::PlanExpired
        );
    }
}
