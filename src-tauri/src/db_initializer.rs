use crate::db_manager;
use crate::db_migrations;
use crate::utils::app_util::get_db_path;
use anyhow::Result;
use log::info;
use std::path::PathBuf;

pub fn init() -> Result<()> {
    let db_path = PathBuf::from(get_db_path());
    info!("init db, path:{}", db_path.display());

    // Initialize database connection manager
    db_manager::init_db_manager(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to init database manager: {}", e))?;

    // Run migrations using the global connection
    let conn = db_manager::get_connection()
        .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;

    db_migrations::init_with_conn(conn.as_conn())?;

    Ok(())
}
