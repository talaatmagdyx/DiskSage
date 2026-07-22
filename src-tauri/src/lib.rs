pub mod app_state;
pub mod applications;
pub mod cleanup;
pub mod commands;
pub mod domain;
pub mod duplicates;
pub mod observability;
pub mod persistence;
pub mod platform;
pub mod rules;
pub mod safety;
pub mod scanner;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            observability::logging::init();
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let data_path = app.path().app_data_dir()?;
            let scans_path = data_path.join("scans");
            let duplicates_path = data_path.join("duplicates");
            let history_path = data_path.join("cleanup-history.ndjson");
            app.manage(app_state::AppState::new(
                settings_path,
                scans_path,
                duplicates_path,
                history_path,
            ));
            tracing::info!(version = env!("CARGO_PKG_VERSION"), "DiskSage initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::applications::scan_applications,
            commands::applications::reveal_application,
            commands::applications::create_application_uninstall_plan,
            commands::applications::execute_application_uninstall_plan,
            commands::app::get_app_info,
            commands::diagnostics::export_diagnostics,
            commands::disk::list_disks,
            commands::disk::get_disk_info,
            commands::scan::get_scan_profiles,
            commands::scan::start_scan,
            commands::scan::cancel_scan,
            commands::scan::get_scan_status,
            commands::scan::get_scan_findings,
            commands::scan::reveal_item,
            commands::duplicates::start_duplicate_scan,
            commands::duplicates::cancel_duplicate_scan,
            commands::duplicates::get_duplicate_scan_status,
            commands::duplicates::get_duplicate_groups,
            commands::duplicates::reveal_duplicate,
            commands::duplicates::create_duplicate_cleanup_plan,
            commands::duplicates::execute_duplicate_cleanup_plan,
            commands::duplicates::cancel_duplicate_cleanup,
            commands::cleanup::create_cleanup_plan,
            commands::cleanup::execute_cleanup_plan,
            commands::cleanup::cancel_cleanup,
            commands::cleanup::get_cleanup_history,
            commands::cleanup::clear_cleanup_history,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run DiskSage");
}
