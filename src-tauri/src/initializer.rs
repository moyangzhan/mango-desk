use crate::db_initializer;
use crate::global::{
    ACTIVE_LOCALE, ACTIVE_MODEL_PLATFORM, CLIENT_ID, CONFIG_NAME_ACTIVE_LOCALE,
    CONFIG_NAME_CLIENT_ID, CONFIG_NAME_INDEXER_SETTING, CONFIG_NAME_PROXY,
    CONFIG_NAME_WATCHER_SETTING, FS_WATCHER_SETTING, INDEXER_SETTING,
    ONNX_EXEC_PROVIDERS_INITIALIZED, PROXY,
};
use crate::repositories::{config_repo, model_platform_repo};
use crate::structs::fs_watcher_setting::FsWatcherSetting;
use crate::structs::indexer_setting::IndexerSetting;
use crate::structs::proxy_setting::ProxyInfo;
use anyhow::Context;
use log::{error, info};
use ort::execution_providers::{CPUExecutionProvider, CUDAExecutionProvider};
use serde::Deserialize;
use serde_json;
use std::sync::LazyLock;
use tokio::sync::{RwLock as AsyncRwLock, RwLockWriteGuard as AsyncRwLockWriteGuard};

pub async fn process() {
    db_initializer::init()
        .context("Failed to initialize database")
        .unwrap_or_else(|e| error!("db init error: {e:?}"));

    // Initialize the client_id
    init_string_setting(
        CONFIG_NAME_CLIENT_ID,
        &CLIENT_ID,
    )
    .await;

    init_setting(
        CONFIG_NAME_PROXY,
        || serde_json::to_string(&ProxyInfo::default()).unwrap_or_default(),
        &PROXY,
    )
    .await;
    init_setting(
        CONFIG_NAME_INDEXER_SETTING,
        || serde_json::to_string(&IndexerSetting::default()).unwrap_or_default(),
        &INDEXER_SETTING,
    )
    .await;
    init_setting(
        CONFIG_NAME_WATCHER_SETTING,
        || serde_json::to_string(&FsWatcherSetting::default()).unwrap_or_default(),
        &FS_WATCHER_SETTING,
    )
    .await;
    init_string_setting(CONFIG_NAME_ACTIVE_LOCALE, &ACTIVE_LOCALE).await;

    //Onnx Runtime initialization
    if ONNX_EXEC_PROVIDERS_INITIALIZED.get().is_none() {
        let result = ort::init()
            .with_execution_providers(vec![
                CUDAExecutionProvider::default().into(),
                CPUExecutionProvider::default().into(),
            ])
            .commit();
        if result.is_err() {
            error!(
                "Failed to initialize execution providers, falling back to CPU only: {}",
                result.unwrap_err()
            );
        } else {
            ONNX_EXEC_PROVIDERS_INITIALIZED
                .set(true)
                .unwrap_or_else(|e| error!("{e}"));
        }
    }

    // Initialize the default model platform
    let config = config_repo::get_one("active_model_platform");
    if let Ok(Some(config)) = config {
        let model_platform = model_platform_repo::get_one(&config.value);
        if let Ok(platform) = model_platform {
            *ACTIVE_MODEL_PLATFORM.write().await = platform;
        } else {
            error!("Cannt find model platform:{}", config.value);
        }
    } else {
        error!("Failed to get config: active_model_platform");
    }
}

pub trait ConfigLock<T: 'static> {
    async fn write(&self) -> AsyncRwLockWriteGuard<'_, T>;
}

impl<T: 'static> ConfigLock<T> for LazyLock<AsyncRwLock<T>> {
    async fn write(&self) -> AsyncRwLockWriteGuard<'_, T> {
        (**self).write().await
    }
}

/// TODO: default_value should be a lazy caculed value
pub async fn init_setting<T: Clone + std::fmt::Debug + for<'de> Deserialize<'de> + 'static>(
    config_name: &str,
    default_value_fn: impl Fn() -> String,
    lock: &impl ConfigLock<T>,
) {
    let config_resp = config_repo::get_one(config_name);
    if let Some(setting) = match config_resp {
        Ok(Some(config)) => Some(config),
        Ok(None) => {
            let default_value = default_value_fn();
            if default_value.is_empty() {
                None
            } else {
                config_repo::insert_or_ignore(config_name, default_value.as_str())
                    .unwrap_or_default();
                config_repo::get_one(config_name).unwrap_or(None)
            }
        }
        Err(error) => {
            error!("get setting from config table error: {error}");
            None
        }
    }
    .and_then(|s| (!s.value.is_empty()).then(|| s.value))
    .and_then(|s| serde_json::from_str(&s).ok())
    {
        let mut setting_guard = lock.write().await;
        *setting_guard = setting;
        info!("init {} setting: {:?}", config_name, *setting_guard);
    }
}

async fn init_string_setting(config_name: &str, lock: &impl ConfigLock<String>) {
    if let Some(setting) = config_repo::get_one(config_name)
        .unwrap_or_else(|error| {
            error!("get setting from config table error: {error}");
            None
        })
        .and_then(|s| (!s.value.is_empty()).then(|| s.value))
    {
        let mut setting_guard = lock.write().await;
        *setting_guard = setting;
        info!("init string {} setting: {:?}", config_name, *setting_guard);
    }
}
