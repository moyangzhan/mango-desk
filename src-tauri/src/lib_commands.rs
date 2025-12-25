use crate::embedding_service::EmbeddingService;
use crate::embedding_service_manager::embedding_service_manager;
use crate::entities::{FileInfo, IndexingTask, ModelPlatform};
use crate::enums::CommandResultCode;
use crate::enums::{DownloadEvent, IndexingEvent, Locale, ModelPlatformName};
use crate::errors::AppError;
use crate::global::{
    ACTIVE_LOCALE, ACTIVE_MODEL_PLATFORM, APP_DIR, INDEXER_SETTING, INDEXING, SCANNING,
    SCANNING_TOTAL, STOP_INDEX_SIGNAL,
};
use crate::model_platform_services::siliconflow::SiliconFlow;
use crate::repositories::{
    ai_model_repo, config_repo, file_content_embedding_repo, file_info_repo,
    file_metadata_embedding_repo, indexing_task_repo, model_platform_repo,
};
use crate::scanner;
use crate::searcher;
use crate::structs::command_result::CommandResult;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::chat_capable::ChatCapable;
use crate::traits::indexing_template::IndexingTemplate;
use crate::utils::{app_util, download_util, frontend_util, indexing_task_util};
use crate::{indexer_service, indexers};
use rust_i18n::t;
use serde_json::json;
use std::fs::read;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::command;
use tauri::{AppHandle, ipc::Channel};

#[command]
pub async fn load_active_locale() -> Result<String, String> {
    let guard = ACTIVE_LOCALE.read().await;
    Ok(guard.clone())
}

#[command]
pub async fn load_model_platforms() -> Vec<ModelPlatform> {
    model_platform_repo::list(&vec![
        ModelPlatformName::OpenAi.text().to_string(),
        ModelPlatformName::SiliconFlow.text().to_string(),
    ])
    .await
    .unwrap_or_else(|e| {
        println!("Failed to load model platforms: {}", e);
        vec![]
    })
}

#[command]
pub async fn load_proxy_info() -> Result<ProxyInfo, String> {
    let result = config_repo::get_one("proxy")
        .await?
        .map(|config| serde_json::from_str(&config.value).map_err(|e| e.to_string()))
        .unwrap_or_else(|| Ok(ProxyInfo::default()))?;
    Ok(result)
}

#[command]
pub async fn load_indexer_setting()
-> Result<crate::structs::indexer_setting::IndexerSetting, String> {
    let result = config_repo::get_one("indexer_setting")
        .await?
        .map(|config| {
            serde_json::from_str(&config.value).map_err(|e| {
                println!("Failed to parse indexer setting: {}", e);
                e.to_string()
            })
        })
        .unwrap_or_else(|| Ok(crate::structs::indexer_setting::IndexerSetting::default()))?;
    Ok(result)
}

#[command]
pub async fn load_embedding_models() -> serde_json::Value {
    let multilang_path = app_util::get_multilingual_embedding_path();
    let multilang_model = Path::new(multilang_path.as_str());
    json!({
        "all-minilm-l6-v2": true,
        "paraphrase-multilingual-MiniLM-L12-v2": multilang_model.exists()
    })
}

#[command]
pub async fn load_model_by_type(
    platform: &str,
    one_type: &str,
) -> Result<Option<crate::entities::AiModel>, String> {
    let result = ai_model_repo::get_one_by_type(platform, one_type).await?;
    Ok(result)
}

#[command]
pub async fn update_indexer_setting(
    indexer_setting: crate::structs::indexer_setting::IndexerSetting,
) -> Result<usize, String> {
    indexer_service::update_indexer_setting(indexer_setting).await
}

#[command]
pub async fn is_embedding_model_changed() -> Result<bool, String> {
    return indexer_service::is_embedding_model_changed().await;
}

#[command]
pub async fn update_proxy_info(proxy_info: ProxyInfo) -> Result<usize, String> {
    let proxy_json = serde_json::to_string(&proxy_info).map_err(|e| AppError::SerializeError(e))?;
    Ok(config_repo::update_by_name("proxy", &proxy_json).await?)
}

#[command]
pub async fn update_model_platform(platform: ModelPlatform) -> Result<usize, AppError> {
    let result = model_platform_repo::update_by_name(&platform.name, &platform).await?;
    if platform.name == ACTIVE_MODEL_PLATFORM.read().await.name {
        match ACTIVE_MODEL_PLATFORM.try_write() {
            Ok(mut guard) => {
                let one = model_platform_repo::get_one(&platform.name).await?;
                *guard = one;
            }
            Err(_) => {
                eprintln!("Failed to acquire write lock for ACTIVE_MODEL_PLATFORM");
            }
        }
    }
    Ok(result)
}

