use anyhow::Result;
use log::info;
use rusqlite::Connection;

/// DB_VERSION = 4
/// Add content_ref_path column to file_info for file-mode content storage.
pub fn exec_ddl_with_conn(conn: &Connection) -> Result<()> {
    info!("dbv4.exec_ddl_with_conn");

    let _ = conn.execute_batch(
        r#"
        ALTER TABLE file_info ADD COLUMN content_ref_path TEXT DEFAULT NULL;
        "#,
    );

    Ok(())
}

pub fn init_data_with_conn(_conn: &Connection) -> Result<()> {
    info!("dbv4.init_data_with_conn");
    Ok(())
}
