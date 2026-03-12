use crate::entities::FileContentEmbedding;
use crate::global::SHORT_QUERY_LEN;
use crate::repositories::RepositoryError;
use crate::structs::sparse_vector::SparseVector;
use crate::utils::app_util::get_db_path;
use crate::utils::vector_util::calculate_content_score;
use rusqlite::{Connection, Result, Row, named_params};

pub fn insert(
    file_content_embedding: &FileContentEmbedding,
) -> Result<Option<FileContentEmbedding>, RepositoryError> {
    let mut conn = Connection::open(get_db_path())?;
    let tx = conn.transaction()?;

    let file_content_embedding = {
        let mut stmt = tx.prepare("insert into file_content_vec(embedding) values (:embedding)")?;
        let embedding_bytes = unsafe {
            std::slice::from_raw_parts(
                file_content_embedding.embedding.as_ptr() as *const u8,
                file_content_embedding.embedding.len() * std::mem::size_of::<f32>(),
            )
        };
        let last_insert_rowid = stmt.insert(named_params! {
            ":embedding": &embedding_bytes,
        })?;
        println!(
            "file_content_embedding_repo.insert() last_insert_rowid: {}",
            last_insert_rowid
        );

        // file_content_data
        let mut stmt = tx.prepare("insert into file_content_data(id, file_id,sparse_weights,chunk_index,chunk_text) values (:id,:file_id,:sparse_weights,:chunk_index,:chunk_text)")?;
        let _ = stmt.insert(named_params! {
            ":id": last_insert_rowid,
            ":file_id": &file_content_embedding.file_id,
            ":chunk_index": &file_content_embedding.chunk_index,
            ":chunk_text": &file_content_embedding.chunk_text,
            ":sparse_weights": &file_content_embedding.sparse_vec.to_blob(),
        })?;

        //where rowid = ?1 will cause error: no such column: rowid ???
        let mut query_stmt = tx.prepare("select v.id, d.file_id, d.chunk_index, d.chunk_text, -0.1 as distance, d.sparse_weights from file_content_vec v join file_content_data d on v.id = d.id where d.file_id = ?1 order by v.id desc limit 1",)?;
        let file_content_embedding = query_stmt
            .query_row([&file_content_embedding.file_id], |row| {
                Ok(Some(build_file_content_embedding(row)?))
            })
            .unwrap_or_else(|e| {
                println!("file_content_embedding_repo.insert() Error: {}", e);
                None
            });

        file_content_embedding
    };
    tx.commit()?;
    Ok(file_content_embedding)
}

pub fn update(file_content_embedding: &FileContentEmbedding) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("update file_content_vec set embedding=:embedding where id = :id")?;
    let embedding_bytes = unsafe {
        std::slice::from_raw_parts(
            file_content_embedding.embedding.as_ptr() as *const u8,
            file_content_embedding.embedding.len() * std::mem::size_of::<f32>(),
        )
    };
    let affected = stmt.execute(named_params! {
        ":id": &file_content_embedding.id,
        ":embedding": embedding_bytes,
    })?;
    println!("update file_content_vec affected: {:?}", affected);
    Ok(affected)
}

/// dense + sparse hybrid search
pub fn hybrid_search(
    dense_embedding: &[f32],
    query_sparse_indices: &[u32],
    query_sparse_values: &[f32],
    min_score: usize, // 1 - 100
) -> Result<Vec<FileContentEmbedding>, RepositoryError> {
    let is_short_query = query_sparse_indices.len() <= SHORT_QUERY_LEN;
    let sparse_threshold = if is_short_query { 0.01 } else { 0.0 };

    let embedding_bytes = unsafe {
        std::slice::from_raw_parts(
            dense_embedding.as_ptr() as *const u8,
            dense_embedding.len() * std::mem::size_of::<f32>(),
        )
    };
    println!(
        "content hybrid_search, query_sparse_indices: {:?},query_sparse_values: {:?}, min_score: {}",
        query_sparse_indices, query_sparse_values, min_score
    );
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select v.id, d.file_id, d.chunk_index, d.chunk_text, v.distance, d.sparse_weights from ( select id, distance from file_content_vec where embedding match :embedding order by distance limit 50 ) v join file_content_data d on v.id = d.id order by v.distance asc")?;
    let rows = stmt.query_map(named_params! {":embedding": embedding_bytes}, |row| {
        let embedding_bytes: Vec<u8> = row.get("embedding").unwrap_or(vec![0; 1024 * 4]); // Default to zero vector if embedding is not available
        let embedding: [f32; 1024] = unsafe {
            let ptr = embedding_bytes.as_ptr() as *const f32;
            std::ptr::read(ptr as *const [f32; 1024])
        };
        let sparse_blob: Vec<u8> = row.get("sparse_weights").unwrap_or(vec![]); // Default to empty vector if sparse_weights is not available
        let sparse_vec: SparseVector = SparseVector::from_blob(&sparse_blob);
        let distance: f32 = row.get("distance").unwrap_or(1.0); // Default to 1.0 if distance is not available
        let sparse_score = sparse_vec.dot_product(query_sparse_indices, query_sparse_values);
        Ok(FileContentEmbedding {
            id: row.get("id")?,
            file_id: row.get("file_id")?,
            chunk_index: row.get("chunk_index")?,
            chunk_text: row.get("chunk_text")?,
            embedding,
            sparse_vec,

            distance: distance,
            sparse_score,
            score: calculate_content_score(distance, sparse_score, is_short_query),
        })
    })?;
    println!("Raw search results count: {}", rows.size_hint().0);
    let filtered_result = rows
        .filter_map(|res| match res {
            Ok(fce) => {
                println!(
                    "Raw Search Result - file_id: {}, chunk_id: {}, distance: {}, sparse_score: {}, score: {}",
                    fce.file_id, fce.chunk_index, fce.distance, fce.sparse_score, fce.score
                );
                // 1. 如果 Sparse 得分太低（说明关键词完全没对上），视为噪音，剔除 | If Sparse score is too low (keywords don't match), treat as noise and filter out
                // 这是解决”中译英”短查询乱码召回的关键 | This is key to fixing Chinese-to-English short query noise recall
                if fce.sparse_score < sparse_threshold {
                    return None;
                }

                // 2. 如果 Dense 距离太大，踢掉（说明语义不相关），无论 Sparse 得分多高都没用 | If Dense distance is too large, filter out (semantically unrelated), regardless of Sparse score
                if min_score > 0 && fce.score < min_score {
                    return None;
                }

                Some(fce)
            }
            Err(e) => {
                eprintln!("Error retrieving file embedding: {}", e);
                None
            }
        })
        .collect::<Vec<_>>();
    return Ok(filtered_result);
}

