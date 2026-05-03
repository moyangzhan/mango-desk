use crate::repositories::config_repo;
use serde::{Deserialize, Serialize};

const CONFIG_NAME_ACTIVE_TASK: &str = "active_task";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveTask {
    pub task_type: String,          // "indexing" | "content_storage_change" | "data_copying"
    pub category: Option<String>,   // only for content_storage_change
    pub old_path: Option<String>,   // only for data_copying
    pub started_at: i64,            // unix timestamp
}

pub fn lock_active_task(task: &ActiveTask) -> Result<(), String> {
    if let Some(existing) = get_active_task()? {
        return Err(format!(
            "Task already in progress: {}",
            existing.task_type
        ));
    }
    let json = serde_json::to_string(task).map_err(|e| e.to_string())?;
    config_repo::upsert(CONFIG_NAME_ACTIVE_TASK, &json)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn unlock_active_task() -> Result<(), String> {
    config_repo::update_by_name(CONFIG_NAME_ACTIVE_TASK, "")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_active_task() -> Result<Option<ActiveTask>, String> {
    let val = config_repo::get_val(CONFIG_NAME_ACTIVE_TASK);
    if val.is_empty() {
        return Ok(None);
    }
    let task: ActiveTask = serde_json::from_str(&val).map_err(|e| e.to_string())?;
    Ok(Some(task))
}

pub fn clear_active_task() -> Result<(), String> {
    unlock_active_task()
}
