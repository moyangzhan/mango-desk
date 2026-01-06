use crate::entities::FileContentEmbedding;
use crate::repositories::RepositoryError;
use crate::utils::app_util::get_db_path;
use rusqlite::{Connection, Error, Result, Row, named_params};

pub fn insert(
    file_content_embedding: &FileContentEmbedding,
) -> Result<Option<FileContentEmbedding>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "insert into file_content_embedding(file_id,embedding,chunk_index,chunk_text) values (:file_id,:embedding,:chunk_index,:chunk_text)",
    )?;

    let embedding_bytes = unsafe {
        std::slice::from_raw_parts(
            file_content_embedding.embedding.as_ptr() as *const u8,
            file_content_embedding.embedding.len() * std::mem::size_of::<f32>(),
        )
    };

    let last_insert_rowid = stmt.insert(named_params! {
        ":file_id": &file_content_embedding.file_id,
        ":embedding": &embedding_bytes,
        ":chunk_index": &file_content_embedding.chunk_index,
        ":chunk_text": &file_content_embedding.chunk_text,
    })?;
    println!(
        "file_content_embedding_repo.insert() last_insert_rowid: {}",
        last_insert_rowid
    );
    //where rowid = ?1 will cause error: no such column: rowid ???
    let mut query_stmt = conn.prepare(
        "select *, -0.1 as distance from file_content_embedding where file_id = ?1 order by id desc limit 1",
    )?;
    let file_content_embedding = query_stmt
        .query_row([&file_content_embedding.file_id], |row| {
            Ok(Some(build_file_content_embedding(row)?))
        })
        .unwrap_or_else(|e| {
            println!("file_content_embedding_repo.insert() Error: {}", e);
            None
        });

    Ok(file_content_embedding)
}

pub fn update(file_content_embedding: &FileContentEmbedding) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "update file_content_embedding set file_id=:file_id,embedding=:embedding where id = :id",
    )?;
    let embedding_bytes = unsafe {
        std::slice::from_raw_parts(
            file_content_embedding.embedding.as_ptr() as *const u8,
            file_content_embedding.embedding.len() * std::mem::size_of::<f32>(),
        )
    };
    let affected = stmt.execute(named_params! {
        ":id": &file_content_embedding.id,
        ":file_id": &file_content_embedding.file_id,
        ":embedding": embedding_bytes,
    })?;
    println!("update file_content_embedding affected: {:?}", affected);
    Ok(affected)
}

pub fn search(
    embedding: &[f32],
    max_distance: f32,
) -> Result<Vec<FileContentEmbedding>, RepositoryError> {
    let embedding_bytes = unsafe {
        std::slice::from_raw_parts(
            embedding.as_ptr() as *const u8,
            embedding.len() * std::mem::size_of::<f32>(),
        )
    };

    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select *,distance from file_content_embedding where embedding match :embedding order by distance limit 10")?;
    let rows = stmt.query_map(named_params! {":embedding": embedding_bytes}, |row| {
        let embedding_bytes: Vec<u8> = row.get("embedding")?;
        let embedding: [f32; 384] = unsafe {
            let ptr = embedding_bytes.as_ptr() as *const f32;
            std::ptr::read(ptr as *const [f32; 384])
        };
        Ok(FileContentEmbedding {
            id: row.get("id")?,
            file_id: row.get("file_id")?,
            chunk_index: row.get("chunk_index")?,
            chunk_text: row.get("chunk_text")?,
            embedding,
            distance: row.get("distance")?,
        })
    })?;
    if max_distance < 0.0 {
        return Ok(rows.collect::<Result<Vec<FileContentEmbedding>, Error>>()?);
    }
    let filtered_result = rows
        .into_iter()
        .filter_map(|res| match res {
            Ok(fe) => {
                if fe.distance <= max_distance {
                    return Some(fe);
                }
                None
            }
            Err(e) => {
                eprintln!("Error retrieving file embedding: {}", e);
                None
            }
        })
        .collect::<Vec<FileContentEmbedding>>();
    return Ok(filtered_result);
}

pub fn delete_by_file_id(file_id: i64) -> Result<usize, RepositoryError> {
    if file_id < 1 {
        return Ok(0);
    }
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_embedding where file_id = :file_id")?;
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
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_content_embedding where file_id in (select id from file_info where path like :prefix_path)")?;
    let affected = stmt.execute(named_params! {":prefix_path": pattern})?;
    Ok(affected)
}

fn build_file_content_embedding(row: &Row<'_>) -> Result<FileContentEmbedding, RepositoryError> {
    let embedding_bytes: Vec<u8> = row.get("embedding")?;
    let embedding: [f32; 384] = unsafe {
        let ptr = embedding_bytes.as_ptr() as *const f32;
        std::ptr::read(ptr as *const [f32; 384])
    };
    return Ok(FileContentEmbedding {
        id: row.get("id")?,
        file_id: row.get("file_id")?,
        embedding,
        chunk_index: row.get("chunk_index")?,
        chunk_text: row.get("chunk_text")?,
        distance: row.get("distance")?,
    });
}
