use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use chrono::Utc;

use crate::domain::{
    error::{CommandError, ErrorCode},
    intelligence::{
        OrphanedApplicationData, PermissionAccess, PermissionLocation, PermissionReport,
        StorageMapEntry, StorageMapReport, StorageMapRequest,
    },
};

const MAP_MAX_ENTRIES: u64 = 150_000;
const MAP_MAX_DURATION: Duration = Duration::from_secs(15);
const MAP_MAX_RESULTS: usize = 24;
const ORPHAN_MAX_ENTRIES: u64 = 75_000;
const ORPHAN_MAX_DURATION: Duration = Duration::from_secs(8);
const ORPHAN_MAX_RESULTS: usize = 200;

#[derive(Debug, Default)]
struct Measurement {
    logical_size: u64,
    allocated_size: u64,
    files_scanned: u64,
    directories_scanned: u64,
    permission_denied_count: u64,
    truncated: bool,
}

struct Budget {
    remaining_entries: u64,
    deadline: Instant,
}

pub fn permission_report(home: &Path, platform: &str) -> Result<PermissionReport, CommandError> {
    if platform != "macos" {
        return Err(CommandError::new(
            ErrorCode::CommandUnavailable,
            "Permission Center is currently available on macOS.",
            true,
        ));
    }
    let checks = [
        ("Home folder", home.to_path_buf(), false),
        (
            "Application Support",
            home.join("Library/Application Support"),
            false,
        ),
        ("App Containers", home.join("Library/Containers"), true),
        (
            "Shared Group Containers",
            home.join("Library/Group Containers"),
            true,
        ),
    ];
    let locations: Vec<_> = checks
        .into_iter()
        .map(|(label, path, restricted)| permission_location(label, &path, home, restricted))
        .collect();
    let full_disk_access_likely = locations
        .iter()
        .filter(|location| location.label.contains("Container"))
        .all(|location| location.access != PermissionAccess::Limited);
    Ok(PermissionReport {
        checked_at: Utc::now(),
        full_disk_access_likely,
        locations,
        note: "This is a read-only access check, not a macOS authorization guarantee. DiskSage never changes privacy settings automatically.".to_owned(),
    })
}

pub fn open_full_disk_access_settings(platform: &str) -> Result<(), CommandError> {
    if platform != "macos" {
        return Err(CommandError::new(
            ErrorCode::CommandUnavailable,
            "Full Disk Access settings are available only on macOS.",
            false,
        ));
    }
    let status = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
        .status()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(CommandError::new(
            ErrorCode::CommandUnavailable,
            "System Settings could not be opened. Open Privacy & Security > Full Disk Access manually.",
            true,
        ))
    }
}

pub fn scan_orphaned_application_data(
    home: &Path,
    installed_bundle_ids: &HashSet<String>,
) -> Result<Vec<OrphanedApplicationData>, CommandError> {
    let roots = [
        (home.join("Library/Caches"), "Cache", "directory name"),
        (
            home.join("Library/Containers"),
            "Container",
            "container identifier",
        ),
        (
            home.join("Library/Group Containers"),
            "Shared Group Container",
            "shared container identifier",
        ),
        (
            home.join("Library/Preferences"),
            "Preference",
            "preference identifier",
        ),
        (
            home.join("Library/Saved Application State"),
            "Saved Application State",
            "saved-state identifier",
        ),
    ];
    let mut budget = Budget {
        remaining_entries: ORPHAN_MAX_ENTRIES,
        deadline: Instant::now() + ORPHAN_MAX_DURATION,
    };
    let mut results = Vec::new();
    for (root, category, evidence) in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            if results.len() >= ORPHAN_MAX_RESULTS || budget_exhausted(&budget) {
                break;
            }
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                continue;
            }
            let Some(identifier) = application_identifier(&path, category) else {
                continue;
            };
            if !looks_like_bundle_identifier(&identifier)
                || identifier.starts_with("com.apple.")
                || identifier == "com.disksage.desktop"
                || identifier_is_installed(&identifier, installed_bundle_ids)
            {
                continue;
            }
            let root_device = device_id(&metadata);
            let measurement = measure_bounded(&path, root_device, &mut budget);
            results.push(OrphanedApplicationData {
                id: blake3::hash(path.to_string_lossy().as_bytes()).to_hex().to_string(),
                path: path.to_string_lossy().into_owned(),
                display_path: display_path(&path, home),
                identifier: identifier.clone(),
                category: category.to_owned(),
                logical_size: measurement.logical_size,
                allocated_size: measurement.allocated_size,
                reason: format!(
                    "No currently scanned application directly matches this {evidence}. Ownership is uncertain, so this item is review-only and never selected automatically."
                ),
                default_selected: false,
            });
        }
    }
    results.sort_by_key(|item| std::cmp::Reverse(item.allocated_size));
    Ok(results)
}

