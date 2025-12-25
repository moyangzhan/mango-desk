use crate::enums::FileContentLanguage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexerSetting {
    pub is_private: bool,
    pub file_content_language: FileContentLanguage, // en, multilingual
    pub ignore_dirs: Vec<String>,
    pub ignore_exts: Vec<String>,
    pub ignore_files: Vec<String>, // File absolute path
}

impl Default for IndexerSetting {
    fn default() -> Self {
        Self {
            is_private: false,
            file_content_language: FileContentLanguage::English,
            ignore_exts: vec![],
            ignore_files: vec![],
            ignore_dirs: vec![],
        }
    }
}
