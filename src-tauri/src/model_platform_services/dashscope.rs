use crate::entities::{AiModel, ModelPlatform};
use crate::enums::ModelPlatformName;
use crate::structs::proxy_setting::ProxyInfo;
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
pub struct DashScope {
    platform: ModelPlatform,
    proxy: ProxyInfo,
    name: &'static str,
}

/**
 * DashScope LLM Service
 * Documentation:
 * https://www.alibabacloud.com/help/zh/model-studio/model-api-reference
 * https://www.alibabacloud.com/help/en/model-studio/model-api-reference
 */
impl DashScope {
    pub async fn new() -> Self {
        let name = ModelPlatformName::DashScope.text();
        let (platform, proxy) = init_service(name, None).await;
        return DashScope {
            platform,
            proxy,
            name,
        };
    }

    /// DashScope's api response : choices[0]["delta"]["content"] or choices[0]["delta"]["reasoning_content"]
    /// Openai's api response :    choices[0]["delta"]["content"]
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
        let client = create_client(&self.platform, &self.proxy)?;

        let mut stream: LlmStreaming = client
            .chat()
            .create_stream_byot(json!({
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "model": ai_model.name.clone(),
                "stream": true,
                "enable_thinking": true,   // Dashscope's parameter for reasoning and openai don't have it
                "stream_options": {
                    "include_usage": true
                }
            }))
            .await?;
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    if chunk["choices"].is_array() {
                        let message = chunk["choices"][0]["delta"].clone();
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
                    eprintln!("DashScope chat error: {}", err);
                    return Err(Box::new(err));
                }
            }
        }
        Ok(())
    }
}

impl WithPlatformConfig for DashScope {
    fn platform(&self) -> &ModelPlatform {
        &self.platform
    }
    fn proxy(&self) -> &ProxyInfo {
        &self.proxy
    }
}

impl ChatCapable for DashScope {}

impl ImageAnalyzer for DashScope {
    fn is_stream(&self) -> bool {
        false
    }
}
