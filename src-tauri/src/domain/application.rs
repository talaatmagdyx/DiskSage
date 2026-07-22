use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::ErrorCode;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApplicationScope {
    User,
    Shared,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApplication {
    pub id: String,
    pub name: String,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub path: String,
    pub display_path: String,
    pub logical_size: u64,
    pub allocated_size: Option<u64>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub scope: ApplicationScope,
    pub uninstall_allowed: bool,
    pub uninstall_block_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApplicationUninstallMode {
    AppOnly,
    Complete,
    DeepCleanup,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RelatedItemConfidence {
    Identified,
    Ambiguous,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedApplicationItem {
    pub id: String,
    pub path: String,
    pub display_path: String,
    pub category: String,
    pub logical_size: u64,
    pub allocated_size: Option<u64>,
    pub may_contain_user_data: bool,
    pub confidence: RelatedItemConfidence,
    pub default_selected: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplicationIdRequest {
    pub application_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateApplicationUninstallPlanRequest {
    pub application_id: String,
    pub mode: ApplicationUninstallMode,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationUninstallPlan {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub application: InstalledApplication,
    pub mode: ApplicationUninstallMode,
    pub related_items: Vec<RelatedApplicationItem>,
    pub total_expected_bytes: u64,
    pub required_confirmation_phrase: Option<String>,
    pub confirmation_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteApplicationUninstallRequest {
    pub plan_id: String,
    pub confirmation_token: String,
    #[serde(default)]
    pub selected_related_item_ids: Vec<String>,
    pub typed_confirmation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationUninstallFailure {
    pub display_path: String,
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationUninstallResult {
    pub application_id: String,
    pub name: String,
    pub display_path: String,
    pub moved_to_trash: bool,
    pub expected_bytes: u64,
    pub mode: ApplicationUninstallMode,
    pub related_items_planned: u64,
    pub related_items_moved: u64,
    pub related_items_failed: u64,
    pub failed_paths: Vec<String>,
    pub failed_items: Vec<ApplicationUninstallFailure>,
}
