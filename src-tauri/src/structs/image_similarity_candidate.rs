/// Candidate for image similarity search with pre-computed scores
/// 图片相似性搜索的候选项（包含预计算的分数）
#[derive(Debug)]
pub struct ImageSimilarityCandidate {
    pub file_id: i64,
    pub semantic_score: usize,
    pub image_hash: Option<Vec<u8>>,
}
