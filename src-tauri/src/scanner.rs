use crate::entities::{FileInfo, IndexingTask};
use crate::enums::{FileCategory, FileIndexStatus, IndexingEvent};
use crate::errors::{AppError, IndexingError};
use crate::global::{
    IGNORE_HIDDEN_DIRS, IGNORE_HIDDEN_FILES, INDEXER_SETTING, SCANNING, SCANNING_TOTAL,
    STOP_INDEX_SIGNAL,
};
use crate::repositories::file_info_repo;
use crate::structs::indexer_setting::IndexerSetting;
use crate::utils::file_util::calculate_md5;
use crate::utils::{datetime_util, file_util, frontend_util};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep};

const MAX_RETRIES: usize = 3;
static UNSCANNED_DIR_COUNT: AtomicUsize = AtomicUsize::new(0);

pub async fn start(
    paths: &Vec<String>,
    indexing_task: Arc<IndexingTask>,
    on_event: Arc<Channel<IndexingEvent>>,
) {
    if paths.is_empty() {
        return;
    }
    if SCANNING.load(Ordering::SeqCst) {
        println!("Scan process already started.");
        return;
    }
    println!("Start scan process.");
    SCANNING.store(true, Ordering::SeqCst);
    SCANNING_TOTAL.store(0, Ordering::SeqCst);
    UNSCANNED_DIR_COUNT.store(0, Ordering::SeqCst);
    let mut tasks = JoinSet::new();
    let (sender, mut rx) = mpsc::channel::<String>(5000);
    for path in paths {
        if path.is_empty() {
            continue;
        }
        let is_file = Path::new(path).is_file();
        if is_file {
            SCANNING_TOTAL.fetch_add(1, Ordering::SeqCst);
            let path_str = path.to_string();
            let task_id = indexing_task.id;
            let file_event = on_event.clone();
            let task = tokio::spawn(async move {
                frontend_util::send_to_frontend(
                    file_event.as_ref(),
                    IndexingEvent::Scan {
                        task_id: task_id,
                        msg: format!("Scanning path: {}", path_str),
                    },
                );
                let indexer_setting = INDEXER_SETTING.read().await.clone();
                let is_valid = is_valid_file(&PathBuf::from(&path_str), &indexer_setting).await;
                if !is_valid {
                    println!("File is not valid: {}", path_str);
                    return;
                }
                if let Err(op) = add_or_update_file_info(path_str.to_string()).await {
                    println!("add_or_update_file_info error:{}", op.to_string());
                }
            });
            tasks.spawn(task);
        } else {
            UNSCANNED_DIR_COUNT.fetch_add(1, Ordering::SeqCst);
            let _ = sender.send(path.to_string()).await.map_err(|op| {
                println!("queue send message error:{}", op.to_string());
                return;
            });
        }
    }
    loop {
        let msg_count_in_queue = rx.len();
        println!(
            "Scan process.  unfinish directory count: {}, msg count in queue: {}",
            UNSCANNED_DIR_COUNT.load(Ordering::SeqCst),
            msg_count_in_queue
        );
        tokio::select! {
            maybe_dir = rx.recv() => {
                let Some(dir) = maybe_dir else { break; };
                let tx_clone = sender.clone();
                let task = indexing_task.clone();
                let event = on_event.clone();
                let task = tokio::spawn(async move {
                    let _ = scan_and_store(dir, tx_clone, task, event).await;
                });
                tasks.spawn(task);
            }

            _ = async {
                // Do not replace [ UNFINISH_DIR_COUNT.load(Ordering::SeqCst) ] with [ rx.len() ].
                while !STOP_INDEX_SIGNAL.load(Ordering::SeqCst)
                    && UNSCANNED_DIR_COUNT.load(Ordering::SeqCst) > 0
                {
                    sleep(Duration::from_millis(500)).await;
                }
            } => {
                println!("Stop signal received or all directories processed.");
                break;
            }
        }
    }
    while let Some(task) = tasks.join_next().await {
        if let Err(e) = task {
            eprintln!("Task failed: {}", e);
        }
    }
    println!("Scan process was finished.");
    SCANNING.store(false, Ordering::SeqCst);
}

