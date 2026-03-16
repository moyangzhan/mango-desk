use crate::entities::FileInfo;
use crate::enums::SimilarityType;
use crate::errors::AppError;
use crate::repositories::{file_content_embedding_repo, file_info_repo};
use crate::structs::search_result::SearchResult;
use crate::traits::similarity_detector::SimilarityDetector;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

pub struct DocumentSimilarityDetector;

#[async_trait]
impl SimilarityDetector for DocumentSimilarityDetector {
    /// Find similar documents using multi-chunk search with score aggregation
    /// 使用多片段搜索和分数聚合查找相似文档
    async fn find_similars(&self, file_info: &FileInfo, limit: usize) -> Result<Vec<SearchResult>, AppError> {
        let embeddings = file_content_embedding_repo::list_by_file_id(file_info.id)?;

        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        // Get all document category file IDs for filtering (Document = 1)
        let doc_file_ids: HashSet<i64> = file_info_repo::list_ids_by_category(1)?
            .into_iter()
            .collect();

        // Aggregate scores from all chunks
        // file_id -> max_score across all chunk searches
        let mut file_scores: HashMap<i64, usize> = HashMap::new();

        // Search with each chunk embedding and aggregate results
        for source_embedding in &embeddings {
            let sparse_indices: Vec<u32> = source_embedding.sparse_vec.indices.clone();
            let sparse_values: Vec<f32> = source_embedding.sparse_vec.values.clone();

            let similar_embeddings = file_content_embedding_repo::hybrid_search(
                &source_embedding.embedding.to_vec(),
                &sparse_indices,
                &sparse_values,
                10, // min_score
            )?;

            // Update scores using max aggregation
            for emb in &similar_embeddings {
                if emb.file_id == file_info.id {
                    continue;
                }
                if !doc_file_ids.contains(&emb.file_id) {
                    continue;
                }
                let max_score = file_scores.entry(emb.file_id).or_insert(0);
                *max_score = (*max_score).max(emb.score);
            }
        }

        // Sort by score and take top N
        let mut sorted_files: Vec<(i64, usize)> = file_scores.into_iter().collect();
        sorted_files.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_files.truncate(limit);

        // Get file info for all similar files
        let file_ids: Vec<i64> = sorted_files.iter().map(|(id, _)| *id).collect();
        let files = file_info_repo::list_by_ids(&file_ids)?;

        // Build search results
        let file_map: HashMap<i64, FileInfo> = files.into_iter().map(|f| (f.id, f)).collect();
        let results: Vec<SearchResult> = sorted_files
            .into_iter()
            .filter_map(|(file_id, score)| {
                file_map.get(&file_id).map(|info| SearchResult {
                    score,
                    hit_types: vec![],
                    file_info: info.clone(),
                    matched_keywords: HashSet::new(),
                    matched_chunk_ids: HashSet::new(),
                    similarity_type: Some(SimilarityType::DocumentSemantic),
                })
            })
            .collect();

        Ok(results)
    }
}
