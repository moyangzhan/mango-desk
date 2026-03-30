//! Similarity search service
//!
//! This module provides unified similarity search functionality that combines
//! local and remote device searches.

use crate::cluster::http_client;
use crate::entities::FileInfo;
use crate::enums::PairingStatus;
use crate::global::CLIENT_ID;
use crate::repositories::{device_repo, file_info_repo};
use crate::similarity::{local_similarity_service, remote_similarity_service};
use crate::structs::search_result::{SearchResult, SourceDevice};

/// Find similar files for a local file
/// 为本地文件查找相似文件
///
/// Searches both local device and remote devices in parallel,
/// then merges and sorts results by score.
pub async fn find_similars_for_local_file(
    file_id: i64,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let file_info =
        file_info_repo::get_by_id(file_id)?.ok_or_else(|| "File not found".to_string())?;

    // Search local and remote devices in parallel
    // 并行搜索本地和远程设备
    let (local_results, mut remote_results) = tokio::join!(
        local_similarity_service::find_similars_by_file_id(&file_info, limit),
        remote_similarity_service::remote_find_similars(&file_info, limit)
    );

    let local_results = local_results.map_err(|e| e.to_string())?;

    // Merge and sort results
    let mut all_results = local_results;
    all_results.append(&mut remote_results);
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(limit);

    Ok(all_results)
}

/// Find similar files for a remote file (cross-device)
/// 为远程文件查找相似文件（跨设备）
///
/// Simplified approach:
/// 1. Send request to source device to find similars (it will search locally + forward to other devices)
/// 2. Search locally in parallel
/// 3. Merge results
pub async fn find_similars_for_remote_file(
    source_device_id: &str,
    remote_file_id: i64,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    log::info!(
        "find_similars_for_remote_file: source_device_id={}, remote_file_id={}, limit={}",
        source_device_id,
        remote_file_id,
        limit
    );

    // Check test mode
    if crate::global::TEST_MODE_REMOTE_DEVICE.load(std::sync::atomic::Ordering::Relaxed) {
        log::info!("Using mock device for test mode");
        let mock_file_info = FileInfo {
            id: remote_file_id,
            category: 2, // Image
            ..Default::default()
        };
        return Ok(crate::cluster::mock::generate_mock_similar_results(
            &mock_file_info,
            limit,
        ));
    }

    // Get source device info
    let source_device = device_repo::get_by_device_id(source_device_id)
        .map_err(|e| format!("Failed to get device: {}", e))?
        .ok_or_else(|| "Device not found".to_string())?;

    if source_device.pairing_status != PairingStatus::Paired {
        return Err("Device is not paired".to_string());
    }

    // Get local device ID for exclude list
    let local_device_id = CLIENT_ID.read().await.clone();
    let exclude_device_ids = vec![local_device_id.clone()];

    // Generate a unique request ID for this similarity search
    let request_id = uuid::Uuid::new_v4().to_string();
    log::info!("Generated request_id for similarity search: {}", request_id);

    // 1. Request source device to find similars (it will search locally + forward to other devices)
    // 1. 请求源设备查找相似文件（它会在本地搜索并转发到其他设备）
    let remote_results = match http_client::find_similars(
        remote_file_id,
        Some(&request_id),
        &exclude_device_ids,
        limit,
        &source_device.ip_address,
        source_device.port,
    )
    .await
    {
        Ok(data) => {
            let mut results = data.results;
            // Add source device info
            for result in &mut results {
                if result.source_device.is_none() {
                    result.source_device = Some(SourceDevice {
                        device_id: source_device.device_id.clone(),
                        device_name: source_device.name.clone(),
                    });
                }
            }
            results
        }
        Err(e) => {
            log::warn!("Failed to find similars on source device: {}", e);
            Vec::new()
        }
    };

    // 2. Also search locally
    // 2. 同时在本地搜索
    // Note: We don't have the file locally, so we can only search if we have features
    // For now, we rely on the source device's search results

    // 3. Return results
    log::info!("Total results from remote: {}", remote_results.len());
    Ok(remote_results)
}