pub async fn scan_and_store(
    dir: String,
    sender: Sender<String>,
    task: Arc<IndexingTask>,
    on_event: Arc<Channel<IndexingEvent>>,
) -> Result<(), IndexingError> {
    if dir.is_empty() {
        return Ok(());
    }
    println!("Scan directory: {}", dir);
    let indexer_setting = INDEXER_SETTING.read().await.clone();
    let mut entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if STOP_INDEX_SIGNAL.load(Ordering::SeqCst) {
            println!("Scanning process was stopped.");
            frontend_util::send_to_frontend(
                on_event.as_ref(),
                IndexingEvent::Stop {
                    task_id: task.id,
                    msg: "Scanning interrupted by stop signal.".to_string(),
                },
            );
            SCANNING.store(false, Ordering::SeqCst);
            break;
        }
        let path_buf = entry.path();
        let path_str = path_buf.to_str().unwrap_or("");
        println!("Scan file: {}", path_str);
        if path_str.is_empty() {
            continue;
        }
        frontend_util::send_to_frontend(
            on_event.as_ref(),
            IndexingEvent::Scan {
                task_id: task.id,
                msg: format!("Scanning path: {}", path_str),
            },
        );
        if path_buf.is_file() {
            SCANNING_TOTAL.fetch_add(1, Ordering::SeqCst);
            let is_valid = is_valid_file(&path_buf, &indexer_setting).await;
            if !is_valid {
                continue;
            }
            if let Err(op) = add_or_update_file_info(path_str.to_string()).await {
                println!("add_or_update_file_info error:{}", op.to_string());
                continue;
            }
        } else if path_buf.is_dir() {
            println!("Found directory: {}", path_buf.display());
            let dir_name: &str = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if indexer_setting.ignore_dirs.contains(&dir_name.to_string()) {
                println!("ignore dirs:{}", indexer_setting.ignore_dirs.join(","));
                continue;
            }
            if IGNORE_HIDDEN_DIRS && dir_name.starts_with(".") {
                println!("ignore hidden dirs");
                continue;
            }
            let path_owned = path_str.to_owned();

            // Try to send, handle full queue case
            let mut retries = 0;
            loop {
                match sender.try_send(path_owned.clone()) {
                    Ok(_) => {
                        UNSCANNED_DIR_COUNT.fetch_add(1, Ordering::SeqCst);
                        break;
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        if retries >= MAX_RETRIES {
                            eprintln!("Max retries exceeded for path: {}", path_owned);
                            break;
                        }
                        retries += 1;
                        // Queue is full, wait briefly and retry
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        println!("Error sending to channel: {}", e);
                        break;
                    }
                }
            }
            // dir_counter.fetch_add(1, Ordering::SeqCst);
        }
    }
    UNSCANNED_DIR_COUNT.fetch_sub(1, Ordering::SeqCst);
    // dir_counter.fetch_sub(1, Ordering::SeqCst);
    Ok(())
}

async fn is_valid_file(path_buf: &PathBuf, indexer_setting: &IndexerSetting) -> bool {
    let ext_str = path_buf
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext_str.is_empty() {
        println!("File has no extension: {}", path_buf.display());
        return false;
    }
    let ext = ext_str.as_str();
    if indexer_setting.ignore_exts.contains(&ext.to_string()) {
        println!("File extension is ignored: {}", path_buf.display());
        return false;
    }
    if !indexer_setting.ignore_files.is_empty()
        && indexer_setting
            .ignore_files
            .contains(&path_buf.to_str().unwrap_or("").to_string())
    {
        println!("File is ignored: {}", path_buf.display());
        return false;
    }
    let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if IGNORE_HIDDEN_FILES && file_name.starts_with(".") {
        println!("File is hidden: {}", path_buf.display());
        return false;
    }
    true
}

