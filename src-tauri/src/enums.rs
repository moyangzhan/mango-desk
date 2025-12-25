use crate::global::{
    SUPPORTED_AUDIO_EXTS, SUPPORTED_IMAGE_EXTS, SUPPORTED_VIDEO_EXTS, supported_doc_exts,
};
use rusqlite::Result as SqlResult;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    EnUs,
    ZhCn,
}

impl Locale {
    pub fn from(value: &str) -> Self {
        match value {
            "en-US" => Locale::EnUs,
            "zh-CN" => Locale::ZhCn,
            _ => Locale::EnUs,
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::ZhCn => "zh-CN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPlatformName {
    OpenAi,
    SiliconFlow,
    DashScope,
    DeepSeek,
    OpenAiCompatable,
}

impl ModelPlatformName {
    pub fn from(value: &str) -> Self {
        match value {
            "openai" => ModelPlatformName::OpenAi,
            "siliconflow" => ModelPlatformName::SiliconFlow,
            "dashscope" => ModelPlatformName::DashScope,
            "deepseek" => ModelPlatformName::DeepSeek,
            "opeai_compatible" => ModelPlatformName::OpenAiCompatable,
            _ => ModelPlatformName::OpenAi,
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            ModelPlatformName::OpenAi => "openai",
            ModelPlatformName::SiliconFlow => "siliconflow",
            ModelPlatformName::DashScope => "dashscope",
            ModelPlatformName::DeepSeek => "deepseek",
            ModelPlatformName::OpenAiCompatable => "opeai_compatible",
        }
    }
}

// Model type: text, image, vision, embedding, rerank, asr, tts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Text,
    Image,
    Vision,
    Embedding,
    Rerank,
    Asr,
    Tts,
}

impl From<&str> for ModelType {
    fn from(s: &str) -> Self {
        match s {
            "text" => ModelType::Text,
            "image" => ModelType::Image,
            "vision" => ModelType::Vision,
            "embedding" => ModelType::Embedding,
            "rerank" => ModelType::Rerank,
            "asr" => ModelType::Asr,
            "tts" => ModelType::Tts,
            _ => ModelType::Text,
        }
    }
}

impl From<ModelType> for &'static str {
    fn from(s: ModelType) -> Self {
        match s {
            ModelType::Text => "text",
            ModelType::Image => "image",
            ModelType::Vision => "vision",
            ModelType::Embedding => "embedding",
            ModelType::Rerank => "rerank",
            ModelType::Asr => "asr",
            ModelType::Tts => "tts",
        }
    }
}

//File index status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileIndexStatus {
    Waiting = 1,
    Indexing = 2,
    Indexed = 3,
    IndexFailed = 4,
}
impl FileIndexStatus {
    pub fn value(self) -> i64 {
        self as i64
    }
}

impl From<i64> for FileIndexStatus {
    fn from(value: i64) -> Self {
        match value {
            1 => FileIndexStatus::Waiting,
            2 => FileIndexStatus::Indexing,
            3 => FileIndexStatus::Indexed,
            4 => FileIndexStatus::IndexFailed,
            _ => FileIndexStatus::Waiting, // Default value
        }
    }
}

// File category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Document = 1,
    Image = 2,
    Audio = 3,
    Video = 4,
    Other = 5,
}

impl FileCategory {
    pub fn value(self) -> i64 {
        self as i64
    }

    pub async fn from_ext(ext: &str) -> Self {
        if ext.is_empty() {
            return FileCategory::Other;
        }
        if supported_doc_exts().await.contains(&ext.to_string()) {
            FileCategory::Document
        } else if SUPPORTED_IMAGE_EXTS.contains(&ext) {
            FileCategory::Image
        } else if SUPPORTED_AUDIO_EXTS.contains(&ext) {
            FileCategory::Audio
        } else if SUPPORTED_VIDEO_EXTS.contains(&ext) {
            FileCategory::Video
        } else {
            FileCategory::Other
        }
    }

    pub fn to_text(self) -> &'static str {
        self.into()
    }

    pub fn value_to_text(value: i64) -> &'static str {
        FileCategory::from(value).into()
    }

    pub fn is_document(&self) -> bool {
        matches!(self, FileCategory::Document)
    }

    pub fn is_media(&self) -> bool {
        matches!(
            self,
            FileCategory::Image | FileCategory::Audio | FileCategory::Video
        )
    }
}

