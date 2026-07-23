use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectionReason {
    pub protected_root: PathBuf,
    pub reason: &'static str,
}

#[derive(Debug, Clone)]
pub struct ProtectedPathPolicy {
    home: PathBuf,
    roots: Vec<(PathBuf, &'static str)>,
    case_insensitive: bool,
}

impl ProtectedPathPolicy {
    pub fn for_platform(home: &Path, platform: &str) -> Self {
        let mut roots = vec![
            (PathBuf::from("/"), "filesystem root"),
            (home.to_path_buf(), "home directory"),
            (home.join(".ssh"), "SSH credentials"),
            (home.join(".gnupg"), "GPG credentials"),
            (home.join(".aws"), "cloud credentials"),
            (home.join(".kube/config"), "Kubernetes credentials"),
            (home.join("Documents"), "user documents"),
            (home.join("Desktop"), "user desktop"),
            (home.join("Pictures"), "user pictures"),
            (home.join("Movies"), "user movies"),
            (home.join("Music"), "user music"),
        ];
        let system_roots: Vec<PathBuf> = match platform {
            "macos" => [
                "/System",
                "/usr",
                "/bin",
                "/sbin",
                "/etc",
                "/var",
                "/Applications",
                "/private/var/db",
            ]
            .into_iter()
            .map(PathBuf::from)
            .collect(),
            "linux" => [
                "/usr",
                "/bin",
                "/sbin",
                "/etc",
                "/var",
                "/proc",
                "/sys",
                "/dev",
                "/run",
                "/root",
                "/lost+found",
            ]
            .into_iter()
            .map(PathBuf::from)
            .collect(),
            "windows" => windows_system_roots(home),
            _ => Vec::new(),
        };
        roots.extend(system_roots.into_iter().map(|path| (path, "system path")));
        if platform == "macos" {
            roots.push((home.join("Library/Keychains"), "macOS keychains"));
            roots.push((home.join("Library/Mail"), "mail data"));
            roots.push((home.join("Library/Messages"), "message data"));
        }
        if platform == "windows" {
            roots.push((
                home.join("AppData/Roaming/Microsoft/Protect"),
                "Windows data-protection keys",
            ));
            roots.push((
                home.join("AppData/Roaming/Microsoft/Credentials"),
                "Windows credentials",
            ));
            roots.push((
                home.join("AppData/Local/Microsoft/Credentials"),
                "Windows credentials",
            ));
        }
        roots.sort_by(|left, right| {
            right
                .0
                .components()
                .count()
                .cmp(&left.0.components().count())
        });
        Self {
            home: home.to_path_buf(),
            roots,
            case_insensitive: platform == "windows",
        }
    }

    pub fn check(&self, candidate: &Path) -> Option<ProtectionReason> {
        let candidate = normalize_lexically(candidate)?;
        self.roots.iter().find_map(|(root, reason)| {
            let root = normalize_lexically(root)?;
            let is_filesystem_root = root.parent().is_none();
            if path_matches(&candidate, &root, is_filesystem_root, self.case_insensitive) {
                Some(ProtectionReason {
                    protected_root: root,
                    reason,
                })
            } else {
                None
            }
        })
    }

    pub fn check_cleanup_candidate(
        &self,
        candidate: &Path,
        exact_known_rule_target: bool,
    ) -> Option<ProtectionReason> {
        self.check(candidate).and_then(|reason| {
            if exact_known_rule_target && reason.protected_root == self.home {
                None
            } else {
                Some(reason)
            }
        })
    }
}

fn windows_system_roots(home: &Path) -> Vec<PathBuf> {
    let drive_root = home.ancestors().last().unwrap_or(home);
    [
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
        "Recovery",
        "System Volume Information",
        "$Recycle.Bin",
    ]
    .into_iter()
    .map(|relative| drive_root.join(relative))
    .collect()
}

fn path_matches(
    candidate: &Path,
    root: &Path,
    is_filesystem_root: bool,
    case_insensitive: bool,
) -> bool {
    if !case_insensitive {
        return candidate == root || (!is_filesystem_root && candidate.starts_with(root));
    }
    let candidate_components: Vec<_> = candidate.components().collect();
    let root_components: Vec<_> = root.components().collect();
    let root_matches = candidate_components
        .iter()
        .zip(&root_components)
        .all(|(left, right)| {
            left.as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
        });
    root_matches
        && (candidate_components.len() == root_components.len()
            || (!is_filesystem_root && candidate_components.len() > root_components.len()))
}

fn normalize_lexically(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
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

    #[cfg(unix)]
    #[test]
    fn blocks_root_home_and_sensitive_descendants() {
        let policy = ProtectedPathPolicy::for_platform(Path::new("/Users/alex"), "macos");
        assert!(policy.check(Path::new("/")).is_some());
        assert!(policy.check(Path::new("/Users/alex")).is_some());
        assert!(policy
            .check(Path::new("/Users/alex/.ssh/id_ed25519"))
            .is_some());
        assert!(policy.check(Path::new("/System/Library")).is_some());
    }

    #[cfg(unix)]
    #[test]
    fn blocks_lexical_traversal_into_protected_path() {
        let policy = ProtectedPathPolicy::for_platform(Path::new("/home/alex"), "linux");
        assert!(policy.check(Path::new("/tmp/../etc/passwd")).is_some());
    }

    #[cfg(unix)]
    #[test]
    fn allows_unprotected_cache_candidate_for_later_rule_validation() {
        let policy = ProtectedPathPolicy::for_platform(Path::new("/home/alex"), "linux");
        assert!(policy.check(Path::new("/opt/cache/npm")).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn allows_only_exact_known_children_inside_home() {
        let policy = ProtectedPathPolicy::for_platform(Path::new("/home/alex"), "linux");
        assert!(policy
            .check_cleanup_candidate(Path::new("/home/alex/.cache/npm"), true)
            .is_none());
        assert!(policy
            .check_cleanup_candidate(Path::new("/home/alex/.cache/npm"), false)
            .is_some());
        assert!(policy
            .check_cleanup_candidate(Path::new("/home/alex/.ssh/key"), true)
            .is_some());
    }

    #[test]
    fn relative_paths_are_not_accepted_for_policy_checks() {
        let policy = ProtectedPathPolicy::for_platform(Path::new("/home/alex"), "linux");
        assert!(policy.check(Path::new("relative/path")).is_none());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_policy_blocks_system_and_credential_paths_case_insensitively() {
        let policy = ProtectedPathPolicy::for_platform(Path::new(r"C:\Users\alex"), "windows");
        assert!(policy.check(Path::new(r"c:\WINDOWS\System32")).is_some());
        assert!(policy
            .check(Path::new(
                r"C:\Users\alex\AppData\Roaming\Microsoft\Credentials\fixture"
            ))
            .is_some());
        assert!(policy
            .check_cleanup_candidate(
                Path::new(r"C:\Users\alex\AppData\Local\npm-cache\_cacache"),
                true,
            )
            .is_none());
    }
}
