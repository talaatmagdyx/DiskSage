use std::path::{Component, Path, PathBuf};

use crate::domain::error::{CommandError, ErrorCode};

#[derive(Debug, Clone, Default)]
pub struct ExclusionMatcher {
    paths: Vec<PathBuf>,
}

impl ExclusionMatcher {
    pub fn new(paths: &[String]) -> Result<Self, CommandError> {
        if paths.len() > 64 {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "A scan can contain at most 64 excluded paths.",
                true,
            ));
        }
        let paths = paths
            .iter()
            .map(|path| {
                normalize(Path::new(path)).ok_or_else(|| {
                    CommandError::new(
                        ErrorCode::InvalidPath,
                        "Excluded paths must be absolute.",
                        true,
                    )
                    .with_path(path)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { paths })
    }

    pub fn is_excluded(&self, candidate: &Path) -> bool {
        normalize(candidate).is_some_and(|candidate| {
            self.paths
                .iter()
                .any(|excluded| candidate.starts_with(excluded))
        })
    }

    pub fn with_additional_paths(&self, paths: &[PathBuf]) -> Self {
        let mut combined = self.paths.clone();
        combined.extend(paths.iter().filter_map(|path| normalize(path)));
        combined.sort();
        combined.dedup();
        Self { paths: combined }
    }
}

fn normalize(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str())
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
        }
    }
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusion_applies_before_descending() {
        let matcher = ExclusionMatcher::new(&["/tmp/cache/keep".to_owned()]).unwrap();
        assert!(matcher.is_excluded(Path::new("/tmp/cache/keep/nested")));
        assert!(!matcher.is_excluded(Path::new("/tmp/cache/remove")));
    }

    #[test]
    fn relative_exclusion_is_rejected() {
        assert_eq!(
            ExclusionMatcher::new(&["relative".to_owned()])
                .unwrap_err()
                .code,
            ErrorCode::InvalidPath
        );
    }
}
