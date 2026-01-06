use crate::entities::{AiModel, FileInfo};
use crate::enums::{FileCategory, ModelPlatformName, ModelType};
use crate::errors::AppError;
use crate::global::ACTIVE_MODEL_PLATFORM;
use crate::model_platform_services::dashscope::DashScope;
use crate::model_platform_services::openai::OpenAi;
use crate::model_platform_services::openai_compatible_service::OpenAiCompatibleService;
use crate::model_platform_services::siliconflow::SiliconFlow;
use crate::repositories::ai_model_repo;
use crate::traits::image_analyzer::ImageAnalyzer;
use crate::traits::indexing_template::IndexingTemplate;

pub struct ImageIndexer {
    category: FileCategory,
    ai_model: AiModel,
    platform_service: Box<dyn ImageAnalyzer>,
}

impl ImageIndexer {
    pub async fn new() -> Result<ImageIndexer, AppError> {
        let (platform_name, base_url) = {
            let active_platform = ACTIVE_MODEL_PLATFORM.read().await;
            (
                active_platform.name.clone(),
                active_platform.base_url.clone(),
            )
        };
        if let Ok(Some(ai_model)) =
            ai_model_repo::get_one_by_type(platform_name.as_str(), ModelType::Vision.into())
        {
            let platform_service: Box<dyn ImageAnalyzer> =
                match ModelPlatformName::from(platform_name.as_str()) {
                    ModelPlatformName::OpenAi => Box::new(OpenAi::new().await),
                    ModelPlatformName::SiliconFlow => Box::new(SiliconFlow::new().await),
                    ModelPlatformName::DashScope => Box::new(DashScope::new().await),
                    ModelPlatformName::DeepSeek => {
                        println!("DeepSeek do not support image analysis yet.");
                        return Err(AppError::UnsupportedImageAnalyze(
                            "Deepseek model platforms".to_string(),
                        ));
                    }
                    _ => Box::new(OpenAiCompatibleService::new(&platform_name, &base_url).await),
                };

            return Ok(Self {
                category: FileCategory::Image,
                ai_model,
                platform_service,
            });
        }
        let vision: &str = ModelType::Vision.into();
        Err(AppError::AiModelNotFound(format!("model type:{}", vision)))
    }
}

impl IndexingTemplate for ImageIndexer {
    fn category(&self) -> &FileCategory {
        &self.category
    }
    async fn load_content(&self, file_info: &FileInfo) -> String {
        match self
            .platform_service
            .analyze_image(&self.ai_model, &file_info.path)
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
