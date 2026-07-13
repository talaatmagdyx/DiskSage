use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{cleanup::CleanupAction, error::CommandError};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateScanPhase {
    Discovering,
    Grouping,
    PartialHashing,
    FullHashing,
    Verifying,
    Finalizing,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartDuplicateScanRequest {
    pub roots: Vec<String>,
    pub minimum_size_bytes: u64,
    #[serde(default)]
    pub byte_for_byte_verification: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDuplicateScanResponse {
    pub scan_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateScanIdRequest {
    pub scan_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetDuplicateGroupsRequest {
    pub scan_id: String,
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_group_page_size")]
    pub limit: usize,
}

fn default_group_page_size() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCopy {
    pub id: String,
    pub path: PathBuf,
    pub display_path: String,
    pub modified_at: Option<DateTime<Utc>>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub id: String,
    pub scan_id: String,
    pub file_size: u64,
    pub reclaimable_bytes: u64,
    pub copies: Vec<DuplicateCopy>,
    pub recommended_keep_id: String,
    pub keep_reason: String,
    pub full_hash: String,
    pub byte_for_byte_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateProgress {
    pub scan_id: String,
    pub phase: DuplicateScanPhase,
    pub current_path: Option<String>,
    pub files_scanned: u64,
    pub candidate_files: u64,
    pub bytes_hashed: u64,
    pub groups_found: u64,
    pub reclaimable_bytes: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSummary {
    pub scan_id: String,
    pub phase: DuplicateScanPhase,
    pub roots: Vec<PathBuf>,
    pub minimum_size_bytes: u64,
    pub byte_for_byte_verification: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub files_scanned: u64,
    pub candidate_files: u64,
    pub bytes_hashed: u64,
    pub groups_found: u64,
    pub duplicate_files: u64,
    pub reclaimable_bytes: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub elapsed_ms: u64,
    pub errors: Vec<CommandError>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateCleanupSelection {
    pub group_id: String,
    pub keep_copy_id: String,
    pub trash_copy_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateDuplicateCleanupPlanRequest {
    pub scan_id: String,
    pub selections: Vec<DuplicateCleanupSelection>,
    pub action: CleanupAction,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCleanupPlanItem {
    pub group_id: String,
    pub copy_id: String,
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub expected_size: u64,
    pub expected_modified_at: Option<DateTime<Utc>>,
    pub full_hash: String,
    pub keep_copy_id: String,
    pub keep_path: PathBuf,
    pub keep_canonical_path: PathBuf,
    pub keep_modified_at: Option<DateTime<Utc>>,
    pub byte_for_byte_verified: bool,
    pub validation_token: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateCleanupPlan {
    pub id: String,
    pub scan_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub action: CleanupAction,
    pub items: Vec<DuplicateCleanupPlanItem>,
    pub expected_reclaimable_bytes: u64,
    pub kept_copy_count: u64,
    pub confirmation_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteDuplicateCleanupRequest {
    pub plan_id: String,
    pub confirmation_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelDuplicateCleanupRequest {
    pub operation_id: String,
}
