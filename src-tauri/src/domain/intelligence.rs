use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionAccess {
    Available,
    Limited,
    NotPresent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionLocation {
    pub label: String,
    pub display_path: String,
    pub access: PermissionAccess,
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionReport {
    pub checked_at: DateTime<Utc>,
    pub full_disk_access_likely: bool,
    pub locations: Vec<PermissionLocation>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrphanedApplicationData {
    pub id: String,
    pub path: String,
    pub display_path: String,
    pub identifier: String,
    pub category: String,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub reason: String,
    pub default_selected: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageMapRequest {
    pub root: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub display_path: String,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub permission_denied_count: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMapReport {
    pub root: String,
    pub display_root: String,
    pub entries: Vec<StorageMapEntry>,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub permission_denied_count: u64,
    pub truncated: bool,
    pub elapsed_ms: u64,
    pub note: String,
}
