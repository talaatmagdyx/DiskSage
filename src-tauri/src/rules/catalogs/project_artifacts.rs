use crate::domain::rule::{
    Platform, RecommendedAction, RegenerationBehavior, RiskLevel, RuleCategory, RuleDefinition,
    RuleMatcher, RuleTarget,
};

#[derive(Debug, Clone)]
pub struct ProjectArtifactSpec {
    pub definition: RuleDefinition,
    pub artifact: &'static str,
    pub indicators: &'static [&'static str],
}

pub fn catalog() -> Vec<ProjectArtifactSpec> {
    vec![
        artifact(
            "artifact.node.modules-v1",
            "Node dependencies",
            "node_modules",
            &["package.json"],
        ),
        artifact(
            "artifact.node.dist-v1",
            "Node distribution output",
            "dist",
            &["package.json"],
        ),
        artifact(
            "artifact.node.build-v1",
            "Node build output",
            "build",
            &["package.json"],
        ),
        artifact(
            "artifact.node.coverage-v1",
            "JavaScript coverage",
            "coverage",
            &["package.json"],
        ),
        artifact(
            "artifact.next.cache-v1",
            "Next.js cache",
            ".next/cache",
            &["package.json"],
        ),
        artifact(
            "artifact.nuxt.output-v1",
            "Nuxt generated output",
            ".nuxt",
            &["package.json"],
        ),
        artifact(
            "artifact.rust.target-v1",
            "Rust build output",
            "target",
            &["Cargo.toml"],
        ),
        artifact(
            "artifact.python.pytest-v1",
            "pytest cache",
            ".pytest_cache",
            &["pyproject.toml", "setup.py"],
        ),
        artifact(
            "artifact.python.mypy-v1",
            "mypy cache",
            ".mypy_cache",
            &["pyproject.toml", "setup.py"],
        ),
        artifact(
            "artifact.python.ruff-v1",
            "Ruff cache",
            ".ruff_cache",
            &["pyproject.toml", "setup.py"],
        ),
        artifact(
            "artifact.python.tox-v1",
            "tox environments",
            ".tox",
            &["pyproject.toml", "tox.ini"],
        ),
        artifact(
            "artifact.python.venv-v1",
            "Python virtual environment",
            ".venv",
            &["pyproject.toml", "requirements.txt"],
        ),
        artifact(
            "artifact.ruby.bundle-v1",
            "Bundler dependencies",
            "vendor/bundle",
            &["Gemfile"],
        ),
        artifact(
            "artifact.gradle.cache-v1",
            "Project Gradle cache",
            ".gradle",
            &["build.gradle", "build.gradle.kts"],
        ),
        artifact(
            "artifact.gradle.build-v1",
            "Gradle build output",
            "build",
            &["build.gradle", "build.gradle.kts"],
        ),
        artifact(
            "artifact.maven.target-v1",
            "Maven build output",
            "target",
            &["pom.xml"],
        ),
        artifact(
            "artifact.dart.tool-v1",
            "Dart tool state",
            ".dart_tool",
            &["pubspec.yaml"],
        ),
        artifact(
            "artifact.dart.build-v1",
            "Dart build output",
            "build",
            &["pubspec.yaml"],
        ),
        artifact(
            "artifact.ios.pods-v1",
            "CocoaPods dependencies",
            "Pods",
            &["Podfile"],
        ),
        artifact(
            "artifact.ios.carthage-v1",
            "Carthage build output",
            "Carthage/Build",
            &["Cartfile"],
        ),
        artifact(
            "artifact.project.tmp-v1",
            "Project temporary files",
            ".tmp",
            &PROJECT_INDICATORS,
        ),
        logs(
            "artifact.project.logs-v1",
            "Project logs",
            "logs",
            &PROJECT_INDICATORS,
        ),
    ]
}

pub const PROJECT_INDICATORS: [&str; 10] = [
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "Gemfile",
    "build.gradle",
    "build.gradle.kts",
    "pom.xml",
    "go.mod",
    "pubspec.yaml",
    "Podfile",
];

fn artifact(
    id: &str,
    name: &str,
    artifact: &'static str,
    indicators: &'static [&'static str],
) -> ProjectArtifactSpec {
    build(
        id,
        name,
        artifact,
        indicators,
        RuleCategory::BuildArtifact,
        RecommendedAction::MoveToTrash,
    )
}

fn logs(
    id: &str,
    name: &str,
    artifact: &'static str,
    indicators: &'static [&'static str],
) -> ProjectArtifactSpec {
    build(
        id,
        name,
        artifact,
        indicators,
        RuleCategory::Log,
        RecommendedAction::Review,
    )
}

fn build(
    id: &str,
    name: &str,
    artifact: &'static str,
    indicators: &'static [&'static str],
    category: RuleCategory,
    recommended_action: RecommendedAction,
) -> ProjectArtifactSpec {
    ProjectArtifactSpec {
        definition: RuleDefinition {
            id: id.to_owned(),
            version: 1,
            category,
            display_name: name.to_owned(),
            description: format!("Detected `{artifact}` only inside a configured root with a matching project manifest."),
            risk: RiskLevel::Careful,
            platforms: vec![Platform::Macos, Platform::Linux, Platform::Windows],
            targets: vec![RuleTarget::SelectedProjectRoots],
            matcher: RuleMatcher::ProjectArtifact {
                indicators: indicators.iter().map(|value| (*value).to_owned()).collect(),
                artifact: artifact.to_owned(),
            },
            minimum_size: Some(1),
            maximum_age_days: None,
            recommended_action,
            regeneration_behavior: RegenerationBehavior::Redownload,
            default_enabled: false,
        },
        artifact,
        indicators,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn has_at_least_twenty_contextual_careful_rules() {
        let rules = catalog();
        let ids: HashSet<_> = rules.iter().map(|rule| &rule.definition.id).collect();
        assert!(rules.len() >= 20);
        assert_eq!(ids.len(), rules.len());
        assert!(rules
            .iter()
            .all(|rule| rule.definition.risk == RiskLevel::Careful));
        assert!(rules.iter().all(|rule| !rule.definition.default_enabled));
        assert!(rules.iter().all(|rule| !rule.indicators.is_empty()));
    }
}
