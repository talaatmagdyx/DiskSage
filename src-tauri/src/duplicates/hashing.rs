use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use crate::{
    domain::error::{CommandError, ErrorCode},
    scanner::cancellation::CancellationToken,
};

const SAMPLE_BYTES: usize = 128 * 1024;
const BUFFER_BYTES: usize = 256 * 1024;

pub fn partial_hash(
    path: &Path,
    size: u64,
    cancellation: &CancellationToken,
) -> Result<(String, u64), CommandError> {
    let mut file = open(path)?;
    let sample = SAMPLE_BYTES as u64;
    let mut offsets = vec![
        0,
        size.saturating_sub(sample) / 2,
        size.saturating_sub(sample),
    ];
    offsets.sort_unstable();
    offsets.dedup();
    let mut hasher = blake3::Hasher::new();
    let mut bytes_hashed = 0_u64;
    let mut buffer = vec![0_u8; SAMPLE_BYTES];
    for offset in offsets {
        check_cancelled(cancellation)?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|error| hash_io_error(path, error))?;
        hasher.update(&offset.to_le_bytes());
        let wanted = SAMPLE_BYTES.min(size.saturating_sub(offset) as usize);
        let mut read_total = 0;
        while read_total < wanted {
            check_cancelled(cancellation)?;
            let read = file
                .read(&mut buffer[read_total..wanted])
                .map_err(|error| hash_io_error(path, error))?;
            if read == 0 {
                break;
            }
            read_total += read;
        }
        hasher.update(&buffer[..read_total]);
        bytes_hashed = bytes_hashed.saturating_add(read_total as u64);
    }
    Ok((hasher.finalize().to_hex().to_string(), bytes_hashed))
}

pub fn full_hash(
    path: &Path,
    cancellation: &CancellationToken,
) -> Result<(String, u64), CommandError> {
    let mut file = open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut total = 0_u64;
    loop {
        check_cancelled(cancellation)?;
        let read = file
            .read(&mut buffer)
            .map_err(|error| hash_io_error(path, error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total = total.saturating_add(read as u64);
    }
    Ok((hasher.finalize().to_hex().to_string(), total))
}

pub fn byte_for_byte_equal(
    left: &Path,
    right: &Path,
    cancellation: &CancellationToken,
) -> Result<(bool, u64), CommandError> {
    let mut left_file = open(left)?;
    let mut right_file = open(right)?;
    let mut left_buffer = vec![0_u8; BUFFER_BYTES];
    let mut right_buffer = vec![0_u8; BUFFER_BYTES];
    let mut compared = 0_u64;
    loop {
        check_cancelled(cancellation)?;
        let left_read = left_file
            .read(&mut left_buffer)
            .map_err(|error| hash_io_error(left, error))?;
        let right_read = right_file
            .read(&mut right_buffer)
            .map_err(|error| hash_io_error(right, error))?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok((false, compared));
        }
        compared = compared.saturating_add(left_read as u64);
        if left_read == 0 {
            return Ok((true, compared));
        }
    }
}

fn open(path: &Path) -> Result<File, CommandError> {
    File::open(path).map_err(|error| hash_io_error(path, error))
}

fn check_cancelled(cancellation: &CancellationToken) -> Result<(), CommandError> {
    if cancellation.is_cancelled() {
        Err(CommandError::new(
            ErrorCode::ScanCancelled,
            "Duplicate hashing was cancelled.",
            true,
        ))
    } else {
        Ok(())
    }
}

fn hash_io_error(path: &Path, error: std::io::Error) -> CommandError {
    let code = match error.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        _ => ErrorCode::HashError,
    };
    CommandError::new(code, "A file could not be hashed safely.", true)
        .with_path(path.to_string_lossy())
        .with_details(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn same_size_non_identical_files_have_distinct_full_hashes() {
        let directory = tempfile::tempdir().unwrap();
        let left = directory.path().join("left");
        let right = directory.path().join("right");
        File::create(&left).unwrap().write_all(b"abcd").unwrap();
        File::create(&right).unwrap().write_all(b"abce").unwrap();
        let token = CancellationToken::default();
        assert_ne!(
            full_hash(&left, &token).unwrap().0,
            full_hash(&right, &token).unwrap().0
        );
    }

    #[test]
    fn cancellation_interrupts_hashing() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("file");
        File::create(&path).unwrap().write_all(&[1; 1024]).unwrap();
        let token = CancellationToken::default();
        token.cancel();
        assert_eq!(
            full_hash(&path, &token).unwrap_err().code,
            ErrorCode::ScanCancelled
        );
    }

    #[test]
    fn optional_byte_verification_detects_difference() {
        let directory = tempfile::tempdir().unwrap();
        let left = directory.path().join("left");
        let right = directory.path().join("right");
        File::create(&left).unwrap().write_all(b"same").unwrap();
        File::create(&right).unwrap().write_all(b"diff").unwrap();
        assert!(
            !byte_for_byte_equal(&left, &right, &CancellationToken::default())
                .unwrap()
                .0
        );
    }
}
