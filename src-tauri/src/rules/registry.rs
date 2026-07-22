use std::{
    collections::HashSet,
    fs,
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
    pub excluded_targets: Vec<PathBuf>,
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
        let mut rules: Vec<_> = safe_caches::catalog()
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
                ScanProfileId::Developer | ScanProfileId::FullAnalysis => true,
                ScanProfileId::Custom => false,
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
                    excluded_targets: Vec::new(),
                }
            })
            .collect();
        expand_chrome_profile_caches(&mut rules, home, platform);
        rules
    }

    pub fn resolve(
        &self,
        rule_id: &str,
        rule_version: u32,
        home: &Path,
        platform: &str,
        expected_target: &Path,
    ) -> Option<ResolvedRule> {
        if let Some(rule) = self
            .rules_for(ScanProfileId::Developer, home, platform)
            .into_iter()
            .find(|rule| {
                rule.definition.id == rule_id
                    && rule.definition.version == rule_version
                    && rule.target == expected_target
            })
        {
            return Some(rule);
        }
        if let Some(rule) = developer_tools::catalog().into_iter().find_map(|spec| {
            if spec.definition.id != rule_id || spec.definition.version != rule_version {
                return None;
            }
            let relative = if platform == "macos" {
                spec.macos_path
            } else {
                spec.linux_path
            }?;
            let target = home.join(relative);
            (target == expected_target).then_some(ResolvedRule {
                definition: spec.definition,
                target,
                excluded_targets: Vec::new(),
            })
        }) {
            return Some(rule);
        }
        project_artifacts::catalog().into_iter().find_map(|spec| {
            if spec.definition.id != rule_id || spec.definition.version != rule_version {
                return None;
            }
            let depth = Path::new(spec.artifact).components().count();
            let project = expected_target.ancestors().nth(depth)?;
            if project == home
                || !project.starts_with(home)
                || project.join(spec.artifact) != expected_target
                || !spec
                    .indicators
                    .iter()
                    .any(|indicator| project.join(indicator).is_file())
            {
                return None;
            }
            Some(ResolvedRule {
                definition: spec.definition,
                target: expected_target.to_path_buf(),
                excluded_targets: Vec::new(),
            })
        })
    }

    pub fn rules_for_scan(
        &self,
        profile: ScanProfileId,
        home: &Path,
        platform: &str,
        configured_project_roots: &[PathBuf],
    ) -> Vec<ResolvedRule> {
        let mut rules = self.rules_for(profile, home, platform);
        if !matches!(
            profile,
            ScanProfileId::Developer | ScanProfileId::FullAnalysis
        ) {
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
                excluded_targets: Vec::new(),
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
                        excluded_targets: Vec::new(),
                    });
                }
            }
        }
        assign_unclassified_cache_exclusions(&mut rules);
        let mut seen = HashSet::new();
        rules.retain(|rule| seen.insert(rule.target.clone()));
        rules
    }
}

fn expand_chrome_profile_caches(rules: &mut Vec<ResolvedRule>, home: &Path, platform: &str) {
    let Some(template_index) = rules
        .iter()
        .position(|rule| rule.definition.id == "cache.browser.chrome-v1")
    else {
        return;
    };
    let template = rules.remove(template_index);
    let targets = chrome_profile_cache_targets(home, platform);
    if targets.is_empty() {
        rules.push(template);
        return;
    }
    rules.extend(targets.into_iter().map(|(target, label)| {
        let mut definition = template.definition.clone();
        definition.display_name = format!("Google Chrome cache ({label})");
        ResolvedRule {
            definition,
            target,
            excluded_targets: Vec::new(),
        }
    }));
}

fn assign_unclassified_cache_exclusions(rules: &mut [ResolvedRule]) {
    let Some(cache_root) = rules
        .iter()
        .find(|rule| rule.definition.id == "inspection.user.dot-cache-v1")
        .map(|rule| rule.target.clone())
    else {
        return;
    };
    let excluded_targets = rules
        .iter()
        .filter(|rule| rule.target != cache_root && rule.target.starts_with(&cache_root))
        .map(|rule| rule.target.clone())
        .collect::<Vec<_>>();
    if let Some(remainder) = rules
        .iter_mut()
        .find(|rule| rule.definition.id == "inspection.user.dot-cache-v1")
    {
        remainder.excluded_targets = excluded_targets;
    }
}

