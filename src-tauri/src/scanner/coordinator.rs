use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::{
    domain::{
        error::{CommandError, ErrorCode},
        finding::{Finding, FindingEvidence, FindingType, GuidedAction},
        rule::RuleCategory,
        scan::{
            ScanPhase, ScanProfile, ScanProfileId, ScanProgress, ScanSummary, StartScanRequest,
        },
    },
    persistence::scans::ScanRepository,
    rules::{registry::ResolvedRule, registry::RulesRegistry},
    safety::protected_paths::ProtectedPathPolicy,
};

use super::{
    analysis::{self, AnalysisConfig},
    cancellation::CancellationToken,
    exclusions::ExclusionMatcher,
    walker::{measure_target, TargetMeasurement},
};

#[derive(Debug, Clone)]
struct ActiveScan {
    id: String,
    cancellation: CancellationToken,
}

struct ScanScope {
    home: PathBuf,
    platform: &'static str,
    project_roots: Vec<PathBuf>,
    analysis: Option<AnalysisConfig>,
}

pub struct ScanManager {
    active: Mutex<Option<ActiveScan>>,
    statuses: Mutex<HashMap<String, ScanSummary>>,
    repository: ScanRepository,
    rules_registry: RulesRegistry,
}

impl ScanManager {
    pub fn new(repository: ScanRepository) -> Self {
        Self {
            active: Mutex::new(None),
            statuses: Mutex::new(HashMap::new()),
            repository,
            rules_registry: RulesRegistry,
        }
    }

    pub fn profiles() -> Vec<ScanProfile> {
        vec![
            profile(ScanProfileId::Quick, "Quick Scan", "Common low-risk caches", "Usually under 30 seconds", true, None),
            profile(ScanProfileId::Developer, "Developer Scan", "Package caches, configured projects, IDEs, Docker, and emulators", "Usually under 2 minutes", true, None),
            profile(
                ScanProfileId::FullAnalysis,
                "Full Analysis",
                "Known rules, project artifacts, large files, and old installers",
                "Can take significant time",
                true,
                Some("Full Analysis inspects configured project roots and can take significant time. Duplicate content remains in the dedicated Duplicates workflow."),
            ),
            profile(
                ScanProfileId::Custom,
                "Custom Scan",
                "User-selected roots and rule categories",
                "Depends on selected roots",
                true,
                Some("Custom analysis is read-only: large files and old installers are review suggestions."),
            ),
        ]
    }

    pub fn start(
        self: &Arc<Self>,
        app: AppHandle,
        request: StartScanRequest,
        home: PathBuf,
        platform: &'static str,
        settings: crate::domain::settings::AppSettings,
    ) -> Result<String, CommandError> {
        let project_roots: Vec<PathBuf> =
            settings.project_roots.iter().map(PathBuf::from).collect();
        let analysis = match request.profile {
            ScanProfileId::Custom => {
                let options = request.custom.as_ref().ok_or_else(|| {
                    CommandError::new(
                        ErrorCode::InvalidSettings,
                        "Custom Scan requires folders and analysis options.",
                        true,
                    )
                })?;
                Some(analysis::validate_custom_options(
                    options,
                    &home,
                    platform,
                    settings.large_file_threshold_bytes,
                    settings.very_large_file_threshold_bytes,
                    settings.huge_file_threshold_bytes,
                    settings.old_file_threshold_days,
                )?)
            }
            ScanProfileId::FullAnalysis => {
                if project_roots.is_empty() {
                    return Err(CommandError::new(
                        ErrorCode::InvalidSettings,
                        "Configure at least one project root before Full Analysis.",
                        true,
                    ));
                }
                let roots = project_roots
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                Some(analysis::validate_custom_options(
                    &crate::domain::scan::CustomScanOptions {
                        roots,
                        enabled_categories: vec![RuleCategory::LargeFile, RuleCategory::OldFile],
                        minimum_file_size_bytes: 0,
                        maximum_depth: 32,
                        include_hidden_files: settings.scan_hidden_files,
                        include_external_drives: settings.scan_external_drives,
                    },
                    &home,
                    platform,
                    settings.large_file_threshold_bytes,
                    settings.very_large_file_threshold_bytes,
                    settings.huge_file_threshold_bytes,
                    settings.old_file_threshold_days,
                )?)
            }
            _ => {
                if request.custom.is_some() {
                    return Err(CommandError::new(
                        ErrorCode::InvalidSettings,
                        "Custom options are accepted only by Custom Scan.",
                        true,
                    ));
                }
                None
            }
        };
        let exclusions = ExclusionMatcher::new(&request.excluded_paths)?;
        let mut active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("scan lock poisoned"))?;
        if active.is_some() {
            return Err(CommandError::new(
                ErrorCode::ScanAlreadyRunning,
                "Another disk scan is already running.",
                true,
            ));
        }

