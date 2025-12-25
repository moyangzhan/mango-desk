use crate::entities::FileInfo;
use crate::repositories::RepositoryError;
use crate::utils::app_util::get_db_path;
use crate::utils::datetime_util;
use rusqlite::{Connection, Result, Row, named_params};

pub async fn insert(file_info: &FileInfo) -> Result<Option<FileInfo>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "insert into file_info(name,category,path,file_ext,file_size,content,metadata,md5,is_invalid,invalid_reason,file_create_time,file_update_time) values (:name,:category,:path,:file_ext,:file_size,:content,:metadata,:md5,:is_invalid,:invalid_reason,:file_create_time,:file_update_time)"
    )?;
    let last_insert_rowid = stmt.insert(named_params! {
        ":name": &file_info.name,
        ":category": file_info.category,
        ":path": &file_info.path,
        ":file_ext": &file_info.file_ext,
        ":file_size": file_info.file_size,
        ":content": &file_info.content,
        ":metadata": &file_info.metadata.to_json(),
        ":md5": &file_info.md5,
        ":is_invalid": file_info.is_invalid,
        ":invalid_reason": &file_info.invalid_reason,
        ":file_create_time": datetime_util::micro_datetime_to_str(&file_info.file_create_time),
        ":file_update_time": datetime_util::micro_datetime_to_str(&file_info.file_update_time),
    })?;
    let mut query_stmt = conn.prepare("select * from file_info where rowid = ?1")?;
    let file_info = query_stmt
        .query_row([last_insert_rowid], |row| Ok(Some(build_file_info(row)?)))
        .unwrap_or_else(|e| {
            println!("file_info_repo.insert() Error: {}", e);
            None
        });

    Ok(file_info)
}

pub async fn update(file_info: &FileInfo) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "update file_info set name =:name,path=:path,file_ext=:file_ext,file_size=:file_size,content=:content,md5=:md5,is_invalid=:is_invalid,invalid_reason=:invalid_reason,metadata=:metadata,file_update_time=:file_update_time where id = :id",
    )?;
    let affected = stmt.execute(named_params! {
        ":id": &file_info.id,
        ":name": &file_info.name,
        ":path": &file_info.path,
        ":file_ext": &file_info.file_ext,
        ":file_size": &file_info.file_size,
        ":content": &file_info.content,
        ":metadata": &file_info.metadata.to_json(),
        ":md5": &file_info.md5,
        ":is_invalid": &file_info.is_invalid,
        ":invalid_reason": &file_info.invalid_reason,
        ":file_update_time": datetime_util::micro_datetime_to_str(&file_info.file_update_time),
    })?;
    println!("update file_info affected: {:?}", affected);
    Ok(affected)
}

pub async fn update_content_meta(
    file_id: i64,
    content: &str,
    meta: &str,
) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("update file_info set content = :content, metadata = :meta where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &file_id,
        ":content": &content,
        ":meta": &meta,
    })?;
    Ok(affected)
}

pub async fn update_invalid(
    file_id: i64,
    is_invalid: bool,
    invalid_reason: &str,
) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("update file_info set is_invalid = :is_invalid,invalid_reason = :invalid_reason where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &file_id,
        ":is_invalid": &is_invalid,
        ":invalid_reason": &invalid_reason,
    })?;
    Ok(affected)
}

pub async fn update_content_index_status(
    file_id: i64,
    index_status: i64,
    index_status_reason: &str,
) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("update file_info set content_index_status = :index_status, content_index_status_msg = :index_status_msg where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &file_id,
        ":index_status": &index_status,
        ":index_status_msg": &index_status_reason,
    })?;
    Ok(affected)
}

pub async fn update_meta_index_status(
    file_id: i64,
    index_status: i64,
    index_status_reason: &str,
) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("update file_info set meta_index_status = :index_status, meta_index_status_msg = :index_status_msg where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &file_id,
        ":index_status": &index_status,
        ":index_status_msg": &index_status_reason,
    })?;
    Ok(affected)
}

