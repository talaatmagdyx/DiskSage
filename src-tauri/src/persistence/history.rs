use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use crate::domain::{
    cleanup::CleanupSummary,
    error::{CommandError, ErrorCode},
};

#[derive(Debug, Clone)]
pub struct HistoryRepository {
    path: PathBuf,
}

impl HistoryRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn append(&self, summary: &CleanupSummary) -> Result<(), CommandError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| self.io_error("create history directory", error))?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| self.io_error("open cleanup history", error))?;
        serde_json::to_writer(&mut file, summary)
            .map_err(|error| self.serialization_error(error))?;
        file.write_all(b"\n")
            .map_err(|error| self.io_error("append cleanup history", error))
    }

    pub fn list(&self, offset: usize, limit: usize) -> Result<Vec<CleanupSummary>, CommandError> {
        if limit == 0 || limit > 200 {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "History page size must be between 1 and 200.",
                true,
            ));
        }
        let file = match fs::File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(self.io_error("read cleanup history", error)),
        };
        let mut entries: Vec<CleanupSummary> = BufReader::new(file)
            .lines()
            .map(|line| {
                let line = line.map_err(|error| self.io_error("read cleanup history", error))?;
                serde_json::from_str(&line).map_err(|error| self.serialization_error(error))
            })
            .collect::<Result<_, _>>()?;
        entries.reverse();
        Ok(entries.into_iter().skip(offset).take(limit).collect())
    }

    pub fn clear(&self) -> Result<(), CommandError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(self.io_error("clear cleanup history", error)),
        }
    }

    fn io_error(&self, operation: &str, error: std::io::Error) -> CommandError {
        let code = if error.kind() == std::io::ErrorKind::PermissionDenied {
            ErrorCode::PermissionDenied
        } else {
            ErrorCode::FilesystemError
        };
        CommandError::new(code, "Local cleanup history could not be accessed.", true)
            .with_details(format!("{operation}: {error}"))
    }

    fn serialization_error(&self, error: serde_json::Error) -> CommandError {
        CommandError::new(
            ErrorCode::SerializationFailed,
            "Local cleanup history is unreadable.",
            true,
        )
        .with_details(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cleanup::{CleanupAction, CleanupSummary};
    use chrono::Utc;

    fn summary() -> CleanupSummary {
        CleanupSummary {
            operation_id: "operation".to_owned(),
            plan_id: "plan".to_owned(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            action: CleanupAction::MoveToTrash,
            selected_count: 0,
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
            expected_bytes: 0,
            actual_free_space_change_bytes: Some(0),
            cancelled: false,
            items: Vec::new(),
            disks: Vec::new(),
        }
    }

    #[test]
    fn persists_lists_and_clears_history() {
        let directory = tempfile::tempdir().unwrap();
        let repository = HistoryRepository::new(directory.path().join("history.ndjson"));
        repository.append(&summary()).unwrap();
        assert_eq!(repository.list(0, 50).unwrap().len(), 1);
        repository.clear().unwrap();
        assert!(repository.list(0, 50).unwrap().is_empty());
    }
}