        let scan_id = Uuid::new_v4().to_string();
        self.repository.initialize(&scan_id)?;
        let summary = initial_summary(&scan_id, request.profile);
        self.repository.save_summary(&summary)?;
        self.statuses
            .lock()
            .map_err(|_| CommandError::internal("scan status lock poisoned"))?
            .insert(scan_id.clone(), summary.clone());

        let cancellation = CancellationToken::default();
        *active = Some(ActiveScan {
            id: scan_id.clone(),
            cancellation: cancellation.clone(),
        });
        drop(active);

        let manager = Arc::clone(self);
        let worker_scan_id = scan_id.clone();
        let scope = ScanScope {
            home,
            platform,
            project_roots,
            analysis,
        };
        tauri::async_runtime::spawn(async move {
            let worker_manager = Arc::clone(&manager);
            let result = tauri::async_runtime::spawn_blocking(move || {
                worker_manager.run_scan(app, summary, scope, exclusions, cancellation)
            })
            .await;
            if let Err(error) = result {
                tracing::error!(scan_id = %worker_scan_id, error = %error, "scan worker panicked");
                manager.fail_worker(&worker_scan_id, error.to_string());
            }
        });
        Ok(scan_id)
    }

    pub fn cancel(&self, scan_id: &str) -> Result<(), CommandError> {
        let active = self
            .active
            .lock()
            .map_err(|_| CommandError::internal("scan lock poisoned"))?;
        let scan = active
            .as_ref()
            .filter(|active| active.id == scan_id)
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::PathNotFound,
                    "That scan is not currently running.",
                    true,
                )
            })?;
        scan.cancellation.cancel();
        Ok(())
    }

    pub fn status(&self, scan_id: &str) -> Result<ScanSummary, CommandError> {
        if let Some(summary) = self
            .statuses
            .lock()
            .map_err(|_| CommandError::internal("scan status lock poisoned"))?
            .get(scan_id)
            .cloned()
        {
            return Ok(summary);
        }
        self.repository.load_summary(scan_id)
    }

    pub fn latest_status(&self) -> Result<Option<ScanSummary>, CommandError> {
        let statuses = self
            .statuses
            .lock()
            .map_err(|_| CommandError::internal("scan status lock poisoned"))?;
        Ok(statuses
            .values()
            .max_by(|left, right| left.started_at.cmp(&right.started_at))
            .cloned())
    }

    pub fn repository(&self) -> &ScanRepository {
        &self.repository
    }

    fn run_scan(
        &self,
        app: AppHandle,
        mut summary: ScanSummary,
        scope: ScanScope,
        exclusions: ExclusionMatcher,
        cancellation: CancellationToken,
    ) {
        let ScanScope {
            home,
            platform,
            project_roots,
            analysis,
        } = scope;
        let started = Instant::now();
        let _ = app.emit("scan://started", &summary);
        summary.phase = ScanPhase::DiscoveringTargets;
        self.publish_status(&summary);
        let rules =
            self.rules_registry
                .rules_for_scan(summary.profile, &home, platform, &project_roots);
        let _ = app.emit("scan://progress", progress_from(&summary, None));
        summary.phase = ScanPhase::Scanning;
        let mut last_progress = Instant::now() - Duration::from_secs(1);

        for rule in rules {
            if cancellation.is_cancelled() {
                break;
            }
            let baseline_files = summary.files_scanned;
            let baseline_directories = summary.directories_scanned;
            let baseline_bytes = summary.bytes_examined;
            let mut current_path: Option<String> = None;
            let rule_exclusions = exclusions.with_additional_paths(&rule.excluded_targets);
            let measurement = measure_target(
                &rule.target,
                &rule_exclusions,
                &cancellation,
                |path, measured| {
                    current_path = Some(display_path(path, &home));
                    if last_progress.elapsed() >= Duration::from_millis(150) {
                        let mut progress = progress_from(&summary, current_path.clone());
                        progress.files_scanned = baseline_files + measured.files_scanned;
                        progress.directories_scanned =
                            baseline_directories + measured.directories_scanned;
                        progress.bytes_examined = baseline_bytes + measured.logical_size;
                        progress.elapsed_ms = started.elapsed().as_millis() as u64;
                        let _ = app.emit("scan://progress", &progress);
                        last_progress = Instant::now();
                    }
                },
            );
            merge_measurement(&mut summary, &measurement);
            summary.elapsed_ms = started.elapsed().as_millis() as u64;

            if measurement.exists
                && measurement.logical_size >= rule.definition.minimum_size.unwrap_or_default()
                && !cancellation.is_cancelled()
            {
                let finding = build_finding(&summary.scan_id, &home, platform, rule, &measurement);
                match self.repository.append_finding(&finding) {
                    Ok(()) => {
                        summary.findings_count += 1;
                        if finding.cleanup_allowed {
                            summary.reclaimable_bytes = summary
                                .reclaimable_bytes
                                .saturating_add(measurement.allocated_size);
                        }
                        let _ = app.emit("scan://finding", &finding);
                    }
                    Err(error) => {
                        summary.errors.push(error);
                        summary.phase = ScanPhase::Failed;
                        break;
                    }
                }
            }
            self.publish_status(&summary);
        }

        if let Some(config) = analysis.filter(|_| !cancellation.is_cancelled()) {
            summary.phase = ScanPhase::Analyzing;
            self.publish_status(&summary);
            let baseline_files = summary.files_scanned;
            let baseline_directories = summary.directories_scanned;
            let baseline_bytes = summary.bytes_examined;
            let finding_count = std::cell::Cell::new(summary.findings_count);
            let reclaimable_bytes = std::cell::Cell::new(summary.reclaimable_bytes);
            let mut analysis_last_progress = Instant::now() - Duration::from_secs(1);
            let result = analysis::analyze(
                &summary.scan_id,
                &home,
                &config,
                &exclusions,
                &cancellation,
                |path, measured| {
                    if analysis_last_progress.elapsed() >= Duration::from_millis(150) {
                        let progress = ScanProgress {
                            scan_id: summary.scan_id.clone(),
                            phase: ScanPhase::Analyzing,
                            current_path: Some(display_path(path, &home)),
                            files_scanned: baseline_files + measured.files_scanned,
                            directories_scanned: baseline_directories
                                + measured.directories_scanned,
                            bytes_examined: baseline_bytes + measured.bytes_examined,
                            findings_count: finding_count.get(),
                            reclaimable_bytes: reclaimable_bytes.get(),
                            skipped_count: summary.skipped_count + measured.skipped_count,
                            permission_denied_count: summary.permission_denied_count
                                + measured.permission_denied_count,
                            elapsed_ms: started.elapsed().as_millis() as u64,
                        };
                        let _ = app.emit("scan://progress", &progress);
                        analysis_last_progress = Instant::now();
                    }
                },
                |finding| {
                    self.repository.append_finding(&finding)?;
                    finding_count.set(finding_count.get() + 1);
                    if finding.cleanup_allowed {
                        reclaimable_bytes.set(reclaimable_bytes.get().saturating_add(
                            finding.allocated_size.unwrap_or(finding.logical_size),
                        ));
                    }
                    let _ = app.emit("scan://finding", &finding);
                    Ok(())
                },
            );
            match result {
                Ok(measured) => {
                    summary.files_scanned = baseline_files + measured.files_scanned;
                    summary.directories_scanned =
                        baseline_directories + measured.directories_scanned;
                    summary.bytes_examined = baseline_bytes + measured.bytes_examined;
                    summary.skipped_count += measured.skipped_count;
                    summary.permission_denied_count += measured.permission_denied_count;
                    summary.errors.extend(measured.errors);
                    summary.errors.truncate(50);
                    summary.findings_count = finding_count.get();
                    summary.reclaimable_bytes = reclaimable_bytes.get();
                }
                Err(error) => {
                    summary.errors.push(error);
                    summary.phase = ScanPhase::Failed;
                }
            }
        }

        summary.elapsed_ms = started.elapsed().as_millis() as u64;
        summary.completed_at = Some(Utc::now().to_rfc3339());
        summary.phase = if cancellation.is_cancelled() {
            ScanPhase::Cancelled
        } else if summary.phase == ScanPhase::Failed {
            ScanPhase::Failed
        } else {
            ScanPhase::Completed
        };
        self.publish_status(&summary);
        if let Err(error) = self.repository.save_summary(&summary) {
            tracing::error!(scan_id = %summary.scan_id, error = %error, "failed to persist scan summary");
        }
        let event = match summary.phase {
            ScanPhase::Cancelled => "scan://cancelled",
            ScanPhase::Failed => "scan://failed",
            _ => "scan://completed",
        };
        let _ = app.emit(event, &summary);
        let _ = app.emit("scan://progress", progress_from(&summary, None));
        self.clear_active(&summary.scan_id);
    }

    fn publish_status(&self, summary: &ScanSummary) {
        if let Ok(mut statuses) = self.statuses.lock() {
            statuses.insert(summary.scan_id.clone(), summary.clone());
        }
    }

    fn clear_active(&self, scan_id: &str) {
        if let Ok(mut active) = self.active.lock() {
            if active.as_ref().is_some_and(|active| active.id == scan_id) {
                *active = None;
            }
        }
    }

    fn fail_worker(&self, scan_id: &str, details: String) {
        if let Ok(mut statuses) = self.statuses.lock() {
            if let Some(summary) = statuses.get_mut(scan_id) {
                summary.phase = ScanPhase::Failed;
                summary.completed_at = Some(Utc::now().to_rfc3339());
                summary.errors.push(CommandError::internal(details));
                let _ = self.repository.save_summary(summary);
            }
        }
        self.clear_active(scan_id);
    }
}

