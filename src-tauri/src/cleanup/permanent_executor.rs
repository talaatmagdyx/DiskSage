use std::{fs, path::Path};

use crate::domain::{
    error::{CommandError, ErrorCode},
    finding::FindingType,
};

pub fn permanently_delete(path: &Path, expected_type: FindingType) -> Result<(), CommandError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| delete_error(path, error))?;
    if metadata.file_type().is_symlink() {
        return Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "Symbolic links are not supported by permanent deletion.",
            false,
        )
        .with_path(path.to_string_lossy()));
    }
    match expected_type {
        FindingType::File if metadata.is_file() => {
            fs::remove_file(path).map_err(|error| delete_error(path, error))
        }
        FindingType::Directory if metadata.is_dir() => {
            fs::remove_dir_all(path).map_err(|error| delete_error(path, error))
        }
        _ => Err(CommandError::new(
            ErrorCode::PlanValidationFailed,
            "The permanent-delete item type changed during execution.",
            false,
        )
        .with_path(path.to_string_lossy())),
    }
}

fn delete_error(path: &Path, error: std::io::Error) -> CommandError {
    CommandError::new(
        ErrorCode::DeleteFailed,
        "The item could not be permanently deleted. No retry was attempted.",
        false,
    )
    .with_path(path.to_string_lossy())
    .with_details(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_a_regular_file_once() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("file");
        fs::write(&path, b"fixture").unwrap();
        permanently_delete(&path, FindingType::File).unwrap();
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_follow_a_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target");
        fs::write(&target, b"keep").unwrap();
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(permanently_delete(&link, FindingType::File).is_err());
        assert!(target.exists());
    }
}
