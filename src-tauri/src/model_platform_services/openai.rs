use crate::enums::ModelPlatformName;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::chat_capable::ChatCapable;
use crate::traits::image_analyzer::ImageAnalyzer;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::utils::llm_client_util::init_service;
use crate::{entities::ModelPlatform, traits::audio_analyzer::AudioAnalyzer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAi {
    platform_info: ModelPlatform,
    proxy: ProxyInfo,
    name: &'static str,
}

impl OpenAi {
    pub async fn new() -> Self {
        let name = ModelPlatformName::OpenAi.text();
        let (platform, proxy) = init_service(name, None).await;
        return OpenAi {
            platform_info: platform,
            proxy,
            name,
        };
    }

    pub fn create_by(platform_info: ModelPlatform, proxy: ProxyInfo) -> Self {
        return OpenAi {
            platform_info,
            proxy,
            name: ModelPlatformName::OpenAi.text(),
        };
    }
}

impl WithPlatformConfig for OpenAi {
    fn platform(&self) -> &ModelPlatform {
        &self.platform_info
    }
    fn proxy(&self) -> &ProxyInfo {
        &self.proxy
    }
}

impl ChatCapable for OpenAi {}

impl ImageAnalyzer for OpenAi {
    fn is_stream(&self) -> bool {
        false
    }
}

impl AudioAnalyzer for OpenAi {}