#[command]
pub async fn load_active_platform() -> String {
    let platform = ACTIVE_MODEL_PLATFORM.read().await;
    platform.name.clone()
}

#[command]
pub async fn set_active_platform(platform_name: &str) -> Result<usize, String> {
    let Ok(platform) = model_platform_repo::get_one(platform_name).await else {
        eprintln!("Failed to get platform: {}", platform_name);
        return Ok(0);
    };

    let result = config_repo::update_by_name("active_model_platform", platform_name)
        .await
        .unwrap_or_else(|e| {
            println!("update config error:{}", e);
            0
        });
    *ACTIVE_MODEL_PLATFORM.write().await = platform;
    return Ok(result);
}

#[command]
pub async fn set_active_locale(app: AppHandle, locale: &str) -> Result<usize, String> {
    if locale.is_empty() {
        return Ok(0);
    }
    if locale != Locale::EnUs.text() && locale != Locale::ZhCn.text() {
        print!("Unsupported locale: {}", locale);
        return Ok(0);
    }
    rust_i18n::set_locale(locale);
    let result = config_repo::update_by_name("active_locale", locale)
        .await
        .unwrap_or_else(|e| {
            println!("update config error:{}", e);
            0
        });
    *ACTIVE_LOCALE.write().await = locale.to_string();
    let _ = app_util::rebuild_tray_menu(&app);
    println!("update db result, {}", result);
    Ok(result)
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
#[command]
pub async fn start_indexing(
    paths: Vec<String>,
    on_event: Channel<IndexingEvent>,
) -> Result<CommandResult, String> {
    if paths.is_empty() {
        let result = CommandResult::error(
            CommandResultCode::INDEXING,
            t!("message.indexing-paths-empty").to_string(),
        );
        return Ok(result);
    }
    if SCANNING.load(Ordering::SeqCst) {
        println!("Scan process already started.");
        let result = CommandResult::error(
            CommandResultCode::INDEXING,
            t!("message.scanning").to_string(),
        );
        return Ok(result);
    }
    if INDEXING.load(Ordering::SeqCst) {
        println!("Indexing process is already running");
        let result = CommandResult::error(
            CommandResultCode::INDEXING,
            t!("message.indexing-processing").to_string(),
        );
        return Ok(result);
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
    scanner::start(&paths, task.clone(), event.clone()).await;

    // Scanned files
    indexing_task_util::set_total(SCANNING_TOTAL.load(Ordering::SeqCst) as i64).await;
    indexing_task_util::summary_to_db().await;

    // Embedding processing
    INDEXING.store(true, Ordering::SeqCst);
    embedding_service_manager().write().await.clear();
    let embedding_service = EmbeddingService::new()
        .await
        .map_err(|op| AppError::InstantNewFailed(op.to_string()))?;

    println!("document indexing...");
    let mut document_indexer = indexers::document_indexer::DocumentIndexer::new(&embedding_service);
    let _ = document_indexer
        .process(task.clone(), event.clone())
        .await
        .unwrap_or_else(|e| println!("Document indexing error,{}", e));
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
            event.as_ref(),
        )
        .await?;
        return Ok(CommandResult::default());
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
            event.as_ref(),
        )
        .await?;

        return Err(format!(
            "Model platform '{}' is missing API key configuration",
            platform_name
        ));
    }

    if let Ok(mut image_indexer) =
        indexers::image_indexer::ImageIndexer::new(&embedding_service).await
    {
        println!("image indexing...");
        let _ = image_indexer
            .process(task.clone(), event.clone())
            .await
            .unwrap_or_else(|e| println!("image indexing error,{}", e));
        indexing_task_util::summary_to_db().await;
    }

    if let Ok(mut audio_indexer) =
        indexers::audio_indexer::AudioIndexer::new(&embedding_service).await
    {
        println!("audio indexing...");
        let _ = audio_indexer
            .process(task.clone(), event.clone())
            .await
            .unwrap_or_else(|e| println!("audio indexing error,{}", e));
        indexing_task_util::summary_to_db().await;
    }

    indexing_finish(task.id, "done", event.as_ref()).await?;

    return Ok(CommandResult::default());
}

