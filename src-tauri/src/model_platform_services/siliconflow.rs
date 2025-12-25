use crate::entities::{AiModel, ModelPlatform};
use crate::enums::ModelPlatformName;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::audio_analyzer::AudioAnalyzer;
use crate::traits::chat_capable::ChatCapable;
use crate::traits::image_analyzer::ImageAnalyzer;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::types::LlmStreaming;
use crate::utils::llm_client_util::{create_client, init_service};
use futures::StreamExt;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
pub struct SiliconFlow {
    platform_info: ModelPlatform,
    proxy: ProxyInfo,
    name: &'static str,
}

/**
 * SiliconFlow LLM Service
 * Documentation: https://docs.siliconflow.com/
 */
impl SiliconFlow {
    pub async fn new() -> Self {
        let name = ModelPlatformName::SiliconFlow.text();
        let (platform_info, proxy) = init_service(name, None).await;
        return SiliconFlow {
            platform_info,
            proxy,
            name,
        };
    }
    /// SiliconFlow's api response : choices[0]["delta"]["content"] or choices[0]["delta"]["reasoning_content"]
    /// Openai's api response :      choices[0]["delta"]["content"]
    pub async fn chat_stream_with_reasoning<F>(
        &self,
        ai_model: &AiModel,
        prompt: &str,
        reasoning_callback: &F,
        callback: &F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        let client = create_client(&self.platform_info, &self.proxy)?;
        let mut stream: LlmStreaming = client
            .chat()
            .create_stream_byot(json!({
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "model": ai_model.name.clone(),
                "enable_thinking": ai_model.is_reasoner,  // SiliconFlow's parameter for reasoning and openai don't have it
                "stream": true,
                "stream_options": {
                    "include_usage": true
                }
            }))
            .await?;
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    let choices = match chunk["choices"].as_array() {
                        Some(choices) => choices,
                        None => continue,
                    };
                    if choices.len() > 0 {
                        let message = choices[0]["delta"].clone();

                        if let Some(content) = message["content"].as_str() {
                            if let Err(e) = callback(content) {
                                eprintln!("Callback error: {}", e);
                            }
                        } else if let Some(reaoning_content) = message["reasoning_content"].as_str()
                        {
                            if let Err(e) = reasoning_callback(reaoning_content) {
                                eprintln!("Callback error: {}", e);
                            }
                        }
                    } else if !chunk["usage"].is_null() {
                        let usage = chunk["usage"].clone();
                        println!("\nUsage:");
                        println!("{}", usage.to_string());
                    } else {
                        println!("Unknown chunk: {}", chunk.to_string());
                    }
                }
                Err(err) => {
                    eprintln!("DeepSeek chat error: {}", err);
                    return Err(Box::new(err));
                }
            }
        }
        Ok(())
    }
}

impl WithPlatformConfig for SiliconFlow {
    fn platform(&self) -> &ModelPlatform {
        &self.platform_info
    }
    fn proxy(&self) -> &ProxyInfo {
        &self.proxy
    }
}

impl ChatCapable for SiliconFlow {}

impl ImageAnalyzer for SiliconFlow {
    fn is_stream(&self) -> bool {
        false
    }
}

impl AudioAnalyzer for SiliconFlow {}
