use std::{fs, path::Path};

pub fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }

    #[cfg(not(target_os = "windows"))]
    false
}

#[cfg(unix)]
pub fn allocated_size(_path: &Path, metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;

    metadata.blocks().saturating_mul(512)
}

#[cfg(target_os = "windows")]
pub fn allocated_size(path: &Path, metadata: &fs::Metadata) -> u64 {
    use std::{iter, os::windows::ffi::OsStrExt};
    use windows_sys::Win32::Storage::FileSystem::GetCompressedFileSizeW;

    let path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let mut high = 0_u32;
    let low = unsafe { GetCompressedFileSizeW(path.as_ptr(), &mut high) };
    if low == u32::MAX && high == 0 {
        metadata.len()
    } else {
        (u64::from(high) << 32) | u64::from(low)
    }
}

#[cfg(not(any(unix, target_os = "windows")))]
pub fn allocated_size(_path: &Path, metadata: &fs::Metadata) -> u64 {
    metadata.len()
}
