use crate::embedding_service::EmbeddingService;
use crate::enums::{FileCategory, IndexingEvent, MigrationEvent};
use crate::errors::AppError;
use crate::global::{
    ACTIVE_MODEL_PLATFORM, CONFIG_NAME_INDEXER_SETTING, INDEXER_SETTING,
    INDEXING, INDEXING_FROM_WATCHER, MIGRATING, SCANNING, SCANNING_TOTAL, STOP_INDEX_SIGNAL, STORAGE_PATH,
};
use crate::indexers;
use crate::initializer;
use crate::repositories::{
    config_repo, file_content_embedding_repo, file_content_fts_repo, file_info_repo,
    file_metadata_embedding_repo, indexing_task_repo,
};
use crate::scanner;
use crate::structs::indexer_setting::IndexerSetting;
use crate::traits::indexing_template::IndexingTemplate;
use crate::utils::{frontend_util, indexing_task_util};
use rust_i18n::t;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub async fn update_indexer_setting(indexer_setting: IndexerSetting) -> Result<usize, String> {
    let json = serde_json::to_string(&indexer_setting).map_err(|e| AppError::SerializeError(e))?;
    let result = config_repo::update_by_name(CONFIG_NAME_INDEXER_SETTING, &json)?;
    initializer::init_setting(
        CONFIG_NAME_INDEXER_SETTING,
        || serde_json::to_string(&IndexerSetting::default()).unwrap_or_default(),
        &INDEXER_SETTING,
    )
    .await;
    Ok(result)
}

/// 启动内容存储迁移任务（后台执行）
///
/// 切换方向：
///   database → file    : 读取 DB 中的 content → 写入 parsed_documents/{md5}.md → 更新 DB 为相对路径
///   database → none    : 清空 DB 中的 content（仅文档）
///   file → database    : 读取 .md 文件 → 写入 DB content → 删除 .md 文件
///   file → none        : 删除 .md 文件 → 清空 DB content（仅文档）
///   none → database    : 重新解析原始文件（仅文档）→ 保存到 DB
///   none → file        : 重新解析原始文件（仅文档）→ 写入 .md → DB 存相对路径
///
/// 注意：none → 其他方向仅支持文档类型，图片和音频不提供 none 选项
pub async fn start_content_migration(category: &str, new_mode: &str) -> Result<(), String> {
    if MIGRATING.load(Ordering::SeqCst) {
        return Err("Migration already in progress".to_string());
    }
    if INDEXING.load(Ordering::SeqCst) {
        return Err("Cannot migrate while indexing is in progress".to_string());
    }

    // Validate mode value
    if new_mode != "database" && new_mode != "file" && new_mode != "none" {
        return Err(format!("Invalid storage mode: {}", new_mode));
    }

    let category_enum = match category {
        "document" => FileCategory::Document,
        "image" => FileCategory::Image,
        "audio" => FileCategory::Audio,
        _ => return Err(format!("Unknown category: {}", category)),
    };

    let old_mode = {
        let setting = INDEXER_SETTING.read().await;
        setting.content_storage.get_for_category(&category_enum).to_string()
    };

    if old_mode == new_mode {
        return Ok(());
    }

    // Update in-memory setting only (persist to DB after migration succeeds)
    {
        let mut setting = INDEXER_SETTING.write().await;
        match category {
            "document" => setting.content_storage.document = new_mode.to_string(),
            "image" => setting.content_storage.image = new_mode.to_string(),
            "audio" => setting.content_storage.audio = new_mode.to_string(),
            _ => {}
        }
    }

    let category_str = category.to_string();
    let new_mode = new_mode.to_string();
    let old_mode = old_mode;

    tokio::spawn(async move {
        MIGRATING.store(true, Ordering::SeqCst);

        let total = match file_info_repo::count_by_category(category_enum.value()) {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to count files for migration: {}", e);
                MIGRATING.store(false, Ordering::SeqCst);
                revert_and_persist(&category_str, &old_mode).await;
                return;
            }
        };

        frontend_util::send_event(
            "migration-event",
            &MigrationEvent::Start {
                category: category_str.clone(),
                total,
            },
        );

        let result = migrate_content_storage_inner(&category_str, &category_enum, &old_mode, &new_mode, total).await;
        MIGRATING.store(false, Ordering::SeqCst);

        let (migrated, failed) = result.unwrap_or_else(|e| {
            log::error!("Content storage migration failed: {}", e);
            (0i64, 0i64)
        });

        // Always send Complete event so frontend can sync state
        frontend_util::send_event(
            "migration-event",
            &MigrationEvent::Complete {
                category: category_str.clone(),
                migrated,
                failed,
            },
        );

        if result.is_err() {
            revert_and_persist(&category_str, &old_mode).await;
            return;
        }

        // Persist setting to DB on success
        persist_indexer_setting().await;
    });

    Ok(())
}