pub async fn list(page: i64, size: i64) -> Result<Vec<FileInfo>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("select * from file_info order by id desc limit :size offset :offset")?;
    let rows = stmt.query_map(
        named_params! {
            ":size": size,
            ":offset": (page - 1) * size,
        },
        |row| Ok(build_file_info(row)?),
    )?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub async fn count() -> Result<i64, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select count(*) from file_info")?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub async fn list_unindexed_files(
    min_id: i64,
    limit: i64,
    category: i64,
) -> Result<Vec<FileInfo>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("select * from file_info where id > :min_id and content_index_status = 1 and category = :category order by id asc limit :limit")?;
    let rows = stmt.query_map(
        named_params! {
            ":min_id": min_id,
            ":category": category,
            ":limit": limit,
        },
        |row| Ok(build_file_info(row)?),
    )?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub async fn count_unindexed() -> Result<i64, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select count(*) from file_info where content_index_status = 1")?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub async fn count_unindexed_files(category: i64) -> Result<i64, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "select count(*) from file_info where content_index_status = 1 and category = :category",
    )?;
    let count = stmt.query_row(named_params! {":category": category}, |row| row.get(0))?;
    Ok(count)
}

pub async fn list_by_ids(ids: &[i64]) -> Result<Vec<FileInfo>, RepositoryError> {
    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join("','");

    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare(format!("select * from file_info where id in ('{}')", ids_str).as_str())?;
    let rows = stmt.query_map([], |row| Ok(build_file_info(row)?))?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub async fn get_by_md5(md5: &str) -> Result<Option<FileInfo>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select * from file_info where md5 = ?1 limit 1")?;
    match stmt.query_row([md5], |row: &Row<'_>| Ok(build_file_info(row)?)) {
        Ok(hit) => return Ok(Some(hit)),
        Err(e) => {
            println!("file_info_repo.get_by_md5() Error: {}", e.to_string());
            return Ok(None);
        }
    }
}

pub async fn get_by_path(path: &str) -> Result<Option<FileInfo>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select * from file_info where path = ?1 limit 1")?;
    match stmt.query_row([path], |row: &Row<'_>| Ok(build_file_info(row)?)) {
        Ok(hit) => return Ok(Some(hit)),
        Err(e) => {
            println!("file_info_repo.get_by_path() Error: {}", e.to_string());
            return Ok(None);
        }
    }
}

pub async fn delete_by_id(file_id: i64) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from file_info where id = ?1")?;
    let affected = stmt.execute([file_id])?;
    println!("delete file_info by id affected: {:?}", affected);
    Ok(affected)
}

fn build_file_info(row: &Row<'_>) -> Result<FileInfo, RepositoryError> {
    let file_create_time: String = row.get("file_create_time")?;
    let file_update_time: String = row.get("file_update_time")?;
    let create_time_str: String = row.get("create_time")?;
    let update_time_str: String = row.get("update_time")?;
    let meta: String = row.get("metadata")?;
    return Ok(FileInfo {
        id: row.get("id")?,
        name: row.get("name")?,
        category: row.get("category")?,
        path: row.get("path")?,
        file_ext: row.get("file_ext")?,
        file_size: row.get("file_size")?,
        content: row.get("content")?,
        content_index_status: row.get("content_index_status")?,
        content_index_status_msg: row.get("content_index_status_msg")?,
        meta_index_status: row.get("meta_index_status")?,
        meta_index_status_msg: row.get("meta_index_status_msg")?,
        is_invalid: row.get("is_invalid")?,
        invalid_reason: row.get("invalid_reason")?,
        md5: row.get("md5")?,
        metadata: crate::structs::file_metadata::FileMetadata::from_json(&meta),
        file_create_time: datetime_util::str_to_micro_datetime(file_create_time.as_str())?,
        file_update_time: datetime_util::str_to_micro_datetime(file_update_time.as_str())?,
        create_time: datetime_util::str_to_datetime(create_time_str.as_str())?,
        update_time: datetime_util::str_to_datetime(update_time_str.as_str())?,
    });
}
