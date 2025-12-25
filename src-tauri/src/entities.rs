use crate::enums::{FileIndexStatus, IndexingTaskStatus};
use crate::structs::file_metadata::FileMetadata;
use crate::utils::datetime_util;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub id: i64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelPlatform {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub logo: String,
    pub base_url: String,
    pub api_key: String,
    pub remark: String,
    pub is_proxy_enable: bool,
    pub is_openai_api_compatible: bool,
    #[serde(with = "datetime_util")]
    pub create_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub update_time: DateTime<Local>,
}

impl Default for ModelPlatform {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            title: "".to_string(),
            logo: "".to_string(),
            base_url: "".to_string(),
            api_key: "".to_string(),
            remark: "".to_string(),
            is_proxy_enable: false,
            is_openai_api_compatible: false,
            create_time: Local::now(),
            update_time: Local::now(),
        }
    }
}

impl ModelPlatform {
    pub fn is_enable(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AiModel {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub remark: String,
    pub model_types: String,
    pub setting: String,
    pub platform: String,
    pub context_window: i32,
    pub max_input_tokens: i32,
    pub max_output_tokens: i32,
    pub input_types: String,
    pub properties: String,
    pub is_reasoner: bool,
    pub is_thinking_closable: bool,
    pub is_free: bool,
    pub is_enable: bool,
    #[serde(with = "datetime_util")]
    pub create_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub update_time: DateTime<Local>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileInfo {
    pub id: i64,
    pub name: String,
    pub category: i64,
    pub path: String,
    pub content: String,
    pub metadata: FileMetadata,
    pub file_ext: String,
    pub file_size: i64,
    pub md5: String,
    pub content_index_status: i64,
    pub content_index_status_msg: String,
    pub meta_index_status: i64,
    pub meta_index_status_msg: String,
    pub is_invalid: bool,
    pub invalid_reason: String,
    #[serde(with = "datetime_util")]
    pub file_create_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub file_update_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub create_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub update_time: DateTime<Local>,
}

impl Default for FileInfo {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_string(),
            category: 0,
            path: "".to_string(),
            content: "".to_string(),
            metadata: FileMetadata::default(),
            file_ext: "".to_string(),
            file_size: 0,
            file_create_time: DateTime::default(),
            file_update_time: DateTime::default(),
            md5: "".to_string(),
            is_invalid: false,
            invalid_reason: "".to_string(),
            content_index_status: FileIndexStatus::Waiting.value(),
            content_index_status_msg: "".to_string(),
            meta_index_status: FileIndexStatus::Waiting.value(),
            meta_index_status_msg: "".to_string(),
            create_time: Local::now(),
            update_time: Local::now(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetaEmbedding {
    pub id: i64,
    pub file_id: i64,
    #[serde(skip, default = "default_embedding")]
    pub embedding: [f32; 384],
    pub distance: f32, // for search result
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileContentEmbedding {
    pub id: i64,
    pub file_id: i64,
    pub chunk_index: i64,
    pub chunk_text: String,
    #[serde(skip, default = "default_embedding")]
    pub embedding: [f32; 384],
    pub distance: f32, // for search result
}

impl Default for FileContentEmbedding {
    fn default() -> Self {
        Self {
            id: 0,
            file_id: 0,
            embedding: default_embedding(),
            chunk_index: 0,
            chunk_text: "".to_string(),
            distance: -0.1,
        }
    }
}

fn default_embedding() -> [f32; 384] {
    [0.0; 384]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingTask {
    pub id: i64,
    pub paths: String,
    pub embedding_model: String,
    pub status: IndexingTaskStatus,
    pub start_time: Option<DateTime<Local>>,
    pub end_time: Option<DateTime<Local>>,
    pub duration: i64, // milliseconds
    pub total_cnt: i64,
    pub content_processed_cnt: i64,
    pub content_indexed_success_cnt: i64,
    pub content_indexed_failed_cnt: i64,
    pub content_indexed_skipped_cnt: i64,
    pub remark: String,
    pub config_json: String,
    #[serde(with = "datetime_util")]
    pub create_time: DateTime<Local>,
    #[serde(with = "datetime_util")]
    pub update_time: DateTime<Local>,
}

impl Default for IndexingTask {
    fn default() -> Self {
        Self {
            id: 0,
            paths: "".to_string(),
            embedding_model: "".to_string(),
            status: IndexingTaskStatus::Pending,
            start_time: None,
            end_time: None,
            duration: 0,
            total_cnt: 0,
            content_processed_cnt: 0,
            content_indexed_success_cnt: 0,
            content_indexed_failed_cnt: 0,
            content_indexed_skipped_cnt: 0,
            remark: "".to_string(),
            config_json: "".to_string(),
            create_time: Local::now(),
            update_time: Local::now(),
        }
    }
}
