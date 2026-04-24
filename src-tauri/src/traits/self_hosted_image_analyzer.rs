use crate::entities::AiModel;
use crate::errors::AppError;
use crate::traits::with_self_hosted_config::WithSelfHostedConfig;
use crate::utils::image_util::image_to_data_uri;
use crate::utils::llm_client_util::create_client_for_self_hosted;
use async_openai::types::{
    ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequestArgs,
};
use async_trait::async_trait;
use rust_i18n::t;

#[async_trait]
pub trait SelfHostedImageAnalyzer: WithSelfHostedConfig + Send + Sync {
    fn is_stream(&self) -> bool;

    /// Analyze image and return the result as a string.
    async fn analyze_image(
        &self,
        ai_model: &AiModel,
        image_path: &str,
    ) -> Result<String, AppError> {
        let data_uri = image_to_data_uri(image_path)?;
        let txt_part = ChatCompletionRequestUserMessageContentPart::Text(
            ChatCompletionRequestMessageContentPartTextArgs::default()
                .text(t!("prompt.image-analyze"))
                .build()?,
        );
        let image_part = ChatCompletionRequestUserMessageContentPart::ImageUrl(
            ChatCompletionRequestMessageContentPartImageArgs::default()
                .image_url(data_uri)
                .build()?,
        );

        let client = create_client_for_self_hosted(self.platform())?;
        let request = CreateChatCompletionRequestArgs::default()
            .model(ai_model.name.clone())
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(vec![txt_part, image_part])
                .build()?
                .into()])
            .stream(self.is_stream())
            .build()?;
        let response = client.chat().create(request).await?;

        Ok(response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default())
    }
}
