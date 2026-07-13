use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::domain::{rule::RuleDefinition, scan::ScanProfileId};

use super::{
    catalogs::{developer_tools, project_artifacts, safe_caches},
    project_detection,
};

#[derive(Debug, Clone)]
pub struct ResolvedRule {
    pub definition: RuleDefinition,
    pub target: PathBuf,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RulesRegistry;

impl RulesRegistry {
    pub fn rules_for(
        &self,
        profile: ScanProfileId,
        home: &Path,
        platform: &str,
    ) -> Vec<ResolvedRule> {
        safe_caches::catalog()
            .into_iter()
            .filter(|rule| match profile {
                ScanProfileId::Quick => matches!(
                    rule.definition.id.as_str(),
                    "cache.npm.content-v1"
                        | "cache.yarn.downloads-v1"
                        | "cache.pip.downloads-v1"
                        | "cache.browser.chrome-v1"
                        | "cache.browser.firefox-v1"
                ),
                ScanProfileId::Developer => true,
                ScanProfileId::FullAnalysis | ScanProfileId::Custom => false,
            })
            .map(|rule| {
                let relative = if platform == "macos" {
                    rule.macos_path
                } else {
                    rule.linux_path
                };
                ResolvedRule {
                    definition: rule.definition,
                    target: home.join(relative),
                }
            })
            .collect()
    }

    pub fn resolve(
        &self,
        rule_id: &str,
        rule_version: u32,
        home: &Path,
        platform: &str,
    ) -> Option<ResolvedRule> {
        self.rules_for(ScanProfileId::Developer, home, platform)
            .into_iter()
            .find(|rule| rule.definition.id == rule_id && rule.definition.version == rule_version)
    }

    pub fn rules_for_scan(
        &self,
        profile: ScanProfileId,
        home: &Path,
        platform: &str,
        configured_project_roots: &[PathBuf],
    ) -> Vec<ResolvedRule> {
        let mut rules = self.rules_for(profile, home, platform);
        if profile != ScanProfileId::Developer {
            return rules;
        }
        rules.extend(developer_tools::catalog().into_iter().filter_map(|spec| {
            let relative = if platform == "macos" {
                spec.macos_path
            } else {
                spec.linux_path
            }?;
            Some(ResolvedRule {
                definition: spec.definition,
                target: home.join(relative),
            })
        }));
        let projects = project_detection::detect(configured_project_roots);
        for project in projects {
            for spec in project_artifacts::catalog() {
                if spec
                    .indicators
                    .iter()
                    .any(|indicator| project.join(indicator).is_file())
                {
                    rules.push(ResolvedRule {
                        definition: spec.definition,
                        target: project.join(spec.artifact),
                    });
                }
            }
        }
        let mut seen = HashSet::new();
        rules.retain(|rule| seen.insert(rule.target.clone()));
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_defines_at_least_twenty_five_versioned_rules() {
        let mut ids = HashSet::new();
        let mut count = 0;
        for definition in safe_caches::catalog()
            .into_iter()
            .map(|rule| rule.definition)
            .chain(
                developer_tools::catalog()
                    .into_iter()
                    .map(|rule| rule.definition),
            )
            .chain(
                project_artifacts::catalog()
                    .into_iter()
                    .map(|rule| rule.definition),
            )
        {
            count += 1;
            assert!(definition.version > 0);
            assert!(ids.insert(definition.id));
        }
        assert!(count >= 25);
        assert!(developer_tools::catalog()
            .iter()
            .all(|rule| !rule.definition.default_enabled));
        assert!(project_artifacts::catalog()
            .iter()
            .all(|rule| !rule.definition.default_enabled));
    }

    #[test]
    fn project_artifacts_require_manifest_context() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(directory.path().join("plain/build")).unwrap();
        let registry = RulesRegistry;
        let without_manifest = registry.rules_for_scan(
            ScanProfileId::Developer,
            directory.path(),
            "linux",
            &[directory.path().join("plain")],
        );
        assert!(!without_manifest
            .iter()
            .any(|rule| rule.definition.id.starts_with("artifact.")));

        std::fs::write(directory.path().join("plain/package.json"), b"{}").unwrap();
        let with_manifest = registry.rules_for_scan(
            ScanProfileId::Developer,
            directory.path(),
            "linux",
            &[directory.path().join("plain")],
        );
        assert!(with_manifest
            .iter()
            .any(|rule| rule.definition.id == "artifact.node.build-v1"));
    }
}
