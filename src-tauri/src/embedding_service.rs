use crate::enums::FileContentLanguage;
use crate::utils::app_util::{
    get_english_embedding_path, get_english_tokenizer_path, get_multilingual_embedding_path,
    get_multilingual_tokenizer_path,
};
use crate::{errors::AppError, global::INDEXER_SETTING};
use log::{error, info};
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::Value,
};
use std::path::Path;
use std::sync::Mutex;
use tokenizers::Tokenizer;

struct ThreadSafeSession {
    session: Mutex<ort::session::Session>,
}

pub struct EmbeddingService {
    session: ThreadSafeSession,
    pub tokenizer: Tokenizer,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self, AppError> {
        info!("Initializing embedding service...");
        let mut model_path = get_english_embedding_path();
        let mut tokenizer_path = get_english_tokenizer_path();
        let multilingual_embedding_path = get_multilingual_embedding_path();

        let multilingual_model = Path::new(&multilingual_embedding_path);
        let content_language: FileContentLanguage =
            { INDEXER_SETTING.read().await.file_content_language.clone() };

        if content_language == FileContentLanguage::Multilingual && multilingual_model.exists() {
            model_path = multilingual_embedding_path;
            tokenizer_path = get_multilingual_tokenizer_path();
        }
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(2).max(2))
            .unwrap_or(2);
        info!("using {} threads", logical_cores);
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_intra_threads(logical_cores)?
            .commit_from_file(model_path)
            .map_err(|e| {
                error!("Failed to load model: {:?}", e);
                e
            })?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;
        info!("EmbeddingService::new succeeded");
        Ok(EmbeddingService {
            session: ThreadSafeSession {
                session: Mutex::new(session),
            },
            tokenizer,
        })
    }

    pub async fn model_name() -> &'static str {
        let content_language: FileContentLanguage =
            { INDEXER_SETTING.read().await.file_content_language.clone() };
        if content_language != FileContentLanguage::English {
            return "paraphrase-multilingual-MiniLM-L12-v2";
        } else {
            return "all-minilm-l6-v2";
        }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let encoding = self.tokenizer.encode(text, true)?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&id| id as i64)
            .collect();
        let token_type_ids = vec![0i64; input_ids.len()];

        let input_tensor = Value::from_array(ndarray::Array::from_shape_vec(
            (1, input_ids.len()),
            input_ids,
        )?)?;
        let attention_tensor = Value::from_array(ndarray::Array::from_shape_vec(
            (1, attention_mask.len()),
            attention_mask,
        )?)?;
        let token_type_tensor = Value::from_array(ndarray::Array::from_shape_vec(
            (1, token_type_ids.len()),
            token_type_ids,
        )?)?;
        let embedding: Vec<f32> = {
            let mut guard = self.session.session.lock().map_err(|err| {
                error!("Failed to lock session: {}", err);
                AppError::EmbeddingSizeMismatch(err.to_string())
            })?;
            // let input_names: Vec<String> = (*guard)
            //     .inputs
            //     .iter()
            //     .map(|input| input.name.clone())
            //     .collect();
            // for name in input_names {
            //     println!("Model input: {}", name);
            // }
            let outputs = (*guard).run(ort::inputs![
                input_tensor,
                attention_tensor,
                token_type_tensor
            ])?;
            let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
            // tensor.1.iter().cloned().collect()
            match shape
                .iter()
                .map(|&x| x as usize)
                .collect::<Vec<_>>()
                .as_slice()
            {
                &[1, seq_len, embed_dim] => {
                    // Shape [1, 128, 384]
                    // take the last sequence
                    let start = ((seq_len - 1) * embed_dim) as usize;
                    let end = (seq_len * embed_dim) as usize;
                    data[start..end].to_vec()
                }
                &[1, embed_dim] => {
                    // Shape [1, 384]
                    data[..embed_dim].to_vec()
                }
                &[embed_dim] => {
                    // Shape [384]
                    data[..embed_dim].to_vec()
                }
                _ => {
                    return Err(AppError::EmbeddingSizeMismatch(format!(
                        "Unexpected output shape: {:?}",
                        shape
                    )));
                }
            }
        };
        Ok(embedding)
    }
}

impl Drop for EmbeddingService {
    fn drop(&mut self) {
        info!("Embedding Service is dropped");
    }
}
