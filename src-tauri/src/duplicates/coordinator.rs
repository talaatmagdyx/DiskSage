use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::Utc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{
    domain::{
        duplicate::{
            DuplicateProgress, DuplicateScanPhase, DuplicateSummary, StartDuplicateScanRequest,
        },
        error::{CommandError, ErrorCode},
    },
    persistence::{duplicates::DuplicateRepository, history::HistoryRepository},
    scanner::cancellation::CancellationToken,
};

use super::{cleanup::DuplicateCleanupManager, scanner};

#[derive(Debug, Clone)]
struct ActiveScan {
    scan_id: String,
    cancellation: CancellationToken,
}

pub struct DuplicateManager {
    active: Mutex<Option<ActiveScan>>,
    repository: DuplicateRepository,
    cleanup: Arc<DuplicateCleanupManager>,
}

impl DuplicateManager {
    pub fn new(repository: DuplicateRepository, history: HistoryRepository) -> Self {
        Self {
            active: Mutex::new(None),
            cleanup: Arc::new(DuplicateCleanupManager::new(repository.clone(), history)),
            repository,
        }
    }

    pub fn start(
        self: &Arc<Self>,
        app: AppHandle,
        request: StartDuplicateScanRequest,
        home: PathBuf,
        platform: &'static str,
    ) -> Result<String, CommandError> {
        if request.minimum_size_bytes == 0 || request.minimum_size_bytes > 1_099_511_627_776 {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Duplicate minimum size must be between 1 byte and 1 TiB.",
                true,
            ));
        }
        let roots = scanner::validate_roots(&request.roots, &home, platform)?;
        let mut active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("duplicate scan lock poisoned"))?;
        if active.is_some() {
            return Err(CommandError::new(
                ErrorCode::ScanAlreadyRunning,
                "Another duplicate scan is already running.",
                true,
            ));
        }
        let scan_id = Uuid::new_v4().to_string();
        self.repository.initialize(&scan_id)?;
        let cancellation = CancellationToken::default();
        *active = Some(ActiveScan {
            scan_id: scan_id.clone(),
            cancellation: cancellation.clone(),
        });
        drop(active);

        let initial = initial_summary(&scan_id, &roots, &request);
        self.repository.save_summary(&initial)?;
        let manager = Arc::clone(self);
        let worker_scan_id = scan_id.clone();
        tauri::async_runtime::spawn(async move {
            let blocking_scan_id = worker_scan_id.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                manager.run_scan(app, blocking_scan_id, roots, request, home, cancellation)
            })
            .await;
            if let Err(error) = result {
                tracing::error!(scan_id = %worker_scan_id, error = %error, "duplicate worker panicked");
            }
        });
        Ok(scan_id)
    }

    pub fn cancel(&self, scan_id: &str) -> Result<(), CommandError> {
        let active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("duplicate scan lock poisoned"))?;
        let scan = active
            .as_ref()
            .filter(|scan| scan.scan_id == scan_id)
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PathNotFound,
                    "That duplicate scan is not running.",
                    true,
                )
            })?;
        scan.cancellation.cancel();
        Ok(())
    }

    pub fn status(&self, scan_id: &str) -> Result<DuplicateSummary, CommandError> {
        self.repository.load_summary(scan_id)
    }

    pub fn repository(&self) -> &DuplicateRepository {
        &self.repository
    }

    pub fn cleanup(&self) -> &Arc<DuplicateCleanupManager> {
        &self.cleanup
    }

    fn run_scan(
        &self,
        app: AppHandle,
        scan_id: String,
        roots: Vec<PathBuf>,
        request: StartDuplicateScanRequest,
        home: PathBuf,
        cancellation: CancellationToken,
    ) {
        let started_at = Utc::now();
        let mut latest_progress: Option<DuplicateProgress> = None;
        let output = scanner::run(
            &scan_id,
            roots.clone(),
            request.minimum_size_bytes,
            request.byte_for_byte_verification,
            &home,
            &cancellation,
            |progress| {
                latest_progress = Some(progress.clone());
                let _ = app.emit("duplicates://progress", progress);
            },
        );
        match output {
            Ok(output) => {
                for group in &output.groups {
                    if let Err(error) = self.repository.append_group(group) {
                        let _ = app.emit("duplicates://failed", &error);
                        self.persist_failure(&scan_id, roots.clone(), &request, started_at, error);
                        self.clear_active(&scan_id);
                        return;
                    }
                    let _ = app.emit("duplicates://group", group);
                }
                if let Err(error) = self.repository.save_summary(&output.summary) {
                    let _ = app.emit("duplicates://failed", &error);
                } else {
                    let completed_progress = progress_from_summary(&output.summary);
                    let _ = app.emit("duplicates://progress", &completed_progress);
                    let _ = app.emit("duplicates://completed", &output.summary);
                }
            }
            Err(error) if error.code == ErrorCode::ScanCancelled => {
                let latest = latest_progress.unwrap_or(DuplicateProgress {
                    scan_id: scan_id.clone(),
                    phase: DuplicateScanPhase::Cancelled,
                    current_path: None,
                    files_scanned: 0,
                    candidate_files: 0,
                    bytes_hashed: 0,
                    groups_found: 0,
                    reclaimable_bytes: 0,
                    skipped_count: 0,
                    permission_denied_count: 0,
                    elapsed_ms: 0,
                });
                let summary = DuplicateSummary {
                    scan_id: scan_id.clone(),
                    phase: DuplicateScanPhase::Cancelled,
                    roots,
                    minimum_size_bytes: request.minimum_size_bytes,
                    byte_for_byte_verification: request.byte_for_byte_verification,
                    started_at,
                    completed_at: Some(Utc::now()),
                    files_scanned: latest.files_scanned,
                    candidate_files: latest.candidate_files,
                    bytes_hashed: latest.bytes_hashed,
                    groups_found: latest.groups_found,
                    duplicate_files: 0,
                    reclaimable_bytes: latest.reclaimable_bytes,
                    skipped_count: latest.skipped_count,
                    permission_denied_count: latest.permission_denied_count,
                    elapsed_ms: latest.elapsed_ms,
                    errors: Vec::new(),
                };
                let _ = self.repository.save_summary(&summary);
                let _ = app.emit("duplicates://completed", &summary);
            }
            Err(error) => {
                self.persist_failure(&scan_id, roots, &request, started_at, error.clone());
                let _ = app.emit("duplicates://failed", &error);
            }
        }
        self.clear_active(&scan_id);
    }

    fn persist_failure(
        &self,
        scan_id: &str,
        roots: Vec<PathBuf>,
        request: &StartDuplicateScanRequest,
        started_at: chrono::DateTime<Utc>,
        error: CommandError,
    ) {
        let summary = DuplicateSummary {
            scan_id: scan_id.to_owned(),
            phase: DuplicateScanPhase::Failed,
            roots,
            minimum_size_bytes: request.minimum_size_bytes,
            byte_for_byte_verification: request.byte_for_byte_verification,
            started_at,
            completed_at: Some(Utc::now()),
            files_scanned: 0,
            candidate_files: 0,
            bytes_hashed: 0,
            groups_found: 0,
            duplicate_files: 0,
            reclaimable_bytes: 0,
            skipped_count: 0,
            permission_denied_count: 0,
            elapsed_ms: 0,
            errors: vec![error],
        };
        let _ = self.repository.save_summary(&summary);
    }

    fn clear_active(&self, scan_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            if active.as_ref().is_some_and(|scan| scan.scan_id == scan_id) {
                *active = None;
            }
        }
    }
}

