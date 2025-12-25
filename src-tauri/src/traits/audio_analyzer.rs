use crate::entities::AiModel;
use crate::errors::AppError;
use crate::traits::with_platform_config::WithPlatformConfig;
use crate::utils::audio_util::is_supported_audio_file;
use crate::utils::llm_client_util::create_client;
use async_openai::types::{AudioResponseFormat, CreateTranscriptionRequestArgs};
use async_trait::async_trait;
use rusqlite::Result;

#[async_trait]
pub trait AudioAnalyzer: WithPlatformConfig + Send + Sync {
    /// Analyze audio file and convert it to text using speech recognition model.
    ///
    /// # Arguments
    /// * `ai_model` - The AI model for audio processing
    /// * `audio_path` - Path to the audio file to be analyzed
    ///
    /// # Returns
    /// * `Result<String, Box<dyn Error>>` - The transcribed text or an error if:
    ///   - The audio format is not supported
    ///   - The transcription request fails
    ///   - The file cannot be accessed
    ///
    /// # Example
    /// ```rust
    /// let result = analyzer.audio_analyze(&model, "path/to/audio.mp3").await?;
    /// println!("Transcribed text: {}", result);
    /// ```
    async fn analyze_audio(
        &self,
        ai_model: &AiModel,
        audio_path: &str,
    ) -> Result<String, AppError> {
        if !is_supported_audio_file(audio_path)? {
            return Err(AppError::UnsupportedAudioFormat(audio_path.to_string()));
        }
        let client = create_client(self.platform(), self.proxy())?;
        let request = CreateTranscriptionRequestArgs::default()
            .file(audio_path)
            .model(ai_model.name.clone())
            .response_format(AudioResponseFormat::Json)
            .build()?;

        let response = client.audio().transcribe(request).await?;
        println!("{}", response.text);
        Ok(response.text)
    }
}
