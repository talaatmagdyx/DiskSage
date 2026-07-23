use std::process::Command;
use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    domain::{
        application::{
            ApplicationIdRequest, ApplicationUninstallPlan, ApplicationUninstallResult,
            CreateApplicationUninstallPlanRequest, ExecuteApplicationUninstallRequest,
            InstalledApplication,
        },
        error::CommandError,
    },
    platform::file_manager,
};

#[tauri::command]
pub async fn scan_applications(
    include_system_apps: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<InstalledApplication>, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let manager = state.application_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.scan(&home, std::env::consts::OS, include_system_apps)
    })
    .await
    .map_err(|error| CommandError::internal(error.to_string()))?
}

#[tauri::command]
pub fn reveal_application(
    request: ApplicationIdRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let application = state
        .application_manager
        .application(&request.application_id)?;
    file_manager::reveal(std::path::Path::new(&application.path))
}

#[tauri::command]
pub async fn create_application_uninstall_plan(
    request: CreateApplicationUninstallPlanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ApplicationUninstallPlan, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let manager = state.application_manager.clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.create_plan(&request.application_id, request.mode, &home)
    })
    .await
    .map_err(|error| CommandError::internal(error.to_string()))?
}

#[tauri::command]
pub async fn execute_application_uninstall_plan(
    request: ExecuteApplicationUninstallRequest,
    state: State<'_, AppState>,
) -> Result<ApplicationUninstallResult, CommandError> {
    let manager = state.application_manager.clone();
    tauri::async_runtime::spawn_blocking(move || manager.execute(&request))
        .await
        .map_err(|error| CommandError::internal(error.to_string()))?
}

#[tauri::command]
pub fn open_installed_apps_settings() -> Result<(), CommandError> {
    if std::env::consts::OS != "windows" {
        return Err(crate::domain::error::CommandError::new(
            crate::domain::error::ErrorCode::CommandUnavailable,
            "Installed Apps settings are available only on Windows.",
            false,
        ));
    }
    let status = Command::new("explorer.exe")
        .arg("ms-settings:appsfeatures")
        .status()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::domain::error::CommandError::new(
            crate::domain::error::ErrorCode::CommandUnavailable,
            "Windows Installed Apps settings could not be opened.",
            true,
        ))
    }
}
