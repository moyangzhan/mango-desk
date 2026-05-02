use crate::entities::FileInfo;
use crate::enums::FileCategory;
use crate::global::{EXT_TO_DOC_LOADER, MAX_DOCUMENT_LOAD_CHARS};
use crate::structs::embedding_progress::EmbeddingProgress;
use crate::traits::indexing_template::IndexingTemplate;
use std::path::PathBuf;

pub struct DocumentIndexer {
    pub category: FileCategory,
    pub status: EmbeddingProgress,
}

impl DocumentIndexer {
    pub fn new() -> Self {
        Self {
            category: FileCategory::Document,
            status: EmbeddingProgress::default(),
        }
    }
}

impl IndexingTemplate for DocumentIndexer {
    fn category(&self) -> &FileCategory {
        &self.category
    }
    async fn load_content(&self, file_info: &FileInfo) -> String {
        let loader = EXT_TO_DOC_LOADER
            .read()
            .await
            .get(&file_info.file_ext)
            .cloned();
        match loader {
            Some(doc_loader) => {
                let path = PathBuf::from(&file_info.path);
                tokio::task::spawn_blocking(move || {
                    doc_loader.load_max(&path, MAX_DOCUMENT_LOAD_CHARS)
                })
                .await
                .unwrap_or_else(|e| {
                    log::error!("Document loading task panicked: {}", e);
                    Ok(String::new())
                })
                .unwrap_or_default()
            }
            None => {
                log::warn!(
                    "No document loader found for extension: {}",
                    &file_info.file_ext
                );
                "".to_string()
            }
        }
    }
}