pub fn storage_map(
    home: &Path,
    request: &StorageMapRequest,
) -> Result<StorageMapReport, CommandError> {
    let started = Instant::now();
    let requested = request
        .root
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| home.to_path_buf());
    if !requested.is_absolute() {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "Storage Map requires an absolute folder path.",
            false,
        ));
    }
    let requested_metadata =
        fs::symlink_metadata(&requested).map_err(|error| filesystem_error(&requested, error))?;
    if requested_metadata.file_type().is_symlink() || !requested_metadata.is_dir() {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "Storage Map requires a real folder and never follows symbolic links.",
            false,
        )
        .with_path(display_path(&requested, home)));
    }
    let root = requested
        .canonicalize()
        .map_err(|error| filesystem_error(&requested, error))?;
    let canonical_home = home
        .canonicalize()
        .map_err(|error| filesystem_error(home, error))?;
    if !root.starts_with(&canonical_home) {
        return Err(CommandError::new(
            ErrorCode::PathProtected,
            "Storage Map is limited to your home folder and folders inside it.",
            false,
        )
        .with_path(root.to_string_lossy()));
    }
    let root_device = device_id(&requested_metadata);
    let mut budget = Budget {
        remaining_entries: MAP_MAX_ENTRIES,
        deadline: Instant::now() + MAP_MAX_DURATION,
    };
    let entries = fs::read_dir(&root).map_err(|error| filesystem_error(&root, error))?;
    let mut mapped = Vec::new();
    let mut report_truncated = false;
    for entry in entries.flatten() {
        if budget_exhausted(&budget) {
            report_truncated = true;
            break;
        }
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || device_id(&metadata) != root_device {
            continue;
        }
        let measurement = measure_bounded(&path, root_device, &mut budget);
        mapped.push(StorageMapEntry {
            id: blake3::hash(path.to_string_lossy().as_bytes())
                .to_hex()
                .to_string(),
            name: entry.file_name().to_string_lossy().into_owned(),
            path: path.to_string_lossy().into_owned(),
            display_path: display_path(&path, home),
            logical_size: measurement.logical_size,
            allocated_size: measurement.allocated_size,
            files_scanned: measurement.files_scanned,
            directories_scanned: measurement.directories_scanned,
            permission_denied_count: measurement.permission_denied_count,
            truncated: measurement.truncated,
        });
    }
    mapped.sort_by_key(|entry| std::cmp::Reverse(entry.allocated_size));
    if mapped.len() > MAP_MAX_RESULTS {
        mapped.truncate(MAP_MAX_RESULTS);
        report_truncated = true;
    }
    let logical_size = mapped.iter().map(|entry| entry.logical_size).sum();
    let allocated_size = mapped.iter().map(|entry| entry.allocated_size).sum();
    let files_scanned = mapped.iter().map(|entry| entry.files_scanned).sum();
    let directories_scanned = mapped.iter().map(|entry| entry.directories_scanned).sum();
    let permission_denied_count = mapped
        .iter()
        .map(|entry| entry.permission_denied_count)
        .sum();
    report_truncated |= mapped.iter().any(|entry| entry.truncated);
    Ok(StorageMapReport {
        root: root.to_string_lossy().into_owned(),
        display_root: display_path(&root, home),
        entries: mapped,
        logical_size,
        allocated_size,
        files_scanned,
        directories_scanned,
        permission_denied_count,
        truncated: report_truncated,
        elapsed_ms: started.elapsed().as_millis() as u64,
        note: "Allocated size reflects blocks currently used by analyzed files. It is not a cleanup recommendation, and APFS snapshots or Trash retention can delay free-space changes.".to_owned(),
    })
}

fn permission_location(
    label: &str,
    path: &Path,
    home: &Path,
    restricted: bool,
) -> PermissionLocation {
    let (access, guidance) = match fs::read_dir(path) {
        Ok(_) => (
            PermissionAccess::Available,
            if restricted {
                "Readable now. macOS can still require approval for individual protected items."
            } else {
                "Readable now."
            },
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            PermissionAccess::NotPresent,
            "This location is not present on this Mac."
        ),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => (
            PermissionAccess::Limited,
            "Access is limited. Grant DiskSage Full Disk Access in System Settings > Privacy & Security, then check again."
        ),
        Err(_) => (
            PermissionAccess::Limited,
            "This location could not be read. Check macOS privacy permissions and try again."
        ),
    };
    PermissionLocation {
        label: label.to_owned(),
        display_path: display_path(path, home),
        access,
        guidance: guidance.to_owned(),
    }
}

fn application_identifier(path: &Path, category: &str) -> Option<String> {
    let name = path.file_name()?.to_string_lossy();
    let identifier = match category {
        "Preference" => name.strip_suffix(".plist")?,
        "Saved Application State" => name.strip_suffix(".savedState")?,
        _ => &name,
    };
    Some(identifier.to_owned())
}

fn looks_like_bundle_identifier(value: &str) -> bool {
    value.contains('.')
        && !value.chars().any(char::is_whitespace)
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.chars().all(|character| {
                    character.is_ascii_alphanumeric() || character == '-' || character == '_'
                })
        })
}

