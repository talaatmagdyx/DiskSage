use std::path::{Path, PathBuf};

use crate::domain::{rule::RuleDefinition, scan::ScanProfileId};

use super::catalogs::safe_caches;

#[derive(Debug, Clone)]
pub struct ResolvedRule {
    pub definition: RuleDefinition,
    pub target: PathBuf,
}

#[derive(Debug, Default)]
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
}
