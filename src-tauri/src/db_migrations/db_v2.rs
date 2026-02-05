use crate::utils::app_util::get_db_path;
use anyhow::Result;
use log::info;
use rusqlite::Connection;

pub fn exec_ddl() -> Result<()> {
    info!("dbv2.exec_ddl");
    let conn: Connection = Connection::open(get_db_path())?;
    // chunk_id => file_content_embedding.id
    conn.execute_batch(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS file_content_fts USING fts5(
            file_id UNINDEXED, chunk_id UNINDEXED, content, tokenize='porter'
        );
        "#,
    )?;

    //triggered_by 1: file selector, 2: file watcher
    conn.execute(
        "ALTER TABLE indexing_task ADD COLUMN triggered_by integer DEFAULT 1",
        (),
    )
    .unwrap_or_else(|error| {
        println!("add triggered_by error:{:?}", error);
        0
    });
    Ok(())
}

pub fn init_data() -> Result<()> {
    info!("dbv2.init_data");
    Ok(())
}
