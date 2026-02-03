use crate::entities::FtsSearchResult;
use crate::repositories::RepositoryError;
use crate::utils::app_util::get_db_path;
use jieba_rs::Jieba;
use regex::Regex;
use rusqlite::{Connection, Result, named_params, params};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Instant;
use tokio::sync::RwLock as AsyncRwLock;

pub static JIEBA: LazyLock<AsyncRwLock<Jieba>> = LazyLock::new(|| AsyncRwLock::new(Jieba::new()));

async fn tokenize(text: &str) -> String {
    let jieba = JIEBA.read().await;
    jieba
        .cut(text, true)
        .into_iter()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn insert(file_id: i64, chunk_id: i64, chunk_text: &str) -> Result<(), RepositoryError> {
    if chunk_text.is_empty() {
        return Ok(());
    }
    let content = tokenize(chunk_text).await;
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("insert into file_content_fts (file_id, chunk_id, content) values (:file_id, :chunk_id, :content)")?;
    let last_insert_rowid = stmt.insert(named_params! {
        ":file_id": file_id,
        ":chunk_id": chunk_id,
        ":content": content,
    })?;
    println!(
        "insert file_content_fts last_insert_rowid:{}",
        last_insert_rowid
    );
    Ok(())
}

pub async fn update(file_id: i64, chunk_id: i64, chunk_text: &str) -> Result<(), RepositoryError> {
    if chunk_text.is_empty() {
        return Ok(());
    }
    let content = tokenize(chunk_text).await;
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("update file_content_fts set content = :content where file_id = :file_id and chunk_id = :chunk_id")?;
    let affected = stmt.execute(named_params! {
        ":file_id": file_id,
        ":chunk_id": chunk_id,
        ":content": content,
    })?;
    println!("update file_content_fts affected:{}", affected);
    Ok(())
}

pub fn delete_by_file_id(file_id: i64) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_fts where file_id = ?1")?;
    let affected = stmt.execute([file_id])?;
    println!(
        "delete file_content_fts by file id affected: {:?}",
        affected
    );
    Ok(affected)
}

pub fn delelte_by_prefix_path(pre_path: &str) -> Result<usize, RepositoryError> {
    if pre_path.is_empty() {
        return Ok(0);
    }
    let pattern = if pre_path.ends_with(std::path::MAIN_SEPARATOR) {
        format!("{}%", pre_path)
    } else {
        format!("{}{}%", pre_path, std::path::MAIN_SEPARATOR)
    };
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_fts join file_info on file_content_fts.file_id = file_info.id where file_info.path = ?1 or file_info.path like ?2")?;
    let affected = stmt.execute((pre_path, pattern))?;
    println!(
        "delete file_content_fts by prefix path affected: {:?}",
        affected
    );
    Ok(affected)
}

pub fn clear() -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_fts")?;
    let affected = stmt.execute([])?;
    println!("clear file_content_fts affected: {:?}", affected);
    Ok(affected)
}

/// Multi-word search set to OR operator by default
pub async fn search(query: &str, limit: usize) -> Result<Vec<FtsSearchResult>, RepositoryError> {
    let start = Instant::now();
    let tokens = tokenize(&query).await;
    let keywords: Vec<&str> = tokens
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .collect();
    let match_query = keywords
        .iter()
        .map(|k| k.to_string())
        .collect::<Vec<_>>()
        .join(" OR ");
    let conn = Connection::open(get_db_path())?;
    let max_rows = if keywords.len() <= 2 { 500 } else { 700 };
    let mut stmt = conn.prepare(
        r#"
        SELECT file_id, chunk_id, content, "rank"
        FROM file_content_fts
        WHERE file_content_fts MATCH ?
        ORDER BY "rank" DESC
        LIMIT ?;
        "#,
    )?;
    let rows = stmt.query_map(params![match_query, max_rows], |row| {
        Ok((
            row.get::<_, i64>("file_id")?,
            row.get::<_, i64>("chunk_id")?,
            row.get::<_, String>("content")?,
            row.get::<_, f64>("rank").unwrap_or(0.0),
        ))
    })?;
    println!("search file_content_fts time: {:?}", start.elapsed());
    let mut file_map: HashMap<i64, FtsSearchResult> = HashMap::new();
    for row in rows {
        let (file_id, chunk_id, content, rank) = row?;
        let mut hit_count = 0;
        let mut matched_keyword = HashSet::new();
        for kw in &keywords {
            if content.contains(kw) {
                hit_count += 1;
                matched_keyword.insert(kw.to_string());
            }
        }
        let score = rank.max(hit_count as f64);
        // Vue will do this
        // let snippet = make_snippet(&content, &keywords, 40);
        let entry = file_map.entry(file_id).or_insert_with(|| FtsSearchResult {
            file_id,
            chunk_ids: HashSet::new(),
            matched_keywords: HashSet::new(),
            score: 0.0,
        });
        entry.chunk_ids.insert(chunk_id);
        entry.matched_keywords.extend(matched_keyword.into_iter());
        entry.score = entry.score.max(score);
    }
    let mut results: Vec<FtsSearchResult> = file_map.into_values().collect();
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    println!("search file_content_fts total cost: {:?}", start.elapsed());
    Ok(results)
}

fn extract_matches(content: &str, offsets: &str) -> Vec<String> {
    let mut result = Vec::new();
    let nums: Vec<usize> = offsets
        .split_whitespace()
        .filter_map(|x| x.parse().ok())
        .collect();
    for chunk in nums.chunks(4) {
        if chunk.len() == 4 {
            let byte_offset = chunk[2];
            let byte_len = chunk[3];
            if let Some(matched) = content.get(byte_offset..byte_offset + byte_len) {
                result.push(matched.to_string());
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

pub fn make_snippet(text: &str, keywords: &[&str], radius: usize) -> String {
    if text.is_empty() || keywords.is_empty() {
        return text.chars().take(radius * 2).collect();
    }
    let mut first_pos: Option<usize> = None;
    for kw in keywords {
        if let Some(pos) = text.find(kw) {
            first_pos = match first_pos {
                Some(existing) => Some(existing.min(pos)),
                None => Some(pos),
            };
        }
    }
    let pos = match first_pos {
        Some(p) => p,
        None => return text.chars().take(radius * 2).collect(),
    };
    let char_positions: Vec<(usize, char)> = text.char_indices().collect();
    let total_chars = char_positions.len();

    let char_index = char_positions
        .iter()
        .position(|(byte_idx, _)| *byte_idx >= pos)
        .unwrap_or(0);

    let start_char = char_index.saturating_sub(radius);
    let end_char = (char_index + radius).min(total_chars - 1);

    let start_byte = char_positions[start_char].0;
    let end_byte = if end_char + 1 < total_chars {
        char_positions[end_char + 1].0
    } else {
        text.len()
    };

    let mut snippet = text[start_byte..end_byte].to_string();

    // highlight keywords
    for kw in keywords {
        if kw.is_empty() {
            continue;
        }
        let pattern = Regex::new(&regex::escape(kw)).unwrap();
        snippet = pattern
            .replace_all(&snippet, |caps: &regex::Captures| {
                format!("<b>{}</b>", &caps[0])
            })
            .to_string();
    }

    if start_byte > 0 {
        snippet = format!("...{}", snippet);
    }
    if end_byte < text.len() {
        snippet = format!("{}...", snippet);
    }

    snippet
}
