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
        package_rule("cache.gradle.wrapper-v1", "Gradle wrapper downloads", "Gradle", ".gradle/wrapper/dists", ".gradle/wrapper/dists"),
        package_rule("cache.maven.wrapper-v1", "Maven wrapper downloads", "Maven", ".m2/wrapper/dists", ".m2/wrapper/dists"),
        package_rule("cache.maven.repository-v1", "Maven local repository", "Maven", ".m2/repository", ".m2/repository"),
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
        package_rule("cache.node-gyp.downloads-v1", "node-gyp download cache", "node-gyp", "Library/Caches/node-gyp", ".cache/node-gyp"),
        package_rule("cache.pip-tools.downloads-v1", "pip-tools cache", "pip-tools", "Library/Caches/pip-tools", ".cache/pip-tools"),
        package_rule("cache.homebrew.downloads-v1", "Homebrew download cache", "Homebrew", "Library/Caches/Homebrew", ".cache/Homebrew"),
        package_rule("cache.aws-cli-v1", "AWS CLI cache", "AWS CLI", "Library/Caches/aws", ".cache/aws"),
        package_rule("cache.uv-v1", "uv package cache", "uv", ".cache/uv", ".cache/uv"),
        package_rule("cache.pre-commit-v1", "pre-commit environments", "pre-commit", ".cache/pre-commit", ".cache/pre-commit"),
        package_rule("cache.cocoapods-v1", "CocoaPods cache", "CocoaPods", "Library/Caches/CocoaPods", ".cache/CocoaPods"),
        package_rule("cache.swiftpm-v1", "SwiftPM cache", "SwiftPM", "Library/Caches/org.swift.swiftpm", ".cache/org.swift.swiftpm"),
        package_rule("cache.cypress-v1", "Cypress browser cache", "Cypress", "Library/Caches/Cypress", ".cache/Cypress"),
        application_cache_rule("cache.updater.notion-todesktop-v1", "Notion updater cache", "Library/Caches/com.todesktop.230313mzl4w4u92.ShipIt", ".cache/notion-todesktop-shipit"),
        application_cache_rule("cache.updater.windsurf-v1", "Windsurf updater cache", "Library/Caches/com.exafunction.windsurf.ShipIt", ".cache/windsurf-shipit"),
        application_cache_rule("cache.updater.claude-v1", "Claude updater cache", "Library/Caches/com.anthropic.claudefordesktop.ShipIt", ".cache/claude-shipit"),
        application_cache_rule("cache.updater.capacities-v1", "Capacities updater cache", "Library/Caches/capacities-updater", ".cache/capacities-updater"),
        application_cache_rule("cache.updater.notion-shipit-v1", "Notion ShipIt cache", "Library/Caches/notion.id.ShipIt", ".cache/notion-shipit"),
        application_cache_rule("cache.updater.whimsical-v1", "Whimsical updater cache", "Library/Caches/whimsical-updater", ".cache/whimsical-updater"),
        application_cache_rule("cache.updater.codex-v1", "Codex updater cache", "Library/Caches/com.openai.codex/org.sparkle-project.Sparkle", ".cache/openai-codex-updater"),
        application_cache_rule("cache.cursor.logs-v1", "Cursor logs", "Library/Application Support/Cursor/logs", ".config/Cursor/logs"),
        application_cache_rule("cache.cursor.cached-data-v1", "Cursor cached data", "Library/Application Support/Cursor/CachedData", ".config/Cursor/CachedData"),
        application_cache_rule("cache.vscode.extension-vsix-v1", "Visual Studio Code extension downloads", "Library/Application Support/Code/CachedExtensionVSIXs", ".config/Code/CachedExtensionVSIXs"),
        application_cache_rule("cache.cursor.extension-vsix-v1", "Cursor extension downloads", "Library/Application Support/Cursor/CachedExtensionVSIXs", ".config/Cursor/CachedExtensionVSIXs"),
    ]
}

fn application_cache_rule(
    id: &str,
    display_name: &str,
    macos_path: &'static str,
    linux_path: &'static str,
) -> CacheRuleSpec {
    build_rule(
        id,
        display_name,
        "Regenerable application cache. Close the owning application before cleanup; the next launch may rebuild or download this data.",
        RuleCategory::ApplicationCache,
        RegenerationBehavior::RestartRequired,
        macos_path,
        linux_path,
    )
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
    fn catalog_has_at_least_thirty_unique_safe_rules() {
        let rules = catalog();
        let ids: HashSet<_> = rules
            .iter()
            .map(|rule| rule.definition.id.as_str())
            .collect();
        assert!(rules.len() >= 30);
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
