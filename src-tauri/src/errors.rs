use crate::repositories::RepositoryError;
use std::{string::FromUtf8Error, sync::PoisonError};
use text_splitter::ChunkConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("An unknown error occurred")]
    Unknown,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Failed to calculate MD5: {0}")]
    CalculateMd5Error(String),
    #[error("Embedding size mismatch: expected {0}")]
    EmbeddingSizeMismatch(String),
    #[error("Failed to parse datetime: {0}")]
    DateTimeParseError(String),
    #[error("Document splitting failed: {0}")]
    DocumentSplitterError(String),
    #[error("Failed to acquire read lock: {0}")]
    RwLockReadError(String),
    #[error("Failed to acquire write lock: {0}")]
    RwLockWriteError(String),
    #[error("Image analysis failed: {0}")]
    AnalyzeImageError(String),
    #[error("Audio analysis failed: {0}")]
    AnalyzeAudioError(String),
    #[error("Video analysis failed: {0}")]
    AnalyzeVideoError(String),
    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),
    #[error("Model service initialization failed: {0}")]
    ModelServiceInitError(String),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid path: {0}")]
    UnsupportedPath(String),
    #[error("Path does not exist: {0}")]
    PathNotExist(String),
    #[error("Invalid file: {0}")]
    FileIsInvalid(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("OpenAI API error: {0}")]
    OpenAiError(#[from] async_openai::error::OpenAIError),
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Model platform not found: {0}")]
    ModelPlatformNotFound(String),
    #[error("Image analysis not supported by: {0}")]
    UnsupportedImageAnalyze(String),
    #[error("Audio analysis not supported by: {0}")]
    UnsupportedAudioAnalyze(String),
    #[error("Video analysis not supported by: {0}")]
    UnsupportedVideoAnalyze(String),
    #[error("AI model not found: {0}")]
    AiModelNotFound(String),
    #[error("Unsupported audio format: {0}")]
    UnsupportedAudioFormat(String),
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("Instant initialization failed: {0}")]
    InstantNewFailed(String),
    #[error("Instant parse failed: {source}")]
    OrtError {
        #[from]
        source: ort::Error,
    },
    #[error("Dynamic error failed: {source}")]
    DynError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Shape error: {source}")]
    ShapeError {
        #[from]
        source: ndarray::ShapeError,
    },
}
unsafe impl Send for AppError {}
unsafe impl Sync for AppError {}

#[derive(Error, Debug)]
pub enum ToFrontendError {
    #[error("backend common error:{0}")]
    Common(String),
}

#[derive(Error, Debug)]
pub enum PptParserError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Slide not found: {0}")]
    SlideNotFound(usize),

    #[error("Invalid slide range: {0}")]
    InvalidSlideRange(String),

    #[error("Empty presentation")]
    EmptyPresentation,

    #[error("ZIP extraction error: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Invalid XML structure: {0}")]
    InvalidXmlStructure(String),

    #[error("Presentation parsing error: {0}")]
    ParsingError(String),

    #[error("Metadata extraction error: {0}")]
    MetadataError(String),

    #[error("Content extraction error: {0}")]
    ContentError(String),
}

impl PptParserError {
    pub fn parsing_error(msg: impl Into<String>) -> Self {
        Self::ParsingError(msg.into())
    }

    pub fn metadata_error(msg: impl Into<String>) -> Self {
        Self::MetadataError(msg.into())
    }

    pub fn content_error(msg: impl Into<String>) -> Self {
        Self::ContentError(msg.into())
    }

    pub fn invalid_xml(msg: impl Into<String>) -> Self {
        Self::InvalidXmlStructure(msg.into())
    }
}

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Error loading document: {0}")]
    LoadDocumentError(String),

    #[error("{0}")]
    TextSplitterError(#[from] DocumentSplitterError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    CSVError(#[from] csv::Error),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Error: {0}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum DocumentSplitterError {
    #[error("Empty input text")]
    EmptyInputText,

    #[error("Mismatch metadata and text")]
    MetadataTextMismatch,

    #[error("Tokenizer not found")]
    TokenizerNotFound,

    #[error("Tokenizer creation failed due to invalid tokenizer")]
    InvalidTokenizer,

    #[error("Tokenizer creation failed due to invalid model")]
    InvalidModel,

    #[error("Invalid chunk overlap and size")]
    InvalidSplitterOptions,

    #[error("Error: {0}")]
    OtherError(String),
}

#[derive(Error, Debug)]
pub enum IndexingError {
    #[error("No content for indexing")]
    EmptyContent,
    #[error("No paths provided for indexing")]
    EmptyPaths,
    #[error("Scanning already in progress")]
    ScanningInProgress,
    #[error("Indexing already in progress")]
    IndexingInProgress,
    #[error("Model platform '{0}' is disabled")]
    PlatformDisabled(String),
    #[error("Failed to create embedding service: {0}")]
    EmbeddingServiceError(#[from] AppError),
    #[error("Failed to process file: {path}")]
    FileProcessing { path: String },
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Dynamic error failed: {source}")]
    DynError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("Database operation failed: {source}")]
    DatabaseError {
        #[from]
        source: rusqlite::Error,
    },
    #[error("Repository operation failed: {source}")]
    RepositoryError {
        #[from]
        source: RepositoryError,
    },
    #[error("Invalid file extension: {ext}")]
    InvalidExtension { ext: String },
    #[error("Send error: {source}")]
    SendError {
        #[from]
        source: tokio::sync::mpsc::error::SendError<String>,
    },
    #[error("Try send error: {source}")]
    TrySendError {
        #[from]
        source: tokio::sync::mpsc::error::TrySendError<String>,
    },
}

impl From<ChunkConfigError> for DocumentSplitterError {
    fn from(_: ChunkConfigError) -> Self {
        Self::InvalidSplitterOptions
    }
}

impl<T> From<PoisonError<std::sync::RwLockWriteGuard<'_, T>>> for AppError {
    fn from(value: PoisonError<std::sync::RwLockWriteGuard<'_, T>>) -> Self {
        Self::RwLockWriteError(value.to_string())
    }
}

impl<T> From<PoisonError<std::sync::RwLockReadGuard<'_, T>>> for AppError {
    fn from(value: PoisonError<std::sync::RwLockReadGuard<'_, T>>) -> Self {
        Self::RwLockReadError(value.to_string())
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for AppError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::SendError("Failed to send message".to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<AppError> for ToFrontendError {
    fn from(f: AppError) -> Self {
        ToFrontendError::Common(f.to_string())
    }
}

impl From<AppError> for String {
    fn from(f: AppError) -> Self {
        f.to_string()
    }
}

impl From<RepositoryError> for String {
    fn from(r: RepositoryError) -> Self {
        r.to_string()
    }
}
