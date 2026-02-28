use crate::enums::{FileContentLanguage, TrayMenuItem};
use crate::global::{
    APP_DATA_PATH, DB_PATH, DOWNLOADING, EMBEDDING_MODEL_NAME, EMBEDDING_MODEL_PATH,
    EMBEDDING_TOKENIZER_NAME, EMBEDDING_TOKENIZER_PATH, EXIT_APP_SIGNAL, HOME_PATH, INDEXING,
    SCANNING, STOP_INDEX_SIGNAL, STORAGE_PATH, TMP_PATH, TRAY_ID, VISION_MODEL_PATH, VISION_NAME,
    VISION_TOKENIZER_NAME, VISION_TOKENIZER_PATH,
};
use crate::utils::file_util;
use log::{error, info, warn};
use rust_i18n::t;
use std::env;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use sys_locale::get_locale;
use tauri::menu::{Menu, MenuItem};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub fn exit_app(app: &AppHandle) {
    let runing_tasks = running_background_tasks();
    if runing_tasks.len() > 0 {
        let answer = app
            .dialog()
            .message(t!(
                "message.abort-tasks-and-exit",
                tasks = runing_tasks.join(", ")
            ))
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                t!("common.yes").to_string(),
                t!("common.no").to_string(),
            ))
            .blocking_show();
        if !answer {
            return;
        }
    }
    STOP_INDEX_SIGNAL.store(true, Ordering::SeqCst);
    EXIT_APP_SIGNAL.store(true, Ordering::SeqCst);
    std::thread::sleep(std::time::Duration::from_millis(500));
    let windows = app.webview_windows();
    for (_, window) in windows {
        let _ = window.destroy();
    }
    app.exit(0);
}

pub fn running_background_tasks() -> Vec<&'static str> {
    let mut tasks = Vec::new();
    if DOWNLOADING.load(Ordering::SeqCst) {
        tasks.push("downloading");
    }
    if SCANNING.load(Ordering::SeqCst) {
        tasks.push("scanning");
    }
    if INDEXING.load(Ordering::SeqCst) {
        tasks.push("indexing");
    }
    tasks
}