fn chrome_profile_cache_targets(home: &Path, platform: &str) -> Vec<(PathBuf, String)> {
    let roots: &[(&str, &str)] = if platform == "macos" {
        &[
            ("Library/Caches/Google/Chrome", "Chrome"),
            ("Library/Caches/Google/Chrome Beta", "Chrome Beta"),
            ("Library/Caches/Google/Chrome Canary", "Chrome Canary"),
            ("Library/Caches/Google/Chrome Dev", "Chrome Dev"),
        ]
    } else {
        &[
            (".cache/google-chrome", "Chrome"),
            (".cache/google-chrome-beta", "Chrome Beta"),
            (".cache/google-chrome-unstable", "Chrome Dev"),
            (".cache/chromium", "Chromium"),
        ]
    };
    let mut targets = Vec::new();
    for (relative_root, channel) in roots {
        let root = home.join(relative_root);
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let profile = entry.file_name().to_string_lossy().into_owned();
            if profile != "Default"
                && profile != "Guest Profile"
                && profile != "System Profile"
                && !profile.starts_with("Profile ")
            {
                continue;
            }
            let target = entry.path().join("Cache/Cache_Data");
            if target.is_dir() {
                targets.push((target, format!("{channel} · {profile}")));
            }
        }
    }
    targets.sort_by(|left, right| left.0.cmp(&right.0));
    targets.dedup_by(|left, right| left.0 == right.0);
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{
        cancellation::CancellationToken, exclusions::ExclusionMatcher, walker::measure_target,
    };

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
        let target = directory.path().join("plain/build");
        assert!(registry
            .resolve(
                "artifact.node.build-v1",
                1,
                directory.path(),
                "linux",
                &target,
            )
            .is_some());
    }

    #[test]
    fn developer_scan_includes_expanded_workstation_targets_but_quick_scan_does_not() {
        let home = tempfile::tempdir().unwrap();
        let registry = RulesRegistry;
        let developer =
            registry.rules_for_scan(ScanProfileId::Developer, home.path(), "macos", &[]);
        let quick = registry.rules_for_scan(ScanProfileId::Quick, home.path(), "macos", &[]);

        for id in [
            "cache.updater.claude-v1",
            "cache.cursor.cached-data-v1",
            "cache.updater.codex-v1",
            "cache.uv-v1",
            "cache.pre-commit-v1",
            "cache.cypress-v1",
            "cache.vscode.extension-vsix-v1",
            "inspection.cursor.state-db-v1",
            "inspection.jetbrains.versioned-data-v1",
            "inspection.runtime.mise-v1",
            "inspection.claude.sessions-v1",
            "inspection.claude.vm-bundles-v1",
            "inspection.codex.runtimes-v1",
            "inspection.xcode.archives-v1",
            "inspection.playwright.browsers-v1",
            "inspection.ml.huggingface-v1",
            "inspection.ml.ollama-v1",
            "inspection.container.colima-v1",
            "inspection.container.lima-v1",
            "inspection.container.orbstack-v1",
        ] {
            assert!(
                developer.iter().any(|rule| rule.definition.id == id),
                "missing {id}"
            );
            assert!(
                !quick.iter().any(|rule| rule.definition.id == id),
                "{id} leaked into Quick Scan"
            );
        }
    }

    #[test]
    fn chrome_cache_rule_discovers_each_known_profile_without_scanning_profile_data() {
        let home = tempfile::tempdir().unwrap();
        let chrome = home.path().join("Library/Caches/Google/Chrome");
        let default_cache = chrome.join("Default/Cache/Cache_Data");
        let profile_cache = chrome.join("Profile 2/Cache/Cache_Data");
        std::fs::create_dir_all(&default_cache).unwrap();
        std::fs::create_dir_all(&profile_cache).unwrap();
        std::fs::create_dir_all(chrome.join("Not a profile/Cache/Cache_Data")).unwrap();

        let registry = RulesRegistry;
        let rules = registry.rules_for(ScanProfileId::Quick, home.path(), "macos");
        let chrome_rules: Vec<_> = rules
            .iter()
            .filter(|rule| rule.definition.id == "cache.browser.chrome-v1")
            .collect();

        assert_eq!(chrome_rules.len(), 2);
        assert!(chrome_rules.iter().any(|rule| rule.target == default_cache));
        assert!(chrome_rules.iter().any(|rule| rule.target == profile_cache));
        assert!(chrome_rules
            .iter()
            .all(|rule| rule.target.ends_with("Cache/Cache_Data")));
        assert!(registry
            .resolve(
                "cache.browser.chrome-v1",
                1,
                home.path(),
                "macos",
                &profile_cache,
            )
            .is_some());
    }

    #[test]
    fn unclassified_cache_remainder_excludes_every_known_nested_rule() {
        let home = tempfile::tempdir().unwrap();
        let cache = home.path().join(".cache");
        std::fs::create_dir_all(cache.join("uv")).unwrap();
        std::fs::create_dir_all(cache.join("other")).unwrap();
        std::fs::write(cache.join("uv/packages"), b"known").unwrap();
        std::fs::write(cache.join("other/unknown"), b"remainder").unwrap();

        let registry = RulesRegistry;
        let rules = registry.rules_for_scan(ScanProfileId::Developer, home.path(), "linux", &[]);
        let remainder = rules
            .iter()
            .find(|rule| rule.definition.id == "inspection.user.dot-cache-v1")
            .unwrap();
        assert!(remainder.excluded_targets.contains(&cache.join("uv")));

        let remainder_measurement = measure_target(
            &remainder.target,
            &ExclusionMatcher::default().with_additional_paths(&remainder.excluded_targets),
            &CancellationToken::default(),
            |_, _| {},
        );
        let uv = rules
            .iter()
            .find(|rule| rule.definition.id == "cache.uv-v1")
            .unwrap();
        let uv_measurement = measure_target(
            &uv.target,
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
        );
        let full_measurement = measure_target(
            &cache,
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
        );

        assert_eq!(remainder_measurement.logical_size, 9);
        assert_eq!(uv_measurement.logical_size, 5);
        assert_eq!(
            remainder_measurement.logical_size + uv_measurement.logical_size,
            full_measurement.logical_size
        );
    }
}