async fn revert_and_persist(category_str: &str, old_mode: &str) {
    let mut setting = INDEXER_SETTING.write().await;
    match category_str {
        "document" => setting.content_storage.document = old_mode.to_string(),
        "image" => setting.content_storage.image = old_mode.to_string(),
        "audio" => setting.content_storage.audio = old_mode.to_string(),
        _ => {}
    }
    drop(setting);
    persist_indexer_setting().await;
}

async fn persist_indexer_setting() {
    let setting = INDEXER_SETTING.read().await;
    if let Ok(json) = serde_json::to_string(&*setting) {
        if let Err(e) = config_repo::update_by_name(CONFIG_NAME_INDEXER_SETTING, &json) {
            log::error!("Failed to persist indexer setting after migration: {}", e);
        }
    }
}

async fn migrate_content_storage_inner(
    category_str: &str,
    category: &FileCategory,
    old_mode: &str,
    new_mode: &str,
    total: i64,
) -> Result<(i64, i64), String> {
    let storage_path = STORAGE_PATH.get().cloned().unwrap_or_default();
    let mut migrated = 0i64;
    let mut failed = 0i64;
    let batch_size = 500i64;
    let mut offset = 0i64;

    while offset < total {
        let files = file_info_repo::list_by_category_paged(
            category.value(), batch_size, offset,
        ).map_err(|e| e.to_string())?;

        if files.is_empty() {
            break;
        }

        for file in &files {
            if STOP_INDEX_SIGNAL.load(Ordering::SeqCst) {
                log::info!("Migration cancelled by stop signal");
                return Ok((migrated, failed));
            }

            match do_migrate_one(file, old_mode, new_mode, &storage_path, category).await {
                Ok(true) => migrated += 1,
                Ok(false) => {} // no change needed
                Err(e) => {
                    log::warn!("Migration failed for file {}: {}", file.path, e);
                    failed += 1;
                }
            }

            frontend_util::send_event(
                "migration-event",
                &MigrationEvent::Progress {
                    category: category_str.to_string(),
                    current: migrated + failed,
                    total,
                },
            );
        }

        offset += files.len() as i64;
    }

    log::info!(
        "Content storage migration complete: category={}, {}→{}, migrated={}, failed={}",
        category_str, old_mode, new_mode, migrated, failed
    );
    Ok((migrated, failed))
}

/// Migrate a single file's content storage. Returns Ok(true) if migrated, Ok(false) if skipped.
async fn do_migrate_one(
    file: &crate::entities::FileInfo,
    old_mode: &str,
    new_mode: &str,
    storage_path: &str,
    category: &FileCategory,
) -> Result<bool, String> {
    let is_file_mode = file.content.starts_with("parsed_documents/");

    match (old_mode, new_mode) {
        // database → file: DB has content, write to .md
        ("database", "file") => {
            if is_file_mode || file.content.is_empty() {
                return Ok(false); // already in file mode or no content
            }
            let md_dir = std::path::Path::new(storage_path).join("parsed_documents");
            let _ = std::fs::create_dir_all(&md_dir);
            let md_filename = format!("{}.md", &file.md5);
            let md_path = md_dir.join(&md_filename);
            std::fs::write(&md_path, &file.content)
                .map_err(|e| format!("write {}: {}", md_path.display(), e))?;
            let relative_path = format!("parsed_documents/{}", md_filename);
            file_info_repo::update_content_only(file.id, &relative_path)
                .map_err(|e| e.to_string())?;
            Ok(true)
        }

        // database → none: clear content (document only)
        ("database", "none") => {
            if file.content.is_empty() || is_file_mode {
                return Ok(false);
            }
            file_info_repo::update_content_only(file.id, "")
                .map_err(|e| e.to_string())?;
            Ok(true)
        }

        // file → database: read .md → write to DB → delete .md
        ("file", "database") => {
            if !is_file_mode {
                return Ok(false); // not in file mode
            }
            let md_path = std::path::Path::new(storage_path).join(&file.content);
            let content = std::fs::read_to_string(&md_path)
                .map_err(|e| format!("read {}: {}", md_path.display(), e))?;
            file_info_repo::update_content_only(file.id, &content)
                .map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(&md_path);
            Ok(true)
        }

        // file → none: delete .md + clear DB (document only)
        ("file", "none") => {
            if is_file_mode {
                let md_path = std::path::Path::new(storage_path).join(&file.content);
                let _ = std::fs::remove_file(&md_path);
            }
            if !file.content.is_empty() {
                file_info_repo::update_content_only(file.id, "")
                    .map_err(|e| e.to_string())?;
            }
            Ok(true)
        }

        // none → database: re-parse file → save to DB
        ("none", "database") => {
            let content = reparse_file(file, category).await?;
            if content.is_empty() {
                return Ok(false);
            }
            file_info_repo::update_content_only(file.id, &content)
                .map_err(|e| e.to_string())?;
            Ok(true)
        }

        // none → file: re-parse file → write .md → save relative path
        ("none", "file") => {
            let content = reparse_file(file, category).await?;
            if content.is_empty() {
                return Ok(false);
            }
            let md_dir = std::path::Path::new(storage_path).join("parsed_documents");
            let _ = std::fs::create_dir_all(&md_dir);
            let md_filename = format!("{}.md", &file.md5);
            let md_path = md_dir.join(&md_filename);
            std::fs::write(&md_path, &content)
                .map_err(|e| format!("write {}: {}", md_path.display(), e))?;
            let relative_path = format!("parsed_documents/{}", md_filename);
            file_info_repo::update_content_only(file.id, &relative_path)
                .map_err(|e| e.to_string())?;
            Ok(true)
        }

        // Fallback: detect actual state from content field and migrate accordingly
        _ => {
            // Handle cross-direction cases (e.g., old was "none" but content exists from prior migration)
            let actual_old = if is_file_mode { "file" } else if file.content.is_empty() { "none" } else { "database" };
            if actual_old == new_mode {
                return Ok(false);
            }
            do_migrate_one(file, actual_old, new_mode, storage_path, category).await
        }
    }
}

