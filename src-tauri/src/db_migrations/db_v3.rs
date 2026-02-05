use crate::utils::app_util::get_db_path;
use anyhow::Result;
use log::info;
use rusqlite::Connection;

/// Execute DDL for db_v3
///
/// Migration: Reset embedding tables and upgrade dimension from float[384] to float[768].
/// Note: This drops existing data; embeddings must be regenerated.
pub fn exec_ddl() -> Result<()> {
    info!("dbv3.exec_ddl");
    let conn: Connection = Connection::open(get_db_path())?;
    // Replace float[384] with float[768]
    conn.execute_batch(
        r#"
        drop table if exists file_metadata_embedding;
        create virtual table if not exists file_metadata_embedding using vec0(
            id integer primary key autoincrement,
            file_id integer default 0 not null,         
            embedding float[768] distance_metric=cosine
        );
        drop table if exists file_content_embedding;
        create virtual table if not exists file_content_embedding using vec0(
            id integer primary key autoincrement,
            file_id integer default 0 not null,
            chunk_index integer default 0 not null,
            chunk_text text default '' not null,
            embedding float[768] distance_metric=cosine
        );
        "#,
    )?;
    Ok(())
}

pub fn init_data() -> Result<()> {
    info!("dbv3.init_data");
    let conn: Connection = Connection::open(get_db_path())?;
    // Reset embedding tables
    conn.execute_batch(
        "
        update file_info set content_index_status = 1, meta_index_status = 1 where 1=1
        ",
    )?;
    Ok(())
}
