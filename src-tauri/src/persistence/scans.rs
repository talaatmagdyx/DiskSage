use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use uuid::Uuid;

use crate::domain::{
    error::{CommandError, ErrorCode},
    finding::Finding,
    scan::ScanSummary,
};

#[derive(Debug, Clone)]
pub struct ScanRepository {
    root: PathBuf,
}

impl ScanRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn initialize(&self, scan_id: &str) -> Result<(), CommandError> {
        let directory = self.scan_directory(scan_id)?;
        fs::create_dir_all(&directory)
            .map_err(|error| self.io_error("create scan directory", error))?;
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(directory.join("findings.ndjson"))
            .map_err(|error| self.io_error("initialize findings", error))?;
        Ok(())
    }

    pub fn append_finding(&self, finding: &Finding) -> Result<(), CommandError> {
        let path = self
            .scan_directory(&finding.scan_id)?
            .join("findings.ndjson");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| self.io_error("open findings", error))?;
        serde_json::to_writer(&mut file, finding)
            .map_err(|error| self.serialization_error(error))?;
        file.write_all(b"\n")
            .map_err(|error| self.io_error("append finding", error))?;
        Ok(())
    }

    pub fn save_summary(&self, summary: &ScanSummary) -> Result<(), CommandError> {
        let directory = self.scan_directory(&summary.scan_id)?;
        fs::create_dir_all(&directory)
            .map_err(|error| self.io_error("create scan directory", error))?;
        let bytes =
            serde_json::to_vec_pretty(summary).map_err(|error| self.serialization_error(error))?;
        let temporary = directory.join("summary.json.tmp");
        fs::write(&temporary, bytes).map_err(|error| self.io_error("write scan summary", error))?;
        fs::rename(temporary, directory.join("summary.json"))
            .map_err(|error| self.io_error("replace scan summary", error))
    }

    pub fn load_summary(&self, scan_id: &str) -> Result<ScanSummary, CommandError> {
        let path = self.scan_directory(scan_id)?.join("summary.json");
        let bytes = fs::read(&path).map_err(|error| self.io_error("read scan summary", error))?;
        serde_json::from_slice(&bytes).map_err(|error| self.serialization_error(error))
    }

    pub fn findings(
        &self,
        scan_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Finding>, CommandError> {
        if limit == 0 || limit > 500 {
            return Err(CommandError::new(
                ErrorCode::InvalidPath,
                "Finding page size must be between 1 and 500.",
                true,
            ));
        }
        let file = fs::File::open(self.scan_directory(scan_id)?.join("findings.ndjson"))
            .map_err(|error| self.io_error("read findings", error))?;
        BufReader::new(file)
            .lines()
            .skip(offset)
            .take(limit)
            .map(|line| {
                let line = line.map_err(|error| self.io_error("read finding", error))?;
                serde_json::from_str(&line).map_err(|error| self.serialization_error(error))
            })
            .collect()
    }

    pub fn finding(&self, scan_id: &str, finding_id: &str) -> Result<Finding, CommandError> {
        let file = fs::File::open(self.scan_directory(scan_id)?.join("findings.ndjson"))
            .map_err(|error| self.io_error("read findings", error))?;
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|error| self.io_error("read finding", error))?;
            let finding: Finding =
                serde_json::from_str(&line).map_err(|error| self.serialization_error(error))?;
            if finding.id == finding_id {
                return Ok(finding);
            }
        }
        Err(CommandError::new(
            ErrorCode::PathNotFound,
            "The finding no longer exists.",
            true,
        ))
    }

    fn scan_directory(&self, scan_id: &str) -> Result<PathBuf, CommandError> {
        Uuid::parse_str(scan_id).map_err(|_| {
            CommandError::new(
                ErrorCode::InvalidPath,
                "The scan identifier is invalid.",
                true,
            )
        })?;
        Ok(self.root.join(scan_id))
    }

    fn io_error(&self, operation: &str, error: std::io::Error) -> CommandError {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::FilesystemError,
        };
        CommandError::new(code, "Local scan data could not be accessed.", true)
            .with_details(format!("{operation}: {error}"))
    }

    fn serialization_error(&self, error: serde_json::Error) -> CommandError {
        CommandError::new(
            ErrorCode::SerializationFailed,
            "Local scan data is unreadable.",
            true,
        )
        .with_details(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        finding::{FindingEvidence, FindingType},
        rule::{RecommendedAction, RiskLevel, RuleCategory},
    };
    use std::path::Path;

    fn finding(scan_id: &str) -> Finding {
        Finding {
            id: Uuid::new_v4().to_string(),
            scan_id: scan_id.to_owned(),
            rule_id: "test.rule".to_owned(),
            rule_version: 1,
            category: RuleCategory::ApplicationCache,
            display_name: "Fixture cache".to_owned(),
            description: "Fixture".to_owned(),
            path: PathBuf::from("/tmp/cache"),
            display_path: "/tmp/cache".to_owned(),
            item_type: FindingType::Directory,
            logical_size: 12,
            allocated_size: Some(4096),
            modified_at: None,
            risk: RiskLevel::Safe,
            recommended_action: RecommendedAction::MoveToTrash,
            evidence: FindingEvidence::KnownPath,
            cleanup_allowed: false,
            cleanup_block_reason: Some("Phase 2 is read-only".to_owned()),
            guided_action: None,
        }
    }

    #[test]
    fn persists_and_pages_flat_findings() {
        let directory = tempfile::tempdir().unwrap();
        let repository = ScanRepository::new(directory.path().to_path_buf());
        let scan_id = Uuid::new_v4().to_string();
        repository.initialize(&scan_id).unwrap();
        repository.append_finding(&finding(&scan_id)).unwrap();
        assert_eq!(repository.findings(&scan_id, 0, 100).unwrap().len(), 1);
    }

    #[test]
    fn rejects_traversal_as_scan_identifier() {
        let repository = ScanRepository::new(Path::new("/tmp").to_path_buf());
        assert_eq!(
            repository.findings("../outside", 0, 100).unwrap_err().code,
            ErrorCode::InvalidPath
        );
    }
}
