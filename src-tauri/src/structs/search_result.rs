use crate::entities::FileInfo;
use crate::enums::{HitType, SimilarityType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Source device information for remote search results
/// Only present when result comes from a remote device
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SourceDevice {
    pub device_id: String,
    pub device_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub score: usize, // 0 - 100
    pub hit_types: Vec<HitType>,
    pub file_info: FileInfo,
    pub matched_keywords: HashSet<String>, // For keyword search
    pub matched_chunk_ids: HashSet<i64>,   // For semantic search
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub similarity_type: Option<SimilarityType>, // For similarity search
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_device: Option<SourceDevice>, // Only set for remote device results
}
