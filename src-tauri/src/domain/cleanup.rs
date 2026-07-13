use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{disk::DiskInfo, error::CommandError, finding::FindingType, rule::RiskLevel};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanupAction {
    MoveToTrash,
    PermanentDelete,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanItem {
    pub scan_id: String,
    pub finding_id: String,
    pub rule_id: String,
    pub rule_version: u32,
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub expected_type: FindingType,
    pub expected_size: u64,
    pub expected_modified_at: Option<DateTime<Utc>>,
    pub risk: RiskLevel,
    pub validation_token: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RiskSummary {
    pub safe: u64,
    pub careful: u64,
    pub expert: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub action: CleanupAction,
    pub items: Vec<CleanupPlanItem>,
    pub expected_reclaimable_bytes: u64,
    pub risk_summary: RiskSummary,
    pub confirmation_token: String,
    pub required_confirmation_phrase: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCleanupPlanRequest {
    pub scan_id: String,
    pub finding_ids: Vec<String>,
    pub action: CleanupAction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteCleanupRequest {
    pub plan_id: String,
    pub confirmation_token: String,
    pub typed_confirmation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteCleanupResponse {
    pub operation_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelCleanupRequest {
    pub operation_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanupItemStatus {
    MovedToTrash,
    PermanentlyDeleted,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItemResult {
    pub finding_id: String,
    pub rule_id: String,
    pub display_path: String,
    pub expected_bytes: u64,
    pub status: CleanupItemStatus,
    pub error: Option<CommandError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupProgress {
    pub operation_id: String,
    pub total_items: u64,
    pub completed_items: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub skipped_count: u64,
    pub processed_bytes: u64,
    pub current_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupSummary {
    pub operation_id: String,
    pub plan_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub action: CleanupAction,
    pub selected_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub skipped_count: u64,
    pub expected_bytes: u64,
    pub actual_free_space_change_bytes: Option<u64>,
    pub cancelled: bool,
    pub items: Vec<CleanupItemResult>,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetCleanupHistoryRequest {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_history_page_size")]
    pub limit: usize,
}

fn default_history_page_size() -> usize {
    50
}
