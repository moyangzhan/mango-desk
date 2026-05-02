pub mod anytomd_loader;
pub mod ods;
pub mod odp;
pub mod odt;
pub mod open_doc;
pub mod pdfplumber_loader;

use crate::global::EXTRACTED_IMAGES_PATH;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub(crate) fn get_images_dir(source_path: &Path) -> PathBuf {
    let extracted_path = EXTRACTED_IMAGES_PATH
        .get()
        .cloned()
        .unwrap_or_default();
    let canonical = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    PathBuf::from(extracted_path).join(hash)
}

/// Truncate content in-place to at most `limit` chars.
/// Uses byte-length heuristic to avoid scanning short strings.
pub(crate) fn truncate_to_char_limit(content: &mut String, limit: usize) {
    if content.len() > limit * 4 {
        if let Some((byte_pos, _)) = content.char_indices().nth(limit) {
            content.truncate(byte_pos);
        }
    }
}
