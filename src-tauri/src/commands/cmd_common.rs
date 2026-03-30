use crate::fs_watcher::watcher;
use crate::global::{ACTIVE_LOCALE, APP_DATA_PATH, CLIENT_ID, UI_MOUNTED};
use crate::repositories::{file_content_embedding_repo, file_info_repo};
use crate::searcher;
use crate::utils::app_util;
use std::fs::read;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{command, AppHandle};

#[command]
pub async fn ui_mounted(app: AppHandle) -> Result<(), String> {
    log::info!("UI mounted");
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

    // Start cluster service if enabled
    let cluster_setting = crate::cluster::get_cluster_setting().await;
    if cluster_setting.enabled {
        tokio::spawn(async move {
            if let Err(e) = crate::cluster::start_cluster_service().await {
                log::error!("Failed to start cluster service: {}", e);
            }
        });
    }

    Ok(())
}

#[command]
pub async fn get_client_id() -> String {
    CLIENT_ID.read().await.clone()
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
pub async fn read_file_data(path: String) -> Result<Vec<u8>, String> {
    read(path).map_err(|e| e.to_string())
}

#[command]
pub async fn open_directory(path: &str) -> Result<(), String> {
    let path = Path::new(&path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