impl From<&str> for FileCategory {
    fn from(s: &str) -> Self {
        match s {
            "document" => FileCategory::Document,
            "image" => FileCategory::Image,
            "audio" => FileCategory::Audio,
            "video" => FileCategory::Video,
            _ => FileCategory::Other,
        }
    }
}

impl From<i64> for FileCategory {
    fn from(value: i64) -> Self {
        match value {
            1 => FileCategory::Document,
            2 => FileCategory::Image,
            3 => FileCategory::Audio,
            4 => FileCategory::Video,
            _ => FileCategory::Other, // Default value
        }
    }
}

impl From<FileCategory> for &'static str {
    fn from(value: FileCategory) -> Self {
        match value {
            FileCategory::Document => "document",
            FileCategory::Image => "image",
            FileCategory::Audio => "audio",
            FileCategory::Video => "video",
            FileCategory::Other => "other",
        }
    }
}

impl Display for FileCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = <&'static str>::from(*self);
        write!(f, "{}", text)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileContentLanguage {
    #[serde(rename = "en")]
    English,
    Multilingual,
}

impl From<&str> for FileContentLanguage {
    fn from(s: &str) -> Self {
        match s {
            "english" => FileContentLanguage::English,
            "multilingual" => FileContentLanguage::Multilingual,
            _ => FileContentLanguage::English,
        }
    }
}

impl From<FileContentLanguage> for &'static str {
    fn from(value: FileContentLanguage) -> Self {
        match value {
            FileContentLanguage::English => "english",
            FileContentLanguage::Multilingual => "multilingual",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexingTaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl From<&str> for IndexingTaskStatus {
    fn from(s: &str) -> Self {
        match s {
            "pending" => IndexingTaskStatus::Pending,
            "running" => IndexingTaskStatus::Running,
            "paused" => IndexingTaskStatus::Paused,
            "completed" => IndexingTaskStatus::Completed,
            "failed" => IndexingTaskStatus::Failed,
            "cancelled" => IndexingTaskStatus::Cancelled,
            _ => IndexingTaskStatus::Pending,
        }
    }
}

impl From<IndexingTaskStatus> for &'static str {
    fn from(status: IndexingTaskStatus) -> Self {
        match status {
            IndexingTaskStatus::Pending => "pending",
            IndexingTaskStatus::Running => "running",
            IndexingTaskStatus::Paused => "paused",
            IndexingTaskStatus::Completed => "completed",
            IndexingTaskStatus::Failed => "failed",
            IndexingTaskStatus::Cancelled => "cancelled",
        }
    }
}

impl ToSql for IndexingTaskStatus {
    fn to_sql(&self) -> SqlResult<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(<&'static str>::from(*self)))
    }
}

impl FromSql for IndexingTaskStatus {
    fn column_result(value: rusqlite::types::ValueRef) -> SqlResult<Self, FromSqlError> {
        let value = value.as_str()?;
        let status = IndexingTaskStatus::from(value);
        Ok(status)
    }
}

impl Display for IndexingTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = <&'static str>::from(*self);
        write!(f, "{}", text)
    }
}

// Communication events with front-end

#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum DownloadEvent {
    Start { url: String, download_id: String },
    Progress { download_id: String, progress: f64 },
    Finish { download_id: String },
    Error { download_id: String, error: String },
}

#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum IndexingEvent {
    Start { task_id: i64, msg: String },
    Scan { task_id: i64, msg: String },
    Stop { task_id: i64, msg: String },
    Embed { task_id: i64, msg: String },
    Finish { task_id: i64, msg: String },
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum CommandResultCode {
    ERROR,
    SUCCESS,
    INDEXING,
}

pub enum TrayMenuItem {
    Show,
    Quit,
}
impl Display for TrayMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayMenuItem::Show => write!(f, "show"),
            TrayMenuItem::Quit => write!(f, "quit"),
        }
    }
}

impl From<TrayMenuItem> for &'static str {
    fn from(value: TrayMenuItem) -> Self {
        match value {
            TrayMenuItem::Show => "show",
            TrayMenuItem::Quit => "quit",
        }
    }
}
