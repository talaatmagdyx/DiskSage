use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Safe,
    Careful,
    Expert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuleCategory {
    ApplicationCache,
    BrowserCache,
    PackageManagerCache,
    BuildArtifact,
    Log,
    Installer,
    Duplicate,
    LargeFile,
    OldFile,
    Container,
    Emulator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Macos,
    Linux,
    Windows,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecommendedAction {
    MoveToTrash,
    Review,
    GuidedCommand,
    NoAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RegenerationBehavior {
    Automatic,
    Redownload,
    Reindex,
    RestartRequired,
    PotentialStateLoss,
    NotRegenerable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuleTarget {
    HomeRelative { path: String },
    CacheDirectory { application: String, path: String },
    SelectedProjectRoots,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuleMatcher {
    ExactPath,
    Extensions {
        extensions: Vec<String>,
    },
    DirectoryName {
        names: Vec<String>,
    },
    Glob {
        pattern: String,
    },
    ProjectArtifact {
        indicators: Vec<String>,
        artifact: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleDefinition {
    pub id: String,
    pub version: u32,
    pub category: RuleCategory,
    pub display_name: String,
    pub description: String,
    pub risk: RiskLevel,
    pub platforms: Vec<Platform>,
    pub targets: Vec<RuleTarget>,
    pub matcher: RuleMatcher,
    pub minimum_size: Option<u64>,
    pub maximum_age_days: Option<u32>,
    pub recommended_action: RecommendedAction,
    pub regeneration_behavior: RegenerationBehavior,
    pub default_enabled: bool,
}
