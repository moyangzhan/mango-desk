use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::Path;

use crate::errors::AppError;

pub fn get_mine_type(path: &Path) -> &'static str {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let mime_type = match extension.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        _ => "application/octet-stream",
    };
    return mime_type;
}

pub fn file_to_data_uri(file_path: &str) -> Result<String, AppError> {
    // Check if the file exists and is an image
    if file_path.is_empty() {
        return Err(AppError::UnsupportedPath(file_path.to_string()));
    }

    // Check if the file exists
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::PathNotExist(file_path.to_string()));
    }

    let file_bytes = fs::read(path)?;

    // File is too short (less than 20 bytes), so it's not an image or audio
    if file_bytes.len() < 20 {
        return Err(AppError::FileIsInvalid(format!(
            "file is too small to be an image or audio:{}",
            file_path.to_string()
        )));
    }

    let base64 = general_purpose::STANDARD.encode(file_bytes);

    let mime_type = get_mine_type(path);
    let data_uri = format!("data:{};base64,{}", mime_type, base64);

    Ok(data_uri)
}
