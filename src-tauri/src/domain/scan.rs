use serde::{Deserialize, Serialize};

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
