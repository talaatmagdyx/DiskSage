use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidPath,
    InvalidSettings,
    PathNotFound,
    PathProtected,
    PermissionDenied,
    ScanAlreadyRunning,
    ScanCancelled,
    FilesystemError,
    HashError,
    TrashFailed,
    DeleteFailed,
    PlanExpired,
    PlanValidationFailed,
    CommandUnavailable,
    DiskInfoFailed,
    SerializationFailed,
    InternalError,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl CommandError {
    pub fn new(code: ErrorCode, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable,
            path: None,
            details: None,
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn internal(details: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::InternalError,
            "An internal local error occurred.",
            true,
        )
        .with_details(details)
    }
}
