use serde::Serialize;

use super::{
    cleanup::{CleanupAction, CleanupSummary},
    disk::DiskInfo,
    error::ErrorCode,
    scan::{ScanPhase, ScanProfileId, ScanSummary},
    settings::{AppSettings, DuplicateVerificationMode, ScanMode, Theme},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsReport {
    pub generated_at: String,
    pub application: ApplicationDiagnostics,
    pub settings: SettingsDiagnostics,
    pub storage: StorageDiagnostics,
    pub latest_scan: Option<ScanDiagnostics>,
    pub recent_cleanup: Vec<CleanupDiagnostics>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDiagnostics {
    pub name: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub architecture: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDiagnostics {
    pub schema_version: u32,
    pub default_scan_mode: ScanMode,
    pub scan_external_drives: bool,
    pub scan_hidden_files: bool,
    pub maximum_concurrency: u8,
    pub duplicate_verification_mode: DuplicateVerificationMode,
    pub permanent_deletion_enabled: bool,
    pub diagnostic_logging: bool,
    pub theme: Theme,
    pub reduced_motion: bool,
    pub configured_project_root_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDiagnostics {
    pub accessible_disk_count: usize,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub removable_disk_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDiagnostics {
    pub profile: ScanProfileId,
    pub phase: ScanPhase,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub bytes_examined: u64,
    pub findings_count: u64,
    pub reclaimable_bytes: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub elapsed_ms: u64,
    pub error_codes: Vec<ErrorCode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupDiagnostics {
    pub action: CleanupAction,
    pub selected_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub skipped_count: u64,
    pub cancelled: bool,
    pub error_codes: Vec<ErrorCode>,
}

impl DiagnosticsReport {
    pub fn from_local_state(
        settings: &AppSettings,
        disks: &[DiskInfo],
        latest_scan: Option<&ScanSummary>,
        cleanup: &[CleanupSummary],
    ) -> Self {
        Self {
            generated_at: chrono::Utc::now().to_rfc3339(),
            application: ApplicationDiagnostics {
                name: "DiskSage",
                version: env!("CARGO_PKG_VERSION"),
                platform: std::env::consts::OS,
                architecture: std::env::consts::ARCH,
            },
            settings: SettingsDiagnostics {
                schema_version: settings.schema_version,
                default_scan_mode: settings.default_scan_mode,
                scan_external_drives: settings.scan_external_drives,
                scan_hidden_files: settings.scan_hidden_files,
                maximum_concurrency: settings.maximum_concurrency,
                duplicate_verification_mode: settings.duplicate_verification_mode,
                permanent_deletion_enabled: settings.permanent_deletion_enabled,
                diagnostic_logging: settings.diagnostic_logging,
                theme: settings.theme,
                reduced_motion: settings.reduced_motion,
                configured_project_root_count: settings.project_roots.len(),
            },
            storage: StorageDiagnostics {
                accessible_disk_count: disks.len(),
                total_bytes: disks.iter().map(|disk| disk.total_bytes).sum(),
                available_bytes: disks.iter().map(|disk| disk.available_bytes).sum(),
                removable_disk_count: disks.iter().filter(|disk| disk.removable).count(),
            },
            latest_scan: latest_scan.map(|scan| ScanDiagnostics {
                profile: scan.profile,
                phase: scan.phase,
                files_scanned: scan.files_scanned,
                directories_scanned: scan.directories_scanned,
                bytes_examined: scan.bytes_examined,
                findings_count: scan.findings_count,
                reclaimable_bytes: scan.reclaimable_bytes,
                skipped_count: scan.skipped_count,
                permission_denied_count: scan.permission_denied_count,
                elapsed_ms: scan.elapsed_ms,
                error_codes: scan.errors.iter().map(|error| error.code).collect(),
            }),
            recent_cleanup: cleanup
                .iter()
                .map(|entry| CleanupDiagnostics {
                    action: entry.action,
                    selected_count: entry.selected_count,
                    success_count: entry.success_count,
                    failure_count: entry.failure_count,
                    skipped_count: entry.skipped_count,
                    cancelled: entry.cancelled,
                    error_codes: entry
                        .items
                        .iter()
                        .filter_map(|item| item.error.as_ref().map(|error| error.code))
                        .collect(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_report_never_contains_configured_paths() {
        let settings = AppSettings {
            project_roots: vec!["/Users/private/secret-client".to_owned()],
            ..AppSettings::default()
        };
        let json = serde_json::to_string(&DiagnosticsReport::from_local_state(
            &settings,
            &[],
            None,
            &[],
        ))
        .unwrap();
        assert!(!json.contains("secret-client"));
        assert!(json.contains("configuredProjectRootCount"));
    }
}
