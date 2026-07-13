use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::domain::{
    error::{CommandError, ErrorCode},
    settings::AppSettings,
};

pub struct SettingsRepository {
    path: PathBuf,
}

impl SettingsRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AppSettings, CommandError> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let bytes = fs::read(&self.path).map_err(|error| self.io_error("read", error))?;
        let settings: AppSettings = serde_json::from_slice(&bytes).map_err(|error| {
            CommandError::new(
                ErrorCode::SerializationFailed,
                "Local settings are unreadable. The file was left unchanged.",
                true,
            )
            .with_details(error.to_string())
        })?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), CommandError> {
        settings.validate()?;
        let parent = self.path.parent().ok_or_else(|| {
            CommandError::new(
                ErrorCode::SerializationFailed,
                "Settings location is invalid.",
                false,
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| self.io_error("create settings directory", error))?;
        let bytes = serde_json::to_vec_pretty(settings).map_err(|error| {
            CommandError::new(
                ErrorCode::SerializationFailed,
                "Settings could not be encoded.",
                true,
            )
            .with_details(error.to_string())
        })?;
        let temporary_path = self.path.with_extension("json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary_path)
            .map_err(|error| self.io_error("create temporary settings file", error))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| self.io_error("write settings", error))?;
        fs::rename(&temporary_path, &self.path)
            .map_err(|error| self.io_error("replace settings", error))?;
        Ok(())
    }

    fn io_error(&self, operation: &str, error: std::io::Error) -> CommandError {
        let code = if error.kind() == std::io::ErrorKind::PermissionDenied {
            ErrorCode::PermissionDenied
        } else {
            ErrorCode::FilesystemError
        };
        CommandError::new(code, "Local settings could not be accessed.", true)
            .with_details(format!("{operation}: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_returns_safe_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let repository = SettingsRepository::new(directory.path().join("settings.json"));
        let settings = repository.load().unwrap();
        assert!(!settings.follow_symlinks);
        assert!(settings.move_to_trash_by_default);
        assert!(!settings.permanent_deletion_enabled);
    }

    #[test]
    fn round_trips_settings_atomically() {
        let directory = tempfile::tempdir().unwrap();
        let repository = SettingsRepository::new(directory.path().join("nested/settings.json"));
        let settings = AppSettings {
            reduced_motion: true,
            ..AppSettings::default()
        };
        repository.save(&settings).unwrap();
        assert_eq!(repository.load().unwrap(), settings);
    }

    #[test]
    fn corrupt_file_is_not_silently_replaced() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("settings.json");
        fs::write(&path, b"not-json").unwrap();
        let repository = SettingsRepository::new(path);
        let error = repository.load().unwrap_err();
        assert_eq!(error.code, ErrorCode::SerializationFailed);
    }
}
