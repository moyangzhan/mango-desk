use crate::entities::{AiModel, FileInfo};
use crate::enums::{FileCategory, ModelPlatformName, ModelType};
use crate::errors::AppError;
use crate::global::ACTIVE_MODEL_PLATFORM;
use crate::model_platform_services::openai::OpenAi;
use crate::model_platform_services::openai_compatible_service::OpenAiCompatibleService;
use crate::model_platform_services::siliconflow::SiliconFlow;
use crate::repositories::ai_model_repo;
use crate::traits::audio_analyzer::AudioAnalyzer;
use crate::traits::indexing_template::IndexingTemplate;

pub struct AudioIndexer {
    category: FileCategory,
    ai_model: AiModel,
    model_platform_service: Box<dyn AudioAnalyzer>,
}

impl<'a> AudioIndexer {
    pub async fn new() -> Result<AudioIndexer, AppError> {
        let (platform_name, base_url) = {
            let active_platform = ACTIVE_MODEL_PLATFORM.read().await;
            (
                active_platform.name.clone(),
                active_platform.base_url.clone(),
            )
        };
        if let Ok(Some(ai_model)) =
            ai_model_repo::get_one_by_type(platform_name.as_str(), ModelType::Asr.into())
        {
            let platform_service: Box<dyn AudioAnalyzer> =
                match ModelPlatformName::from(platform_name.as_str()) {
                    ModelPlatformName::OpenAi => Box::new(OpenAi::new().await),
                    ModelPlatformName::SiliconFlow => Box::new(SiliconFlow::new().await),
                    ModelPlatformName::DashScope | ModelPlatformName::DeepSeek => {
                        println!("DeepSeek and DashScope do not support audio analysis yet.");
                        return Err(AppError::UnsupportedAudioAnalyze(
                            "Deepseek and Dashscope".to_string(),
                        ));
                    }
                    _ => Box::new(OpenAiCompatibleService::new(&platform_name, &base_url).await),
                };

            return Ok(Self {
                category: FileCategory::Audio,
                ai_model,
                model_platform_service: platform_service,
            });
        }
        Err(AppError::AiModelNotFound(format!(
            "platform name:{}, model type:{}",
            platform_name,
            <&'static str>::from(ModelType::Vision)
        )))
    }
}

impl IndexingTemplate for AudioIndexer {
    fn category(&self) -> &FileCategory {
        &self.category
    }

    async fn load_content(&self, file_info: &FileInfo) -> String {
        match self
            .model_platform_service
            .analyze_audio(&self.ai_model, &file_info.path)
            .await
        {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error analyzing image: {}", e);
                String::new()
            }
        }
    }
}
