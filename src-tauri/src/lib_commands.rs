use crate::embedding_service_manager::get_manager;
use crate::entities::{FileInfo, IndexingTask, ModelPlatform};
use crate::enums::CommandResultCode;
use crate::enums::{DownloadEvent, IndexingEvent, Locale, ModelPlatformName};
use crate::errors::AppError;
use crate::fs_watcher::watcher;
use crate::global::{
    ACTIVE_LOCALE, ACTIVE_MODEL_PLATFORM, APP_DATA_PATH, CLIENT_ID, CONFIG_NAME_INDEXER_SETTING,
    CONFIG_NAME_PROXY, INDEXING, SCANNING, STOP_INDEX_SIGNAL, UI_MOUNTED,
};
use crate::indexer_service;
use crate::model_platform_services::siliconflow::SiliconFlow;
use crate::repositories::{
    ai_model_repo, config_repo, file_content_embedding_repo, file_info_repo,
    file_metadata_embedding_repo, indexing_task_repo, model_platform_repo,
};
use crate::searcher;
use crate::structs::command_result::CommandResult;
use crate::structs::proxy_setting::ProxyInfo;
use crate::structs::search_result::SearchResult;
use crate::traits::chat_capable::ChatCapable;
use crate::utils::{app_util, download_util};
use rust_i18n::t;
use serde_json::json;
use std::fs::read;
use std::path::Path;
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
    .unwrap_or_else(|e| {
        println!("Failed to load model platforms: {}", e);
        vec![]
    })
}

#[command]
pub async fn load_proxy_info() -> Result<ProxyInfo, String> {
    let result = config_repo::get_one(CONFIG_NAME_PROXY)?
        .map(|config| serde_json::from_str(&config.value).map_err(|e| e.to_string()))
        .unwrap_or_else(|| Ok(ProxyInfo::default()))?;
    Ok(result)
}

#[command]
pub async fn load_indexer_setting()
-> Result<crate::structs::indexer_setting::IndexerSetting, String> {
    let result = config_repo::get_one(CONFIG_NAME_INDEXER_SETTING)?
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
    let result = ai_model_repo::get_one_by_type(platform, one_type)?;
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
    Ok(config_repo::update_by_name("proxy", &proxy_json)?)
}

#[command]
pub async fn update_model_platform(platform: ModelPlatform) -> Result<usize, AppError> {
    let result = model_platform_repo::update_by_name(&platform.name, &platform)?;
    if platform.name == ACTIVE_MODEL_PLATFORM.read().await.name {
        match ACTIVE_MODEL_PLATFORM.try_write() {
            Ok(mut guard) => {
                let one = model_platform_repo::get_one(&platform.name)?;
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
    let Ok(platform) = model_platform_repo::get_one(platform_name) else {
        eprintln!("Failed to get platform: {}", platform_name);
        return Ok(0);
    };

    let result = config_repo::update_by_name("active_model_platform", platform_name)
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
    let result = config_repo::update_by_name("active_locale", locale).unwrap_or_else(|e| {
        println!("update config error:{}", e);
        0
    });
    *ACTIVE_LOCALE.write().await = locale.to_string();
    let _ = app_util::rebuild_tray_menu(&app);
    println!("update db result, {}", result);
    Ok(result)
}

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
    let result = indexer_service::start_indexing(paths, on_event).await?;
    if result {
        return Ok(CommandResult::default());
    }
    let result = CommandResult::error(CommandResultCode::ERROR, "Error".to_string());
    return Ok(result);
}

#[command]
pub async fn stop_indexing() {
    STOP_INDEX_SIGNAL.store(true, Ordering::SeqCst);
}

#[command]
pub async fn load_indexing_tasks(
    page: i64,
    page_size: i64,
    column_key: &str,
    sort_order: &str,
) -> Result<Vec<IndexingTask>, String> {
    let tasks = indexing_task_repo::list(page, page_size, column_key, sort_order)?;
    Ok(tasks)
}

#[command]
pub async fn count_indexing_tasks() -> Result<i64, String> {
    let count = indexing_task_repo::count()?;
    Ok(count)
}

#[command]
pub async fn load_files(page: i64, page_size: i64) -> Result<Vec<FileInfo>, String> {
    let files = file_info_repo::list(page, page_size)?;
    Ok(files)
}

#[command]
pub async fn count_files() -> Result<i64, String> {
    let count = file_info_repo::count()?;
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
pub fn delete_indexing_task(task_id: i64) -> Result<(), String> {
    indexing_task_repo::delete_by_id(task_id)?;
    Ok(())
}

#[command]
pub async fn load_file_detail(file_id: i64) -> Result<Option<FileInfo>, String> {
    let file = file_info_repo::get_by_id(file_id)?;
    Ok(file)
}

#[command]
pub async fn delete_index_item(file_id: i64) -> Result<(), String> {
    file_content_embedding_repo::delete_by_file_id(file_id)?;
    file_metadata_embedding_repo::delete_by_file_id(file_id)?;
    file_info_repo::delete_by_id(file_id)?;
    Ok(())
}

#[command]
pub async fn clear_index() -> Result<(), String> {
    file_content_embedding_repo::clear()?;
    file_metadata_embedding_repo::clear()?;
    file_info_repo::clear()?;
    Ok(())
}

#[command]
pub async fn quick_search(query: &str) -> Result<(), String> {
    match get_manager().try_write() {
        Ok(mut manager) => {
            manager.warmup().await.map_err(|e| e.to_string())?;
        }
        Err(_) => {
            println!("Failed to get manager lock")
        }
    }
    // let results = searcher::quick_search(query).await?; // todo
    Ok(())
}

#[command]
pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let results = searcher::search_with_intent(query).await;
    Ok(results)
}

#[command]
pub async fn path_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let results = searcher::path_search(query).await;
    Ok(results)
}

#[command]
pub async fn semantic_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let results = searcher::semantic_search(query).await;
    Ok(results)
}

