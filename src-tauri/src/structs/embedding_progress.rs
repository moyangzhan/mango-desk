use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct EmbeddingProgress {
    pub total: i64,
    pub processed: i64,
    pub success: i64,
    pub skipped: i64,
    pub failed: i64,
    //Milliseconds
    pub duration: i64,
}