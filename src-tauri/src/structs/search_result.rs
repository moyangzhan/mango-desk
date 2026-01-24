use crate::entities::FileInfo;
use crate::enums::SearchSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub source: SearchSource,
    pub file_info: FileInfo,
    pub matched_keywords: Vec<String>, // For path search
    pub matched_chunk_ids: Vec<i64>, // For semantic search
}