#[command]
pub async fn read_file_data(path: String) -> Result<Vec<u8>, String> {
    read(path).map_err(|e| e.to_string())
}

#[command]
pub async fn get_data_path() -> Result<String, String> {
    Ok(APP_DATA_PATH.read().await.to_string())
}

#[command]
pub async fn set_data_path(path: String, force: bool, app: AppHandle) -> Result<String, String> {
    app_util::set_data_path(&path, force, &app).await
}

#[command]
pub async fn reset_data_path(force: bool, app: AppHandle) -> Result<String, String> {
    app_util::reset_data_path(force, &app).await
}

#[command]
pub async fn load_config_value(config_name: &str) -> Result<String, String> {
    let config = config_repo::get_one(config_name)?;
    if let Some(config) = config {
        return Ok(config.value);
    }
    Ok(String::new())
}

#[command]
pub async fn add_watch_path(path: &str) -> Result<(), String> {
    watcher::add_path(path).await.unwrap_or_else(|error| {
        log::error!("add watch path error:{}", error);
    });
    Ok(())
}

#[command]
pub async fn remove_watch_path(path: &str) -> Result<(), String> {
    watcher::remove_path(path).await.unwrap_or_else(|error| {
        log::error!("remove watch path error:{}", error);
    });
    Ok(())
}

#[command]
pub async fn ui_mounted(app: AppHandle) -> Result<(), String> {
    println!("UI mounted");
    UI_MOUNTED.store(true, Ordering::SeqCst);
    let locale = ACTIVE_LOCALE.read().await.clone();
    rust_i18n::set_locale(locale.as_str());
    let _ = app_util::rebuild_tray_menu(&app);
    tokio::spawn(async move {
        crate::timers::start_after_ui_mounted(); // Timer start
        watcher::init_after_ui_mounted()
            .await
            .unwrap_or_else(|error| {
                log::error!("init file watch error:{}", error);
            }); // File watcher start
        if file_content_embedding_repo::count().unwrap_or(0) > 0 {
            searcher::semantic_search_engine::warmup_embedding_service()
                .await
                .unwrap_or_else(|error| {
                    log::error!("first warming up embedding service error: {}", error);
                });
        }
    });
    tokio::spawn(async {
        searcher::path_search_engine::init().await;
    });
    Ok(())
}

#[command]
pub async fn load_chunks(ids: Vec<u32>) -> Result<Vec<String>, String> {
    let segments =
        file_content_embedding_repo::list_chunks_by_ids(&ids).map_err(|e| e.to_string())?;
    Ok(segments)
}

async fn chat() {
    let ai_model = ai_model_repo::get_one(
        ModelPlatformName::SiliconFlow.text(),
        "thudm/glm-z1-9b-0414",
    )
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

#[command]
pub async fn get_client_id() -> String {
    CLIENT_ID.read().await.clone()
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
