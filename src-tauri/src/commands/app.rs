use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::domain::error::{CommandError, ErrorCode};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
    architecture: &'static str,
    build_profile: &'static str,
    runtime: &'static str,
    destructive_commands_available: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAppLinkRequest {
    link: AppLink,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum AppLink {
    Repository,
    Releases,
    Privacy,
    Security,
    License,
}

impl AppLink {
    fn url(&self) -> &'static str {
        match self {
            Self::Repository => "https://github.com/talaatmagdyx/disk_sage",
            Self::Releases => "https://github.com/talaatmagdyx/disk_sage/releases",
            Self::Privacy => "https://github.com/talaatmagdyx/disk_sage/blob/main/PRIVACY.md",
            Self::Security => "https://github.com/talaatmagdyx/disk_sage/blob/main/SECURITY.md",
            Self::License => "https://github.com/talaatmagdyx/disk_sage/blob/main/LICENSE",
        }
    }
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "DiskSage",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        build_profile: if cfg!(debug_assertions) {
            "development"
        } else {
            "release"
        },
        runtime: "Tauri 2",
        destructive_commands_available: true,
    }
}

#[tauri::command]
pub fn open_app_link(request: OpenAppLinkRequest) -> Result<(), CommandError> {
    let url = request.link.url();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(url).spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err(CommandError::new(
        ErrorCode::CommandUnavailable,
        "Opening product links is not supported on this platform.",
        false,
    ));

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    result.map(|_| ()).map_err(|error| {
        CommandError::new(
            ErrorCode::CommandUnavailable,
            "The default browser could not be opened.",
            true,
        )
        .with_details(error.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::AppLink;

    #[test]
    fn product_links_are_fixed_https_destinations() {
        for link in [
            AppLink::Repository,
            AppLink::Releases,
            AppLink::Privacy,
            AppLink::Security,
            AppLink::License,
        ] {
            let url = link.url();
            assert!(url.starts_with("https://github.com/talaatmagdyx/disk_sage"));
            assert!(!url.contains(' '));
        }
    }
}
