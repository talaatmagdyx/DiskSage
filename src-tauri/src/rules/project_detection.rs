use std::{
    fs,
    path::{Path, PathBuf},
};

use super::catalogs::project_artifacts::PROJECT_INDICATORS;

const MAX_DEPTH: usize = 4;
const MAX_PROJECTS: usize = 200;

pub fn detect(configured_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut projects = Vec::new();
    for configured in configured_roots {
        if projects.len() >= MAX_PROJECTS {
            break;
        }
        let Ok(metadata) = fs::symlink_metadata(configured) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let mut pending = vec![(configured.clone(), 0_usize)];
        while let Some((path, depth)) = pending.pop() {
            if projects.len() >= MAX_PROJECTS {
                break;
            }
            if is_project(&path) {
                projects.push(path.clone());
            }
            if depth >= MAX_DEPTH {
                continue;
            }
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                let child = entry.path();
                if should_skip(&child) {
                    continue;
                }
                if let Ok(metadata) = fs::symlink_metadata(&child) {
                    if metadata.is_dir() && !metadata.file_type().is_symlink() {
                        pending.push((child, depth + 1));
                    }
                }
            }
        }
    }
    projects.sort();
    projects.dedup();
    projects
}

fn is_project(path: &Path) -> bool {
    PROJECT_INDICATORS
        .iter()
        .any(|indicator| path.join(indicator).is_file())
}

fn should_skip(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "node_modules" | "target" | "build" | "dist" | ".venv" | "Pods")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_manifest_context_without_descending_into_artifacts() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("nested/app");
        fs::create_dir_all(project.join("node_modules/ignored")).unwrap();
        fs::write(project.join("package.json"), b"{}").unwrap();
        fs::write(project.join("node_modules/ignored/package.json"), b"{}").unwrap();
        assert_eq!(detect(&[directory.path().to_path_buf()]), vec![project]);
    }
}
