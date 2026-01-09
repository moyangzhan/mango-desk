use crate::enums::{FileCategory, FileContentLanguage};
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
    pub is_private: bool,
    pub file_content_language: FileContentLanguage, // en, multilingual
    pub ignore_dirs: Vec<String>,
    pub ignore_exts: Vec<String>,
    pub ignore_files: Vec<String>, // File absolute path
    #[serde(default)]
    pub save_parsed_content: SaveParsedContent,
}

impl Default for IndexerSetting {
    fn default() -> Self {
        Self {
            is_private: false,
            file_content_language: FileContentLanguage::English,
            ignore_exts: vec![],
            ignore_files: vec![],
            ignore_dirs: vec![],
            save_parsed_content: SaveParsedContent {
                document: false,
                image: true,
                video: true,
                audio: true,
            },
        }
    }
}
