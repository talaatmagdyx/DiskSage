use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    domain::{
        cleanup::ExecuteCleanupResponse,
        duplicate::{
            CancelDuplicateCleanupRequest, CreateDuplicateCleanupPlanRequest, DuplicateCleanupPlan,
            DuplicateGroup, DuplicateScanIdRequest, DuplicateSummary,
            ExecuteDuplicateCleanupRequest, GetDuplicateGroupsRequest, StartDuplicateScanRequest,
            StartDuplicateScanResponse,
        },
        error::CommandError,
    },
    platform::file_manager,
};

#[tauri::command]
pub fn start_duplicate_scan(
    request: StartDuplicateScanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<StartDuplicateScanResponse, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let scan_id = state
        .duplicate_manager
        .start(app, request, home, std::env::consts::OS)?;
    Ok(StartDuplicateScanResponse { scan_id })
}

#[tauri::command]
pub fn cancel_duplicate_scan(
    request: DuplicateScanIdRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state.duplicate_manager.cancel(&request.scan_id)
}

#[tauri::command]
pub fn get_duplicate_scan_status(
    request: DuplicateScanIdRequest,
    state: State<'_, AppState>,
) -> Result<DuplicateSummary, CommandError> {
    state.duplicate_manager.status(&request.scan_id)
}

#[tauri::command]
pub fn get_duplicate_groups(
    request: GetDuplicateGroupsRequest,
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateGroup>, CommandError> {
    state
        .duplicate_manager
        .repository()
        .groups(&request.scan_id, request.offset, request.limit)
}

#[tauri::command]
pub fn reveal_duplicate(
    request: RevealDuplicateRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let group = state
        .duplicate_manager
        .repository()
        .group(&request.scan_id, &request.group_id)?;
    let copy = group
        .copies
        .iter()
        .find(|copy| copy.id == request.copy_id)
        .ok_or_else(|| {
            CommandError::new(
                crate::domain::error::ErrorCode::PathNotFound,
                "That duplicate copy no longer exists in the group.",
                true,
            )
        })?;
    file_manager::reveal(&copy.path)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevealDuplicateRequest {
    pub scan_id: String,
    pub group_id: String,
    pub copy_id: String,
}

#[tauri::command]
pub fn create_duplicate_cleanup_plan(
    request: CreateDuplicateCleanupPlanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<DuplicateCleanupPlan, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    state
        .duplicate_manager
        .cleanup()
        .create_plan(request, &home, std::env::consts::OS)
}

#[tauri::command]
pub fn execute_duplicate_cleanup_plan(
    request: ExecuteDuplicateCleanupRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExecuteCleanupResponse, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let operation_id =
        state
            .duplicate_manager
            .cleanup()
            .execute(app, request, home, std::env::consts::OS)?;
    Ok(ExecuteCleanupResponse { operation_id })
}

#[tauri::command]
pub fn cancel_duplicate_cleanup(
    request: CancelDuplicateCleanupRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .duplicate_manager
        .cleanup()
        .cancel(&request.operation_id)
}
