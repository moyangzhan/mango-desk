use crate::errors::AppError;
use crate::global::SUPPORTED_IMAGE_EXTS;
use crate::utils::base64_util::file_to_data_uri;

fn is_supported_image_ext(ext: &str) -> bool {
    SUPPORTED_IMAGE_EXTS.contains(&ext)
}

/// Determine if a file is an image (based on magic number verification)
/// Returns Ok(true) for an image, Ok(false) for non-image, and Err for file operation failure
pub fn is_supported_image_file(path: &str) -> Result<bool, AppError> {
    if let Some(kind) = infer::get_from_path(path)? {
        let ext = kind.extension();
        if SUPPORTED_IMAGE_EXTS.contains(&ext) {
            return Ok(true);
        }
    }
    // No image format matched
    Ok(false)
}

/// Convert image file to Data URI
pub fn image_to_data_uri(image_path: &str) -> Result<String, AppError> {
    // Check if the file is an image with magic number verification
    let supported = is_supported_image_file(image_path)?;
    if !supported {
        return Err(AppError::UnsupportedFormat("".to_string()));
    }

    let data_uri = file_to_data_uri(image_path)?;
    Ok(data_uri)
}

// Example usage
fn main() {
    let test_files = [
        "test.jpg",     // Should return true
        "document.txt", // Should return false
        "image.png",    // Should return true
        "invalid.webp", // Should return false if file content is not WebP
    ];

    for path in test_files {
        match is_supported_image_file(path) {
            Ok(true) => println!("✅ {}: Is an supported image file", path),
            Ok(false) => println!("❌ {}: Is not an supported image file", path),
            Err(e) => println!("⚠️ {}: Check failed - {}", path, e),
        }
    }

    // Example usage of image_to_data_uri
    match image_to_data_uri("test.jpg") {
        Ok(data_uri) => println!(
            "Data URI for test.jpg: {}",
            &data_uri[..std::cmp::min(60, data_uri.len())]
        ),
        Err(e) => println!("Failed to convert image to data URI: {}", e),
    }
}
