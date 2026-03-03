use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SparseVector {
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

impl Default for SparseVector {
    fn default() -> Self {
        SparseVector {
            indices: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl SparseVector {
    /// 将模型输出的 HashMap 转换为有序存储结构
    pub fn from_map(map: HashMap<u32, f32>) -> Self {
        let mut pairs: Vec<(u32, f32)> = map.into_iter().collect();
        // 关键：必须按索引升序排列，才能支持搜索时的双指针 O(n) 计算
        pairs.sort_by_key(|&(id, _)| id);

        let (indices, values) = pairs.into_iter().unzip();
        Self { indices, values }
    }

    /// 转换为存入 SQLite 的 BLOB
    pub fn to_blob(&self) -> Vec<u8> {
        bincode::serialize(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize sparse vector: {}", e))
            .unwrap_or_default()
    }

    /// 从 SQLite BLOB 解析
    pub fn from_blob(blob: &[u8]) -> Self {
        bincode::deserialize(blob).unwrap_or_else(|_| SparseVector {
            indices: vec![],
            values: vec![],
        })
    }

    /// 计算两个有序稀疏向量的内积 (极速版)
    pub fn dot_product(&self, query_indices: &[u32], query_values: &[f32]) -> f32 {
        let mut score = 0.0;
        let mut i = 0; // Doc 指针
        let mut j = 0; // Query 指针

        let doc_indices = &self.indices;
        let doc_values = &self.values;

        while i < doc_indices.len() && j < query_indices.len() {
            if doc_indices[i] == query_indices[j] {
                score += doc_values[i] * query_values[j];
                i += 1;
                j += 1;
            } else if doc_indices[i] < query_indices[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        // 归一化到 0-1 范围
        let query_norm = query_values.iter().map(|x| x * x).sum::<f32>().sqrt() + 1e-8;
        let doc_norm = self.values.iter().map(|x| x * x).sum::<f32>().sqrt() + 1e-8;
        (score / (query_norm * doc_norm)).min(1.0).max(0.0)
    }
}
