use crate::enums::TrayMenuItem;
use crate::global::{
    APP_DIR, ASSETS_PATH, DB_PATH, DOWNLOADING, EN_EMBEDDING_PATH, EN_TOKENIZER_PATH,
    EXIT_APP_SIGNAL, INDEXING, MULTI_LANG_EMBEDDING_PATH, MULTI_LANG_TOKENIZER_PATH, SCANNING,
    STOP_INDEX_SIGNAL, STORAGE_PATH, TMP_DOWNLOAD_PATH, TRAY_ID,
};
use log::{error, info, warn};
use rust_i18n::t;
use std::env;
use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
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

pub fn init_paths(app: &AppHandle) {
    let mut data_dir = app
        .path()
        .data_dir()
        .unwrap_or_else(|error| {
            error!("Failed to get user data directory:{}", error);
            PathBuf::from("./")
        })
        .join(env!("CARGO_PKG_NAME"));
    info!("use dir: {}", data_dir.display());
    let config_path = data_dir.join(".config");
    info!("config_path: {}", config_path.display());
    if !config_path.exists() {
        if let Err(e) = std::fs::write(&config_path, data_dir.to_str().unwrap_or("")) {
            error!("Failed to write data path record: {}", e);
        }
    } else {
        match std::fs::File::open(&config_path) {
            Ok(file) => {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                if let Some(Ok(first_line)) = reader.lines().next() {
                    if !first_line.is_empty() {
                        data_dir = PathBuf::from(first_line);
                        info!("read data_dir from config file: {}", data_dir.display());
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
    info!("data_dir: {}", data_dir.display());
    #[cfg(debug_assertions)]
    {
        data_dir = env::current_dir()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|e| {
                error!("Failed to get current directory: {}", e);
                PathBuf::from("./")
            });
        info!(
            "Debug mode: using current directory as data directory: {}",
            data_dir.display()
        );
    }
    let app_data_path = data_dir.to_string_lossy().into_owned();
    info!("App data directory: {}", app_data_path);
    if APP_DIR.set(app_data_path).is_err() {
        error!("Warning: APP_DIR was already set");
    }
    if !Path::new(&data_dir).exists() {
        create_dir(&data_dir).unwrap_or_else(|error| {
            error!("Failed to create app directory: {}", error);
        });
    }
    // Define storage path
    let storage_path = Path::new(&data_dir).join("storage");
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
    let db_path = Path::new(&data_dir).join("storage").join("mango-desk.db");
    DB_PATH
        .set(db_path.to_string_lossy().into_owned())
        .unwrap_or_else(|error| error!("Failed to set DB_PATH: {}", error));
    info!(
        "Database path: {}",
        DB_PATH.get().unwrap_or(&String::new()).to_string()
    );
    // Assets directory
    let assets_dir = Path::new(&data_dir).join("assets");
    if !assets_dir.exists() {
        create_dir(&assets_dir).unwrap_or_else(|error| {
            error!("Failed to create assets directory: {}", error);
        });
    }
    ASSETS_PATH
        .set(assets_dir.to_string_lossy().into_owned())
        .unwrap_or_else(|e| error!("Failed to set ASSETS_PATH: {}", e));
    info!(
        "Assets directory: {}",
        ASSETS_PATH.get().unwrap_or(&String::new()).to_string()
    );

    let assets_model_dir = assets_dir.join("model");
    // Multi-language embedding model paths
    let multilingual_embedding_path = assets_model_dir
        .join("model.onnx")
        .to_string_lossy()
        .into_owned();
    MULTI_LANG_EMBEDDING_PATH
        .set(multilingual_embedding_path)
        .unwrap_or_else(|e| error!("Failed to set MULTI_LANG_EMBEDDING_PATH: {}", e));
    info!(
        "Multi-language embedding model path: {}",
        MULTI_LANG_EMBEDDING_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
    let multilingual_tokenizer_path = assets_model_dir
        .join("tokenizer.json")
        .to_string_lossy()
        .into_owned();
    MULTI_LANG_TOKENIZER_PATH
        .set(multilingual_tokenizer_path)
        .unwrap_or_else(|e| error!("Failed to set MULTI_LANG_TOKENIZER_PATH: {}", e));
    info!(
        "Multi-language tokenizer path: {}",
        MULTI_LANG_TOKENIZER_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
    // English embedding model paths
    let en_embedding_path = assets_model_dir
        .join("all-minilm-l6-v2.onnx")
        .to_string_lossy()
        .into_owned();
    EN_EMBEDDING_PATH
        .set(en_embedding_path)
        .unwrap_or_else(|e| error!("Failed to set EN_EMBEDDING_PATH: {}", e));
    info!(
        "English embedding model path: {}",
        EN_EMBEDDING_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
    let en_tokenizer_path = assets_model_dir
        .join("all-minilm-l6-v2-tokenizer.json")
        .to_string_lossy()
        .into_owned();
    EN_TOKENIZER_PATH
        .set(en_tokenizer_path)
        .unwrap_or_else(|e| error!("Failed to set EN_TOKENIZER_PATH: {}", e));
    info!(
        "English tokenizer path: {}",
        EN_TOKENIZER_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
    // Tmp download directory
    let assets_tmp_dir = assets_dir.join("tmp");
    if !assets_tmp_dir.exists() {
        create_dir(&assets_tmp_dir).unwrap_or_else(|error| {
            error!("Failed to create assets tmp directory: {}", error);
        });
    }
    TMP_DOWNLOAD_PATH
        .set(assets_tmp_dir.to_string_lossy().into_owned())
        .unwrap_or_else(|e| error!("Failed to set TMP_DOWNLOAD_DIRECTORY: {}", e));
    info!(
        "Assets tmp directory: {}",
        TMP_DOWNLOAD_PATH
            .get()
            .unwrap_or(&String::new())
            .to_string()
    );
}

pub fn get_db_path() -> String {
    DB_PATH.get().unwrap_or(&String::new()).to_string()
}

pub fn get_assets_tmp_path() -> String {
    TMP_DOWNLOAD_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_multilingual_embedding_path() -> String {
    MULTI_LANG_EMBEDDING_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_multilingual_tokenizer_path() -> String {
    MULTI_LANG_TOKENIZER_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_english_embedding_path() -> String {
    EN_EMBEDDING_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}

pub fn get_english_tokenizer_path() -> String {
    EN_TOKENIZER_PATH
        .get()
        .unwrap_or(&String::new())
        .to_string()
}
