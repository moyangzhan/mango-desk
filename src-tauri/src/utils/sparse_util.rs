use crate::structs::sparse_vector::SparseVector;
use std::collections::HashMap;
use tokenizers::Tokenizer;

pub fn prepare_sparse_for_storage(weights: HashMap<String, f32>, tokenizer: &Tokenizer) -> Vec<u8> {
    let mut id_weights = HashMap::new();

    for (token, weight) in weights {
        // 将 Token 字符串转回 ID (BGE-M3 的 Sparse 实际上是基于词表的权重)
        if let Some(id) = tokenizer.token_to_id(&token) {
            id_weights.insert(id, weight);
        }
    }

    let sv = SparseVector::from_map(id_weights);
    sv.to_blob()
}
