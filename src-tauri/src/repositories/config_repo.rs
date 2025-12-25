use crate::entities::Config;
use crate::repositories::RepositoryError;
use crate::utils::app_util::get_db_path;
use rusqlite::{Connection, Result};

pub async fn get_one(config_name: &str) -> Result<Option<Config>, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("select * from config where name = ?1 limit 1")?;
    let hit_config = match stmt.query_row([config_name], |row| {
        Ok(Config {
            id: row.get("id")?,
            name: row.get("name")?,
            value: row.get("value")?,
        })
    }) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("get one error, config_name:{}, error: {}", config_name, e);
            return Ok(None);
        }
    };
    return Ok(Some(hit_config));
}

pub async fn get_val(config_name: &str) -> String {
    let conn = match Connection::open(get_db_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return "".to_string();
        }
    };
    let mut stmt = match conn.prepare("select value from config where name = ?1 limit 1") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to prepare stmt: {}", e);
            return "".to_string();
        }
    };
    let config = match stmt.query_row([config_name], |row| Ok(row.get(0)?)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to query map: {}", e);
            return "".to_string();
        }
    };
    return config;
}

pub async fn insert(config_name: &str, config_value: &str) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("insert into config (name, value) values (?1, ?2)")?;
    let affected = stmt.execute([config_name, config_value])?;
    Ok(affected)
}

pub async fn update_by_name(config_name: &str, new_value: &str) -> Result<usize, RepositoryError> {
    let conn = Connection::open(get_db_path())?;
    let mut stmt = conn.prepare("update config set value = ?1 where name = ?2")?;
    let affected = stmt.execute([new_value, config_name])?;
    Ok(affected)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_update_by_name() {
        let result = update_by_name("active_locale", "zh-CN").await;
        println!("update result: {:?}", result);
    }
}
