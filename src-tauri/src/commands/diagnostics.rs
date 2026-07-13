use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    domain::{
        diagnostics::DiagnosticsReport,
        error::{CommandError, ErrorCode},
    },
    platform::{disk_info, file_manager},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportDiagnosticsResponse {
    path: String,
}

#[tauri::command]
pub fn export_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportDiagnosticsResponse, CommandError> {
    let settings = state
        .settings_repository
        .lock()
        .map_err(|_| CommandError::internal("settings lock poisoned"))?
        .load()?;
    let disks = disk_info::list_disks().unwrap_or_default();
    let latest_scan = state.scan_manager.latest_status()?;
    let cleanup = state.cleanup_manager.history(0, 50)?;
    let report =
        DiagnosticsReport::from_local_state(&settings, &disks, latest_scan.as_ref(), &cleanup);
    let encoded = serde_json::to_vec_pretty(&report).map_err(|error| {
        CommandError::new(
            ErrorCode::SerializationFailed,
            "The diagnostics report could not be encoded.",
            true,
        )
        .with_details(error.to_string())
    })?;

    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?
        .join("diagnostics");
    fs::create_dir_all(&directory).map_err(|error| {
        CommandError::new(
            ErrorCode::FilesystemError,
            "The local diagnostics folder could not be created.",
            true,
        )
        .with_details(error.to_string())
    })?;
    let filename = format!(
        "disksage-diagnostics-{}-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S"),
        &Uuid::new_v4().to_string()[..8]
    );
    let path = directory.join(filename);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            CommandError::new(
                ErrorCode::FilesystemError,
                "The diagnostics report could not be created.",
                true,
            )
            .with_details(error.to_string())
        })?;
    file.write_all(&encoded).map_err(|error| {
        CommandError::new(
            ErrorCode::FilesystemError,
            "The diagnostics report could not be written.",
            true,
        )
        .with_details(error.to_string())
    })?;
    file.sync_all()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    file_manager::reveal(&path)?;
    Ok(ExportDiagnosticsResponse {
        path: path.to_string_lossy().into_owned(),
    })
}
