mod db_initializer;
mod document_loaders;
mod embedding_service;
mod embedding_service_manager;
mod entities;
mod enums;
mod errors;
mod global;
mod indexer_service;
mod indexers;
mod initializer;
mod lib_commands;
mod model_platform_services;
mod repositories;
mod scanner;
mod searcher;
mod structs;
mod timers;
mod traits;
mod types;
mod utils;

use crate::lib_commands::{
    check_path_type, count_files, count_indexing_tasks, delete_file, delete_indexing_task,
    download_multilingual_model, is_embedding_model_changed, load_active_locale,
    load_active_platform, load_embedding_models, load_files, load_indexer_setting,
    load_indexing_tasks, load_model_by_type, load_model_platforms, load_proxy_info, quick_search,
    read_file_data, search, set_active_locale, set_active_platform, start_indexing, stop_indexing,
    ui_mounted, update_indexer_setting, update_model_platform, update_proxy_info, get_app_dir
};
use crate::utils::app_util;
use global::TRAY_ID;
use log::{error, info};
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use std::env;
use std::panic;
use tauri::Manager;
use tauri::WindowEvent;
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en-US");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }
    panic::set_hook(Box::new(|panic_info| {
        let message = format!("Application crashed: {:?}", panic_info);
        error!("{}", message);
        //TODO add file logger
    }));
    let mut builder = tauri::Builder::default();
    let log_path = {
        #[cfg(debug_assertions)]
        {
            let current_dir = env::current_dir()
                .map(|p| p.join("src-tauri"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|e| {
                    error!("Failed to get current directory: {}, using app data dir", e);
                    String::new()
                });
            Some(current_dir)
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    };
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }
    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .targets([
                    Target::new(TargetKind::LogDir {
                        file_name: log_path,
                    }),
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .rotation_strategy(RotationStrategy::KeepSome(5))
                .max_file_size(1024 * 1024 * 5)
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            load_active_locale,
            load_model_platforms,
            load_proxy_info,
            load_active_platform,
            load_indexer_setting,
            load_model_by_type,
            load_embedding_models,
            load_indexing_tasks,
            load_files,
            count_indexing_tasks,
            count_files,
            set_active_platform,
            set_active_locale,
            update_model_platform,
            update_proxy_info,
            update_indexer_setting,
            start_indexing,
            stop_indexing,
            download_multilingual_model,
            check_path_type,
            delete_indexing_task,
            delete_file,
            search,
            quick_search,
            read_file_data,
            ui_mounted,
            is_embedding_model_changed,
            get_app_dir
        ])
        .setup(|app| {
            let app_handle = app.handle();
            app_util::init_paths(app_handle);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                initializer::process().await;
            });
            let menu = app_util::create_tray_menu(app_handle)?;
            let tray_builder = TrayIconBuilder::with_id(TRAY_ID);
            let tray_builder = if let Some(icon) = app.default_window_icon() {
                tray_builder.icon(icon.clone())
            } else {
                tray_builder
            };
            let _ = tray_builder
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        app_util::show_app(app);
                    }
                    _ => {}
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        app_util::show_app(app);
                    }
                    "quit" => {
                        info!("Exit menu item was clicked");
                        app_util::exit_app(&app);
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if let Err(error) = window.hide() {
                    error!("error hiding window: {}", error);
                } else {
                    api.prevent_close();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
