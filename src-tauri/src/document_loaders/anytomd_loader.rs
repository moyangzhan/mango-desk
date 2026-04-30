use crate::global::{
    ANYTOMD_EXTRA_EXTS, DOCX_EXTS, EXCEL_EXTS, EXTRACTED_IMAGES_PATH, INDEXER_SETTING,
    MAX_DOCUMENT_LOAD_CHARS, PLAIN_TEXT_EXTS, PPTX_EXTS,
};
use crate::ocr_service;
use crate::traits::document_loader::DocumentLoader;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct AnyToMdLoader {
    exts: Vec<String>,
}

impl Default for AnyToMdLoader {
    fn default() -> Self {
        let exts: Vec<&[&str]> = vec![
            DOCX_EXTS,
            PPTX_EXTS,
            EXCEL_EXTS,
            PLAIN_TEXT_EXTS,
            ANYTOMD_EXTRA_EXTS,
        ];
        Self {
            exts: exts
                .into_iter()
                .flat_map(|e| e.iter().map(|s| s.to_string()))
                .collect(),
        }
    }
}

impl DocumentLoader for AnyToMdLoader {
    fn get_exts(&self) -> &[String] {
        &self.exts
    }

    fn add_ext(&mut self, ext: String) {
        self.exts.push(ext);
    }

    fn load(&self, path: &Path) -> io::Result<String> {
        self.load_max(path, 0)
    }

    fn load_max(&self, path: &Path, max_load_chars: usize) -> io::Result<String> {
        let output_format = INDEXER_SETTING
            .read()
            .map(|s| s.document_output_format.clone())
            .unwrap_or_else(|_| "text".to_string());

        let options = anytomd::ConversionOptions {
            extract_images: true,
            max_total_image_bytes: 10 * 1024 * 1024,
            ..Default::default()
        };

        let result = anytomd::convert_file(path, &options).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse document {}: {}", path.display(), e),
            )
        })?;

        let mut content = if output_format == "markdown" {
            result.markdown
        } else {
            result.plain_text
        };

        if !result.images.is_empty() {
            let images_dir = get_images_dir(path);
            if !images_dir.exists() {
                fs::create_dir_all(&images_dir).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to create images dir: {}", e),
                    )
                })?;
            }

            let mut ocr_texts = Vec::new();
            for (filename, bytes) in &result.images {
                let img_path = images_dir.join(filename);
                fs::write(&img_path, bytes)?;
                let ocr_text = ocr_service::recognize_file(&img_path);
                if !ocr_text.is_empty() {
                    ocr_texts.push(format!("[OCR]: {}", ocr_text));
                }
            }

            if !ocr_texts.is_empty() {
                content.push_str("\n\n");
                content.push_str(&ocr_texts.join("\n"));
            }
        }

        let limit = if max_load_chars > 0 {
            max_load_chars
        } else {
            MAX_DOCUMENT_LOAD_CHARS
        };

        if content.chars().count() > limit {
            content = content.chars().take(limit).collect();
        }

        Ok(content)
    }

    fn load_file_max(&self, _file: &std::fs::File, _max_load_chars: usize) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "load_file_max is not supported for AnyToMdLoader, use load_max with Path instead",
        ))
    }
}

fn get_images_dir(source_path: &Path) -> std::path::PathBuf {
    let extracted_path = EXTRACTED_IMAGES_PATH
        .get()
        .cloned()
        .unwrap_or_default();
    let canonical = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());
    std::path::PathBuf::from(extracted_path).join(hash)
}
