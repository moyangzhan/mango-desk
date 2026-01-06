use crate::entities::IndexingTask;
use crate::repositories::RepositoryError;
use crate::utils::app_util::get_db_path;
use crate::utils::datetime_util;
use chrono::{DateTime, Local};
use rusqlite::{Connection, Result, Row, named_params};

pub fn insert_by_paths(
    paths: &Vec<String>,
    embedding_model: &str,
    status: &str,
    start_time: &DateTime<Local>,
) -> Result<IndexingTask, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "insert into indexing_task (paths,embedding_model,status,start_time) values (:paths,:embedding_model,:status,:start_time)",
    )?;

    let last_insert_rowid = stmt.insert(named_params! {
        ":paths": &paths.join(","),
        ":embedding_model": embedding_model,
        ":status": status,
        ":start_time": datetime_util::datetime_to_str(start_time),
    })?;
    let mut query_stmt = conn.prepare("select * from indexing_task where rowid = ?1")?;
    let entity = query_stmt.query_row([last_insert_rowid], |row| Ok(build_entity(row)?))?;
    Ok(entity)
}

pub fn insert(entity: &IndexingTask) -> Result<IndexingTask, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "insert into indexing_task (paths, embedding_model, status, start_time, end_time, duration, total_cnt, content_indexed_success_cnt, content_indexed_failed_cnt, content_indexed_skipped_cnt, remark, config_json) values (:paths,:embedding_model,:status,:start_time,:end_time,:duration,:total_cnt,:content_indexed_success_cnt,:content_indexed_failed_cnt,:content_indexed_skipped_cnt,:remark,:config_json)"
    )?;
    let start_time = entity
        .start_time
        .map(|t| datetime_util::datetime_to_str(&t))
        .unwrap_or_default();
    let end_time = entity
        .end_time
        .map(|t| datetime_util::datetime_to_str(&t))
        .unwrap_or_default();
    let last_insert_rowid = stmt.insert(named_params! {
        ":paths": &entity.paths,
        ":embedding_model": &entity.embedding_model,
        ":status": <&'static str>::from(entity.status),
        ":start_time": start_time,
        ":end_time": end_time,
        ":duration": &entity.duration,
        ":total_cnt": &entity.total_cnt,
        ":content_processed_cnt": &entity.content_processed_cnt,
        ":content_indexed_success_cnt": &entity.content_indexed_success_cnt,
        ":content_indexed_failed_cnt": &entity.content_indexed_failed_cnt,
        ":content_indexed_skipped_cnt": &entity.content_indexed_skipped_cnt,
        ":remark": &entity.remark,
        ":config_json": &entity.config_json,
    })?;
    let mut query_stmt = conn.prepare("select * from indexing_task where rowid = ?1")?;
    let entity = query_stmt.query_row([last_insert_rowid], |row| Ok(build_entity(row)?))?;
    Ok(entity)
}

pub fn update(entity: &IndexingTask) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "update indexing_task set paths=:paths,status=:status,start_time=:start_time,end_time=:end_time,duration=:duration,total_cnt=:total_cnt,content_processed_cnt=:content_processed_cnt,content_indexed_success_cnt=:content_indexed_success_cnt,content_indexed_failed_cnt=:content_indexed_failed_cnt,content_indexed_skipped_cnt=:content_indexed_skipped_cnt,status=:status,remark=:remark,config_json=:config_json where id = :id")?;
    let start_time = entity
        .start_time
        .map(|t| datetime_util::datetime_to_str(&t))
        .unwrap_or_default();
    let end_time = entity
        .end_time
        .map(|t| datetime_util::datetime_to_str(&t))
        .unwrap_or_default();
    let affected = stmt.execute(named_params! {
       ":id": &entity.id,
       ":paths": &entity.paths,
       ":status": &(<&'static str>::from(entity.status)),
       ":start_time": start_time,
       ":end_time": end_time,
       ":duration": &entity.duration,
       ":total_cnt": &entity.total_cnt,
       ":content_processed_cnt": &entity.content_processed_cnt,
       ":content_indexed_success_cnt": &entity.content_indexed_success_cnt,
       ":content_indexed_failed_cnt": &entity.content_indexed_failed_cnt,
       ":content_indexed_skipped_cnt": &entity.content_indexed_skipped_cnt,
       ":status": &entity.status,
       ":remark": &entity.remark,
       ":config_json": &entity.config_json,
    })?;
    println!("update indexing_task affected: {:?}", affected);
    Ok(affected)
}

