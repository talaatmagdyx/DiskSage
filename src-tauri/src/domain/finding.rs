use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::rule::{RecommendedAction, RiskLevel, RuleCategory};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FindingType {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FindingEvidence {
    KnownPath,
    ExtensionMatch { extensions: Vec<String> },
    DirectoryNameMatch { directory_name: String },
    AgeThreshold { older_than_days: u32 },
    SizeThreshold { minimum_bytes: u64 },
    DuplicateHash { algorithm: String, group_id: String },
    PackageManagerCache { manager: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub scan_id: String,
    pub rule_id: String,
    pub rule_version: u32,
    pub category: RuleCategory,
    pub display_name: String,
    pub description: String,
    pub path: PathBuf,
    pub display_path: String,
    pub item_type: FindingType,
    pub logical_size: u64,
    pub allocated_size: Option<u64>,
    pub modified_at: Option<DateTime<Utc>>,
    pub risk: RiskLevel,
    pub recommended_action: RecommendedAction,
    pub evidence: FindingEvidence,
    pub cleanup_allowed: bool,
    pub cleanup_block_reason: Option<String>,
}
