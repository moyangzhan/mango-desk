use crate::entities::{ModelPlatform, SelfHostedPlatform};
use crate::enums::{Locale, ModelPlatformName};
use crate::errors::AppError;
use crate::global::{
    ACTIVE_LOCALE, ACTIVE_MODEL_PLATFORM, ACTIVE_SELF_HOSTED_PLATFORM,
    CONFIG_NAME_ACTIVE_SELF_HOSTED_PLATFORM, CONFIG_NAME_INDEXER_SETTING, CONFIG_NAME_PROXY,
};
use crate::indexer_service;
use crate::repositories::{ai_model_repo, config_repo, model_platform_repo, self_hosted_platform_repo};
use crate::structs::proxy_setting::ProxyInfo;
use crate::utils::app_util;
use rust_i18n::t;
use std::sync::atomic::Ordering;
use tauri::{command, AppHandle};

// ========== Locale commands ==========

#[command]
pub async fn load_active_locale() -> Result<String, String> {
    let guard = ACTIVE_LOCALE.read().await;
    Ok(guard.clone())
}

#[command]
pub async fn set_active_locale(app: AppHandle, locale: &str) -> Result<usize, String> {
    if locale.is_empty() {
        return Ok(0);
    }
    if locale != Locale::EnUs.text() && locale != Locale::ZhCn.text() {
        log::warn!("Unsupported locale: {}", locale);
        return Ok(0);
    }
    rust_i18n::set_locale(locale);
    let result = config_repo::update_by_name("active_locale", locale).unwrap_or_else(|e| {
        log::error!("update config error: {}", e);
        0
    });
    *ACTIVE_LOCALE.write().await = locale.to_string();
    let _ = app_util::rebuild_tray_menu(&app);
    log::debug!("update db result: {}", result);
    Ok(result)
}

// ========== Model platform commands ==========

#[command]
pub async fn load_model_platforms() -> Vec<ModelPlatform> {
    model_platform_repo::list(&vec![
        ModelPlatformName::OpenAi.text().to_string(),
        ModelPlatformName::SiliconFlow.text().to_string(),
    ])
    .unwrap_or_else(|e| {
        log::error!("Failed to load model platforms: {}", e);
        vec![]
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
pub async fn update_ai_model(
    id: i64,
    name: String,
    title: String,
    remark: String,
) -> Result<usize, String> {
    let result = ai_model_repo::update_basic(id, &name, &title, &remark)?;
    Ok(result)
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
                log::error!("Failed to acquire write lock for ACTIVE_MODEL_PLATFORM");
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
        log::error!("Failed to get platform: {}", platform_name);
        return Ok(0);
    };

    let result = config_repo::update_by_name("active_model_platform", platform_name)
        .unwrap_or_else(|e| {
            log::error!("update config error: {}", e);
            0
        });
    *ACTIVE_MODEL_PLATFORM.write().await = platform;
    return Ok(result);
}

// ========== Self-hosted platform commands ==========

#[command]
pub async fn load_self_hosted_platforms() -> Vec<SelfHostedPlatform> {
    self_hosted_platform_repo::list().unwrap_or_else(|e| {
        log::error!("Failed to load self-hosted platforms: {}", e);
        vec![]
    })
}

#[command]
pub async fn load_active_self_hosted_platform() -> String {
    let platform = ACTIVE_SELF_HOSTED_PLATFORM.read().await;
    platform.name.clone()
}

#[command]
pub async fn set_active_self_hosted_platform(platform_name: &str) -> Result<usize, String> {
    let Ok(platform) = self_hosted_platform_repo::get_one(platform_name) else {
        log::error!("Failed to get self-hosted platform: {}", platform_name);
        return Ok(0);
    };

    let result =
        config_repo::update_by_name(CONFIG_NAME_ACTIVE_SELF_HOSTED_PLATFORM, platform_name)
            .unwrap_or_else(|e| {
                log::error!("update config error: {}", e);
                0
            });
    *ACTIVE_SELF_HOSTED_PLATFORM.write().await = platform;
    return Ok(result);
}

#[command]
pub async fn update_self_hosted_platform(platform: SelfHostedPlatform) -> Result<usize, AppError> {
    let result = self_hosted_platform_repo::update_by_name(&platform.name, &platform)?;
    if platform.name == ACTIVE_SELF_HOSTED_PLATFORM.read().await.name {
        match ACTIVE_SELF_HOSTED_PLATFORM.try_write() {
            Ok(mut guard) => {
                let one = self_hosted_platform_repo::get_one(&platform.name)?;
                *guard = one;
            }
            Err(_) => {
                log::error!("Failed to acquire write lock for ACTIVE_SELF_HOSTED_PLATFORM");
            }
        }
    }
    Ok(result)
}

// ========== Proxy commands ==========

#[command]
pub async fn load_proxy_info() -> Result<ProxyInfo, String> {
    let result = config_repo::get_one(CONFIG_NAME_PROXY)?
        .map(|config| serde_json::from_str(&config.value).map_err(|e| e.to_string()))
        .unwrap_or_else(|| Ok(ProxyInfo::default()))?;
    Ok(result)
}

#[command]
pub async fn update_proxy_info(proxy_info: ProxyInfo) -> Result<usize, String> {
    let proxy_json = serde_json::to_string(&proxy_info).map_err(|e| AppError::SerializeError(e))?;
    Ok(config_repo::update_by_name("proxy", &proxy_json)?)
}

// ========== Indexer setting commands ==========

#[command]
pub async fn load_indexer_setting()
-> Result<crate::structs::indexer_setting::IndexerSetting, String> {
    let mut result = config_repo::get_one(CONFIG_NAME_INDEXER_SETTING)?
        .map(|config| {
            serde_json::from_str(&config.value).map_err(|e| {
                log::error!("Failed to parse indexer setting: {}", e);
                e.to_string()
            })
        })
        .unwrap_or_else(|| Ok(crate::structs::indexer_setting::IndexerSetting::default()))?;

    // Add HOME_PATH to ignore_path_prefixes if not already present
    if let Some(home_path) = crate::global::HOME_PATH.get() {
        if !result.ignore_path_prefixes.contains(&home_path.to_string()) {
            result.ignore_path_prefixes.push(home_path.to_string());
        }
    }

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

// ========== Config commands ==========

#[command]
pub async fn load_config_value(config_name: &str) -> Result<String, String> {
    let config = config_repo::get_one(config_name)?;
    if let Some(config) = config {
        return Ok(config.value);
    }
    Ok(String::new())
}

#[command]
pub async fn update_config(name: &str, value: &str) -> Result<(), String> {
    config_repo::update_by_name(name, value)?;
    Ok(())
}