fn profile(
    id: ScanProfileId,
    display_name: &str,
    description: &str,
    expected_duration: &str,
    available: bool,
    warning: Option<&str>,
) -> ScanProfile {
    ScanProfile {
        id,
        display_name: display_name.to_owned(),
        description: description.to_owned(),
        expected_duration: expected_duration.to_owned(),
        available,
        warning: warning.map(str::to_owned),
    }
}

fn initial_summary(scan_id: &str, profile: ScanProfileId) -> ScanSummary {
    ScanSummary {
        scan_id: scan_id.to_owned(),
        profile,
        phase: ScanPhase::Preparing,
        started_at: Utc::now().to_rfc3339(),
        completed_at: None,
        files_scanned: 0,
        directories_scanned: 0,
        bytes_examined: 0,
        findings_count: 0,
        reclaimable_bytes: 0,
        skipped_count: 0,
        permission_denied_count: 0,
        elapsed_ms: 0,
        errors: Vec::new(),
    }
}

fn merge_measurement(summary: &mut ScanSummary, measurement: &TargetMeasurement) {
    summary.files_scanned += measurement.files_scanned;
    summary.directories_scanned += measurement.directories_scanned;
    summary.bytes_examined = summary
        .bytes_examined
        .saturating_add(measurement.logical_size);
    summary.skipped_count += measurement.skipped_count;
    summary.permission_denied_count += measurement.permission_denied_count;
    summary.errors.extend(measurement.errors.iter().cloned());
    summary.errors.truncate(50);
}

