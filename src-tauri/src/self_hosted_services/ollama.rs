use crate::entities::SelfHostedPlatform;
use crate::errors::AppError;
use crate::repositories::self_hosted_platform_repo;
use crate::traits::self_hosted_audio_analyzer::SelfHostedAudioAnalyzer;
use crate::traits::self_hosted_chat_capable::SelfHostedChatCapable;
use crate::traits::self_hosted_image_analyzer::SelfHostedImageAnalyzer;
use crate::traits::with_self_hosted_config::WithSelfHostedConfig;
use async_trait::async_trait;
use rusqlite::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Ollama {
    platform: SelfHostedPlatform,
    name: &'static str,
}

/**
 * Ollama Self-hosted LLM Service
 * Documentation: https://github.com/ollama/ollama/blob/main/docs/api.md
 * Ollama API is compatible with OpenAI API format
 */
impl Ollama {
    pub async fn new() -> Result<Self, AppError> {
        let name = "ollama";
        let platform = self_hosted_platform_repo::get_one(name).map_err(|e| {
            AppError::InternalError(format!("Failed to get Ollama platform config: {}", e))
        })?;
        Ok(Ollama { platform, name })
    }
}

impl WithSelfHostedConfig for Ollama {
    fn platform(&self) -> &SelfHostedPlatform {
        &self.platform
    }
}

#[async_trait]
impl SelfHostedChatCapable for Ollama {}

#[async_trait]
impl SelfHostedImageAnalyzer for Ollama {
    fn is_stream(&self) -> bool {
        false
    }
}

#[async_trait]
impl SelfHostedAudioAnalyzer for Ollama {}
