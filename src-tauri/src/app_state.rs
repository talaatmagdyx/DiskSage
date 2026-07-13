use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    persistence::{scans::ScanRepository, settings::SettingsRepository},
    scanner::coordinator::ScanManager,
};

pub struct AppState {
    pub settings_repository: Mutex<SettingsRepository>,
    pub scan_manager: Arc<ScanManager>,
}

impl AppState {
    pub fn new(settings_path: PathBuf, scans_path: PathBuf) -> Self {
        Self {
            settings_repository: Mutex::new(SettingsRepository::new(settings_path)),
            scan_manager: Arc::new(ScanManager::new(ScanRepository::new(scans_path))),
        }
    }
}
