use crate::embedding_service_manager::get_manager;
use crate::entities::{FileContentEmbedding, FileInfo, FileMetaEmbedding};
use crate::enums::SearchSource;
use crate::errors::AppError;
use crate::repositories::{
    file_content_embedding_repo, file_info_repo, file_metadata_embedding_repo,
};
use crate::structs::search_result::SearchResult;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tokio::{task, try_join};

struct SearchTmp {
    file_id: i64,
    distance: f32,
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
            move || file_content_embedding_repo::search(&embedding, -1.0).unwrap_or_default()
        }),
        task::spawn_blocking({
            let embedding = embedding.clone();
            move || file_metadata_embedding_repo::search(&embedding, -1.0).unwrap_or_default()
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
    let mut seen = HashSet::new();
    let mut tmps: Vec<SearchTmp> = content_result
        .into_iter()
        .map(|item| SearchTmp {
            file_id: item.file_id,
            distance: item.distance,
        })
        .chain(meta_result.into_iter().map(|item| SearchTmp {
            file_id: item.file_id,
            distance: item.distance,
        }))
        .filter_map(|item| {
            if seen.insert(item.file_id) {
                Some(SearchTmp {
                    file_id: item.file_id,
                    distance: item.distance,
                })
            } else {
                None
            }
        })
        .collect();

    tmps.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let file_ids: Vec<i64> = tmps.iter().map(|t| t.file_id).collect();
    println!("file_ids: {:?}", file_ids);
    let file_infos = file_info_repo::list_by_ids(&file_ids).unwrap_or_default();
    if file_infos.is_empty() {
        println!("file_infos is empty");
        return Vec::new();
    }

    let file_map: HashMap<i64, FileInfo> =
        file_infos.into_iter().map(|info| (info.id, info)).collect();

    // Return results in the sorted order
    tmps.into_iter()
        .filter_map(|tmp| file_map.get(&tmp.file_id).cloned())
        .map(|info| SearchResult {
            file_info: info,
            score: 1.0,
            source: SearchSource::Semantic,
            matched_keywords: Vec::new(),
        })
        .collect()
}