pub fn show_app(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn create_tray_menu(app: &AppHandle) -> Result<Menu<Wry>, String> {
    let show_i = MenuItem::with_id(
        app,
        TrayMenuItem::Show.to_string(),
        t!("common.show"),
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let quit_i = MenuItem::with_id(
        app,
        TrayMenuItem::Quit.to_string(),
        t!("common.quit"),
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    let result = Menu::with_items(app, &[&show_i, &quit_i]).map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn rebuild_tray_menu(app: &AppHandle) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = create_tray_menu(app)?;
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn set_data_path(
    new_data_path: &str,
    force: bool,
    app: &AppHandle,
) -> Result<String, String> {
    if new_data_path.is_empty() {
        error!("new data path is empty");
        return Err("new data path is empty".to_string());
    }
    if new_data_path == *APP_DATA_PATH.read().await {
        return Ok("same".to_string());
    }
    let ndp = PathBuf::from(new_data_path);
    if !force {
        let mut exist_files = "".to_string();
        if ndp.join("storage").join("mango-finder.db").exists() {
            exist_files.push_str("mango-finder.db, ");
        }
        if !exist_files.is_empty() {
            return Ok("exist:".to_string() + &exist_files);
        }
    }
    let sys_data_path = app
        .path()
        .data_dir()
        .unwrap_or_else(|error| {
            error!("Failed to get user data directory:{}", error);
            PathBuf::from("./")
        })
        .join(env!("CARGO_PKG_NAME"));
    info!("system data path: {}", sys_data_path.display());
    let old_data_path = APP_DATA_PATH.read().await.clone();
    let config_path = sys_data_path.join(".config");
    if let Err(e) = std::fs::write(&config_path, new_data_path) {
        error!("Failed to write data path record: {}", e);
    }
    let old_path_buf = PathBuf::from(old_data_path);
    if old_path_buf.exists() {
        let old_db = old_path_buf.join("storage").join("mango-finder.db");
        if old_db.exists() {
            let new_db = ndp.join("storage").join("mango-finder.db");
            file_util::copy_file(&old_db, &new_db)
                .map_err(|e| format!("Failed to copy db file: {}", e))?;
        }
    }
    let mut guard = APP_DATA_PATH.write().await;
    *guard = new_data_path.to_string();
    drop(guard);
    Ok("success".to_string())
}

pub async fn reset_data_path(force: bool, app: &AppHandle) -> Result<String, String> {
    let sys_data_path = app
        .path()
        .data_dir()
        .unwrap_or_else(|error| {
            error!("Failed to get user data directory:{}", error);
            PathBuf::from("./")
        })
        .join(env!("CARGO_PKG_NAME"));
    set_data_path(sys_data_path.to_str().unwrap_or(""), force, app).await
}

/// If the {user data directory}/.config file exists, read the path from it. Otherwise, use {user data directory} as default.
pub async fn init_paths(app: &AppHandle) {
    let mut data_path = app
        .path()
        .data_dir()
        .unwrap_or_else(|error| {
            error!("Failed to get user data directory:{}", error);
            PathBuf::from("./")
        })
        .join(env!("CARGO_PKG_NAME"));
    info!("system data path: {}", data_path.display());
    let config_path = data_path.join(".config");
    info!("MangoFinder config file: {}", config_path.display());
    // Default to use user data directory. Otherwise, use the path from .config file.
    if !config_path.exists() {
        if let Err(e) = std::fs::write(&config_path, data_path.to_str().unwrap_or("")) {
            error!("Failed to write data path record: {}", e);
        }
    } else {
        match std::fs::File::open(&config_path) {
            Ok(file) => {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                if let Some(Ok(first_line)) = reader.lines().next() {
                    if !first_line.is_empty() {
                        data_path = PathBuf::from(first_line);
                        info!("read data_dir from config file: {}", data_path.display());
                    } else {
                        warn!("Empty line in data path record, using default directory");
                    }
                } else {
                    error!("Failed to read data path record: empty line");
                }
            }
            Err(e) => {
                error!("Failed to read data path record: {}", e);
            }
        }
    }
    info!("data_path: {}", data_path.display());
    let data_path_str = data_path.to_string_lossy().into_owned();
    info!("MangoFinder data directory: {}", data_path_str);
    let mut guard = APP_DATA_PATH.write().await;
    *guard = data_path_str;
    drop(guard);
    if !Path::new(&data_path).exists() {
        create_dir(&data_path).unwrap_or_else(|error| {
            error!("Failed to create app directory: {}", error);
        });
    }
    // Define storage path
    let storage_path = Path::new(&data_path).join("storage");
    if !storage_path.exists() {
        create_dir(&storage_path).unwrap_or_else(|error| {
            error!("Failed to create storage directory: {}", error);
        });
    }
    STORAGE_PATH
        .set(storage_path.to_string_lossy().into_owned())
        .unwrap_or_else(|e| error!("Failed to set STORAGE_PATH: {}", e));
    info!(
        "Storage directory: {}",
        STORAGE_PATH.get().unwrap_or(&String::new())
    );
    let db_path = Path::new(&data_path).join("storage").join("mango-finder.db");
    DB_PATH
        .set(db_path.to_string_lossy().into_owned())
        .unwrap_or_else(|error| error!("Failed to set DB_PATH: {}", error));
    info!(
        "Database path: {}",
        DB_PATH.get().unwrap_or(&String::new()).to_string()
    );
    // Tmp download directory
    let tmp_path = data_path.join("tmp");
    if !tmp_path.exists() {
        create_dir(&tmp_path).unwrap_or_else(|error| {
            error!("Failed to create tmp directory: {}", error);
        });
    }
    TMP_PATH
        .set(tmp_path.to_string_lossy().into_owned())
        .unwrap_or_else(|e| error!("Failed to set TMP_DOWNLOAD_DIRECTORY: {}", e));
    info!(
        "Temp path: {}",
        TMP_PATH.get().unwrap_or(&String::new()).to_string()
    );

    init_embedding_model_path(app);
}

fn init_embedding_model_path(app_handle: &AppHandle) {
    let app_dir = {
        #[cfg(debug_assertions)]
        {
            env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|e| {
                    error!("Failed to get current directory: {}", e);
                    "./".to_string()
                })
        }
        #[cfg(not(debug_assertions))]
        {
            env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| {
                    error!("Failed to get executable directory");
                    "./".to_string()
                })
        }
    };
    if HOME_PATH.set(app_dir.clone()).is_err() {
        error!("Warning: HOME_PATH was already set");
    }
    info!("Home directory: {}", app_dir);
    let resource_dir = app_handle.path().resource_dir().unwrap_or_else(|e| {
        error!("Failed to get resource directory: {}", e);
        PathBuf::from(app_dir).join("assets")
    });
    info!("Resource directory: {}", resource_dir.display());
    let build_in_model_path = resource_dir.join("assets").join("model");
    if !build_in_model_path.exists() {
        create_dir(&build_in_model_path).unwrap_or_else(|error| {
            error!("Failed to create assets directory: {}", error);
        });
    }

    let multilingual_embedding_path = build_in_model_path
        .join(EMBEDDING_MODEL_NAME)
        .to_string_lossy()
        .into_owned();
    EMBEDDING_MODEL_PATH
        .set(multilingual_embedding_path)
        .unwrap_or_else(|e| error!("Failed to set EMBEDDING_MODEL_PATH: {}", e));
    info!(
        "Multi-language embedding model path: {}",
        EMBEDDING_MODEL_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
    let multilingual_tokenizer_path = build_in_model_path
        .join(EMBEDDING_TOKENIZER_NAME)
        .to_string_lossy()
        .into_owned();
    EMBEDDING_TOKENIZER_PATH
        .set(multilingual_tokenizer_path)
        .unwrap_or_else(|e| error!("Failed to set EMBEDDING_TOKENIZER_PATH: {}", e));
    info!(
        "Multi-language tokenizer path: {}",
        EMBEDDING_TOKENIZER_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );

    // Vision model
    let vision_path = build_in_model_path
        .join(VISION_NAME)
        .to_string_lossy()
        .into_owned();
    VISION_MODEL_PATH
        .set(vision_path)
        .unwrap_or_else(|e| error!("Failed to set VISION_0_PATH: {}", e));
    info!(
        "Vision model path: {}",
        VISION_MODEL_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );

    let vision_tokenizer_path = build_in_model_path
        .join(VISION_TOKENIZER_NAME)
        .to_string_lossy()
        .into_owned();
    VISION_TOKENIZER_PATH
        .set(vision_tokenizer_path)
        .unwrap_or_else(|e| error!("Failed to set VISION_TOKENIZER_PATH: {}", e));
}

pub fn get_db_path() -> String {
    DB_PATH.get().unwrap_or(&String::new()).to_string()
}

pub fn get_assets_tmp_path() -> String {
    TMP_PATH.get().unwrap_or(&String::new()).to_string()
}

pub fn get_multilingual_embedding_path() -> String {
    EMBEDDING_MODEL_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_multilingual_tokenizer_path() -> String {
    EMBEDDING_TOKENIZER_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_vision_0_path() -> String {
    VISION_MODEL_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_vision_tokenizer_path() -> String {
    VISION_TOKENIZER_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_default_file_content_language() -> FileContentLanguage {
    let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
    println!("locale: {}", locale);
    match locale.as_str() {
        l if l.starts_with("zh") => FileContentLanguage::Chinese,
        l if l.starts_with("en") => FileContentLanguage::English,
        _ => FileContentLanguage::English, // 默认回退
    }
}
