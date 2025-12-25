use crate::entities::ModelPlatform;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::audio_analyzer::AudioAnalyzer;
use crate::traits::chat_capable::ChatCapable;
use crate::traits::image_analyzer::ImageAnalyzer;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::utils::llm_client_util::init_service;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAiCompatibleService {
    platform: ModelPlatform,
    proxy: ProxyInfo,
    name: String,
}

impl OpenAiCompatibleService {
    pub async fn new(platform_name: &str, base_url: &str) -> Self {
        let (platform, proxy) = init_service(platform_name, Some(base_url.to_string())).await;
        return OpenAiCompatibleService {
            platform,
            proxy,
            name: platform_name.to_string(),
        };
    }
}

impl WithPlatformConfig for OpenAiCompatibleService {
    fn proxy(&self) -> &ProxyInfo {
        &self.proxy
    }
    fn platform(&self) -> &ModelPlatform {
        &self.platform
    }
}

impl ChatCapable for OpenAiCompatibleService {}

impl ImageAnalyzer for OpenAiCompatibleService {
    fn is_stream(&self) -> bool {
        false
    }
}

impl AudioAnalyzer for OpenAiCompatibleService {}