/// Re-parse a file to recover its content (used for none → database/file migration).
/// Only supports documents. Images and audio would require expensive model inference.
async fn reparse_file(
    file: &crate::entities::FileInfo,
    category: &FileCategory,
) -> Result<String, String> {
    let path = std::path::Path::new(&file.path);
    if !path.exists() {
        return Err(format!("File no longer exists: {}", file.path));
    }

    match category {
        FileCategory::Document => {
            use crate::global::{EXT_TO_DOC_LOADER, MAX_DOCUMENT_LOAD_CHARS};
            let loader_map = EXT_TO_DOC_LOADER.read().await;
            let loader = loader_map.get(&file.file_ext).cloned();
            drop(loader_map);

            match loader {
                Some(doc_loader) => {
                    tokio::task::spawn_blocking(move || {
                        doc_loader.load_max(&path, MAX_DOCUMENT_LOAD_CHARS)
                    })
                    .await
                    .map_err(|e| format!("Re-parse task panicked: {}", e))?
                    .map_err(|e| format!("Re-parse failed: {}", e))
                }
                None => Ok(String::new()),
            }
        }
        _ => {
            // Image/audio re-parsing requires model inference, not supported in migration
            log::warn!("Re-parsing not supported for {:?} files in migration", category);
            Ok(String::new())
        }
    }
}

