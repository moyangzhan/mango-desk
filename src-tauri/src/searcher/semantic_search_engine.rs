use crate::embedding_service_manager::get_manager;
use crate::entities::{FileContentEmbedding, FileInfo, FileMetaEmbedding};
use crate::enums::SearchSource;
use crate::errors::AppError;
use crate::global::{QUERY_MIN_SCORE, SHORT_QUERY_LEN};
use crate::repositories::{
    file_content_embedding_repo, file_info_repo, file_metadata_embedding_repo,
};
use crate::searcher::fts_search_engine;
use crate::structs::embed_result::EmbedResult;
use crate::structs::search_result::SearchResult;
use crate::utils::search_util::STOPWORDS;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tokio::{task, try_join};

#[derive(Debug, Clone)]
struct SearchTmp {
    file_id: i64,
    max_score: usize,
    chunk_ids: HashSet<i64>,
    sources: HashSet<SearchSource>,
    match_keywords: HashSet<String>,
}

pub async fn warmup_embedding_service() -> Result<(), AppError> {
    let mut manager = get_manager().write().await;
    manager.warmup().await?;
    Ok(())
}

pub async fn search(query: &str) -> Vec<SearchResult> {
    println!("Search: {}", query);
    let start = Instant::now();
    let embed_result = {
        let mut manager = get_manager().write().await;
        manager.embed(query).await.unwrap_or_else(|e| {
            log::error!("embed query failed, error: {:?}", e);
            EmbedResult::default()
        })
    };
    if embed_result.dense.is_empty() {
        return Vec::new();
    }
    let short_query = embed_result.sparse.indices.len() <= SHORT_QUERY_LEN;
    let (content_result, meta_result, fts_result) = try_join!(
        task::spawn_blocking({
            let embedding = embed_result.dense.clone();
            let sparse_indices = embed_result.sparse.indices.clone();
            let sparse_values = embed_result.sparse.values.clone();
            move || match file_content_embedding_repo::hybrid_search(
                &embedding,
                &sparse_indices,
                &sparse_values,
                QUERY_MIN_SCORE,
            ) {
                Ok(result) => result,
                Err(e) => {
                    log::error!("content search failed, error: {:?}", e);
                    Vec::new()
                }
            }
        }),
        task::spawn_blocking({
            let embedding = embed_result.dense.clone();
            let sparse_indices = embed_result.sparse.indices.clone();
            let sparse_values = embed_result.sparse.values.clone();
            move || match file_metadata_embedding_repo::hybrid_search(
                &embedding,
                &sparse_indices,
                &sparse_values,
                QUERY_MIN_SCORE,
            ) {
                Ok(result) => result,
                Err(e) => {
                    log::error!("meta search failed, error: {:?}", e);
                    Vec::new()
                }
            }
        }),
        task::spawn_blocking({
            let query = query.to_string();
            move || {
                if embed_result.sparse.indices.is_empty() || !short_query {
                    return Vec::new();
                }
                let words: Vec<&str> = query
                    .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
                    .filter(|word| !word.is_empty())
                    .filter(|word| {
                        let lower = word.to_lowercase();
                        !STOPWORDS.contains(lower.as_str())
                    })
                    .collect();
                let q = words.join(" ");
                tokio::runtime::Handle::current()
                    .block_on(async { fts_search_engine::search(&q).await })
            }
        }),
    )
    .unwrap_or_default();
    let result = merge_and_filter_results(content_result, meta_result, fts_result);
    println!("cost: {:?}", start.elapsed());
    result
}

/// Merge and filter the results from content and meta search
fn merge_and_filter_results(
    content_result: Vec<FileContentEmbedding>,
    meta_result: Vec<FileMetaEmbedding>,
    fts_result: Vec<SearchResult>,
) -> Vec<SearchResult> {
    if content_result.is_empty() && meta_result.is_empty() {
        return Vec::new();
    }
    let mut file_map: HashMap<i64, SearchTmp> = HashMap::new();

    // 1. 处理内容路结果
    for item in content_result {
        println!(
            "Content Search - file_id: {}, chunk_id: {}, score: {}",
            item.file_id, item.chunk_index, item.score
        );
        let entry = file_map.entry(item.file_id).or_insert(SearchTmp {
            file_id: item.file_id,
            max_score: 0,
            chunk_ids: HashSet::new(),
            sources: HashSet::new(),
            match_keywords: HashSet::new(),
        });
        entry.chunk_ids.insert(item.id);
        entry.sources.insert(SearchSource::ContentSemantic);
        if item.score > entry.max_score {
            entry.max_score = item.score;
        }
    }

    // 2. 处理元数据路结果 (赋予 1.1 倍的加成)
    for item in meta_result {
        println!(
            "Meta Search - file_id: {}, score: {}, distance: {}, sparse_score: {}",
            item.file_id, item.score, item.distance, item.sparse_score
        );
        let boosted_score = (item.score as f32 * 1.1).min(100.0) as usize;
        let entry = file_map.entry(item.file_id).or_insert(SearchTmp {
            file_id: item.file_id,
            max_score: 0,
            chunk_ids: HashSet::new(),
            sources: HashSet::new(),
            match_keywords: HashSet::new(),
        });
        entry.sources.insert(SearchSource::MetaSemantic);
        if boosted_score > entry.max_score {
            entry.max_score = boosted_score;
        }
    }

    let extact_words_match = !fts_result.is_empty();
    // 3. 处理 FTS 全文检索结果 (通常关键词匹配非常精准，给予固定高分或权重)
    for item in fts_result {
        //补偿10分
        let fts_boosted_score = (item.score.min(100) as usize + 10).min(100);
        let entry = file_map.entry(item.file_info.id).or_insert(SearchTmp {
            file_id: item.file_info.id,
            max_score: 0,
            chunk_ids: HashSet::new(),
            sources: HashSet::new(),
            match_keywords: HashSet::new(),
        });
        entry.sources.insert(SearchSource::ContentKeyword);
        for match_keyword in item.matched_keywords {
            println!(
                "FTS Search - file_id: {}, keyword: {}",
                item.file_info.id, match_keyword
            );
            entry.match_keywords.insert(match_keyword);
        }
        for chunk_id in item.matched_chunk_ids {
            entry.chunk_ids.insert(chunk_id);
        }
        if fts_boosted_score > entry.max_score {
            entry.max_score = fts_boosted_score;
        }
    }

    // 4. 如果有完全匹配关键词，则针对未命中关键词的文档进行【降权惩罚】
    if extact_words_match {
        for entry in file_map.values_mut() {
            // 如果该文档在 FTS 全文检索中完全没有出现
            if !entry.sources.contains(&SearchSource::ContentKeyword) {
                // 策略：得分打 6 折。例如：90分降至 54分，70分降至 42分
                // 这能有效让出高位给包含硬核关键词的结果
                entry.max_score = (entry.max_score as f32 * 0.6) as usize;
            }
        }
    }

    let mut tmps: Vec<SearchTmp> = file_map.into_values().collect();
    tmps.sort_by(|a, b| b.max_score.cmp(&a.max_score));
    tmps.retain(|t| t.max_score >= QUERY_MIN_SCORE || t.sources.contains(&SearchSource::ContentKeyword));

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
                score: tmp.max_score,
                sources: tmp.sources.into_iter().collect(),
                matched_keywords: tmp.match_keywords.into_iter().collect(),
                matched_chunk_ids: tmp.chunk_ids.into_iter().collect(),
            })
        })
        .collect()
}
