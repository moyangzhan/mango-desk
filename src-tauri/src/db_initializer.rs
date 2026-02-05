use crate::global::{CONFIG_NAME_DB_VERSION, DB_VERSION};
use crate::db_migrations::{db_v1, db_v2, db_v3};
use crate::utils::app_util::get_db_path;
use anyhow::Result;
use log::{error, info};
use rusqlite::Connection;

pub fn init() -> Result<()> {
    info!("init db, path:{}", get_db_path());
    let mut is_latest: bool = false;
    let conn: Connection = Connection::open(get_db_path())?;
    let mut check_stmt =
        conn.prepare("select name from sqlite_master where type='table' and name='config'")?;
    let mut current_db_version: i32 = 0;
    if check_stmt.exists([])? {
        current_db_version = conn
            .query_row(
                &format!(
                    "select * from config where name='{}'",
                    CONFIG_NAME_DB_VERSION
                ),
                [],
                |row| {
                    let value: String = row.get("value")?;
                    Ok(value.parse().unwrap_or(0))
                },
            )
            .unwrap_or_else(|e| {
                error!("db_version not found, set to 0,error:{:?}", e);
                0
            });
        if current_db_version == DB_VERSION {
            is_latest = true;
        }
    }

    info!("db is latest:{}", is_latest);
    if is_latest {
        return Ok(());
    }

    for a in current_db_version + 1..DB_VERSION + 1 {
        info!("db version:{}", a);
        match a {
            1 => {
                db_v1::exec_ddl()?;
                db_v1::init_data()?;
            }
            2 => {
                db_v2::exec_ddl()?;
                db_v2::init_data()?;
            }
            3 => {
                db_v3::exec_ddl()?;
                db_v3::init_data()?;
            }
            _ => {}
        }
    }

    conn.execute(
        "update config set value=? where name=?",
        [DB_VERSION.to_string(), CONFIG_NAME_DB_VERSION.to_string()],
    )?;
    return Ok(());
}
