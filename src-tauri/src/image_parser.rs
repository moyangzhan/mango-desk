use crate::utils::app_util::{get_vision_0_path, get_vision_tokenizer_path};
use anyhow::{Ok, Result};
use image::ImageReader;
use log::info;
use ndarray::{Array, Array2, Array4, IxDyn, s};
use ort::{
    session::SessionInputValue,
    session::{Session, builder::GraphOptimizationLevel},
    value::TensorValueType,
    value::Value,
};
use std::path::Path;
use tokenizers::Tokenizer;
use tokio::sync::Mutex as AsyncMutex;

pub struct ImageParser {
    vision_session: AsyncMutex<ort::session::Session>,
    tokenizer: Tokenizer,
}

impl ImageParser {
    pub fn new() -> Result<Self> {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(2).max(2))
            .unwrap_or(2);
        info!("vision model using {} threads", logical_cores);
        let vision_session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(logical_cores)?
            .commit_from_file(get_vision_0_path())?;
        println!("Successfully loaded vision and decoder models.");
        Ok(ImageParser {
            vision_session: AsyncMutex::new(vision_session),
            tokenizer: Tokenizer::from_file(get_vision_tokenizer_path())
                .map_err(|e| anyhow::anyhow!("Failed to load vision tokenizer: {}", e))?,
        })
    }

    pub async fn generate_caption_from_path(
        &self,
        image_path: &str, // Changed from Array4 to &str
    ) -> Result<String> {
        if !Path::new(image_path).exists() {
            return Err(anyhow::anyhow!("Image file not found: {}", image_path));
        }

        let ext = std::path::Path::new(image_path)
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file extension for: {}", image_path))?;

        let img = ImageReader::open(image_path)
            .map_err(|e| anyhow::anyhow!("Failed to open image file: {}", e))?
            .with_guessed_format()
            .map_err(|e| {
                let ext = ext.to_lowercase();
                match ext.as_str() {
                    "png" => anyhow::anyhow!(
                        "Invalid PNG file: {}. Please verify the file is not corrupted.",
                        e
                    ),
                    "jpg" | "jpeg" => anyhow::anyhow!(
                        "Invalid JPEG file: {}. Please verify the file is not corrupted.",
                        e
                    ),
                    _ => anyhow::anyhow!("Invalid image file: {}. Supported formats: PNG, JPEG", e),
                }
            })?
            .decode()
            .map_err(|e| {
                let ext = ext.to_lowercase();
                match ext.as_str() {
                    "png" => anyhow::anyhow!(
                        "Invalid PNG file: {}. Please verify the file is not corrupted.",
                        e
                    ),
                    "jpg" | "jpeg" => anyhow::anyhow!(
                        "Invalid JPEG file: {}. Please verify the file is not corrupted.",
                        e
                    ),
                    _ => anyhow::anyhow!("Invalid image file: {}. Supported formats: PNG, JPEG", e),
                }
            })?;

        if img.width() == 0 || img.height() == 0 {
            return Err(anyhow::anyhow!("Invalid image dimensions"));
        }
        // --- 1. Preprocess Image String ---
        // BLIP Large expects 384x384. Use FilterType::Triangle (Biliniar) for speed
        let resized = img.resize_exact(384, 384, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();

        // Convert Image to (1, 3, 384, 384) Float Tensor
        let mut image_tensor = Array4::<f32>::zeros((1, 3, 384, 384));
        for (x, y, pixel) in rgb_img.enumerate_pixels() {
            // Normalization (HuggingFace BlipProcessor standard: Mean [0.481, 0.457, 0.408], Std [0.268, 0.261, 0.275])
            image_tensor[[0, 0, y as usize, x as usize]] =
                (pixel[0] as f32 / 255.0 - 0.48145466) / 0.26862954;
            image_tensor[[0, 1, y as usize, x as usize]] =
                (pixel[1] as f32 / 255.0 - 0.4578275) / 0.26130258;
            image_tensor[[0, 2, y as usize, x as usize]] =
                (pixel[2] as f32 / 255.0 - 0.40821073) / 0.27577711;
        }

        // --- 2. Existing Loop Logic ---
        let mut tokens = vec![101_i64]; // BOS/SOS token
        let eos_token_id = 102_i64;
        let max_length = 20;
        let mut vision_session = self.vision_session.lock().await;
        let image_input: Value<TensorValueType<f32>> = Value::from_array(image_tensor)?;

        for _ in 0..max_length {
            let input_ids = Array2::from_shape_vec((1, tokens.len()), tokens.clone())?;
            let input_ids_value: Value<TensorValueType<i64>> = Value::from_array(input_ids)?;
            let inputs: Vec<(&str, SessionInputValue)> = vec![
                ("pixel_values", SessionInputValue::from(&image_input)),
                ("input_ids", SessionInputValue::from(&input_ids_value)),
            ];
            let outputs = vision_session.run(inputs)?;
            let (shape, logits_data) = outputs["logits"].try_extract_tensor::<f32>()?;
            let seq_len = shape[1] as usize;
            let logits = Array::from_shape_vec(
                IxDyn(&shape.iter().map(|&x| x as usize).collect::<Vec<_>>()),
                logits_data.to_vec(),
            )?;
            let last_token_logits = logits.slice(s![0, seq_len - 1, ..]);

            let next_token_id = last_token_logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index as i64)
                .unwrap_or_else(|| {
                    println!("Failed to get next token id");
                    0
                });

            if next_token_id == eos_token_id {
                break;
            }
            tokens.push(next_token_id);
        }

        let final_text = self
            .tokenizer
            .decode(
                &tokens[1..].iter().map(|&x| x as u32).collect::<Vec<_>>(),
                true,
            )
            .map_err(|e| anyhow::anyhow!("Failed to decode tokens: {}", e))?;

        Ok(final_text)
    }
}
