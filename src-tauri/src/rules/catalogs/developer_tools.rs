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
            RecommendedAction::MoveToTrash,
            RegenerationBehavior::Reindex,
        ),
        tool(
            "inspection.xcode.archives-v1",
            "Xcode Archives",
            Some("Library/Developer/Xcode/Archives"),
            None,
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::NotRegenerable,
        ),
        tool(
            "inspection.xcode.device-support-v1",
            "Xcode iOS DeviceSupport",
            Some("Library/Developer/Xcode/iOS DeviceSupport"),
            None,
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.ide.jetbrains-cache-v1",
            "JetBrains IDE caches",
            Some("Library/Caches/JetBrains"),
            Some(".cache/JetBrains"),
            RiskLevel::Careful,
            RecommendedAction::MoveToTrash,
            RegenerationBehavior::Reindex,
        ),
        tool(
            "inspection.ide.vscode-cache-v1",
            "Visual Studio Code cache",
            Some("Library/Application Support/Code/Cache"),
            Some(".config/Code/Cache"),
            RiskLevel::Careful,
            RecommendedAction::MoveToTrash,
            RegenerationBehavior::RestartRequired,
        ),
        tool(
            "inspection.android.sdk-temp-v1",
            "Android SDK temporary data",
            Some("Library/Android/sdk/.temp"),
            Some("Android/Sdk/.temp"),
            RiskLevel::Careful,
            RecommendedAction::MoveToTrash,
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
        tool(
            "inspection.cursor.state-db-v1",
            "Cursor state database",
            Some("Library/Application Support/Cursor/User/globalStorage/state.vscdb"),
            Some(".config/Cursor/User/globalStorage/state.vscdb"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.codex.runtimes-v1",
            "Codex workspace runtimes",
            Some(".cache/codex-runtimes"),
            Some(".cache/codex-runtimes"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.playwright.browsers-v1",
            "Playwright browsers and profiles",
            Some("Library/Caches/ms-playwright"),
            Some(".cache/ms-playwright"),
            RiskLevel::Careful,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.zed.node-runtime-v1",
            "Zed bundled Node runtime",
            Some("Library/Application Support/Zed/node"),
            Some(".local/share/zed/node"),
            RiskLevel::Careful,
            RecommendedAction::MoveToTrash,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.zed.language-runtimes-v1",
            "Zed language runtimes",
            Some("Library/Application Support/Zed/languages"),
            Some(".local/share/zed/languages"),
            RiskLevel::Careful,
            RecommendedAction::MoveToTrash,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.user.dot-cache-v1",
            "Unclassified user cache remainder",
            Some(".cache"),
            Some(".cache"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.jetbrains.versioned-data-v1",
            "JetBrains versioned application data",
            Some("Library/Application Support/JetBrains"),
            Some(".config/JetBrains"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.notion.partitions-v1",
            "Notion offline partitions",
            Some("Library/Application Support/Notion/Partitions"),
            Some(".config/Notion/Partitions"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.claude.sessions-v1",
            "Claude local agent sessions",
            Some("Library/Application Support/Claude/local-agent-mode-sessions"),
            None,
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.claude.vm-bundles-v1",
            "Claude VM bundles",
            Some("Library/Application Support/Claude/vm_bundles"),
            None,
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.swiftpm.repositories-v1",
            "SwiftPM repository data",
            Some("Library/org.swift.swiftpm"),
            Some(".local/share/org.swift.swiftpm"),
            RiskLevel::Careful,
            RecommendedAction::Review,
            RegenerationBehavior::Redownload,
        ),
        tool(
            "inspection.runtime.rbenv-v1",
            "rbenv installations",
            Some(".rbenv"),
            Some(".rbenv"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.runtime.asdf-v1",
            "asdf installations",
            Some(".asdf"),
            Some(".asdf"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.runtime.mise-v1",
            "mise language installations",
            Some(".local/share/mise/installs"),
            Some(".local/share/mise/installs"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.kubernetes.minikube-v1",
            "minikube local cluster data",
            Some(".minikube"),
            Some(".minikube"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.ml.unsloth-v1",
            "Unsloth models and training data",
            Some(".unsloth"),
            Some(".unsloth"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.ml.huggingface-v1",
            "Hugging Face cache and models",
            Some(".cache/huggingface"),
            Some(".cache/huggingface"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.ml.pytorch-v1",
            "PyTorch cache and models",
            Some(".cache/torch"),
            Some(".cache/torch"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.ml.ollama-v1",
            "Ollama models",
            Some(".ollama/models"),
            None,
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.container.colima-v1",
            "Colima virtual machines and container data",
            Some(".colima"),
            Some(".colima"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.container.lima-v1",
            "Lima virtual machines",
            Some(".lima"),
            Some(".lima"),
            RiskLevel::Expert,
            RecommendedAction::GuidedCommand,
            RegenerationBehavior::PotentialStateLoss,
        ),
        tool(
            "inspection.container.orbstack-v1",
            "OrbStack container and machine data",
            Some("Library/Group Containers/HUAQ24HBR6.dev.orbstack"),
            None,
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
    let description = if id.contains("xcode-derived-data") {
        "Regenerable Xcode build and index output. Quit Xcode first; manual selection requires Careful confirmation and the next build may be slower."
    } else if id.contains("jetbrains-cache") {
        "Regenerable JetBrains indexes and caches. Quit every JetBrains IDE first; manual selection requires Careful confirmation and reindexing may take time."
    } else if id.contains("vscode-cache") {
        "Regenerable Visual Studio Code cache. Quit Visual Studio Code first; manual selection requires Careful confirmation."
    } else if id.contains("android.sdk-temp") {
        "Android SDK temporary downloads and staging data. Quit Android Studio first; manual selection requires Careful confirmation and downloads may restart."
    } else if id.contains("docker") {
        "Docker Desktop virtual disk inspection only. Review `docker system df`; DiskSage never removes Docker.raw directly."
    } else if id.contains("cursor.state-db") {
        "Cursor's live SQLite state database. Close Cursor and inspect a guided VACUUM workflow; DiskSage never moves or modifies the database directly. VACUUM may require substantial temporary free space."
    } else if id.contains("codex.runtimes") {
        "Codex-managed workspace runtimes may be in use by active tasks. Close Codex and verify the runtime is no longer needed; DiskSage never selects it for cleanup."
    } else if id.contains("claude.sessions") {
        "Claude session data may include the currently active task. DiskSage cannot reliably infer ownership or activity, so this is inspection-only and never cleanup-authorized."
    } else if id.contains("claude.vm-bundles") {
        "Claude VM bundles power its local sandbox and may be active. Removing them can interrupt or break Claude until they are rebuilt, so DiskSage provides inspection only."
    } else if id.contains("playwright") {
        "Playwright browser downloads can be reinstalled, but this cache can also contain persistent MCP browser profiles and signed-in state. Review `npx playwright uninstall --all` and profile ownership first."
    } else if id.contains("xcode.archives") {
        "Xcode Archives may contain the only retained build used for distribution, symbolication, or crash reports. Inspect them in Xcode Organizer; they are not automatically regenerable."
    } else if id.contains("xcode.device-support") {
        "Xcode downloads device-support files again when required. Remove obsolete OS versions through Xcode after confirming no connected device or debugging workflow needs them."
    } else if id.contains("swiftpm.repositories") {
        "SwiftPM repository data can normally be fetched again, but custom or offline sources may be costly to restore. Review it separately from the safe SwiftPM download cache."
    } else if id.contains("mise") {
        "Language installations managed by mise. Review `mise prune --dry-run`; DiskSage never removes runtimes directly."
    } else if id.contains("rbenv") || id.contains("asdf") {
        "Installed language runtimes may still be referenced by projects or shells. Verify configuration with the owning version manager before removal."
    } else if id.contains("minikube") {
        "Local Kubernetes clusters can contain workloads and volumes. Inspect with minikube before using its guided deletion commands."
    } else if id.contains("unsloth") {
        "Local ML models and training artifacts may be expensive or impossible to reproduce. Review ownership before removal."
    } else if id.contains("huggingface") || id.contains("pytorch") {
        "Model weights, datasets, checkpoints, and compiled artifacts may be large, private, or expensive to reproduce. Use the owning ML tool to inspect them; DiskSage never selects them automatically."
    } else if id.contains("ollama") {
        "Installed Ollama models can be tens or hundreds of gigabytes. Review `ollama list` and remove individual models with `ollama rm`; DiskSage never deletes the model store directly."
    } else if id.contains("colima") {
        "Colima virtual machines may contain images, containers, Kubernetes state, and persistent volumes. Inspect with `colima status` and use `colima delete --data` only when intentional."
    } else if id.contains("container.lima") {
        "Lima instances are virtual machines that may contain irreplaceable local state. Inspect and remove instances with `limactl`; DiskSage never deletes their storage directly."
    } else if id.contains("orbstack") {
        "OrbStack data can include containers, images, volumes, and Linux machines. Use OrbStack's own management interface for cleanup; DiskSage provides inspection only."
    } else if id.contains("notion.partitions") {
        "Notion offline data may require a full cloud re-sync. Close Notion and confirm all important content is synchronized before review."
    } else if id.contains("user.dot-cache") {
        "Mixed cache data not covered by a more specific finding. Known child caches are excluded from this measurement to prevent double-counting; the remainder is review-only."
    } else if id.contains("jetbrains.versioned-data") {
        "Versioned JetBrains application data can contain settings and plugins as well as caches. Confirm an IDE version is unused before removing it with JetBrains tools."
    } else if id.contains("zed") {
        "Zed-managed runtimes can be downloaded again, but removal interrupts language tooling until Zed rebuilds them."
    } else if id.contains("android") || id.contains("simulator") {
        "Emulator state inspection only. Use the platform's device manager for any removal."
    } else {
        "IDE cache inspection only. Rebuilding may require downloads, reindexing, or an application restart."
    };
    DeveloperToolSpec {
        definition: RuleDefinition {
            id: id.to_owned(),
            version: 1,
            category: if id.contains("docker")
                || id.contains("minikube")
                || id.contains("vm-bundles")
                || id.contains("container.colima")
                || id.contains("container.lima")
                || id.contains("orbstack")
            {
                RuleCategory::Container
            } else if id.contains("android")
                || id.contains("simulator")
                || id.contains("device-support")
            {
                RuleCategory::Emulator
            } else if id.contains("xcode.archives") {
                RuleCategory::BuildArtifact
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
