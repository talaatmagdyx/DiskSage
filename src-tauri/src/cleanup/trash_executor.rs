use std::path::Path;

use crate::domain::error::{CommandError, ErrorCode};

pub fn move_to_trash(path: &Path) -> Result<(), CommandError> {
    trash::delete(path).map_err(|error| classify_trash_error(path, &error.to_string()))
}

fn classify_trash_error(path: &Path, details: &str) -> CommandError {
    let normalized = details.to_ascii_lowercase();
    let display_path = path.to_string_lossy();
    let permission_denied = normalized.contains("permission denied")
        || normalized.contains("operation not permitted")
        || normalized.contains("access denied")
        || normalized.contains("os error 1")
        || normalized.contains("os error 13");
    let item_in_use = normalized.contains("resource busy")
        || normalized.contains("device or resource busy")
        || normalized.contains("being used")
        || normalized.contains("in use");

    let (code, message) = if permission_denied && is_private_application_data(path) {
        (
            ErrorCode::PermissionDenied,
            "macOS blocked access to this app data. Quit the app and grant DiskSage Full Disk Access in System Settings > Privacy & Security, then review the uninstall again.",
        )
    } else if permission_denied && path.starts_with("/Applications") {
        (
            ErrorCode::PermissionDenied,
            "macOS denied permission to move this application. Quit the app and grant DiskSage Full Disk Access; administrator-owned apps may need to be moved with Finder.",
        )
    } else if permission_denied {
        (
            ErrorCode::PermissionDenied,
            "macOS denied access to this item. Grant DiskSage Full Disk Access in System Settings > Privacy & Security, then try again.",
        )
    } else if item_in_use {
        (
            ErrorCode::TrashFailed,
            "This item is currently in use. Close the application using it, then review the cleanup again.",
        )
    } else {
        (
            ErrorCode::TrashFailed,
            "macOS could not move this item to Trash. Close the related application and try again; if it still fails, review the item in Finder.",
        )
    };

    CommandError::new(code, message, true)
        .with_path(display_path)
        .with_details(details)
}

fn is_private_application_data(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.contains("/Library/Containers/") || text.contains("/Library/Group Containers/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_permission_errors_explain_full_disk_access() {
        let error = classify_trash_error(
            Path::new("/Users/fixture/Library/Containers/com.example.app"),
            "Operation not permitted (os error 1)",
        );
        assert_eq!(error.code, ErrorCode::PermissionDenied);
        assert!(error.message.contains("Full Disk Access"));
        assert!(!error.message.contains("/Users/fixture"));
    }

    #[test]
    fn application_permission_errors_explain_finder_fallback() {
        let error = classify_trash_error(
            Path::new("/Applications/Fixture.app"),
            "Permission denied (os error 13)",
        );
        assert_eq!(error.code, ErrorCode::PermissionDenied);
        assert!(error.message.contains("Finder"));
    }

    #[test]
    fn busy_errors_recommend_closing_the_application() {
        let error = classify_trash_error(
            Path::new("/Applications/Fixture.app"),
            "Device or resource busy",
        );
        assert_eq!(error.code, ErrorCode::TrashFailed);
        assert!(error.message.contains("Close the application"));
    }
}
