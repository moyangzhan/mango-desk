use crate::global::{EXTRACTED_IMAGES_PATH, INDEXER_SETTING, MAX_DOCUMENT_LOAD_CHARS, PDF_EXTS};
use crate::ocr_service;
use crate::traits::document_loader::DocumentLoader;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct PdfPlumberLoader {
    exts: Vec<String>,
}

impl Default for PdfPlumberLoader {
    fn default() -> Self {
        Self {
            exts: PDF_EXTS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl DocumentLoader for PdfPlumberLoader {
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

        let pdf = pdfplumber::Pdf::open_file(path, None).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to open PDF {}: {}", path.display(), e),
            )
        })?;

        let mut pages_content = Vec::new();
        let mut ocr_texts = Vec::new();

        let images_dir = get_images_dir(path);
        let needs_images = !images_dir.exists();

        for page_idx in 0..pdf.page_count() {
            let page = match pdf.page(page_idx) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!(
                        "Failed to read page {} of {}: {}",
                        page_idx,
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            let page_text = if output_format == "markdown" {
                let chars = page.chars();
                let tables = page.find_tables(&pdfplumber::TableSettings::default());
                pdfplumber::MarkdownRenderer::render(
                    &chars,
                    &tables,
                    &pdfplumber::MarkdownOptions::default(),
                )
            } else {
                page.extract_text(&pdfplumber::TextOptions::default())
                    .unwrap_or_default()
            };

            if !page_text.is_empty() {
                pages_content.push(page_text);
            }

            // Extract images from page
            let images = page.images();
            if !images.is_empty() {
                if needs_images {
                    if let Err(e) = fs::create_dir_all(&images_dir) {
                        log::warn!("Failed to create images dir: {}", e);
                    }
                }
                for (img_idx, img) in images.iter().enumerate() {
                    if let Ok(img_content) = pdf.extract_image_content(page_idx, &img.name) {
                        let ext = match img_content.format {
                            pdfplumber::ImageFormat::Jpeg => "jpg",
                            pdfplumber::ImageFormat::Png => "png",
                            _ => "bin",
                        };
                        let img_filename = format!("p{}_img{}.{}", page_idx, img_idx, ext);
                        let img_path = images_dir.join(&img_filename);
                        if fs::write(&img_path, &img_content.data).is_ok() {
                            let ocr_text = ocr_service::recognize_file(&img_path);
                            if !ocr_text.is_empty() {
                                ocr_texts.push(format!("[OCR]: {}", ocr_text));
                            }
                        }
                    }
                }
            }
        }

        let separator = if output_format == "markdown" {
            "\n\n---\n\n"
        } else {
            "\n\n"
        };
        let mut content = pages_content.join(separator);

        if !ocr_texts.is_empty() {
            content.push_str("\n\n");
            content.push_str(&ocr_texts.join("\n"));
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
            "load_file_max is not supported for PdfPlumberLoader, use load_max with Path instead",
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
