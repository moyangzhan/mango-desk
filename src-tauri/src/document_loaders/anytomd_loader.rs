use crate::global::{
    ANYTOMD_EXTRA_EXTS, DOCX_EXTS, EXCEL_EXTS, MAX_DOCUMENT_LOAD_CHARS, PLAIN_TEXT_EXTS, PPTX_EXTS,
};
use crate::ocr_service;
use crate::traits::document_loader::DocumentLoader;
use super::get_images_dir;
use std::fs;
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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Short-circuit for plain text files: read directly without anytomd parsing
        if PLAIN_TEXT_EXTS.contains(&ext.as_str()) {
            return load_plain_text(path, max_load_chars);
        }

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

        let mut content = result.markdown;

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

        super::truncate_to_char_limit(&mut content, limit);

        Ok(content)
    }

    fn load_file_max(&self, _file: &std::fs::File, _max_load_chars: usize) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "load_file_max is not supported for AnyToMdLoader, use load_max with Path instead",
        ))
    }
}

fn load_plain_text(path: &Path, max_load_chars: usize) -> io::Result<String> {
    use std::io::Read;

    let limit = if max_load_chars > 0 {
        max_load_chars
    } else {
        MAX_DOCUMENT_LOAD_CHARS
    };

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut content = String::new();

    // Read up to limit * 4 bytes (UTF-8 worst case), then truncate to char limit
    reader
        .take((limit as u64) * 4)
        .read_to_string(&mut content)?;
    super::truncate_to_char_limit(&mut content, limit);

    Ok(content)
}