pub async fn is_embedding_model_changed() -> Result<bool, String> {
    let tasks = indexing_task_repo::list(1, 1, "id", "desc");
    if tasks.is_err() {
        return Ok(false);
    }
    match tasks {
        Ok(tasks) => {
            if tasks.is_empty() {
                return Ok(false);
            }
            let latest = tasks
                .first()
                .map(|item| item.embedding_model.clone())
                .unwrap_or_default();
            let embedding_name: &'static str = EmbeddingService::model_name().await;
            if latest != embedding_name {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(_) => {
            return Ok(false);
        }
    }
}

/// Indexing workflow consists of four main phases:
///
/// 1. File scanning:
///    Scan specified paths and store file metadata in database
///
/// 2. Document processing:
///    Extract text content and generate document embeddings
///
/// 3. Image processing:
///    Extract visual features and generate image embeddings
///    Prerequisites: Private mode disabled, Model platform configured
///
/// 4. Audio processing:
///    Extract audio features and generate audio embeddings
///    Prerequisites: Private mode disabled, Model platform configured
pub async fn start_indexing(paths: Vec<String>, from: &str) -> Result<bool, String> {
    if paths.is_empty() {
        return Ok(false);
    }
    if SCANNING.load(Ordering::SeqCst) {
        return Ok(false);
    }
    if INDEXING.load(Ordering::SeqCst) {
        return Ok(false);
    }
    STOP_INDEX_SIGNAL.store(false, Ordering::SeqCst);
    let embedding_model = EmbeddingService::model_name().await;
    let task = indexing_task_util::task_new(&paths, embedding_model).await?;

    let task = Arc::new(task);

    frontend_util::send_event(
        get_event_from(from),
        &IndexingEvent::Start {
            task_id: task.id,
            msg: "Start".to_string(),
        },
    );

    // Scan specified paths and store file metadata in database
    scanner::start(&paths, task.clone(), from).await;

    // Scanned files
    indexing_task_util::set_total(SCANNING_TOTAL.load(Ordering::SeqCst) as i64).await;
    indexing_task_util::summary_to_db().await;

    // Embedding processing
    INDEXING.store(true, Ordering::SeqCst);

    log::info!("Starting document indexing...");
    let mut document_indexer = indexers::document_indexer::DocumentIndexer::new();
    let _ = document_indexer
        .process(task.clone(), from)
        .await
        .unwrap_or_else(|e| log::error!("start_indexing => Document indexing error,{}", e));
    log::info!(
        "Document indexing done, status: {}",
        serde_json::json!(document_indexer.status)
    );
    indexing_task_util::summary_to_db().await;

    let unindex_images_cnt = file_info_repo::count_unindexed_files(FileCategory::Image.value())?;
    log::info!("Total images to index: {}", unindex_images_cnt);
    if unindex_images_cnt > 0 {
        if let Ok(mut image_indexer) = indexers::image_indexer::ImageIndexer::new().await {
            log::info!("Starting image indexing...");
            let _ = image_indexer
                .process(task.clone(), from)
                .await
                .unwrap_or_else(|e| log::error!("Image indexing error: {}", e));
            indexing_task_util::summary_to_db().await;
        }
    }

    let unindex_audio_cnt = file_info_repo::count_unindexed_files(FileCategory::Audio.value())?;
    log::info!("Total audio files to index: {}", unindex_audio_cnt);
    if unindex_audio_cnt > 0 {
        if let Ok(mut audio_indexer) = indexers::audio_indexer::AudioIndexer::new().await {
            log::info!("Starting audio indexing...");
            let _ = audio_indexer
                .process(task.clone(), from)
                .await
                .unwrap_or_else(|e| log::error!("Audio indexing error: {}", e));
            indexing_task_util::summary_to_db().await;
        }
    }

    indexing_finish(task.id, "done", from).await?;

    return Ok(true);
}

pub async fn index_file(path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let is_valid = scanner::is_valid_file_with(&path_buf).await;
    if !is_valid {
        return Ok(());
    }
    if let Err(add_info_result) = scanner::add_or_update_file_info(path.to_string()).await {
        log::error!("add_or_update_file_info error: {:?}", add_info_result);
        return Ok(());
    }
    let ext = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    let Some(file_info) = file_info_repo::get_by_path(path)? else {
        return Ok(());
    };
    let category = FileCategory::from_ext(&ext);
    match category {
        FileCategory::Document => {
            let document_indexer = indexers::document_indexer::DocumentIndexer::new();
            document_indexer.embedding_one_file(&file_info).await?;
        }
        FileCategory::Image => {
            let image_indexer = indexers::image_indexer::ImageIndexer::new().await?;
            image_indexer.embedding_one_file(&file_info).await?;
        }
        FileCategory::Audio => {
            let audio_indexer = indexers::audio_indexer::AudioIndexer::new().await?;
            audio_indexer.embedding_one_file(&file_info).await?;
        }
        _ => {}
    }
    Ok(())
}

pub fn remove_file_index(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    let file_info = file_info_repo::get_by_path(path)?;
    if let Some(file_info) = file_info {
        file_info_repo::delete_by_id(file_info.id)?;
        file_content_fts_repo::delete_by_file_id(file_info.id)?;
        file_content_embedding_repo::delete_by_file_id(file_info.id)?;
        file_metadata_embedding_repo::delete_by_file_id(file_info.id)?;
    }
    Ok(())
}

pub fn remove_directory_index(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    file_content_fts_repo::delete_by_prefix_path(path)?;
    file_content_embedding_repo::delete_by_file_prefix_path(path)?;
    file_metadata_embedding_repo::delete_by_file_prefix_path(path)?;
    file_info_repo::delete_by_prefix_path(path)?;
    Ok(())
}

async fn indexing_finish(task_id: i64, msg: &str, from: &str) -> Result<(), String> {
    SCANNING.store(false, Ordering::SeqCst);
    INDEXING.store(false, Ordering::SeqCst);
    SCANNING_TOTAL.store(0, Ordering::SeqCst);
    STOP_INDEX_SIGNAL.store(false, Ordering::SeqCst);

    indexing_task_util::task_done().await?;

    frontend_util::send_event(
        get_event_from(from),
        &IndexingEvent::Finish {
            task_id,
            msg: msg.to_string(),
        },
    );
    Ok(())
}

pub fn get_event_from(from: &str) -> &'static str {
    if from.is_empty() {
        return crate::global::EVENT_SELECTOR_INDEXING;
    }
    if from == crate::global::INDEXING_FROM_SELECTOR {
        return crate::global::EVENT_SELECTOR_INDEXING;
    }
    return crate::global::EVENT_WATCHER_INDEXING;
}