fn build_finding(
    scan_id: &str,
    home: &Path,
    platform: &str,
    rule: ResolvedRule,
    measurement: &TargetMeasurement,
) -> Finding {
    let evidence = match rule.definition.category {
        RuleCategory::PackageManagerCache => FindingEvidence::PackageManagerCache {
            manager: rule.definition.display_name.clone(),
        },
        RuleCategory::BuildArtifact | RuleCategory::Log => FindingEvidence::DirectoryNameMatch {
            directory_name: rule
                .target
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| rule.definition.display_name.clone()),
        },
        _ => FindingEvidence::KnownPath,
    };
    let policy_block_reason = ProtectedPathPolicy::for_platform(home, platform)
        .check_cleanup_candidate(&rule.target, true)
        .map(|reason| format!("Protected by the {} policy.", reason.reason));
    let cleanup_allowed = policy_block_reason.is_none()
        && matches!(
            rule.definition.risk,
            crate::domain::rule::RiskLevel::Safe | crate::domain::rule::RiskLevel::Careful
        )
        && rule.definition.recommended_action
            == crate::domain::rule::RecommendedAction::MoveToTrash;
    let cleanup_block_reason = if cleanup_allowed {
        None
    } else if rule.definition.recommended_action
        == crate::domain::rule::RecommendedAction::GuidedCommand
    {
        Some(format!(
            "{} finding: use the guided owner command; direct filesystem cleanup is unavailable.",
            match rule.definition.risk {
                crate::domain::rule::RiskLevel::Careful => "Careful",
                crate::domain::rule::RiskLevel::Expert => "Expert",
                crate::domain::rule::RiskLevel::Safe => "Safe",
            }
        ))
    } else if rule.definition.risk != crate::domain::rule::RiskLevel::Safe {
        Some(format!(
            "{} finding: review-only; direct cleanup is unavailable.",
            match rule.definition.risk {
                crate::domain::rule::RiskLevel::Careful => "Careful",
                crate::domain::rule::RiskLevel::Expert => "Expert",
                crate::domain::rule::RiskLevel::Safe => "Safe",
            }
        ))
    } else {
        policy_block_reason
    };
    let item_type = std::fs::symlink_metadata(&rule.target)
        .ok()
        .map(|metadata| {
            if metadata.is_file() {
                FindingType::File
            } else {
                FindingType::Directory
            }
        })
        .unwrap_or(FindingType::Directory);
    let guided_action = guided_action(&rule.definition.id);
    Finding {
        id: Uuid::new_v4().to_string(),
        scan_id: scan_id.to_owned(),
        rule_id: rule.definition.id,
        rule_version: rule.definition.version,
        category: rule.definition.category,
        display_name: rule.definition.display_name,
        description: rule.definition.description,
        path: rule.target.clone(),
        display_path: display_path(&rule.target, home),
        item_type,
        logical_size: measurement.logical_size,
        allocated_size: Some(measurement.allocated_size),
        modified_at: measurement.modified_at.map(DateTime::<Utc>::from),
        risk: rule.definition.risk,
        recommended_action: rule.definition.recommended_action,
        evidence,
        cleanup_allowed,
        cleanup_block_reason,
        guided_action,
    }
}

