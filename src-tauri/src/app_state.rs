use std::{path::PathBuf, sync::Mutex};

use crate::persistence::settings::SettingsRepository;

pub struct AppState {
    pub settings_repository: Mutex<SettingsRepository>,
}

impl AppState {
    pub fn new(settings_path: PathBuf) -> Self {
        Self {
            settings_repository: Mutex::new(SettingsRepository::new(settings_path)),
        }
    }
}
