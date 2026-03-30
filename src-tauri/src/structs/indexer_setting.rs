use crate::enums::{FileCategory, FileParserMode};
use serde::{Deserialize, Serialize};

/// @see enums.rs FileCategory
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct SaveParsedContent {
    document: bool,
    image: bool,
    video: bool,
    audio: bool,
}

impl Default for SaveParsedContent {
    fn default() -> Self {
        Self {
            document: false,
            image: true,
            video: true,
            audio: true,
        }
    }
}

impl SaveParsedContent {
    pub fn need_store(&self, file_category: &FileCategory) -> bool {
        match file_category {
            FileCategory::Document => self.document,
            FileCategory::Image => self.image,
            FileCategory::Video => self.video,
            FileCategory::Audio => self.audio,
            _ => false,
        }
    }
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
    pub ignore_files: Vec<String>, // File absolute path
    #[serde(default)]
    pub save_parsed_content: SaveParsedContent,
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
            save_parsed_content: SaveParsedContent {
                document: false,
                image: true,
                video: true,
                audio: true,
            },
        }
    }
}
