pub mod app_state;
pub mod commands;
pub mod domain;
pub mod observability;
pub mod persistence;
pub mod platform;
pub mod rules;
pub mod safety;
pub mod scanner;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            observability::logging::init();
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let scans_path = app.path().app_data_dir()?.join("scans");
            app.manage(app_state::AppState::new(settings_path, scans_path));
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "DiskSage initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::disk::list_disks,
            commands::disk::get_disk_info,
            commands::scan::get_scan_profiles,
            commands::scan::start_scan,
            commands::scan::cancel_scan,
            commands::scan::get_scan_status,
            commands::scan::get_scan_findings,
            commands::scan::reveal_item,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run DiskSage");
}
