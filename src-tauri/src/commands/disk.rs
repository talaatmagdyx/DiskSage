use serde::Deserialize;

use crate::{domain::disk::DiskInfo, domain::error::CommandError, platform::disk_info};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetDiskInfoRequest {
    pub mount_path: String,
}

#[tauri::command]
pub async fn list_disks() -> Result<Vec<DiskInfo>, CommandError> {
    tauri::async_runtime::spawn_blocking(disk_info::list_disks)
        .await
        .map_err(|error| CommandError::internal(format!("disk worker failed: {error}")))?
}

#[tauri::command]
pub async fn get_disk_info(request: GetDiskInfoRequest) -> Result<DiskInfo, CommandError> {
    tauri::async_runtime::spawn_blocking(move || disk_info::get_disk(&request.mount_path))
        .await
        .map_err(|error| CommandError::internal(format!("disk worker failed: {error}")))?
}
