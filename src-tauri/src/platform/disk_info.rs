use std::path::Path;

use sysinfo::Disks;

use crate::domain::{
    disk::DiskInfo,
    error::{CommandError, ErrorCode},
};

pub fn list_disks() -> Result<Vec<DiskInfo>, CommandError> {
    let disks = Disks::new_with_refreshed_list();
    let mut result: Vec<_> = disks.list().iter().map(to_disk_info).collect();
    result.sort_by(|left, right| left.mount_path.cmp(&right.mount_path));
    if result.is_empty() {
        tracing::warn!("no accessible disks reported by the operating system");
    }
    Ok(result)
}

pub fn get_disk(mount_path: &str) -> Result<DiskInfo, CommandError> {
    if mount_path.is_empty() || mount_path.contains('\0') {
        return Err(CommandError::new(
            ErrorCode::InvalidPath,
            "The mount path is invalid.",
            true,
        ));
    }
    list_disks()?
        .into_iter()
        .find(|disk| disk.mount_path == mount_path)
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::PathNotFound,
                "The mounted disk is no longer available.",
                true,
            )
            .with_path(mount_path)
        })
}

fn to_disk_info(disk: &sysinfo::Disk) -> DiskInfo {
    let total_bytes = disk.total_space();
    let available_bytes = disk.available_space().min(total_bytes);
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let percentage_used = if total_bytes == 0 {
        0.0
    } else {
        used_bytes as f64 / total_bytes as f64 * 100.0
    };
    let mount_path = display_path(disk.mount_point());
    DiskInfo {
        id: mount_path.clone(),
        name: disk.name().to_string_lossy().into_owned(),
        mount_path,
        file_system: disk.file_system().to_string_lossy().into_owned(),
        total_bytes,
        used_bytes,
        available_bytes,
        percentage_used,
        removable: disk.is_removable(),
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_mount_path() {
        let error = get_disk("").unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidPath);
    }
}
