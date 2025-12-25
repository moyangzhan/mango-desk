use crate::enums::{FileCategory, IndexingTaskStatus};
use crate::global::INDEXING_SUMMARY;
use crate::repositories::indexing_task_repo;
use crate::structs::indexing_summary::IndexingSummary;
use chrono::Local;

pub async fn task_new(
    paths: &Vec<String>,
    embedding_model: &str,
) -> Result<crate::entities::IndexingTask, crate::repositories::RepositoryError> {
    let now = Local::now();
    let task = indexing_task_repo::insert_by_paths(paths, embedding_model, IndexingTaskStatus::Running.into(), &now)
        .await?;

    let mut summary = IndexingSummary::default();
    summary.task_id = task.id;
    summary.start_time = now;
    *INDEXING_SUMMARY.write().await = summary;

    Ok(task)
}

pub async fn task_done() -> Result<usize, String> {
    let (task_id) = {
        let mut summary = INDEXING_SUMMARY.write().await;
        summary.end_time = Local::now();
        summary.duration = summary.end_time.timestamp() - summary.start_time.timestamp();
        summary.task_id
    };
    let mut task = indexing_task_repo::get(task_id).await?;

    let summary = INDEXING_SUMMARY.read().await;
    let embedding_progress = summary.calculate_all_embedding();

    task.status = IndexingTaskStatus::Completed;
    task.end_time = Some(summary.end_time);
    task.duration = summary.duration;
    task.content_processed_cnt = embedding_progress.processed;
    task.content_indexed_success_cnt = embedding_progress.success;
    task.content_indexed_failed_cnt = embedding_progress.failed;
    task.content_indexed_skipped_cnt = summary.total - embedding_progress.total;
    let resut = indexing_task_repo::update(&task).await?;

    Ok(resut)
}

pub async fn summary_to_db() {
    let summary = INDEXING_SUMMARY.read().await;
    let embedding_progress = summary.calculate_all_embedding();
    indexing_task_repo::update_cnt(
        summary.task_id,
        summary.total,
        embedding_progress.processed,
        embedding_progress.success,
        embedding_progress.failed,
        embedding_progress.skipped,
        summary.duration,
    )
    .await
    .unwrap_or_else(|e| {
        println!("update_cnt error:{}", e);
        0
    });
}

pub async fn set_total(total: i64) {
    (*INDEXING_SUMMARY).write().await.total = total;
}

pub async fn set_total_by_category(file_category: &FileCategory, total: i64) {
    INDEXING_SUMMARY
        .write()
        .await
        .get_embedding_progress(file_category)
        .total = total;
}

pub async fn processed_incr(file_category: &FileCategory, incr: i64) {
    INDEXING_SUMMARY
        .write()
        .await
        .get_embedding_progress(file_category)
        .processed += incr;
}

pub async fn success_incr(file_category: &FileCategory, incr: i64) {
    (*INDEXING_SUMMARY)
        .write()
        .await
        .get_embedding_progress(file_category)
        .success += incr;
}

pub async fn failed_incr(file_category: &FileCategory, incr: i64) {
    (*INDEXING_SUMMARY)
        .write()
        .await
        .get_embedding_progress(file_category)
        .failed += incr;
}

pub async fn skipped_incr(file_category: &FileCategory, incr: i64) {
    (*INDEXING_SUMMARY)
        .write()
        .await
        .get_embedding_progress(file_category)
        .skipped += incr;
}
