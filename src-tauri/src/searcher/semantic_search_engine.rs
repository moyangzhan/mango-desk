use crate::embedding_service_manager::get_manager;
use crate::entities::{FileContentEmbedding, FileInfo, FileMetaEmbedding};
use crate::enums::SearchSource;
use crate::errors::AppError;
use crate::repositories::{
    file_content_embedding_repo, file_info_repo, file_metadata_embedding_repo,
};
use crate::structs::search_result::SearchResult;
use std::collections::HashMap;
use std::time::Instant;
use tokio::{task, try_join};

#[derive(Debug, Clone)]
struct SearchTmp {
    file_id: i64,
    distance: f32,
    chunk_ids: Vec<i64>,
}

pub async fn warmup_embedding_service() -> Result<(), AppError> {
    let mut manager = get_manager().write().await;
    manager.warmup().await?;
    Ok(())
}

pub async fn search(query: &str) -> Vec<SearchResult> {
    let start = Instant::now();
    let embedding = {
        let mut manager = get_manager().write().await;
        manager.embed(query).await.unwrap_or_default()
    };
    if embedding.is_empty() {
        return Vec::new();
    }
    let checkpoint1 = start.elapsed();
    println!("checkpoint1 {:?}", checkpoint1);
    let (content_result, meta_result) = try_join!(
        task::spawn_blocking({
            let embedding = embedding.clone();
            move || file_content_embedding_repo::search(&embedding, 0.7).unwrap_or_default()
        }),
        task::spawn_blocking({
            let embedding = embedding.clone();
            move || file_metadata_embedding_repo::search(&embedding, 0.7).unwrap_or_default()
        }),
    )
    .unwrap_or_default();
    let checkpoint2 = start.elapsed();
    println!("checkpoint2: {:?}", checkpoint2 - checkpoint1);
    let result = merge_and_filter_results(content_result, meta_result);
    let checkpoint3 = start.elapsed();
    println!("checkpoint3: {:?}", checkpoint3 - checkpoint2);
    result
}

/// Merge and filter the results from content and meta search
fn merge_and_filter_results(
    content_result: Vec<FileContentEmbedding>,
    meta_result: Vec<FileMetaEmbedding>,
) -> Vec<SearchResult> {
    if content_result.is_empty() && meta_result.is_empty() {
        return Vec::new();
    }
    let mut file_map: HashMap<i64, SearchTmp> = HashMap::new();

    for item in content_result {
        let entry = file_map.entry(item.file_id).or_insert(SearchTmp {
            file_id: item.file_id,
            distance: item.distance,
            chunk_ids: Vec::new(),
        });
        entry.chunk_ids.push(item.id);
        // Keep the minimum distance value
        if item.distance < entry.distance {
            entry.distance = item.distance;
        }
    }

    for item in meta_result {
        file_map.entry(item.file_id).or_insert(SearchTmp {
            file_id: item.file_id,
            distance: item.distance,
            chunk_ids: Vec::new(),
        });
        // Metadata results don't add segment_ids, but may update distance
        if let Some(entry) = file_map.get_mut(&item.file_id) {
            if item.distance < entry.distance {
                entry.distance = item.distance;
            }
        }
    }

    let mut tmps: Vec<SearchTmp> = file_map.into_values().collect();
    tmps.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let file_ids: Vec<i64> = tmps.iter().map(|t| t.file_id).collect();
    let file_infos = file_info_repo::list_by_ids(&file_ids).unwrap_or_default();
    if file_infos.is_empty() {
        println!("file_infos is empty");
        return Vec::new();
    }

    let file_map: HashMap<i64, FileInfo> =
        file_infos.into_iter().map(|info| (info.id, info)).collect();
    // Return results in the sorted order
    tmps.into_iter()
        .filter_map(|tmp| {
            let file_id = tmp.file_id;
            let info = file_map.get(&file_id).cloned();
            if info.is_none() {
                return None;
            }
            Some(SearchResult {
                file_info: info.unwrap_or_default(),
                score: tmp.distance,
                source: SearchSource::Semantic,
                matched_keywords: Vec::new(),
                matched_chunk_ids: tmp.chunk_ids,
            })
        })
        .collect()
}
