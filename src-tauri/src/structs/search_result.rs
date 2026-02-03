use crate::entities::FileInfo;
use crate::enums::SearchSource;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub sources: Vec<SearchSource>,
    pub file_info: FileInfo,
    pub matched_keywords: HashSet<String>, // For keyword search
    pub matched_chunk_ids: HashSet<i64>,   // For semantic search
}
