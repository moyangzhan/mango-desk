use crate::embedding_service::EmbeddingService;
use crate::enums::{FileCategory, IndexingEvent};
use crate::errors::AppError;
use crate::global::{
    ACTIVE_MODEL_PLATFORM, CONFIG_NAME_INDEXER_SETTING, INDEXER_SETTING, INDEXING, SCANNING,
    SCANNING_TOTAL, STOP_INDEX_SIGNAL,
};
use crate::initializer;
use crate::repositories::{
    config_repo, file_content_embedding_repo, file_info_repo, file_metadata_embedding_repo,
    indexing_task_repo,
};
use crate::scanner;
use crate::structs::indexer_setting::IndexerSetting;
use crate::traits::indexing_template::IndexingTemplate;
use crate::utils::{frontend_util, indexing_task_util};
use crate::{embedding_service_manager, indexers};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;

pub async fn update_indexer_setting(indexer_setting: IndexerSetting) -> Result<usize, String> {
    let content_language_changed =
        indexer_setting.file_content_language != INDEXER_SETTING.read().await.file_content_language;
    let json = serde_json::to_string(&indexer_setting).map_err(|e| AppError::SerializeError(e))?;
    let result = config_repo::update_by_name(CONFIG_NAME_INDEXER_SETTING, &json)?;
    initializer::init_setting(
        CONFIG_NAME_INDEXER_SETTING,
        || serde_json::to_string(&IndexerSetting::default()).unwrap_or_default(),
        &INDEXER_SETTING,
    )
    .await;
    if content_language_changed {
        embedding_service_manager::get_manager()
            .write()
            .await
            .clear();
    }
    Ok(result)
}

