use crate::errors::AppError;
use crate::global::SUPPORTED_AUDIO_EXTS;
use crate::utils::base64_util::file_to_data_uri;

pub fn is_supported_audio_ext(ext: &str) -> bool {
    SUPPORTED_AUDIO_EXTS.contains(&ext)
}

/// Determine if a file is an audio file
/// Returns Ok(true) for an audio, Ok(false) for non-audio, and Err for file operation failure
pub fn is_supported_audio_file(path: &str) -> Result<bool, AppError> {
    if let Some(kind) = infer::get_from_path(path)? {
        let ext = kind.extension();
        if SUPPORTED_AUDIO_EXTS.contains(&ext) {
            return Ok(true);
        }
    }
    // No audio format matched
    Ok(false)
}

/// Convert audio file to Data URI
pub fn audio_to_data_uri(path: &str) -> Result<String, AppError> {
    let supported = is_supported_audio_file(path)?;
    if !supported {
        return Err(AppError::UnsupportedAudioFormat(path.to_string()));
    }
    let data_uri = file_to_data_uri(path)?;
    Ok(data_uri)
}
