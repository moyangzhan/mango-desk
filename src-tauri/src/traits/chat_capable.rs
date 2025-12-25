use crate::entities::AiModel;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::utils::llm_client_util::create_client;
use async_openai::types::{
    ChatCompletionRequestUserMessageArgs, ChatCompletionStreamOptions,
    CreateChatCompletionRequestArgs,
};
use futures::stream::StreamExt;
use rusqlite::Result;

pub trait ChatCapable: WithPlatformConfig {
    async fn chat(
        &self,
        ai_model: &AiModel,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = create_client(&self.platform(), &self.proxy())?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(ai_model.name.clone())
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into()])
            .build()?;

        let mut result = String::from("");
        let response = client.chat().create(request).await?;
        for choice in response.choices {
            if let Some(ref content) = choice.message.content {
                result.push_str(content);
            }
        }
        return Ok("".to_string());
    }

    async fn chat_stream<F>(
        &self,
        ai_model: &AiModel,
        prompt: &str,
        callback: &F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(&str) -> Result<(), Box<dyn std::error::Error>>,
    {
        let client = create_client(&self.platform(), &self.proxy())?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(ai_model.name.clone())
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into()])
            .stream(true)
            .stream_options(ChatCompletionStreamOptions {
                include_usage: true,
            })
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if response.choices.len() > 0 {
                        response.choices.iter().for_each(|chat_choice| {
                            if let Some(ref content) = chat_choice.delta.content {
                                if let Err(e) = callback(content) {
                                    eprintln!("Callback error: {}", e);
                                }
                            }
                        });
                    } else if response.usage.is_some() {
                        response.usage.iter().for_each(|usage| {
                            println!("usage: {}", usage.total_tokens);
                        })
                    }
                }
                Err(err) => {
                    eprintln!("chat error: {}", err);
                    return Err(Box::new(err));
                }
            }
        }

        Ok(())
    }
}
