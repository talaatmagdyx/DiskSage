use crate::domain::rule::{
    Platform, RecommendedAction, RegenerationBehavior, RiskLevel, RuleCategory, RuleDefinition,
    RuleMatcher, RuleTarget,
};

pub struct DeveloperToolSpec {
    pub definition: RuleDefinition,
    pub macos_path: Option<&'static str>,
    pub linux_path: Option<&'static str>,
}

pub fn catalog() -> Vec<DeveloperToolSpec> {
    vec![
        tool(
            "inspection.ide.xcode-derived-data-v1",
            "Xcode DerivedData",
            Some("Library/Developer/Xcode/DerivedData"),
            None,
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Reindex,
        ),
        tool(
            "inspection.ide.jetbrains-cache-v1",
            "JetBrains IDE caches",
            Some("Library/Caches/JetBrains"),
            Some(".cache/JetBrains"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Reindex,
        ),
        tool(
            "inspection.ide.vscode-cache-v1",
            "Visual Studio Code cache",
            Some("Library/Application Support/Code/Cache"),
            Some(".config/Code/Cache"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::RestartRequired,
        ),
        tool(
            "inspection.android.sdk-temp-v1",
            "Android SDK temporary data",
            Some("Library/Android/sdk/.temp"),
            Some("Android/Sdk/.temp"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.android.avd-v1",
            "Android virtual devices",
            Some(".android/avd"),
            Some(".android/avd"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.apple.simulators-v1",
            "Apple simulator devices",
            Some("Library/Developer/CoreSimulator/Devices"),
            None,
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.docker.raw-v1",
            "Docker Desktop virtual disk",
            Some("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"),
            Some(".docker/desktop/vms/0/data/Docker.raw"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
    ]
}

fn tool(
    id: &str,
    name: &str,
    macos_path: Option<&'static str>,
    linux_path: Option<&'static str>,
    risk: RiskLevel,
    action: RecommendedAction,
    regeneration_behavior: RegenerationBehavior,
) -> DeveloperToolSpec {
    let mut platforms = Vec::new();
    if macos_path.is_some() {
        platforms.push(Platform::Macos);
    }
    if linux_path.is_some() {
        platforms.push(Platform::Linux);
    }
    let paths = [macos_path, linux_path]
        .into_iter()
        .flatten()
        .map(|path| RuleTarget::HomeRelative {
            path: path.to_owned(),
        })
        .collect();
    let description = if id.contains("docker") {
        "Docker Desktop virtual disk inspection only. Review `docker system df`; DiskSage never removes Docker.raw directly."
    } else if id.contains("android") || id.contains("simulator") {
        "Emulator state inspection only. Use the platform's device manager for any removal."
    } else {
        "IDE cache inspection only. Rebuilding may require downloads, reindexing, or an application restart."
    };
    DeveloperToolSpec {
        definition: RuleDefinition {
            id: id.to_owned(),
            version: 1,
            category: if id.contains("docker") {
                RuleCategory::Container
            } else if id.contains("android") || id.contains("simulator") {
                RuleCategory::Emulator
            } else {
                RuleCategory::ApplicationCache
            },
            display_name: name.to_owned(),
            description: description.to_owned(),
            risk,
            platforms,
            targets: paths,
            matcher: RuleMatcher::ExactPath,
            minimum_size: Some(1),
            maximum_age_days: None,
            recommended_action: action,
            regeneration_behavior,
            default_enabled: false,
        },
        macos_path,
        linux_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_and_emulators_are_never_cleanup_defaults() {
        let rules = catalog();
        assert!(rules.iter().all(|rule| !rule.definition.default_enabled));
        assert!(rules
            .iter()
            .filter(|rule| rule.definition.risk == RiskLevel::Expert)
            .all(|rule| rule.definition.recommended_action == RecommendedAction::GuidedCommand));
        assert!(rules
            .iter()
            .flat_map(|rule| rule.definition.targets.iter())
            .all(|target| {
                matches!(target, RuleTarget::HomeRelative { path } if !path.starts_with('/'))
            }));
    }
}