fn initial_summary(
    scan_id: &str,
    roots: &[PathBuf],
    request: &StartDuplicateScanRequest,
) -> DuplicateSummary {
    DuplicateSummary {
        scan_id: scan_id.to_owned(),
        phase: DuplicateScanPhase::Discovering,
        roots: roots.to_vec(),
        minimum_size_bytes: request.minimum_size_bytes,
        byte_for_byte_verification: request.byte_for_byte_verification,
        started_at: Utc::now(),
        completed_at: None,
        files_scanned: 0,
        candidate_files: 0,
        bytes_hashed: 0,
        groups_found: 0,
        duplicate_files: 0,
        reclaimable_bytes: 0,
        skipped_count: 0,
        permission_denied_count: 0,
        elapsed_ms: 0,
        errors: Vec::new(),
    }
}

fn progress_from_summary(summary: &DuplicateSummary) -> DuplicateProgress {
    DuplicateProgress {
        scan_id: summary.scan_id.clone(),
        phase: summary.phase,
        current_path: None,
        files_scanned: summary.files_scanned,
        candidate_files: summary.candidate_files,
        bytes_hashed: summary.bytes_hashed,
        groups_found: summary.groups_found,
        reclaimable_bytes: summary.reclaimable_bytes,
        skipped_count: summary.skipped_count,
        permission_denied_count: summary.permission_denied_count,
        elapsed_ms: summary.elapsed_ms,
    }
}
