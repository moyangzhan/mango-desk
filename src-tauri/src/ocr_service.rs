use crate::utils::app_util::{get_ocr_cls_model_path, get_ocr_det_model_path, get_ocr_dict_path, get_ocr_rec_model_path};
use kreuzberg_paddle_ocr::OcrLite;
use log::{info, warn};
use std::path::Path;
use std::sync::OnceLock;

static OCR_ENGINE: OnceLock<Option<OcrLite>> = OnceLock::new();

fn get_engine() -> Option<&'static OcrLite> {
    OCR_ENGINE
        .get_or_init(|| {
            let det_path = get_ocr_det_model_path();
            let cls_path = get_ocr_cls_model_path();
            let rec_path = get_ocr_rec_model_path();
            let dict_path = get_ocr_dict_path();

            if !Path::new(&det_path).exists() {
                warn!("OCR detection model not found: {}", det_path);
                return None;
            }
            if !Path::new(&cls_path).exists() {
                warn!("OCR classification model not found: {}", cls_path);
                return None;
            }
            if !Path::new(&rec_path).exists() {
                warn!("OCR recognition model not found: {}", rec_path);
                return None;
            }
            if !Path::new(&dict_path).exists() {
                warn!("OCR dictionary not found: {}", dict_path);
                return None;
            }

            info!("Loading PaddleOCR models...");
            let mut engine = OcrLite::new();
            let num_threads = std::thread::available_parallelism()
                .map(|n| n.get().saturating_sub(2).max(2))
                .unwrap_or(2);
            match engine.init_models_with_dict(&det_path, &cls_path, &rec_path, &dict_path, num_threads) {
                Ok(()) => {
                    info!("PaddleOCR engine initialized successfully");
                    Some(engine)
                }
                Err(e) => {
                    warn!("Failed to initialize PaddleOCR engine: {}", e);
                    None
                }
            }
        })
        .as_ref()
}

pub fn recognize_file(image_path: &Path) -> String {
    let engine = match get_engine() {
        Some(e) => e,
        None => return String::new(),
    };

    if !image_path.exists() {
        warn!("OCR image file not found: {}", image_path.display());
        return String::new();
    }

    // Skip images over 40 MB to avoid excessive memory usage
    if let Ok(meta) = std::fs::metadata(image_path) {
        const MAX_IMAGE_SIZE: u64 = 40 * 1024 * 1024;
        if meta.len() > MAX_IMAGE_SIZE {
            warn!("OCR skipped, image too large ({} bytes): {}", meta.len(), image_path.display());
            return String::new();
        }
    }

    let path_str = match image_path.to_str() {
        Some(s) => s,
        None => return String::new(),
    };

    match engine.detect_from_path(path_str, 50, 1024, 0.5, 0.3, 2.0, true, false) {
        Ok(result) => {
            let text: String = result
                .text_blocks
                .iter()
                .map(|b| b.text.as_str())
                .collect::<Vec<&str>>()
                .join("\n");
            text.trim().to_string()
        }
        Err(e) => {
            warn!("OCR failed for {}: {}", image_path.display(), e);
            String::new()
        }
    }
}