pub async fn is_embedding_model_changed() -> Result<bool, String> {
    let tasks = indexing_task_repo::list(1, 1);
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

/// Indexing workflow consists of four main phases:
///
/// 1. File scanning:
///    Scan specified paths and store file metadata in database
///
/// 2. Document processing:
///    Extract text content and generate document embeddings
///
/// 3. Image processing:
///    Extract visual features and generate image embeddings
///    Prerequisites: Private mode disabled, Model platform configured
///
/// 4. Audio processing:
///    Extract audio features and generate audio embeddings
///    Prerequisites: Private mode disabled, Model platform configured
pub async fn start_indexing(
    paths: Vec<String>,
    on_event: Channel<IndexingEvent>,
) -> Result<bool, String> {
    if paths.is_empty() {
        return Ok(false);
    }
    if SCANNING.load(Ordering::SeqCst) {
        return Ok(false);
    }
    if INDEXING.load(Ordering::SeqCst) {
        return Ok(false);
    }
    STOP_INDEX_SIGNAL.store(false, Ordering::SeqCst);
    let embedding_model = EmbeddingService::model_name().await;
    let task = indexing_task_util::task_new(&paths, embedding_model).await?;

    let task = Arc::new(task);
    let event = Arc::new(on_event);

    frontend_util::send_to_frontend(
        event.as_ref(),
        IndexingEvent::Start {
            task_id: task.id,
            msg: "Start".to_string(),
        },
    );

    // Scan specified paths and store file metadata in database
    scanner::start(&paths, task.clone(), Some(event.clone())).await;

    // Scanned files
    indexing_task_util::set_total(SCANNING_TOTAL.load(Ordering::SeqCst) as i64).await;
    indexing_task_util::summary_to_db().await;

    // Embedding processing
    INDEXING.store(true, Ordering::SeqCst);

    println!("document indexing...");
    let mut document_indexer = indexers::document_indexer::DocumentIndexer::new();
    let _ = document_indexer
        .process(task.clone(), Some(event.clone()))
        .await
        .unwrap_or_else(|e| log::error!("start_indexing => Document indexing error,{}", e));
    println!(
        "Document indexing done,status:{}",
        serde_json::json!(document_indexer.status)
    );
    indexing_task_util::summary_to_db().await;

    if INDEXER_SETTING.read().await.is_private {
        println!("--- private mode, skip indexing image and audio ---");
        indexing_finish(
            task.id,
            format!("Privacy setting: skip indexing image and audio").as_str(),
            Some(event.clone()),
        )
        .await?;
        return Ok(true);
    }

    let (enabled, platform_name) = {
        let platform = ACTIVE_MODEL_PLATFORM.read().await;
        (platform.is_enable(), platform.name.clone())
    };
    if !enabled {
        println!(
            "--- active model platform disabled, name: {} ---",
            platform_name
        );
        indexing_finish(
            task.id,
            format!("Active model platform disabled, name: {}", platform_name).as_str(),
            Some(event.clone()),
        )
        .await?;

        return Err(format!(
            "Model platform '{}' is missing API key configuration",
            platform_name
        ));
    }

    if let Ok(mut image_indexer) = indexers::image_indexer::ImageIndexer::new().await {
        println!("image indexing...");
        let _ = image_indexer
            .process(task.clone(), Some(event.clone()))
            .await
            .unwrap_or_else(|e| println!("image indexing error,{}", e));
        indexing_task_util::summary_to_db().await;
    }

    if let Ok(mut audio_indexer) = indexers::audio_indexer::AudioIndexer::new().await {
        println!("audio indexing...");
        let _ = audio_indexer
            .process(task.clone(), Some(event.clone()))
            .await
            .unwrap_or_else(|e| println!("audio indexing error,{}", e));
        indexing_task_util::summary_to_db().await;
    }

    indexing_finish(task.id, "done", Some(event.clone())).await?;

    return Ok(true);
}

pub async fn background_indexing(path: &str) -> Result<bool, String> {
    log::info!("background indexing... path:{}", path);
    if path.is_empty() {
        return Ok(false);
    }
    let paths = vec![path.to_string()];
    let embedding_model = EmbeddingService::model_name().await;
    let task = indexing_task_util::task_new(&paths, embedding_model).await?;
    let task = Arc::new(task);

    // Scan specified paths and store file metadata in database
    scanner::start(&paths, task.clone(), None).await;
    // Embedding processing

    let mut document_indexer = indexers::document_indexer::DocumentIndexer::new();
    let _ = document_indexer
        .process(task.clone(), None)
        .await
        .unwrap_or_else(|e| log::error!("Document indexing error,{}", e));

    if INDEXER_SETTING.read().await.is_private {
        log::info!("--- private mode, skip indexing image and audio ---");
        return Ok(true);
    }

    let (enabled, platform_name) = {
        let platform = ACTIVE_MODEL_PLATFORM.read().await;
        (platform.is_enable(), platform.name.clone())
    };
    if !enabled {
        log::info!(
            "--- active model platform disabled, name: {} ---",
            platform_name
        );
        indexing_finish(
            task.id,
            format!("Active model platform disabled, name: {}", platform_name).as_str(),
            None,
        )
        .await?;

        return Err(format!(
            "Model platform '{}' is missing API key configuration",
            platform_name
        ));
    }

    if let Ok(mut image_indexer) = indexers::image_indexer::ImageIndexer::new().await {
        let _ = image_indexer
            .process(task.clone(), None)
            .await
            .unwrap_or_else(|e| log::error!("image indexing error,{}", e));
    }

    if let Ok(mut audio_indexer) = indexers::audio_indexer::AudioIndexer::new().await {
        let _ = audio_indexer
            .process(task.clone(), None)
            .await
            .unwrap_or_else(|e| log::error!("audio indexing error,{}", e));
    }

    indexing_finish(task.id, "done", None).await?;
    return Ok(true);
}

pub async fn index_file(path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let is_valid = scanner::is_valid_file_with(&path_buf).await;
    if !is_valid {
        return Ok(());
    }
    if let Err(add_info_result) = scanner::add_or_update_file_info(path.to_string()).await {
        log::error!("add_or_update_file_info error: {:?}", add_info_result);
        return Ok(());
    }
    let ext = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    let Some(file_info) = file_info_repo::get_by_path(path)? else {
        return Ok(());
    };
    let category = FileCategory::from_ext(&ext);
    match category {
        FileCategory::Document => {
            let document_indexer = indexers::document_indexer::DocumentIndexer::new();
            document_indexer.embedding_one_file(&file_info).await?;
        }
        FileCategory::Image => {
            let image_indexer = indexers::image_indexer::ImageIndexer::new().await?;
            image_indexer.embedding_one_file(&file_info).await?;
        }
        FileCategory::Audio => {
            let audio_indexer = indexers::audio_indexer::AudioIndexer::new().await?;
            audio_indexer.embedding_one_file(&file_info).await?;
        }
        _ => {}
    }
    Ok(())
}

pub fn remove_file_index(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    let file_info = file_info_repo::get_by_path(path)?;
    if let Some(file_info) = file_info {
        file_info_repo::delete_by_id(file_info.id)?;
        file_content_embedding_repo::delete_by_file_id(file_info.id)?;
        file_metadata_embedding_repo::delete_by_file_id(file_info.id)?;
    }
    Ok(())
}

pub fn remove_directory_index(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    file_content_embedding_repo::delete_by_file_prefix_path(path)?;
    file_metadata_embedding_repo::delete_by_file_prefix_path(path)?;
    file_info_repo::delete_by_prefix_path(path)?;
    Ok(())
}

async fn indexing_finish(
    task_id: i64,
    msg: &str,
    event: Option<Arc<Channel<IndexingEvent>>>,
) -> Result<(), String> {
    SCANNING.store(false, Ordering::SeqCst);
    INDEXING.store(false, Ordering::SeqCst);
    SCANNING_TOTAL.store(0, Ordering::SeqCst);
    STOP_INDEX_SIGNAL.store(false, Ordering::SeqCst);

    indexing_task_util::task_done().await?;

    if let Some(event) = event {
        frontend_util::send_to_frontend(
            event.as_ref(),
            IndexingEvent::Finish {
                task_id,
                msg: msg.to_string(),
            },
        );
    }
    Ok(())
}