async fn add_or_update_file_info(input_path: String) -> Result<(), IndexingError> {
    let path_str = input_path.as_str();
    let path = PathBuf::from(path_str);
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mut file_handle = tokio::fs::File::open(path.as_path()).await?;
    let md5_hash = calculate_md5(&mut file_handle)
        .await
        .map_err(|op| AppError::CalculateMd5Error(op.to_string()))?;

    // Check the file by md5 hash
    if let Some(mut file_record) = file_info_repo::get_by_md5(&md5_hash).await? {
        if file_record.is_invalid
            || file_record.content_index_status == FileIndexStatus::Indexing.value()
        {
            return Ok(());
        }
        let modified_time = datetime_util::systemtime_to_datetime(
            (&file_handle).metadata().await?.modified()?.into(),
        );
        if file_record.content_index_status == FileIndexStatus::Indexed.value()
            && file_record.file_update_time.ge(&modified_time)
        {
            println!("File is already indexed: {}", path.display());
            return Ok(());
        } else {
            println!(
                "File is not indexed, updating record for reindexing. file: {}",
                path.display()
            );
            let meta = file_util::get_meta_by_record(path.as_path(), &file_record).await?;
            // Path/name/extension may have changed
            file_record.category = FileCategory::from_ext(&ext).await.value();
            file_record.path = path_str.to_string();
            file_record.metadata = meta.clone();
            file_record.name = file_record.metadata.name.clone();
            file_record.content_index_status = FileIndexStatus::Waiting.value();
            file_record.file_ext = ext.clone();
            file_record.file_update_time = modified_time; // Update mtime with touch command ?
            file_info_repo::update(&file_record).await?;
        }
    }
    // Check the file by path
    else if let Some(mut file_record) = file_info_repo::get_by_path(path_str).await? {
        // File has been modified: path exists but MD5 hash mismatch, updating record for reindexing
        println!(
            "File modified, path record exists but md5 hash mismatch - Path: {}, Expected MD5: {}, Actual MD5: {}",
            path.display(),
            file_record.md5,
            md5_hash
        );
        let mut meta = file_util::get_meta_by_record(path.as_path(), &file_record).await?;
        meta.extension = ext.clone();
        meta.category = FileCategory::from_ext(&meta.extension)
            .await
            .to_text()
            .to_string();
        let new_meta = meta.clone();
        file_record.name = new_meta.name.clone();
        file_record.md5 = md5_hash;
        file_record.is_invalid = false;
        file_record.invalid_reason = "".to_string();
        file_record.category = FileCategory::from_ext(&file_record.file_ext).await.value();
        file_record.file_ext = ext.clone();
        file_record.file_size = new_meta.size;
        file_record.file_create_time = new_meta.created;
        file_record.file_update_time = new_meta.modified;
        file_record.metadata = meta.clone();
        file_record.content_index_status = FileIndexStatus::Waiting.value();
        file_record.content_index_status_msg = "".to_string();
        file_record.meta_index_status = FileIndexStatus::Waiting.value();
        file_record.meta_index_status_msg = "".to_string();
        file_info_repo::update(&file_record).await?;
    }
    // New file
    else {
        println!("New file: {}, create record for indexing", path.display());
        let mut meta = file_util::get_meta_by_local(path.as_path(), &file_handle).await?;
        meta.extension = ext.clone();
        meta.category = FileCategory::from_ext(&meta.extension)
            .await
            .to_text()
            .to_string();
        let new_meta = meta.clone();
        let mut new_file_record = FileInfo::default();
        new_file_record.name = new_meta.name.clone();
        new_file_record.category = FileCategory::from_ext(&ext).await.value();
        new_file_record.path = path_str.to_string();
        new_file_record.md5 = md5_hash;
        new_file_record.path = path_str.to_string();
        new_file_record.file_ext = ext.clone();
        new_file_record.file_size = new_meta.size;
        new_file_record.file_create_time = new_meta.created;
        new_file_record.file_update_time = new_meta.modified;
        new_file_record.metadata = meta.clone();
        match file_info_repo::insert(&new_file_record).await {
            Ok(Some(new_file_record)) => {
                println!("New file record created: {}", new_file_record.id);
            }
            Ok(None) => {
                println!("Failed to create file record: {}", path.display());
            }
            Err(op) => println!(
                "Failed to create file record: {}, error: {}",
                path.display(),
                op.to_string()
            ),
        }
    }
    Ok(())
}
