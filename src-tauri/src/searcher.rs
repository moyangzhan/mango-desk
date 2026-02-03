pub mod fts_search_engine;
pub mod path_search_engine;
pub mod semantic_search_engine;

use crate::enums::QueryIntent;
use crate::structs::search_result::SearchResult;
use crate::utils::search_util;
use std::collections::HashMap;
use tokio::{task, try_join};

pub async fn keyword_search(query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }
    println!("keyword_search query: {:?}", query);
    let query = query.to_owned();
    let (path_results, fts_results) = try_join!(
        task::spawn({
            let query = query.clone();
            async move { Ok(path_search_engine::search(&query).await) }
        }),
        task::spawn(async move { Ok(fts_search_engine::search(&query).await) }),
    )
    .map(|(path_res, fts_res)| {
        (
            path_res.unwrap_or_else(|_: Vec<SearchResult>| Vec::new()),
            fts_res.unwrap_or_else(|_: Vec<SearchResult>| Vec::new()),
        )
    })
    .unwrap_or_else(|_| (Vec::new(), Vec::new()));
    fuse_results(path_results, fts_results)
}

pub async fn semantic_search(query: &str) -> Vec<SearchResult> {
    semantic_search_engine::search(query).await
}

pub async fn search_with_intent(query: &str) -> Vec<SearchResult> {
    let intent = search_util::detect_intent(query);
    match intent {
        QueryIntent::PathOnly => path_search_engine::search(query).await,

        QueryIntent::SemanticOnly => semantic_search_engine::search(query).await,

        QueryIntent::Hybrid => parallel_search(query).await,
    }
}

async fn parallel_search(query: &str) -> Vec<SearchResult> {
    let query = query.to_owned();
    let (path_results, semantic_results) = try_join!(
        task::spawn({
            let query = query.clone();
            async move { Ok(path_search_engine::search(&query).await) }
        }),
        task::spawn(async move { Ok(semantic_search_engine::search(&query).await) }),
    )
    .map(|(path_res, semantic_res)| {
        (
            path_res.unwrap_or_else(|_: Vec<SearchResult>| Vec::new()),
            semantic_res.unwrap_or_else(|_: Vec<SearchResult>| Vec::new()),
        )
    })
    .unwrap_or_else(|_| (Vec::new(), Vec::new()));

    fuse_results(path_results, semantic_results)
}

fn fuse_results(
    left_results: Vec<SearchResult>,
    right_results: Vec<SearchResult>,
) -> Vec<SearchResult> {
    let mut map: HashMap<String, SearchResult> = HashMap::new();
    for r in left_results {
        map.entry(r.file_info.path.clone()).or_insert(r);
    }
    for mut r in right_results {
        let matched_chunk_ids = r.matched_chunk_ids.clone();
        let matched_keywords = r.matched_keywords.clone();
        map.entry(r.file_info.path.clone())
            .and_modify(|e| {
                e.score = e.score * 0.6 + r.score * 0.4;
                e.sources.append(&mut r.sources);
                e.matched_chunk_ids.extend(matched_chunk_ids.clone());
                e.matched_keywords.extend(matched_keywords.clone());
            })
            .or_insert_with(|| {
                let score = r.score;
                let file_info = r.file_info.clone();
                let sources = r.sources.clone();
                SearchResult {
                    file_info,
                    sources,
                    matched_chunk_ids,
                    matched_keywords,
                    score: score * 0.4,
                }
            });
    }
    let mut results: Vec<_> = map.into_values().collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}
