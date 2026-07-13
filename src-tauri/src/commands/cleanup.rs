use tauri::{AppHandle, Manager, State};

use crate::{
    app_state::AppState,
    domain::{
        cleanup::{
            CancelCleanupRequest, CleanupPlan, CleanupSummary, CreateCleanupPlanRequest,
            ExecuteCleanupRequest, ExecuteCleanupResponse, GetCleanupHistoryRequest,
        },
        error::CommandError,
    },
};

#[tauri::command]
pub fn create_cleanup_plan(
    request: CreateCleanupPlanRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CleanupPlan, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    state
        .cleanup_manager
        .create_plan(request, &home, std::env::consts::OS)
}

#[tauri::command]
pub fn execute_cleanup_plan(
    request: ExecuteCleanupRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExecuteCleanupResponse, CommandError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|error| CommandError::internal(error.to_string()))?;
    let operation_id = state
        .cleanup_manager
        .execute(app, request, home, std::env::consts::OS)?;
    Ok(ExecuteCleanupResponse { operation_id })
}

#[tauri::command]
pub fn cancel_cleanup(
    request: CancelCleanupRequest,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    state.cleanup_manager.cancel(&request.operation_id)
}

#[tauri::command]
pub fn get_cleanup_history(
    request: GetCleanupHistoryRequest,
    state: State<'_, AppState>,
) -> Result<Vec<CleanupSummary>, CommandError> {
    state.cleanup_manager.history(request.offset, request.limit)
}

#[tauri::command]
pub fn clear_cleanup_history(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.cleanup_manager.clear_history()
}
