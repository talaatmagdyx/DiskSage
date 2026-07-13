use crate::domain::rule::{
    Platform, RecommendedAction, RegenerationBehavior, RiskLevel, RuleCategory, RuleDefinition,
    RuleMatcher, RuleTarget,
};

pub struct CacheRuleSpec {
    pub definition: RuleDefinition,
    pub macos_path: &'static str,
    pub linux_path: &'static str,
}

pub fn catalog() -> Vec<CacheRuleSpec> {
    vec![
        package_rule("cache.npm.content-v1", "npm download cache", "npm", ".npm/_cacache", ".npm/_cacache"),
        package_rule("cache.pnpm.store-v1", "pnpm package store", "pnpm", "Library/pnpm/store", ".local/share/pnpm/store"),
        package_rule("cache.yarn.downloads-v1", "Yarn download cache", "Yarn", "Library/Caches/Yarn", ".cache/yarn"),
        package_rule("cache.pip.downloads-v1", "pip download cache", "pip", "Library/Caches/pip", ".cache/pip"),
        package_rule("cache.cargo.registry-v1", "Cargo registry cache", "Cargo", ".cargo/registry/cache", ".cargo/registry/cache"),
        package_rule("cache.gradle.modules-v1", "Gradle module cache", "Gradle", ".gradle/caches/modules-2/files-2.1", ".gradle/caches/modules-2/files-2.1"),
        package_rule("cache.maven.wrapper-v1", "Maven wrapper downloads", "Maven", ".m2/wrapper/dists", ".m2/wrapper/dists"),
        package_rule("cache.go.modules-v1", "Go module download cache", "Go", "go/pkg/mod/cache", "go/pkg/mod/cache"),
        browser_rule(
            "cache.browser.chrome-v1",
            "Google Chrome cache",
            "Closing Chrome before cleanup may be required. Browser profiles, cookies, history, sessions, and credentials are excluded.",
            "Library/Caches/Google/Chrome/Default/Cache/Cache_Data",
            ".cache/google-chrome/Default/Cache/Cache_Data",
        ),
        browser_rule(
            "cache.browser.firefox-v1",
            "Firefox cache",
            "Only the operating system cache location is scanned. Firefox profiles, history, sessions, and credentials are excluded.",
            "Library/Caches/Firefox/Profiles",
            ".cache/mozilla/firefox",
        ),
    ]
}

fn package_rule(
    id: &str,
    display_name: &str,
    manager: &str,
    macos_path: &'static str,
    linux_path: &'static str,
) -> CacheRuleSpec {
    build_rule(
        id,
        display_name,
        &format!("Downloaded {manager} packages that can be fetched again when needed."),
        RuleCategory::PackageManagerCache,
        RegenerationBehavior::Redownload,
        macos_path,
        linux_path,
    )
}

fn browser_rule(
    id: &str,
    display_name: &str,
    description: &str,
    macos_path: &'static str,
    linux_path: &'static str,
) -> CacheRuleSpec {
    build_rule(
        id,
        display_name,
        description,
        RuleCategory::BrowserCache,
        RegenerationBehavior::Automatic,
        macos_path,
        linux_path,
    )
}

fn build_rule(
    id: &str,
    display_name: &str,
    description: &str,
    category: RuleCategory,
    regeneration_behavior: RegenerationBehavior,
    macos_path: &'static str,
    linux_path: &'static str,
) -> CacheRuleSpec {
    CacheRuleSpec {
        definition: RuleDefinition {
            id: id.to_owned(),
            version: 1,
            category,
            display_name: display_name.to_owned(),
            description: description.to_owned(),
            risk: RiskLevel::Safe,
            platforms: vec![Platform::Macos, Platform::Linux],
            targets: vec![
                RuleTarget::HomeRelative {
                    path: macos_path.to_owned(),
                },
                RuleTarget::HomeRelative {
                    path: linux_path.to_owned(),
                },
            ],
            matcher: RuleMatcher::ExactPath,
            minimum_size: Some(1),
            maximum_age_days: None,
            recommended_action: RecommendedAction::MoveToTrash,
            regeneration_behavior,
            default_enabled: true,
        },
        macos_path,
        linux_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_ten_unique_safe_rules() {
        let rules = catalog();
        let ids: HashSet<_> = rules
            .iter()
            .map(|rule| rule.definition.id.as_str())
            .collect();
        assert_eq!(rules.len(), 10);
        assert_eq!(ids.len(), rules.len());
        assert!(rules
            .iter()
            .all(|rule| rule.definition.risk == RiskLevel::Safe));
        for rule in rules {
            assert!(
                !rule.definition.description.is_empty(),
                "{} needs an explanation",
                rule.definition.id
            );
            assert_eq!(
                rule.definition.recommended_action,
                RecommendedAction::MoveToTrash
            );
            assert!(rule.definition.minimum_size.is_some());
            assert!(!rule.macos_path.starts_with('/'));
            assert!(!rule.linux_path.starts_with('/'));
        }
    }
}
