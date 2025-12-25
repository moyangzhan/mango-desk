use crate::embedding_service::EmbeddingService;
use crate::errors::AppError;
use crate::global::INDEXER_SETTING;
use crate::initializer;
use crate::repositories::{config_repo, indexing_task_repo};

pub async fn update_indexer_setting(
    indexer_setting: crate::structs::indexer_setting::IndexerSetting,
) -> Result<usize, String> {
    let json = serde_json::to_string(&indexer_setting).map_err(|e| AppError::SerializeError(e))?;
    let result = config_repo::update_by_name("indexer_setting", &json).await?;
    initializer::init_setting("indexer_setting", &INDEXER_SETTING).await;
    Ok(result)
}

pub async fn is_embedding_model_changed() -> Result<bool, String> {
    let tasks = indexing_task_repo::list(1, 1).await;
    if tasks.is_err() {
        return Ok(false);
    }
    match tasks {
        Ok(tasks) => {
            if tasks.is_empty() {
                return Ok(false);
            }
            let latest = tasks
                .first()
                .map(|item| item.embedding_model.clone())
                .unwrap_or_default();
            let embedding_name: &'static str = EmbeddingService::model_name().await;
            if latest != embedding_name {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(_) => {
            return Ok(false);
        }
    }
}