fn guided_action(rule_id: &str) -> Option<GuidedAction> {
    let (title, command, explanation) = match rule_id {
        "inspection.docker.raw-v1" => (
            "Inspect Docker usage",
            "docker system df -v",
            "Review Docker-owned images, containers, volumes, and build cache before using Docker's own cleanup commands.",
        ),
        "inspection.cursor.state-db-v1" => (
            "Compact Cursor's database",
            "sqlite3 \"$HOME/Library/Application Support/Cursor/User/globalStorage/state.vscdb\" \"VACUUM;\"",
            "Quit Cursor first. VACUUM modifies the database in place and can require substantial temporary free space.",
        ),
        "inspection.android.avd-v1" => (
            "List Android virtual devices",
            "avdmanager list avd",
            "Remove unused devices through Android Studio's Device Manager after reviewing this inventory.",
        ),
        "inspection.apple.simulators-v1" => (
            "List Apple simulators",
            "xcrun simctl list devices",
            "Use Xcode or simctl to remove only simulator devices you no longer need.",
        ),
        "inspection.codex.runtimes-v1" => (
            "Inspect Codex runtimes",
            "du -sh \"$HOME/.cache/codex-runtimes\"/*",
            "Quit Codex before changing managed runtimes; an active task may depend on them.",
        ),
        "inspection.playwright.browsers-v1" => (
            "List Playwright browsers",
            "npx playwright install --list",
            "Review installed browsers and persistent MCP profiles before running Playwright's uninstall command.",
        ),
        "inspection.claude.sessions-v1" => (
            "Inspect Claude sessions",
            "du -sh \"$HOME/Library/Application Support/Claude/local-agent-mode-sessions\"/*",
            "Quit Claude and identify the active session before removing anything with Claude's own tools.",
        ),
        "inspection.claude.vm-bundles-v1" => (
            "Inspect Claude VM bundles",
            "du -sh \"$HOME/Library/Application Support/Claude/vm_bundles\"/*",
            "These bundles power Claude's sandbox. Removing an active bundle can break running tasks.",
        ),
        "inspection.runtime.rbenv-v1" => (
            "List rbenv runtimes",
            "rbenv versions",
            "Check project version files before uninstalling a runtime with rbenv.",
        ),
        "inspection.runtime.asdf-v1" => (
            "List asdf runtimes",
            "asdf list",
            "Check project tool-version files before uninstalling a runtime with asdf.",
        ),
        "inspection.runtime.mise-v1" => (
            "Preview mise pruning",
            "mise prune --dry-run",
            "The dry run identifies unused managed runtimes without deleting them.",
        ),
        "inspection.kubernetes.minikube-v1" => (
            "List minikube profiles",
            "minikube profile list",
            "Inspect profile status and persistent workloads before using minikube delete.",
        ),
        "inspection.ml.ollama-v1" => (
            "List Ollama models",
            "ollama list",
            "Remove individual models intentionally with ollama rm instead of deleting the model store.",
        ),
        "inspection.ml.huggingface-v1" => (
            "Inspect Hugging Face storage",
            "du -sh \"$HOME/.cache/huggingface\"/*",
            "Distinguish downloadable models from private datasets and local checkpoints before cleanup.",
        ),
        "inspection.ml.pytorch-v1" => (
            "Inspect PyTorch storage",
            "du -sh \"$HOME/.cache/torch\"/*",
            "Review model weights and compiled artifacts before removing any cache subtree.",
        ),
        "inspection.ml.unsloth-v1" => (
            "Inspect Unsloth storage",
            "du -sh \"$HOME/.unsloth\"/*",
            "Keep private training data and checkpoints unless you have verified backups.",
        ),
        "inspection.container.colima-v1" => (
            "Inspect Colima",
            "colima status",
            "Review the active profile before using Colima's delete commands.",
        ),
        "inspection.container.lima-v1" => (
            "List Lima instances",
            "limactl list",
            "Use limactl to stop and remove only virtual machines you recognize.",
        ),
        "inspection.container.orbstack-v1" => (
            "List OrbStack machines",
            "orb list",
            "Use OrbStack's interface or CLI to remove owned containers and machines.",
        ),
        _ => return None,
    };
    Some(GuidedAction {
        title: title.to_owned(),
        command: command.to_owned(),
        explanation: explanation.to_owned(),
    })
}

