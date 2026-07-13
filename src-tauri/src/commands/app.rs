use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    platform: &'static str,
    destructive_commands_available: bool,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "DiskSage",
        version: env!("CARGO_PKG_VERSION"),
        platform: std::env::consts::OS,
        destructive_commands_available: true,
    }
}
