use crate::utils::app_util::{get_ocr_detection_model_path, get_ocr_recognition_model_path};
use log::{info, warn};
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use std::path::Path;
use std::sync::OnceLock;

static OCR_ENGINE: OnceLock<Option<OcrEngine>> = OnceLock::new();

fn get_engine() -> Option<&'static OcrEngine> {
    OCR_ENGINE
        .get_or_init(|| {
            let detection_path = get_ocr_detection_model_path();
            let recognition_path = get_ocr_recognition_model_path();

            if !Path::new(&detection_path).exists() || !Path::new(&recognition_path).exists() {
                warn!("OCR model files not found, OCR disabled. Detection: {}, Recognition: {}", detection_path, recognition_path);
                return None;
            }

            info!("Loading OCR models...");
            let detection_model = match Model::load_file(&detection_path) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to load OCR detection model: {}", e);
                    return None;
                }
            };
            let recognition_model = match Model::load_file(&recognition_path) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to load OCR recognition model: {}", e);
                    return None;
                }
            };

            match OcrEngine::new(OcrEngineParams {
                detection_model: Some(detection_model),
                recognition_model: Some(recognition_model),
                ..Default::default()
            }) {
                Ok(engine) => {
                    info!("OCR engine initialized successfully");
                    Some(engine)
                }
                Err(e) => {
                    warn!("Failed to create OCR engine: {}", e);
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

    let img = match image::open(image_path) {
        Ok(img) => img.to_rgb8(),
        Err(e) => {
            warn!("OCR failed to open image {}: {}", image_path.display(), e);
            return String::new();
        }
    };

    let ocr_input = match engine.prepare_input(img) {
        Ok(input) => input,
        Err(e) => {
            warn!("OCR prepare input failed for {}: {}", image_path.display(), e);
            return String::new();
        }
    };

    match engine.get_text(&ocr_input) {
        Ok(text) => text.trim().to_string(),
        Err(e) => {
            warn!("OCR text extraction failed for {}: {}", image_path.display(), e);
            String::new()
        }
    }
}
