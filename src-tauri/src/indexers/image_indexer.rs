use crate::entities::{AiModel, FileInfo};
use crate::enums::{FileCategory, FileParserMode, ModelPlatformName, ModelType};
use crate::errors::AppError;
use crate::global::{ACTIVE_MODEL_PLATFORM, INDEXER_SETTING};
use crate::image_parser::ImageParser;
use crate::model_platform_services::dashscope::DashScope;
use crate::model_platform_services::openai::OpenAi;
use crate::model_platform_services::openai_compatible_service::OpenAiCompatibleService;
use crate::model_platform_services::siliconflow::SiliconFlow;
use crate::repositories::ai_model_repo;
use crate::traits::image_analyzer::ImageAnalyzer;
use crate::traits::indexing_template::IndexingTemplate;

pub struct ImageIndexer {
    category: FileCategory,
    ai_model: Option<AiModel>,
    platform_service: Option<Box<dyn ImageAnalyzer>>,
    local_parser: Option<ImageParser>,
}

impl ImageIndexer {
    pub async fn new() -> Result<ImageIndexer, AppError> {
        let mut platform_service_option: Option<Box<dyn ImageAnalyzer>> = None;
        let mut local_parser: Option<ImageParser> = None;
        let mut ai_model_option: Option<AiModel> = None;
        if INDEXER_SETTING.read().await.image_parser_mode == FileParserMode::Local {
            local_parser = Some(ImageParser::new().map_err(|e| {
                AppError::ImageParserInitError(format!(
                    "Failed to initialize local image parser: {:?}",
                    e
                ))
            })?);
        } else {
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
                        _ => {
                            Box::new(OpenAiCompatibleService::new(&platform_name, &base_url).await)
                        }
                    };
                platform_service_option = Some(platform_service);
                ai_model_option = Some(ai_model);
            }
        }
        if platform_service_option.is_some() || local_parser.is_some() {
            return Ok(Self {
                category: FileCategory::Image,
                ai_model: ai_model_option,
                platform_service: platform_service_option,
                local_parser: local_parser,
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
        match &self.local_parser {
            Some(parser) => match parser.generate_caption_from_path(&file_info.path).await {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error parsing image locally: {}", e);
                    String::new()
                }
            },
            None => match &self.platform_service {
                Some(service) => {
                    let model = match &self.ai_model {
                        Some(m) => m,
                        None => {
                            eprintln!("No AI model configured for image analysis");
                            return String::new();
                        }
                    };
                    service
                        .analyze_image(model, &file_info.path)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("Error analyzing image: {}", e);
                            String::new()
                        })
                }
                None => {
                    eprintln!("No image parser available");
                    String::new()
                }
            },
        }
    }
}