fn progress_from(summary: &ScanSummary, current_path: Option<String>) -> ScanProgress {
    ScanProgress {
        scan_id: summary.scan_id.clone(),
        phase: summary.phase,
        current_path,
        files_scanned: summary.files_scanned,
        directories_scanned: summary.directories_scanned,
        bytes_examined: summary.bytes_examined,
        findings_count: summary.findings_count,
        reclaimable_bytes: summary.reclaimable_bytes,
        skipped_count: summary.skipped_count,
        permission_denied_count: summary.permission_denied_count,
        elapsed_ms: summary.elapsed_ms,
    }
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| format!("~/{}", relative.to_string_lossy()))
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_phase_six_profiles_are_available() {
        let profiles = ScanManager::profiles();
        assert_eq!(profiles.len(), 4);
        assert!(profiles.iter().all(|profile| profile.available));
    }

    #[cfg(unix)]
    #[test]
    fn sparse_virtual_disks_report_allocated_bytes_and_remain_expert_only() {
        let home = tempfile::tempdir().unwrap();
        let docker = home
            .path()
            .join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw");
        let claude = home
            .path()
            .join("Library/Application Support/Claude/vm_bundles/fixture.bundle");
        std::fs::create_dir_all(docker.parent().unwrap()).unwrap();
        std::fs::create_dir_all(claude.parent().unwrap()).unwrap();
        std::fs::File::create(&docker)
            .unwrap()
            .set_len(8 * 1024 * 1024 * 1024)
            .unwrap();
        std::fs::File::create(&claude)
            .unwrap()
            .set_len(4 * 1024 * 1024 * 1024)
            .unwrap();

        let rules =
            RulesRegistry.rules_for_scan(ScanProfileId::Developer, home.path(), "macos", &[]);
        for id in [
            "inspection.docker.raw-v1",
            "inspection.claude.vm-bundles-v1",
        ] {
            let rule = rules
                .iter()
                .find(|rule| rule.definition.id == id)
                .unwrap()
                .clone();
            let measurement = measure_target(
                &rule.target,
                &ExclusionMatcher::default(),
                &CancellationToken::default(),
                |_, _| {},
            );
            let finding = build_finding("fixture-scan", home.path(), "macos", rule, &measurement);

            assert!(measurement.logical_size >= 4 * 1024 * 1024 * 1024);
            assert!(measurement.allocated_size < measurement.logical_size);
            assert_eq!(finding.risk, crate::domain::rule::RiskLevel::Expert);
            assert!(!finding.cleanup_allowed);
            assert!(finding.guided_action.is_some());
        }
    }
}
