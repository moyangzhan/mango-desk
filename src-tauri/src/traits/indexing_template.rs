use crate::embedding_service_manager::embedding_service_manager;
use crate::entities::{FileContentEmbedding, FileInfo, FileMetaEmbedding, IndexingTask};
use crate::enums::{FileCategory, FileIndexStatus, IndexingEvent};
use crate::errors::{AppError, IndexingError};
use crate::global::STOP_INDEX_SIGNAL;
use crate::repositories::{
    file_content_embedding_repo, file_info_repo, file_metadata_embedding_repo,
};
use crate::structs::file_metadata::FileMetadata;
use crate::utils::{file_util, frontend_util, indexing_task_util, text_util};
use rust_i18n::t;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;

pub trait IndexingTemplate {
    async fn process(
        &mut self,
        task: Arc<IndexingTask>,
        on_event: Option<Arc<Channel<IndexingEvent>>>,
    ) -> Result<(), IndexingError> {
        let mut min_id = 0i64;
        let mut loop_count = 0;
        let limit = 1000;
        let total = file_info_repo::count_unindexed_files(self.category().value())?;
        println!("Total documents to index: {}", total);
        if total == 0 {
            return Ok(());
        }
        let max_loop = total / limit + 1; // Max times to avoid infinite loop
        indexing_task_util::set_total_by_category(self.category(), total).await;
        'outer: loop {
            if loop_count > max_loop {
                println!("Max loop reached, exiting...");
                break;
            }
            if STOP_INDEX_SIGNAL.load(Ordering::SeqCst) {
                println!("stopping indexing process");
                if let Some(event) = on_event.as_ref() {
                    frontend_util::send_to_frontend(
                        event,
                        IndexingEvent::Stop {
                            task_id: task.id,
                            msg: "Stop indexing, Stopped by user.".to_string(),
                        },
                    );
                }
                break;
            }
            println!(
                "list_unindexed_files by min_id: {},category:{}",
                min_id,
                self.category()
            );

            let file_infos =
                file_info_repo::list_unindexed_files(min_id, limit, self.category().value())?;
            if file_infos.is_empty() {
                println!("No documents");
                break;
            }
            loop_count += 1;
            println!("Found {} documents to index.", file_infos.len());
            min_id = file_infos
                .iter()
                .map(|info| info.id)
                .max()
                .unwrap_or(min_id + 1000);
            for file_info in file_infos {
                if STOP_INDEX_SIGNAL.load(Ordering::SeqCst) {
                    println!("Indexing process interrupted by stop signal");
                    break 'outer;
                }
                indexing_task_util::processed_incr(self.category(), 1).await;
                if !Path::new(&file_info.path).exists() {
                    println!("File not exist: {}", file_info.path);
                    indexing_task_util::failed_incr(self.category(), 1).await;
                    file_info_repo::delete_by_id(file_info.id)?;
                    file_content_embedding_repo::delete_by_file_id(file_info.id)?;
                    file_metadata_embedding_repo::delete_by_file_id(file_info.id)?;
                    continue;
                }
                if let Some(event) = on_event.as_ref() {
                    frontend_util::send_to_frontend(
                        event,
                        IndexingEvent::Embed {
                            task_id: task.id,
                            msg: format!("Embedding path: {}", &file_info.path),
                        },
                    );
                }
                if let Err(error) = self.embedding_one_file(&file_info).await {
                    println!("Embedding failed: {}", error.to_string());
                    indexing_task_util::failed_incr(self.category(), 1).await;
                }
            }
        }
        Ok(())
    }

    async fn embedding_one_file(&self, file_info: &FileInfo) -> Result<(), IndexingError> {
        let filtered_content = {
            let content = self.load_content(&file_info).await;
            text_util::collapse_newlines(&content)
        };
        let file_id = file_info.id;
        let path_str = file_info.path.as_str();
        let path = Path::new(path_str);
        let file_meta = file_util::get_meta_by_record(path, &file_info).await?;
        let _ =
            file_info_repo::update_content_meta(file_id, &filtered_content, &file_meta.to_json())?;
        println!("File metadata loaded: {}", file_meta.to_text());

        //Remove old index
        file_content_embedding_repo::delete_by_file_id(file_id)?;
        file_metadata_embedding_repo::delete_by_file_id(file_id)?;

        embedding_metadata(file_id, &file_meta).await?;
        if filtered_content.is_empty() {
            let _ = file_info_repo::update_content_index_status(
                file_id,
                FileIndexStatus::Indexed.value(),
                t!("message.indexing-skip-empty-content").as_ref(),
            );
            indexing_task_util::skipped_incr(self.category(), 1).await;
        } else {
            match embedding_content(file_id, &filtered_content).await {
                Ok(_) => {
                    indexing_task_util::success_incr(self.category(), 1).await;
                }
                Err(error) => {
                    println!("Embedding content error: {}", error.to_string());
                    let _ = indexing_task_util::failed_incr(self.category(), 1).await;
                }
            }
        }
        return Ok(());
    }

    async fn load_content(&self, file_info: &FileInfo) -> String;
    fn category(&self) -> &FileCategory;
}

