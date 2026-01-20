pub mod path_search_engine;
pub mod semantic_search_engine;

use crate::enums::QueryIntent;
use crate::structs::search_result::SearchResult;
use crate::utils::search_util;
use tokio::{task, try_join};

pub async fn path_search(query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }
    let result = path_search_engine::search(query).await;
    result
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
    path_results: Vec<SearchResult>,
    semantic_results: Vec<SearchResult>,
) -> Vec<SearchResult> {
    use std::collections::HashMap;

    let mut map: HashMap<String, SearchResult> = HashMap::new();

    for r in path_results {
        map.entry(r.file_info.path.clone()).or_insert(r);
    }

    for mut r in semantic_results {
        map.entry(r.file_info.path.clone())
            .and_modify(|e| {
                e.score = e.score * 0.6 + r.score * 0.4;
            })
            .or_insert_with(|| {
                r.score *= 0.4;
                r
            });
    }

    let mut results: Vec<_> = map.into_values().collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}
