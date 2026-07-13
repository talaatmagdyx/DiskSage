use serde::Deserialize;
use tauri::State;

use crate::{
    app_state::AppState,
    domain::{error::CommandError, settings::AppSettings},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateSettingsRequest {
    pub settings: AppSettings,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    state
        .settings_repository
        .lock()
        .map_err(|_| CommandError::internal("settings lock poisoned"))?
        .load()
}

#[tauri::command]
pub fn update_settings(
    request: UpdateSettingsRequest,
    state: State<'_, AppState>,
) -> Result<AppSettings, CommandError> {
    request.settings.validate()?;
    state
        .settings_repository
        .lock()
        .map_err(|_| CommandError::internal("settings lock poisoned"))?
        .save(&request.settings)?;
    Ok(request.settings)
}
