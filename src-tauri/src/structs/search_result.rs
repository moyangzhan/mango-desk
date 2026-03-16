use crate::entities::FileInfo;
use crate::enums::{HitType, SimilarityType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub score: usize, // 0 - 100
    pub hit_types: Vec<HitType>,
    pub file_info: FileInfo,
    pub matched_keywords: HashSet<String>, // For keyword search
    pub matched_chunk_ids: HashSet<i64>,   // For semantic search
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub similarity_type: Option<SimilarityType>, // For similarity search
}
