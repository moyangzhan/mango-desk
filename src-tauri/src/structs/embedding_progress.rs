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

impl EmbeddingProgress {
    pub fn failed_incr(&mut self) {
        self.failed += 1;
    }
    pub fn success_incr(&mut self) {
        self.success += 1;
    }
    pub fn processed_incr(&mut self) {
        self.processed += 1;
    }
    pub fn skipped_incr(&mut self) {
        self.skipped += 1;
    }
    pub fn duration_incr(&mut self, ms: i64) {
        self.duration += ms;
    }
}