pub async fn embedding_content(file_id: i64, content: &str) -> Result<(), IndexingError> {
    if content.is_empty() {
        return Err(IndexingError::EmptyContent);
    }

    // Embedding content
    let chunks = {
        let mut manager = embedding_service_manager().write().await;
        let embedding_service = manager.service().await?;
        text_util::split_text(&content, &embedding_service.tokenizer)
            .map_err(|op| AppError::DocumentSplitterError(op.to_string()))?
    };
    for (chunk_index, chunk_text) in chunks.into_iter().enumerate() {
        println!("Chunk text: {}", chunk_text.len());
        let mut keep_run = true;
        let chunk_embedding = {
            let mut manager = embedding_service_manager().write().await;
            match manager.embed(&chunk_text).await {
                Ok(embedding) => embedding,
                Err(op) => {
                    drop(manager);
                    println!("embedding chunk error:{}", op.to_string());
                    let _ = file_info_repo::update_content_index_status(
                        file_id,
                        FileIndexStatus::IndexFailed.value(),
                        op.to_string().as_str(),
                    );
                    keep_run = false;
                    Vec::new()
                }
            }
        };
        if !keep_run {
            continue;
        }
        let content_array: [f32; 384] = match chunk_embedding.try_into() {
            Ok(embedding) => embedding,
            Err(_) => {
                let _ = file_info_repo::update_content_index_status(
                    file_id,
                    FileIndexStatus::IndexFailed.value(),
                    "Failed to convert embedding to array",
                );
                keep_run = false;
                [0.0; 384]
            }
        };
        if !keep_run {
            continue;
        }
        file_content_embedding_repo::insert(
            &(FileContentEmbedding {
                id: 0,
                file_id,
                embedding: content_array,
                chunk_index: chunk_index as i64,
                chunk_text,
                distance: -0.1,
            }),
        )?;
        let _ = file_info_repo::update_content_index_status(
            file_id,
            FileIndexStatus::Indexed.value(),
            "success",
        )?;
    }
    Ok(())
}

pub async fn embedding_metadata(
    file_id: i64,
    file_meta: &FileMetadata,
) -> Result<(), IndexingError> {
    // File meta embedding
    let mut guard = embedding_service_manager().write().await;
    let meta_embedding = match guard.embed(file_meta.to_text().as_str()).await {
        Ok(embedding) => {
            drop(guard);
            embedding
        }
        Err(op) => {
            drop(guard);
            println!("embedding meta error:{}", op.to_string());
            file_info_repo::update_meta_index_status(
                file_id,
                FileIndexStatus::IndexFailed.value(),
                op.to_string().as_str(),
            )?;
            return Ok(());
        }
    };
    let meta_array: [f32; 384] = meta_embedding.try_into().unwrap_or([0.0; 384]);
    file_metadata_embedding_repo::insert(
        &(FileMetaEmbedding {
            id: 0,
            file_id,
            embedding: meta_array,
            distance: -0.1,
        }),
    )?;
    file_info_repo::update_meta_index_status(file_id, FileIndexStatus::Indexed.value(), "success")?;
    return Ok(());
}
