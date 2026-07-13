use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{finding::FindingType, rule::RiskLevel};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanupAction {
    MoveToTrash,
    PermanentDelete,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanItem {
    pub finding_id: String,
    pub path: PathBuf,
    pub expected_type: FindingType,
    pub expected_size: u64,
    pub expected_modified_at: Option<DateTime<Utc>>,
    pub risk: RiskLevel,
    validation_token: String,
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
    pub action: CleanupAction,
    pub items: Vec<CleanupPlanItem>,
    pub expected_reclaimable_bytes: u64,
    pub risk_summary: RiskSummary,
    confirmation_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCleanupPlanRequest {
    pub finding_ids: Vec<String>,
    pub action: CleanupAction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteCleanupRequest {
    pub plan_id: String,
    pub confirmation_token: String,
}