pub fn list_chunks_by_ids(ids: &Vec<u32>) -> Result<Vec<String>, RepositoryError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let conn = Connection::open(get_db_path())?;
    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let mut stmt = conn.prepare(&format!(
        "select chunk_text from file_content_data where id in ({}) order by chunk_index asc",
        ids_str
    ))?;
    let rows = stmt.query_map((), |row| {
        let chunk_text: String = row.get("chunk_text")?;
        Ok(chunk_text)
    })?;
    let mut segments: Vec<String> = Vec::new();
    for row_result in rows {
        match row_result {
            Ok(chunk_text) => segments.push(chunk_text),
            Err(e) => eprintln!("Error retrieving chunk text: {}", e),
        }
    }
    Ok(segments)
}

pub fn delete_by_file_id(file_id: i64) -> Result<usize, RepositoryError> {
    if file_id < 1 {
        return Ok(0);
    }
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_vec where id in (select id from file_content_data where file_id = :file_id)")?;
    let _ = stmt.execute(named_params! {":file_id": file_id})?;
    let mut stmt = conn.prepare("delete from file_content_data where file_id = :file_id")?;
    let affected = stmt.execute(named_params! {":file_id": file_id})?;
    Ok(affected)
}

pub fn delete_by_file_prefix_path(file_prefix_path: &str) -> Result<usize, RepositoryError> {
    if file_prefix_path.is_empty() {
        return Ok(0);
    }
    let pattern = if file_prefix_path.ends_with(std::path::MAIN_SEPARATOR) {
        format!("{}%", file_prefix_path)
    } else {
        format!("{}{}%", file_prefix_path, std::path::MAIN_SEPARATOR)
    };
    let mut conn = Connection::open(get_db_path())?;
    let tx = conn.transaction()?;
    let affected = {
        let mut stmt_vec = tx.prepare( "delete from file_content_vec where id in ( 
        select d.id from file_content_data d join file_info f on d.file_id = f.id where f.path like :prefix_path 
        )", )?;
        stmt_vec.execute(named_params! {":prefix_path": pattern})?;

        let mut stmt_data = tx.prepare(
            "delete from file_content_data where file_id in (
                select id from file_info where path like :prefix_path
            )",
        )?;
        stmt_data.execute(named_params! {":prefix_path": pattern})?
    };
    tx.commit()?;
    Ok(affected)
}

pub fn clear() -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_vec")?;
    let affected = stmt.execute([])?;
    let mut stmt = conn.prepare("delete from file_content_data")?;
    let affected_data = stmt.execute([])?;
    Ok(affected + affected_data)
}

fn build_file_content_embedding(row: &Row<'_>) -> Result<FileContentEmbedding, RepositoryError> {
    let embedding_bytes: Vec<u8> = row.get("embedding").unwrap_or(vec![0; 1024 * 4]); // Default to zero vector if embedding is not available
    let embedding: [f32; 1024] = unsafe {
        let ptr = embedding_bytes.as_ptr() as *const f32;
        std::ptr::read(ptr as *const [f32; 1024])
    };
    let sparse_blob: Vec<u8> = row.get("sparse_weights")?;
    return Ok(FileContentEmbedding {
        id: row.get("id")?,
        file_id: row.get("file_id")?,
        embedding,
        chunk_index: row.get("chunk_index")?,
        chunk_text: row.get("chunk_text")?,
        distance: row.get("distance")?,
        sparse_vec: SparseVector::from_blob(&sparse_blob),
        sparse_score: 0.0,
        score: 0,
    });
}

pub fn count() -> Result<i64, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM file_content_vec")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    return Ok(count);
}
