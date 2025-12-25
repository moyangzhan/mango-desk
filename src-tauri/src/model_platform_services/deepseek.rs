use crate::entities::{AiModel, ModelPlatform};
use crate::enums::ModelPlatformName;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::chat_capable::ChatCapable;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::types::LlmStreaming;
use crate::utils::llm_client_util::{create_client, init_service};
use futures::StreamExt;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
pub struct DeepSeek {
    platform: ModelPlatform,
    proxy: ProxyInfo,
    name: &'static str,
}

/**
 * DeepSeek LLM Service
 * Documentation: https://api-docs.deepseek.com/
 */
impl DeepSeek {
    pub async fn new() -> Self {
        let name = ModelPlatformName::DeepSeek.text();
        let (platform, proxy) = init_service(name, None).await;
        return DeepSeek {
            platform,
            proxy,
            name,
        };
    }

    /// Deepseek's api response : choices[0]["message"]["content"] or choices[0]["message"]["reasoning_content"]
    /// Openai's api response : choices[0]["delta"]["content"]
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
                "stream": true
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
                        let message = choices[0]["message"].clone();
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

impl WithPlatformConfig for DeepSeek {
    fn platform(&self) -> &ModelPlatform {
        &self.platform
    }
    fn proxy(&self) -> &ProxyInfo {
        &self.proxy
    }
}

impl ChatCapable for DeepSeek {}
