use crate::entities::FileInfo;
use crate::enums::{FileCategory, SearchSource};
use crate::global::{PATHS_CACHE, PATHS_CACHE_BUILD_TIME};
use crate::repositories::file_info_repo;
use crate::structs::search_result::SearchResult;
use crate::utils::file_util;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use anyhow::{Result, anyhow};
use chrono::Local;
use rayon::prelude::*;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

const LIMIT: usize = 20;
pub async fn search(query: &str) -> Vec<SearchResult> {
    let start = Instant::now();
    let keywords: Vec<&str> = query.split_whitespace().collect();
    let Ok(automaton) = create_automaton(&keywords) else {
        return vec![];
    };
    let take_num = if keywords.len() == 1 {
        LIMIT
    } else {
        LIMIT * 10
    };
    let paths_cache = PATHS_CACHE.read().await;
    let mut result: Vec<SearchResult> = (*paths_cache)
        .par_iter()
        .enumerate()
        .filter_map(|(line_num, line)| {
            if line.is_empty() {
                return None;
            }
            //Scan for all keywords in the line
            let matches: SmallVec<[usize; 5]> = automaton
                .find_iter(line)
                .map(|m| m.pattern().as_usize())
                .collect();

            if matches.is_empty() {
                None
            } else {
                let (file_name, ext) = file_util::get_name_ext(line);
                let match_keywords = keywords
                    .iter()
                    .enumerate()
                    .filter_map(|(i, k)| {
                        if matches.contains(&i) {
                            Some(k.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                let result = SearchResult {
                    file_info: FileInfo {
                        id: line_num as i64,
                        path: line.to_string(),
                        name: file_name,
                        category: FileCategory::from_ext(ext.as_str()).value(),
                        file_ext: ext,
                        ..Default::default()
                    },
                    score: matches.len() as f32,
                    source: SearchSource::Path,
                    matched_keywords: match_keywords,
                };
                Some(result)
            }
        })
        .take_any(take_num)
        .collect();
    result.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.file_info.id.cmp(&b.file_info.id))
    });
    result.truncate(LIMIT);
    println!("path search time: {:?}", start.elapsed());
    result
}

pub async fn init() {
    build_index().await;
    tokio::spawn(paths_index_timer());
}
async fn build_index() {
    let size = 1000;
    {
        let mut cache = PATHS_CACHE.write().await;
        cache.clear();
    }
    let total = file_info_repo::count().unwrap_or_default();
    if total == 0 {
        println!("file_info total == 0");
        return;
    }
    let pages = (total + size - 1) / size;
    println!("total: {}, pages:{}", total, pages);
    for page in 1..=pages {
        let paths = match file_info_repo::list_paths(page, size, true) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error reading page {}: {}", page, e);
                break;
            }
        };
        if paths.is_empty() {
            break;
        }
        let mut cache = PATHS_CACHE.write().await;
        for path in paths {
            if path.is_empty() {
                continue;
            }
            (*cache).push(path);
        }
    }
    let mut guard = PATHS_CACHE_BUILD_TIME.write().await;
    *guard = Local::now()
}

pub fn create_automaton(keywords: &[&str]) -> Result<AhoCorasick> {
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .ascii_case_insensitive(true)
        .build(keywords)
        .map_err(|e| anyhow!("create automaton error:{}", e))
}

// PATHS_CACHE only accept two events: Remove and Add, the [rename] event from fs_event is seperate into Remove and Add
async fn paths_index_timer() {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        push_to_index().await;
    }
}

pub async fn push_to_index() {
    let last_build_time = PATHS_CACHE_BUILD_TIME.read().await.clone();
    let total = file_info_repo::count_by_min_update_time(&last_build_time).unwrap_or_default();
    if total == 0 {
        return;
    }
    let page_size = 1000;
    let pages = (total as f64 / page_size as f64).ceil() as u32;
    for page in 1..=pages {
        let paths =
            file_info_repo::list_paths_by_min_update_time(&last_build_time, page as i64, page_size)
                .unwrap_or_default();
        if paths.is_empty() {
            println!("no new paths");
            break;
        }
        let mut paths_guard = PATHS_CACHE.write().await;
        for path in paths {
            if !paths_guard.contains(&path) {
                paths_guard.push(path);
            }
        }
    }
    let mut guard = PATHS_CACHE_BUILD_TIME.write().await;
    *guard = Local::now();
}

// Handle delete path
pub async fn remove_from_index(path: &str, is_file: bool) {
    let mut paths = PATHS_CACHE.write().await;
    if is_file {
        (*paths).retain(|item| item != path);
    } else {
        let directory = format!("{}{}", path, std::path::MAIN_SEPARATOR);
        (*paths).retain(|item| !item.starts_with(directory.as_str()));
    }
}
