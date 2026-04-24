use crate::entities::AiModel;
use crate::errors::AppError;
use crate::traits::with_self_hosted_config::WithSelfHostedConfig;
use async_trait::async_trait;

#[async_trait]
pub trait SelfHostedAudioAnalyzer: WithSelfHostedConfig + Send + Sync {
    /// Self-hosted platforms (Ollama, vLLM) do not support audio transcription (ASR).
    /// This method always returns an error indicating the feature is not supported.
    ///
    /// # Arguments
    /// * `_ai_model` - The AI model (unused)
    /// * `_audio_path` - Path to the audio file (unused)
    ///
    /// # Returns
    /// * `Result<String, AppError>` - Always returns UnsupportedAudioAnalyze error
    async fn analyze_audio(
        &self,
        _ai_model: &AiModel,
        _audio_path: &str,
    ) -> Result<String, AppError> {
        Err(AppError::UnsupportedAudioAnalyze(
            "Self-hosted platforms (Ollama, vLLM) do not support audio transcription".to_string(),
        ))
    }
}