pub fn update_status(id: i64, status: &str, remark: &str) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("update indexing_task set status = :status,remark=:remark where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &id,
        ":status": status,
        ":remark": remark,
    })?;
    println!("update indexing_task affected: {:?}", affected);
    Ok(affected)
}

pub fn update_cnt(
    id: i64,
    total_cnt: i64,
    processed_cnt: i64,
    success_cnt: i64,
    failed_cnt: i64,
    skipped_cnt: i64,
    duration: i64,
) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare(
        "update indexing_task set total_cnt =:total_cnt,content_processed_cnt=:content_processed_cnt,content_indexed_success_cnt=:content_indexed_success_cnt,content_indexed_failed_cnt=:content_indexed_failed_cnt,content_indexed_skipped_cnt=:content_indexed_skipped_cnt,duration=:duration where id = :id",
    )?;
    let affected = stmt.execute(named_params! {
        ":id": &id,
        ":total_cnt": &total_cnt,
        ":content_processed_cnt": &processed_cnt,
        ":content_indexed_success_cnt": &success_cnt,
        ":content_indexed_failed_cnt": &failed_cnt,
        ":content_indexed_skipped_cnt": &skipped_cnt,
        ":duration": &duration,
    })?;
    println!("update indexing_task affected: {:?}", affected);
    Ok(affected)
}

pub fn get(id: i64) -> Result<IndexingTask, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select * from indexing_task where id = :id")?;
    let entity = stmt.query_row([id], |row| Ok(build_entity(row)?))?;
    Ok(entity)
}

pub fn list(page: i64, page_size: i64) -> Result<Vec<IndexingTask>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt =
        conn.prepare("select * from indexing_task order by id desc limit :limit offset :offset")?;
    let rows = stmt.query_map(
        named_params! {
            ":offset": (page -1)*page_size,
            ":limit": page_size,
        },
        |row| Ok(build_entity(row)?),
    )?;
    let mut result = Vec::new();
    for item in rows {
        result.push(item?);
    }
    Ok(result)
}

pub fn count() -> Result<i64, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select count(*) from indexing_task")?;
    let count = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn delete_by_id(id: i64) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("delete from indexing_task where id = :id")?;
    let affected = stmt.execute(named_params! {
        ":id": &id,
    })?;
    println!("delete indexing_task affected: {:?}", affected);
    Ok(affected)
}

fn build_entity(row: &Row<'_>) -> Result<IndexingTask, RepositoryError> {
    let create_time_str: String = row.get("create_time")?;
    let update_time_str: String = row.get("update_time")?;

    let start_time = match row.get::<_, String>("start_time") {
        Ok(s) if !s.is_empty() => datetime_util::str_to_datetime(&s).ok(),
        _ => None,
    };
    let end_time = match row.get::<_, String>("end_time") {
        Ok(s) if !s.is_empty() => datetime_util::str_to_datetime(&s).ok(),
        _ => None,
    };
    return Ok(IndexingTask {
        id: row.get("id")?,
        paths: row.get("paths")?,
        embedding_model: row.get("embedding_model")?,
        status: row.get("status")?,
        start_time,
        end_time,
        duration: row.get("duration")?,
        total_cnt: row.get("total_cnt")?,
        content_processed_cnt: row.get("content_processed_cnt")?,
        content_indexed_success_cnt: row.get("content_indexed_success_cnt")?,
        content_indexed_failed_cnt: row.get("content_indexed_failed_cnt")?,
        content_indexed_skipped_cnt: row.get("content_indexed_skipped_cnt")?,
        remark: row.get("remark")?,
        config_json: row.get("config_json")?,
        create_time: datetime_util::str_to_datetime(create_time_str.as_str())?,
        update_time: datetime_util::str_to_datetime(update_time_str.as_str())?,
    });
}
