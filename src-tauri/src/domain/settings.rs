use std::{
    collections::HashSet,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};

use super::error::{CommandError, ErrorCode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ScanMode {
    Quick,
    Developer,
    FullAnalysis,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DuplicateVerificationMode {
    FullHash,
    ByteForByte,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppSettings {
    pub schema_version: u32,
    pub default_scan_mode: ScanMode,
    pub follow_symlinks: bool,
    pub scan_external_drives: bool,
    pub scan_hidden_files: bool,
    pub maximum_concurrency: u8,
    pub large_file_threshold_bytes: u64,
    pub duplicate_minimum_size_bytes: u64,
    pub duplicate_verification_mode: DuplicateVerificationMode,
    pub move_to_trash_by_default: bool,
    pub permanent_deletion_enabled: bool,
    pub preselect_safe_items: bool,
    pub require_cleanup_confirmation: bool,
    pub show_expert_recommendations: bool,
    pub diagnostic_logging: bool,
    pub theme: Theme,
    pub reduced_motion: bool,
    #[serde(default)]
    pub project_roots: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            default_scan_mode: ScanMode::Quick,
            follow_symlinks: false,
            scan_external_drives: false,
            scan_hidden_files: false,
            maximum_concurrency: 3,
            large_file_threshold_bytes: 1_073_741_824,
            duplicate_minimum_size_bytes: 1_048_576,
            duplicate_verification_mode: DuplicateVerificationMode::FullHash,
            move_to_trash_by_default: true,
            permanent_deletion_enabled: false,
            preselect_safe_items: false,
            require_cleanup_confirmation: true,
            show_expert_recommendations: false,
            diagnostic_logging: false,
            theme: Theme::System,
            reduced_motion: false,
            project_roots: Vec::new(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), CommandError> {
        if self.schema_version != 1 {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Unsupported settings version.",
                false,
            ));
        }
        if self.follow_symlinks {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Following symlinks is disabled for safety.",
                true,
            ));
        }
        if !self.move_to_trash_by_default {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Move to Trash must remain the default action.",
                true,
            ));
        }
        if !(1..=8).contains(&self.maximum_concurrency) {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Filesystem concurrency must be between 1 and 8.",
                true,
            ));
        }
        if self.large_file_threshold_bytes < 10 * 1_048_576 {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Large-file threshold must be at least 10 MB.",
                true,
            ));
        }
        if self.duplicate_minimum_size_bytes < 1_024 {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "Duplicate minimum size must be at least 1 KB.",
                true,
            ));
        }
        if self.project_roots.len() > 20 {
            return Err(CommandError::new(
                ErrorCode::InvalidSettings,
                "At most 20 project roots can be configured.",
                true,
            ));
        }
        let mut unique = HashSet::new();
        for root in &self.project_roots {
            let path = Path::new(root);
            if root.len() > 4096
                || !path.is_absolute()
                || path.parent().is_none()
                || path
                    .components()
                    .any(|component| matches!(component, Component::ParentDir))
                || !unique.insert(root)
            {
                return Err(CommandError::new(
                    ErrorCode::InvalidSettings,
                    "Project roots must be unique absolute non-root paths without traversal.",
                    true,
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_roots_reject_roots_relative_paths_and_traversal() {
        for invalid in ["/", "relative/project", "/tmp/../etc"] {
            let settings = AppSettings {
                project_roots: vec![invalid.to_owned()],
                ..AppSettings::default()
            };
            assert_eq!(
                settings.validate().unwrap_err().code,
                ErrorCode::InvalidSettings
            );
        }
    }
}