async fn indexing_finish(
    task_id: i64,
    msg: &str,
    event: &Channel<IndexingEvent>,
) -> Result<(), String> {
    SCANNING.store(false, Ordering::SeqCst);
    INDEXING.store(false, Ordering::SeqCst);
    SCANNING_TOTAL.store(0, Ordering::SeqCst);
    STOP_INDEX_SIGNAL.store(false, Ordering::SeqCst);

    indexing_task_util::task_done().await?;

    frontend_util::send_to_frontend(
        event,
        IndexingEvent::Finish {
            task_id,
            msg: msg.to_string(),
        },
    );

    Ok(())
}

#[command]
pub async fn stop_indexing() {
    STOP_INDEX_SIGNAL.store(true, Ordering::SeqCst);
}

#[command]
pub async fn load_indexing_tasks(page: i64, page_size: i64) -> Result<Vec<IndexingTask>, String> {
    let tasks = indexing_task_repo::list(page, page_size).await?;
    Ok(tasks)
}

#[command]
pub async fn count_indexing_tasks() -> Result<i64, String> {
    let count = indexing_task_repo::count().await?;
    Ok(count)
}

#[command]
pub async fn load_files(page: i64, page_size: i64) -> Result<Vec<FileInfo>, String> {
    let files = file_info_repo::list(page, page_size).await?;
    Ok(files)
}

#[command]
pub async fn count_files() -> Result<i64, String> {
    let count = file_info_repo::count().await?;
    Ok(count)
}

#[command]
pub async fn download_multilingual_model(proxy: bool, on_event: Channel<DownloadEvent>) -> bool {
    println!("download multilingual model");
    if let Err(e) = download_util::download_multilingual_model(proxy, &on_event).await {
        eprintln!("download multilingual model error: {e}");
        return false;
    };
    return true;
}

#[command]
pub async fn check_path_type(path: &str) -> Result<String, String> {
    let path = Path::new(path);
    if path.is_dir() {
        Ok("directory".to_string())
    } else if path.is_file() {
        Ok("file".to_string())
    } else {
        Err("Path does not exist".to_string())
    }
}

#[command]
pub async fn delete_indexing_task(task_id: i64) -> Result<(), String> {
    indexing_task_repo::delete_by_id(task_id).await?;
    Ok(())
}

#[command]
pub async fn delete_file(file_id: i64) -> Result<(), String> {
    file_content_embedding_repo::delete_by_file_id(file_id).await?;
    file_metadata_embedding_repo::delete_by_file_id(file_id).await?;
    file_info_repo::delete_by_id(file_id).await?;
    Ok(())
}

#[command]
pub async fn quick_search(query: &str) -> Result<(), String> {
    searcher::warmup_embedding_service().await?;
    // let results = searcher::quick_search(query).await?; // todo
    Ok(())
}

#[command]
pub async fn search(query: &str) -> Result<Vec<FileInfo>, String> {
    let results = searcher::search(query).await?;
    Ok(results)
}

#[command]
pub async fn read_file_data(path: String) -> Result<Vec<u8>, String> {
    read(path).map_err(|e| e.to_string())
}

#[command]
pub async fn get_app_dir() -> Result<String, String> {
    Ok(APP_DIR.get().unwrap_or(&String::new()).to_string())
}

#[command]
pub async fn ui_mounted(app: AppHandle) -> Result<(), String> {
    println!("UI mounted");
    let locale = ACTIVE_LOCALE.read().await.clone();
    rust_i18n::set_locale(locale.as_str());
    let _ = app_util::rebuild_tray_menu(&app);
    crate::timers::start_after_ui_mounted(); // Timer start
    Ok(())
}

async fn chat() {
    let ai_model = ai_model_repo::get_one(
        ModelPlatformName::SiliconFlow.text(),
        "thudm/glm-z1-9b-0414",
    )
    .await
    .unwrap_or_else(|e| {
        println!("get ai model error: {e}");
        return None;
    });
    if let Some(aimodel) = ai_model {
        let callback = |content: &str| -> Result<(), Box<dyn std::error::Error>> {
            println!("Received: {}", content);
            Ok(())
        };
        SiliconFlow::new()
            .await
            .chat_stream(&aimodel, "Give me five", &callback)
            .await
            .unwrap_or_else(|e| {
                eprintln!("ask error: {e}");
            });
    } else {
        println!("No ai model found");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_locale() {
        // set_active_locale("en-US").await.unwrap_or_else(|e| {
        //     println!("set locale error: {}", e);
        //     0
        // });
    }
}
