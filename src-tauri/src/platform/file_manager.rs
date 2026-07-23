use std::{path::Path, process::Command};

use crate::domain::error::{CommandError, ErrorCode};

pub fn reveal(path: &Path) -> Result<(), CommandError> {
    if !path.exists() {
        return Err(
            CommandError::new(ErrorCode::PathNotFound, "The item no longer exists.", true)
                .with_path(path.to_string_lossy()),
        );
    }

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg("-R").arg(path).spawn();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open")
        .arg(path.parent().unwrap_or(path))
        .spawn();

    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe")
        .arg(format!("/select,{}", path.display()))
        .spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return Err(CommandError::new(
        ErrorCode::CommandUnavailable,
        "Reveal is not supported on this platform.",
        true,
    ));

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    result.map(|_| ()).map_err(|error| {
        CommandError::new(
            ErrorCode::CommandUnavailable,
            "The system file manager could not be opened.",
            true,
        )
        .with_details(error.to_string())
    })
}
