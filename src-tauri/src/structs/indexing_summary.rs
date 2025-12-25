use crate::{enums::FileCategory, structs::embedding_progress::EmbeddingProgress};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Indexing files = Embedding files + other files( video/binary etc.)
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct IndexingSummary {
    pub task_id: i64,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    // Indexing time in seconds = Files scanning duration + document embedding duration + image embedding duration + audio embedding duration
    pub duration: i64,
    pub total: i64, // Total files count
    pub document: EmbeddingProgress,
    pub image: EmbeddingProgress,
    pub audio: EmbeddingProgress,
}

impl IndexingSummary {
    pub fn calculate_all_embedding(&self) -> EmbeddingProgress {
        EmbeddingProgress {
            total: self.document.total + self.image.total + self.audio.total,
            processed: self.document.processed + self.image.processed + self.audio.processed,
            success: self.document.success + self.image.success + self.audio.success,
            failed: self.document.failed + self.image.failed + self.audio.failed,
            skipped: self.document.skipped + self.image.skipped + self.audio.skipped,
            duration: self.document.duration + self.image.duration + self.audio.duration,
        }
    }

    pub fn get_embedding_progress(&mut self, category: &FileCategory) -> &mut EmbeddingProgress {
        match category {
            FileCategory::Document => &mut self.document,
            FileCategory::Image => &mut self.image,
            FileCategory::Audio => &mut self.audio,
            _ => {
                println!("Unknown support file category");
                &mut self.document
            }
        }
    }
}
