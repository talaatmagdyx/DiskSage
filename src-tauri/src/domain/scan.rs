use serde::{Deserialize, Serialize};

use super::{error::CommandError, rule::RuleCategory};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScanProfileId {
    Quick,
    Developer,
    FullAnalysis,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProfile {
    pub id: ScanProfileId,
    pub display_name: String,
    pub description: String,
    pub expected_duration: String,
    pub available: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScanPhase {
    Preparing,
    DiscoveringTargets,
    Scanning,
    Analyzing,
    Hashing,
    Finalizing,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanProgress {
    pub scan_id: String,
    pub phase: ScanPhase,
    pub current_path: Option<String>,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub bytes_examined: u64,
    pub findings_count: u64,
    pub reclaimable_bytes: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub scan_id: String,
    pub profile: ScanProfileId,
    pub phase: ScanPhase,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub bytes_examined: u64,
    pub findings_count: u64,
    pub reclaimable_bytes: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub elapsed_ms: u64,
    pub errors: Vec<CommandError>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartScanRequest {
    pub profile: ScanProfileId,
    #[serde(default)]
    pub excluded_paths: Vec<String>,
    #[serde(default)]
    pub custom: Option<CustomScanOptions>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CustomScanOptions {
    pub roots: Vec<String>,
    pub enabled_categories: Vec<RuleCategory>,
    pub minimum_file_size_bytes: u64,
    pub maximum_depth: u16,
    pub include_hidden_files: bool,
    pub include_external_drives: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanResponse {
    pub scan_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanIdRequest {
    pub scan_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetScanFindingsRequest {
    pub scan_id: String,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_page_size")]
    pub limit: usize,
}

fn default_page_size() -> usize {
    100
}
