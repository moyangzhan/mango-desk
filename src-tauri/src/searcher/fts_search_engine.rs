use crate::enums::SearchSource;
use crate::global::DEFAULT_PAGE_SIZE;
use crate::repositories::file_content_fts_repo;
use crate::repositories::file_info_repo;
use crate::structs::search_result::SearchResult;
use std::time::Instant;

/// Keyword-based exact match search for file content.
pub async fn search(query: &str) -> Vec<SearchResult> {
    let start = Instant::now();
    let mut results = Vec::new();
    let fts_results = file_content_fts_repo::search(&query, DEFAULT_PAGE_SIZE)
        .await
        .unwrap_or_else(|error| {
            println!("fts search error: {}", error);
            Vec::new()
        });

    let mut file_ids = Vec::new();
    fts_results.iter().for_each(|fts_result| {
        file_ids.push(fts_result.file_id);
    });
    let file_infos = file_info_repo::list_by_ids(&file_ids).unwrap_or_default();
    file_infos.iter().for_each(|file_info| {
        let match_fts_item = fts_results
            .iter()
            .find(|fts_result| fts_result.file_id == file_info.id);
        if let Some(mfts_item) = match_fts_item {
            results.push(SearchResult {
                file_info: file_info.clone(),
                score: mfts_item.score,
                sources: vec![SearchSource::ContentKeyword],
                matched_keywords: mfts_item.matched_keywords.clone(),
                matched_chunk_ids: mfts_item.chunk_ids.clone(),
            });
        }
    });
    println!("content search time: {:?}", start.elapsed());
    results
}
