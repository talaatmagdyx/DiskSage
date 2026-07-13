use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    cleanup::coordinator::CleanupManager,
    persistence::{
        history::HistoryRepository, scans::ScanRepository, settings::SettingsRepository,
    },
    scanner::coordinator::ScanManager,
};

pub struct AppState {
    pub settings_repository: Mutex<SettingsRepository>,
    pub scan_manager: Arc<ScanManager>,
    pub cleanup_manager: Arc<CleanupManager>,
}

impl AppState {
    pub fn new(settings_path: PathBuf, scans_path: PathBuf, history_path: PathBuf) -> Self {
        let scan_repository = ScanRepository::new(scans_path);
        Self {
            settings_repository: Mutex::new(SettingsRepository::new(settings_path)),
            scan_manager: Arc::new(ScanManager::new(scan_repository.clone())),
            cleanup_manager: Arc::new(CleanupManager::new(
                scan_repository,
                HistoryRepository::new(history_path),
            )),
        }
    }
}
