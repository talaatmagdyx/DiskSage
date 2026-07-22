use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    applications::coordinator::ApplicationManager,
    cleanup::coordinator::CleanupManager,
    duplicates::coordinator::DuplicateManager,
    persistence::{
        duplicates::DuplicateRepository, history::HistoryRepository, scans::ScanRepository,
        settings::SettingsRepository,
    },
    scanner::coordinator::ScanManager,
};

pub struct AppState {
    pub application_manager: Arc<ApplicationManager>,
    pub settings_repository: Mutex<SettingsRepository>,
    pub scan_manager: Arc<ScanManager>,
    pub cleanup_manager: Arc<CleanupManager>,
    pub duplicate_manager: Arc<DuplicateManager>,
}

impl AppState {
    pub fn new(
        settings_path: PathBuf,
        scans_path: PathBuf,
        duplicates_path: PathBuf,
        history_path: PathBuf,
    ) -> Self {
        let scan_repository = ScanRepository::new(scans_path);
        let history_repository = HistoryRepository::new(history_path);
        let duplicate_repository = DuplicateRepository::new(duplicates_path);
        Self {
            application_manager: Arc::new(ApplicationManager::default()),
            settings_repository: Mutex::new(SettingsRepository::new(settings_path)),
            scan_manager: Arc::new(ScanManager::new(scan_repository.clone())),
            cleanup_manager: Arc::new(CleanupManager::new(
                scan_repository,
                history_repository.clone(),
            )),
            duplicate_manager: Arc::new(DuplicateManager::new(
                duplicate_repository,
                history_repository,
            )),
        }
    }
}
