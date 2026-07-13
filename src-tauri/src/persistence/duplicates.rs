use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use uuid::Uuid;

use crate::domain::{
    duplicate::{DuplicateGroup, DuplicateSummary},
    error::{CommandError, ErrorCode},
};

#[derive(Debug, Clone)]
pub struct DuplicateRepository {
    root: PathBuf,
}

impl DuplicateRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn initialize(&self, scan_id: &str) -> Result<(), CommandError> {
        let directory = self.scan_directory(scan_id)?;
        fs::create_dir_all(&directory)
            .map_err(|error| self.io_error("create duplicate scan directory", error))?;
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(directory.join("groups.ndjson"))
            .map_err(|error| self.io_error("initialize duplicate groups", error))?;
        Ok(())
    }

    pub fn append_group(&self, group: &DuplicateGroup) -> Result<(), CommandError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.scan_directory(&group.scan_id)?.join("groups.ndjson"))
            .map_err(|error| self.io_error("open duplicate groups", error))?;
        serde_json::to_writer(&mut file, group).map_err(|error| self.serialization_error(error))?;
        file.write_all(b"\n")
            .map_err(|error| self.io_error("append duplicate group", error))
    }

    pub fn save_summary(&self, summary: &DuplicateSummary) -> Result<(), CommandError> {
        let directory = self.scan_directory(&summary.scan_id)?;
        fs::create_dir_all(&directory)
            .map_err(|error| self.io_error("create duplicate scan directory", error))?;
        let bytes =
            serde_json::to_vec_pretty(summary).map_err(|error| self.serialization_error(error))?;
        let temporary = directory.join("summary.json.tmp");
        fs::write(&temporary, bytes)
            .map_err(|error| self.io_error("write duplicate scan summary", error))?;
        fs::rename(temporary, directory.join("summary.json"))
            .map_err(|error| self.io_error("replace duplicate scan summary", error))
    }

    pub fn load_summary(&self, scan_id: &str) -> Result<DuplicateSummary, CommandError> {
        let bytes = fs::read(self.scan_directory(scan_id)?.join("summary.json"))
            .map_err(|error| self.io_error("read duplicate scan summary", error))?;
        serde_json::from_slice(&bytes).map_err(|error| self.serialization_error(error))
    }

    pub fn groups(
        &self,
        scan_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<DuplicateGroup>, CommandError> {
        if limit == 0 || limit > 500 {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "Duplicate group page size must be between 1 and 500.",
                true,
            ));
        }
        let file = fs::File::open(self.scan_directory(scan_id)?.join("groups.ndjson"))
            .map_err(|error| self.io_error("read duplicate groups", error))?;
        BufReader::new(file)
            .lines()
            .skip(offset)
            .take(limit)
            .map(|line| {
                let line = line.map_err(|error| self.io_error("read duplicate group", error))?;
                serde_json::from_str(&line).map_err(|error| self.serialization_error(error))
            })
            .collect()
    }

    pub fn group(&self, scan_id: &str, group_id: &str) -> Result<DuplicateGroup, CommandError> {
        Uuid::parse_str(group_id).map_err(|_| invalid_identifier("group"))?;
        let file = fs::File::open(self.scan_directory(scan_id)?.join("groups.ndjson"))
            .map_err(|error| self.io_error("read duplicate groups", error))?;
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| self.io_error("read duplicate group", error))?;
            let group: DuplicateGroup =
                serde_json::from_str(&line).map_err(|error| self.serialization_error(error))?;
            if group.id == group_id {
                return Ok(group);
            }
        }
        Err(CommandError::new(
            ErrorCode::PathNotFound,
            "The duplicate group no longer exists.",
            true,
        ))
    }

    fn scan_directory(&self, scan_id: &str) -> Result<PathBuf, CommandError> {
        Uuid::parse_str(scan_id).map_err(|_| invalid_identifier("scan"))?;
        Ok(self.root.join(scan_id))
    }

    fn io_error(&self, operation: &str, error: std::io::Error) -> CommandError {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::FilesystemError,
        };
        CommandError::new(
            code,
            "Local duplicate scan data could not be accessed.",
            true,
        )
        .with_details(format!("{operation}: {error}"))
    }

    fn serialization_error(&self, error: serde_json::Error) -> CommandError {
        CommandError::new(
            ErrorCode::SerializationFailed,
            "Local duplicate scan data is unreadable.",
            true,
        )
        .with_details(error.to_string())
    }
}

fn invalid_identifier(kind: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidPath,
        format!("The duplicate {kind} identifier is invalid."),
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::duplicate::DuplicateCopy;

    #[test]
    fn persists_and_pages_groups() {
        let directory = tempfile::tempdir().unwrap();
        let repository = DuplicateRepository::new(directory.path().to_path_buf());
        let scan_id = Uuid::new_v4().to_string();
        repository.initialize(&scan_id).unwrap();
        repository
            .append_group(&DuplicateGroup {
                id: Uuid::new_v4().to_string(),
                scan_id: scan_id.clone(),
                file_size: 10,
                reclaimable_bytes: 10,
                copies: vec![DuplicateCopy {
                    id: Uuid::new_v4().to_string(),
                    path: PathBuf::from("/tmp/a"),
                    display_path: "/tmp/a".to_owned(),
                    modified_at: None,
                    owner: None,
                }],
                recommended_keep_id: "copy".to_owned(),
                keep_reason: "fixture".to_owned(),
                full_hash: "hash".to_owned(),
                byte_for_byte_verified: false,
            })
            .unwrap();
        assert_eq!(repository.groups(&scan_id, 0, 100).unwrap().len(), 1);
    }

    #[test]
    fn rejects_traversal_identifiers() {
        let repository = DuplicateRepository::new(PathBuf::from("/tmp"));
        assert_eq!(
            repository.groups("../outside", 0, 100).unwrap_err().code,
            ErrorCode::InvalidPath
        );
    }
}