fn identifier_is_installed(identifier: &str, installed: &HashSet<String>) -> bool {
    installed.contains(identifier)
        || identifier
            .strip_prefix("group.")
            .is_some_and(|value| installed.contains(value))
}

fn measure_bounded(path: &Path, root_device: u64, budget: &mut Budget) -> Measurement {
    let mut measurement = Measurement::default();
    let mut pending = vec![(path.to_path_buf(), 0_u8)];
    while let Some((current, depth)) = pending.pop() {
        if budget_exhausted(budget) {
            measurement.truncated = true;
            break;
        }
        budget.remaining_entries = budget.remaining_entries.saturating_sub(1);
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::PermissionDenied {
                    measurement.permission_denied_count += 1;
                }
                continue;
            }
        };
        if metadata.file_type().is_symlink() || device_id(&metadata) != root_device {
            continue;
        }
        if metadata.is_file() {
            measurement.files_scanned += 1;
            measurement.logical_size = measurement.logical_size.saturating_add(metadata.len());
            measurement.allocated_size = measurement
                .allocated_size
                .saturating_add(allocated_size(&metadata));
        } else if metadata.is_dir() {
            measurement.directories_scanned += 1;
            if depth >= 32 {
                measurement.truncated = true;
                continue;
            }
            match fs::read_dir(&current) {
                Ok(entries) => {
                    pending.extend(entries.flatten().map(|entry| (entry.path(), depth + 1)))
                }
                Err(error) => {
                    if error.kind() == std::io::ErrorKind::PermissionDenied {
                        measurement.permission_denied_count += 1;
                    }
                }
            }
        }
    }
    measurement
}

fn budget_exhausted(budget: &Budget) -> bool {
    budget.remaining_entries == 0 || Instant::now() >= budget.deadline
}

fn display_path(path: &Path, home: &Path) -> String {
    path.strip_prefix(home)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                "~".to_owned()
            } else {
                format!("~/{}", relative.to_string_lossy())
            }
        })
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

fn filesystem_error(path: &Path, error: std::io::Error) -> CommandError {
    CommandError::new(
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            ErrorCode::PermissionDenied
        } else if error.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::PathNotFound
        } else {
            ErrorCode::FilesystemError
        },
        "The requested folder could not be analyzed.",
        true,
    )
    .with_path(path.to_string_lossy())
    .with_details(error.to_string())
}

#[cfg(unix)]
fn device_id(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.dev()
}

#[cfg(not(unix))]
fn device_id(_metadata: &fs::Metadata) -> u64 {
    0
}

#[cfg(unix)]
fn allocated_size(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn allocated_size(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_identifier_filter_is_narrow() {
        assert!(looks_like_bundle_identifier("com.example.fixture"));
        assert!(looks_like_bundle_identifier("group.com.example.fixture"));
        assert!(!looks_like_bundle_identifier("Application Support"));
        assert!(!looks_like_bundle_identifier("single"));
    }

    #[test]
    fn storage_map_rejects_roots_outside_home() {
        let home = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let error = storage_map(
            home.path(),
            &StorageMapRequest {
                root: Some(outside.path().to_string_lossy().into_owned()),
            },
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::PathProtected);
    }

    #[test]
    fn storage_map_measures_immediate_children_without_following_symlinks() {
        let home = tempfile::tempdir().unwrap();
        let folder = home.path().join("Documents");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("fixture.bin"), vec![1_u8; 4096]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&folder, home.path().join("linked")).unwrap();
        let report = storage_map(home.path(), &StorageMapRequest { root: None }).unwrap();
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].name, "Documents");
        assert_eq!(report.entries[0].files_scanned, 1);
    }

    #[test]
    fn orphan_scan_excludes_identifiers_owned_by_installed_apps() {
        let home = tempfile::tempdir().unwrap();
        let orphan = home.path().join("Library/Caches/com.example.retired");
        let installed = home.path().join("Library/Caches/com.example.current");
        fs::create_dir_all(&orphan).unwrap();
        fs::create_dir_all(&installed).unwrap();
        fs::write(orphan.join("cache.bin"), vec![1_u8; 1024]).unwrap();
        fs::write(installed.join("cache.bin"), vec![1_u8; 1024]).unwrap();
        let installed_ids = HashSet::from(["com.example.current".to_owned()]);

        let results = scan_orphaned_application_data(home.path(), &installed_ids).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].identifier, "com.example.retired");
        assert!(!results[0].default_selected);
    }

    #[test]
    fn permission_report_classifies_readable_and_missing_locations() {
        let home = tempfile::tempdir().unwrap();
        fs::create_dir_all(home.path().join("Library/Application Support")).unwrap();
        fs::create_dir_all(home.path().join("Library/Containers")).unwrap();

        let report = permission_report(home.path(), "macos").unwrap();

        assert_eq!(report.locations[0].access, PermissionAccess::Available);
        assert_eq!(report.locations[2].access, PermissionAccess::Available);
        assert_eq!(report.locations[3].access, PermissionAccess::NotPresent);
    }
}
