use crate::enums::{FileCategory, FileParserMode};
use serde::{Deserialize, Serialize};

/// Per-content-type storage setting: "database" | "file" | "none"
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ContentStorage {
    #[serde(default = "default_storage")]
    pub document: String,
    #[serde(default = "default_storage")]
    pub image: String,
    #[serde(default = "default_storage")]
    pub audio: String,
}

impl Default for ContentStorage {
    fn default() -> Self {
        Self {
            document: default_storage(),
            image: default_storage(),
            audio: default_storage(),
        }
    }
}

impl ContentStorage {
    pub fn get_for_category(&self, category: &FileCategory) -> &str {
        match category {
            FileCategory::Document => &self.document,
            FileCategory::Image => &self.image,
            FileCategory::Audio => &self.audio,
            _ => &self.document,
        }
    }
}

fn default_storage() -> String {
    "database".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexerSetting {
    pub is_private: bool, // Deprecated: kept for backward compatibility
    #[serde(default)]
    pub parser_mode: String, // "local" | "selfhosted" | "remote" | "mixed" - quick preset
    #[serde(default)]
    pub image_parser_mode: FileParserMode,
    #[serde(default)]
    pub audio_parser_mode: FileParserMode,
    pub ignore_dirs: Vec<String>,
    #[serde(default)]
    pub ignore_path_prefixes: Vec<String>, // Full path prefixes to ignore
    pub ignore_exts: Vec<String>,
    pub ignore_files: Vec<String], // File absolute path
    #[serde(default)]
    pub content_storage: ContentStorage,
}

impl Default for IndexerSetting {
    fn default() -> Self {
        Self {
            is_private: true,
            parser_mode: "local".to_string(),
            image_parser_mode: FileParserMode::Local,
            audio_parser_mode: FileParserMode::Local,
            ignore_exts: vec![],
            ignore_files: vec![],
            ignore_dirs: vec![],
            ignore_path_prefixes: vec![],
            content_storage: ContentStorage::default(),
        }
    }
}
