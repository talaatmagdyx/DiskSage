use std::path::Path;

use crate::domain::error::{CommandError, ErrorCode};

pub fn move_to_trash(path: &Path) -> Result<(), CommandError> {
    trash::delete(path).map_err(|error| {
        CommandError::new(
            ErrorCode::TrashFailed,
            "The item could not be moved to Trash.",
            true,
        )
        .with_path(path.to_string_lossy())
        .with_details(error.to_string())
    })
}
