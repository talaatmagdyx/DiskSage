use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    domain::{
        error::CommandError,
        finding::Finding,
        scan::{
            GetScanFindingsRequest, ScanIdRequest, ScanProfile, ScanSummary, StartScanRequest,
            StartScanResponse,
        },
    },
    platform::file_manager,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevealFindingRequest {
    pub scan_id: String,
    pub finding_id: String,
}

#[tauri::command]
pub fn get_scan_profiles() -> Vec<ScanProfile> {
    crate::scanner::coordinator::ScanManager::profiles()
}

#[tauri::command]
pub fn start_scan(
    request: StartScanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<StartScanResponse, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let scan_id = state
        .scan_manager
        .start(app, request, home, std::env::consts::OS)?;
    Ok(StartScanResponse { scan_id })
}

#[tauri::command]
pub fn cancel_scan(request: ScanIdRequest, state: State<'_, AppState>) -> Result<(), CommandError> {
    state.scan_manager.cancel(&request.scan_id)
}

#[tauri::command]
pub fn get_scan_status(
    request: ScanIdRequest,
    state: State<'_, AppState>,
) -> Result<ScanSummary, CommandError> {
    state.scan_manager.status(&request.scan_id)
}

#[tauri::command]
pub fn get_scan_findings(
    request: GetScanFindingsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<Finding>, CommandError> {
    state
        .scan_manager
        .repository()
        .findings(&request.scan_id, request.offset, request.limit)
}

#[tauri::command]
pub fn reveal_item(
    request: RevealFindingRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let finding = state
        .scan_manager
        .repository()
        .finding(&request.scan_id, &request.finding_id)?;
    file_manager::reveal(&finding.path)
}
