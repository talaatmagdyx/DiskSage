use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    domain::{
        error::CommandError,
        intelligence::{
            OrphanedApplicationData, PermissionReport, StorageMapReport, StorageMapRequest,
        },
    },
    intelligence,
};

#[tauri::command]
pub fn get_permission_report(app: AppHandle) -> Result<PermissionReport, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    intelligence::permission_report(&home, std::env::consts::OS)
}

#[tauri::command]
pub fn open_full_disk_access_settings() -> Result<(), CommandError> {
    intelligence::open_full_disk_access_settings(std::env::consts::OS)
}

#[tauri::command]
pub async fn scan_orphaned_application_data(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<OrphanedApplicationData>, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let manager = state.application_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if manager.inventory_is_empty()? {
            manager.scan(&home, std::env::consts::OS, false)?;
        }
        let installed_bundle_ids = manager.installed_bundle_ids()?;
        intelligence::scan_orphaned_application_data(
            &home,
            &installed_bundle_ids,
            std::env::consts::OS,
        )
    })
    .await
    .map_err(|error| CommandError::internal(error.to_string()))?
}

#[tauri::command]
pub async fn scan_storage_map(
    request: StorageMapRequest,
    app: AppHandle,
) -> Result<StorageMapReport, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    tauri::async_runtime::spawn_blocking(move || intelligence::storage_map(&home, &request))
        .await
        .map_err(|error| CommandError::internal(error.to_string()))?
}
