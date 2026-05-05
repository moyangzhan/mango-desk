use crate::utils::app_util::{get_ocr_cls_model_path, get_ocr_det_model_path, get_ocr_dict_path, get_ocr_rec_model_path};
use kreuzberg_paddle_ocr::OcrLite;
use log::{info, warn};
use std::path::Path;
use std::sync::Mutex;

// PaddleOCR detection parameters (PP-OCRv4 defaults)
const DET_PADDING: u32 = 50;
const DET_MAX_SIDE_LEN: u32 = 1024;
const DET_BOX_SCORE_THRESH: f32 = 0.5;
const DET_BOX_THRESH: f32 = 0.3;
const DET_UNCLIP_RATIO: f32 = 2.0;
const DET_DO_ANGLE: bool = true;
const DET_MOST_ANGLE: bool = false;

static OCR_ENGINE: Mutex<Option<OcrLite>> = Mutex::new(None);

fn try_init_engine() -> Option<OcrLite> {
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
}

/// Ensure the OCR engine is initialized. Retries on each call if previously failed.
fn ensure_engine() {
    // Fast path: check without mutation
    {
        let guard = OCR_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_some() {
            return;
        }
    }
    // Slow path: try initialization
    {
        let mut guard = OCR_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            *guard = try_init_engine();
        }
    }
}

pub fn recognize_file(image_path: &Path) -> String {
    // File checks outside the lock — no contention
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
        None => {
            warn!("OCR skipped, non-UTF-8 path: {}", image_path.display());
            return String::new();
        }
    };

    ensure_engine();

    // Lock only for the actual inference call
    let guard = OCR_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
    let engine = match guard.as_ref() {
        Some(e) => e,
        None => return String::new(),
    };

    match engine.detect_from_path(
        path_str,
        DET_PADDING,
        DET_MAX_SIDE_LEN,
        DET_BOX_SCORE_THRESH,
        DET_BOX_THRESH,
        DET_UNCLIP_RATIO,
        DET_DO_ANGLE,
        DET_MOST_ANGLE,
    ) {
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
