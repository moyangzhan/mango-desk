use crate::entities::FileInfo;
use crate::enums::FileCategory;
use crate::errors::AppError;
use crate::structs::file_metadata::FileMetadata;
use crate::utils::datetime_util;
use chrono::{DateTime, Local};
use md5::{Digest, Md5};
use std::path::Path;
use tokio::io::AsyncReadExt;

pub fn list_files_by_directory(scan_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(scan_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            println!("Found file: {}", path.display());
        } else if path.is_dir() {
            list_files_by_directory(path.to_str().unwrap())?;
        }
    }
    Ok(())
}

pub async fn calculate_md5(
    file_handle: &mut tokio::fs::File,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = Md5::new();
    let mut buffer = [0; 8192];
    loop {
        match file_handle.read(&mut buffer).await? {
            0 => break,
            n => hasher.update(&buffer[..n]),
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(unix)]
pub fn get_file_attributes_desc(attributes: u32) -> Vec<String> {
    return vec![];
}

#[cfg(windows)]
pub fn get_file_attributes_desc(attributes: u32) -> Vec<String> {
    let mut attrs = Vec::new();
    const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
    const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
    const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x00000020;
    const FILE_ATTRIBUTE_NORMAL: u32 = 0x00000080;
    const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x00000100;
    const FILE_ATTRIBUTE_COMPRESSED: u32 = 0x00000800;
    const FILE_ATTRIBUTE_OFFLINE: u32 = 0x00001000;
    const FILE_ATTRIBUTE_ENCRYPTED: u32 = 0x00004000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;
    const FILE_ATTRIBUTE_SPARSE_FILE: u32 = 0x00000200;

    if attributes & FILE_ATTRIBUTE_READONLY != 0 {
        attrs.push("Read only".to_string());
    }
    if attributes & FILE_ATTRIBUTE_HIDDEN != 0 {
        attrs.push("Hidden".to_string());
    }
    if attributes & FILE_ATTRIBUTE_SYSTEM != 0 {
        attrs.push("System".to_string());
    }
    if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
        attrs.push("Directory".to_string());
    }
    if attributes & FILE_ATTRIBUTE_ARCHIVE != 0 {
        attrs.push("Archive".to_string());
    }
    if attributes & FILE_ATTRIBUTE_NORMAL != 0 {
        attrs.push("Normal".to_string());
    }
    if attributes & FILE_ATTRIBUTE_TEMPORARY != 0 {
        attrs.push("Temporary".to_string());
    }
    if attributes & FILE_ATTRIBUTE_COMPRESSED != 0 {
        attrs.push("Compressed".to_string());
    }
    if attributes & FILE_ATTRIBUTE_OFFLINE != 0 {
        attrs.push("Offline".to_string());
    }
    if attributes & FILE_ATTRIBUTE_ENCRYPTED != 0 {
        attrs.push("Encrypted".to_string());
    }
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        attrs.push("Reparse Point".to_string());
    }
    if attributes & FILE_ATTRIBUTE_SPARSE_FILE != 0 {
        attrs.push("Sparse File".to_string());
    }

    attrs
}

pub async fn get_meta_by_record(
    path: &Path,
    file_info: &FileInfo,
) -> Result<FileMetadata, AppError> {
    let local_file = tokio::fs::File::open(path).await?;
    let mut file_meta = get_meta_by_local(path, &local_file).await?;
    file_meta.extension = file_info.file_ext.clone();
    file_meta.category = FileCategory::value_to_text(file_info.category).to_string();
    return Ok(file_meta);
}

/// Get file meta by local file
pub async fn get_meta_by_local(
    path: &Path,
    file_handle: &tokio::fs::File,
) -> Result<FileMetadata, AppError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string();
    let mut file_meta = FileMetadata::default();
    file_meta.name = file_name;
    file_handle.metadata().await.map(|meta| {
        file_meta.size = meta.len() as i64;
        file_meta.created = datetime_util::systemtime_to_datetime(
            meta.created()
                .unwrap_or((DateTime::default() as DateTime<Local>).into()),
        );
        file_meta.modified = datetime_util::systemtime_to_datetime(
            meta.modified()
                .unwrap_or((DateTime::default() as DateTime<Local>).into()),
        );
    })?;
    return Ok(file_meta);
}
