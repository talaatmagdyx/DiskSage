use std::{fs, path::Path, time::SystemTime};

use crate::domain::error::{CommandError, ErrorCode};

use super::{cancellation::CancellationToken, exclusions::ExclusionMatcher};

#[derive(Debug, Default)]
pub struct TargetMeasurement {
    pub exists: bool,
    pub logical_size: u64,
    pub allocated_size: u64,
    pub files_scanned: u64,
    pub directories_scanned: u64,
    pub skipped_count: u64,
    pub permission_denied_count: u64,
    pub modified_at: Option<SystemTime>,
    pub errors: Vec<CommandError>,
}

pub fn measure_target(
    root: &Path,
    exclusions: &ExclusionMatcher,
    cancellation: &CancellationToken,
    mut progress: impl FnMut(&Path, &TargetMeasurement),
) -> TargetMeasurement {
    let mut measurement = TargetMeasurement::default();
    if exclusions.is_excluded(root) {
        return measurement;
    }
    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return measurement,
        Err(error) => {
            record_error(&mut measurement, root, error);
            return measurement;
        }
    };
    measurement.exists = true;
    let root_device = Some(device_id(&root_metadata));
    let mut pending = vec![root.to_path_buf()];

    while let Some(path) = pending.pop() {
        if cancellation.is_cancelled() {
            break;
        }
        if exclusions.is_excluded(&path) {
            measurement.skipped_count += 1;
            continue;
        }
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                record_error(&mut measurement, &path, error);
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            measurement.skipped_count += 1;
            continue;
        }
        if metadata.is_file() {
            measurement.files_scanned += 1;
            measurement.logical_size = measurement.logical_size.saturating_add(metadata.len());
            measurement.allocated_size = measurement
                .allocated_size
                .saturating_add(allocated_size(&metadata));
            if let Ok(modified) = metadata.modified() {
                measurement.modified_at = Some(
                    measurement
                        .modified_at
                        .map_or(modified, |current| current.max(modified)),
                );
            }
        } else if metadata.is_dir() {
            if path != root && root_device.is_some_and(|device| device != device_id(&metadata)) {
                measurement.skipped_count += 1;
                continue;
            }
            measurement.directories_scanned += 1;
            match fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries {
                        if cancellation.is_cancelled() {
                            break;
                        }
                        match entry {
                            Ok(entry) => pending.push(entry.path()),
                            Err(error) => record_error(&mut measurement, &path, error),
                        }
                    }
                }
                Err(error) => record_error(&mut measurement, &path, error),
            }
        } else {
            measurement.skipped_count += 1;
        }
        progress(&path, &measurement);
    }
    measurement
}

fn record_error(measurement: &mut TargetMeasurement, path: &Path, error: std::io::Error) {
    let (code, message) = if error.kind() == std::io::ErrorKind::PermissionDenied {
        measurement.permission_denied_count += 1;
        (
            ErrorCode::PermissionDenied,
            "A location could not be read because of its permissions.",
        )
    } else {
        measurement.skipped_count += 1;
        (
            ErrorCode::FilesystemError,
            "A filesystem item changed or could not be read.",
        )
    };
    if measurement.errors.len() < 20 {
        measurement.errors.push(
            CommandError::new(code, message, true)
                .with_path(path.to_string_lossy())
                .with_details(error.to_string()),
        );
    }
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
    use std::io::Write;

    #[test]
    fn measures_files_without_following_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let file_path = directory.path().join("cache.bin");
        fs::File::create(&file_path)
            .unwrap()
            .write_all(&[1_u8; 64])
            .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, directory.path().join("link")).unwrap();
        let result = measure_target(
            directory.path(),
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
        );
        assert_eq!(result.logical_size, 64);
        assert_eq!(result.files_scanned, 1);
        #[cfg(unix)]
        assert_eq!(result.skipped_count, 1);
    }

    #[cfg(unix)]
    #[test]
    fn distinguishes_sparse_logical_and_allocated_size() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("virtual-disk.raw");
        let file = fs::File::create(&file_path).unwrap();
        file.set_len(256 * 1024 * 1024).unwrap();

        let result = measure_target(
            &file_path,
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
        );

        assert_eq!(result.logical_size, 256 * 1024 * 1024);
        assert!(result.allocated_size < result.logical_size);
    }

    #[test]
    fn cancellation_stops_an_active_walk() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..100 {
            fs::write(dir.path().join(format!("fixture-{index}")), b"cache").unwrap();
        }
        let token = CancellationToken::default();
        let result = measure_target(
            dir.path(),
            &ExclusionMatcher::default(),
            &token,
            |_, measurement| {
                if measurement.files_scanned > 0 {
                    token.cancel();
                }
            },
        );

        assert!(result.files_scanned > 0);
        assert!(result.files_scanned < 100);
    }

    #[test]
    fn permission_denial_is_counted_without_aborting_the_scan() {
        let mut measurement = TargetMeasurement::default();
        record_error(
            &mut measurement,
            Path::new("restricted"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "fixture"),
        );

        assert_eq!(measurement.permission_denied_count, 1);
        assert_eq!(measurement.errors.len(), 1);
        assert_eq!(measurement.errors[0].code, ErrorCode::PermissionDenied);
    }

    #[test]
    #[ignore = "release-candidate performance fixture"]
    fn scans_one_hundred_thousand_files() {
        let dir = tempfile::tempdir().unwrap();
        for index in 0..100_000 {
            fs::File::create(dir.path().join(format!("fixture-{index}"))).unwrap();
        }
        let started = std::time::Instant::now();
        let result = measure_target(
            dir.path(),
            &ExclusionMatcher::default(),
            &CancellationToken::default(),
            |_, _| {},
        );

        assert_eq!(result.files_scanned, 100_000);
        eprintln!("scanned 100,000 files in {:?}", started.elapsed());
    }

    #[test]
    fn cancellation_stops_before_work() {
        let directory = tempfile::tempdir().unwrap();
        let token = CancellationToken::default();
        token.cancel();
        let result = measure_target(
            directory.path(),
            &ExclusionMatcher::default(),
            &token,
            |_, _| {},
        );
        assert_eq!(result.files_scanned, 0);
        assert_eq!(result.directories_scanned, 0);
    }
}
