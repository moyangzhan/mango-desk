use crate::cluster::api_types::{
    CommonResponse, PingData, SearchData, ChunksData, ChunksRequest, FindSimilarsData,
    FileContentData, PairingRequestData, PairingRespondData, ResetNotifyData,
    CODE_SUCCESS, CODE_MISSING_DEVICE_ID, CODE_DEVICE_NOT_PAIRED, CODE_NOT_FOUND, CODE_INTERNAL_ERROR,
};
use crate::entities::{Device, PairingRequest};
use crate::enums::FileCategory;
use crate::enums::{OnlineStatus, PairingRequestStatus, PairingResponseStatus, PairingStatus};
use crate::global::{APP_HANDLE, CLIENT_ID};
use crate::repositories::{
    device_repo, file_content_embedding_repo, file_info_repo, pairing_request_repo,
};
use crate::searcher;
use crate::structs::cluster_config::{
    DeviceCapabilities, PairingRequestPayload, PairingResponsePayload, RemoteFindSimilarsRequest,
    RemoteSearchRequest, ResetNotifyPayload,
};
use crate::structs::search_result::SearchResult;
use rust_i18n::t;
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Json, Response, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;
use tokio::sync::watch;

/// HTTP server shutdown signal sender
static SHUTDOWN_SENDER: std::sync::RwLock<Option<watch::Sender<bool>>> =
    std::sync::RwLock::new(None);

/// Default port for cluster service
/// 集群服务默认端口
pub const DEFAULT_PORT: i32 = 15678;

/// Cache for processed request IDs to prevent loops
/// 用于防止循环的已处理请求ID缓存
/// Key: request_id, Value: timestamp when processed
static PROCESSED_REQUESTS: std::sync::RwLock<Option<HashMap<String, Instant>>> =
    std::sync::RwLock::new(None);

/// Cache expiration time in seconds (10 minutes)
/// 缓存过期时间（10分钟）
const REQUEST_CACHE_TTL_SECS: u64 = 600;

// ============================================================================
// Helper Functions / 辅助函数
// ============================================================================

/// Verify that the requester device is paired (returns CommonResponse error)
/// 验证请求者设备是否已配对（返回 CommonResponse 错误）
fn verify_paired_device<T: Serialize>(headers: &HeaderMap) -> Result<&str, CommonResponse<T>> {
    let requester_id = headers
        .get("X-Device-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            log::warn!("Missing X-Device-ID header");
            CommonResponse::error(CODE_MISSING_DEVICE_ID, "Missing X-Device-ID header")
        })?;

    let device = device_repo::get_by_device_id(requester_id).map_err(|e| {
        log::error!("Failed to get device: {}", e);
        CommonResponse::error(CODE_INTERNAL_ERROR, "Failed to get device")
    })?;

    match device {
        Some(d) if d.pairing_status == PairingStatus::Paired => Ok(requester_id),
        Some(d) => {
            log::warn!("Device {} not paired, status: {:?}", requester_id, d.pairing_status);
            Err(CommonResponse::error(CODE_DEVICE_NOT_PAIRED, "Device not paired"))
        }
        None => {
            log::warn!("Device {} not found", requester_id);
            Err(CommonResponse::error(CODE_DEVICE_NOT_PAIRED, "Device not found"))
        }
    }
}

/// Verify that the requester device is paired (returns StatusCode for binary responses)
/// 验证请求者设备是否已配对（为二进制响应返回 StatusCode）
fn verify_paired_device_status(headers: &HeaderMap) -> Result<&str, StatusCode> {
    let requester_id = headers
        .get("X-Device-ID")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            log::warn!("Missing X-Device-ID header");
            StatusCode::BAD_REQUEST
        })?;

    let device = device_repo::get_by_device_id(requester_id).map_err(|e| {
        log::error!("Failed to get device: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match device {
        Some(d) if d.pairing_status == PairingStatus::Paired => Ok(requester_id),
        Some(d) => {
            log::warn!("Device {} not paired, status: {:?}", requester_id, d.pairing_status);
            Err(StatusCode::FORBIDDEN)
        }
        None => {
            log::warn!("Device {} not found", requester_id);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Initialize the processed requests cache
fn init_request_cache() {
    let mut cache = PROCESSED_REQUESTS.write().unwrap();
    if cache.is_none() {
        *cache = Some(HashMap::new());
    }
}

/// Check if request has been processed, and mark as processed if not
/// Returns true if the request was already processed (should be ignored)
/// Returns false if this is a new request (should be processed)
fn check_and_mark_request(request_id: &str) -> bool {
    let mut cache_guard = PROCESSED_REQUESTS.write().unwrap();
    if let Some(ref mut cache) = *cache_guard {
        let now = Instant::now();

        // Check if already processed
        if let Some(&timestamp) = cache.get(request_id) {
            if now.duration_since(timestamp).as_secs() < REQUEST_CACHE_TTL_SECS {
                log::warn!("Request {} already processed, ignoring to prevent loop", request_id);
                return true;
            }
            // Expired, remove old entry
            cache.remove(request_id);
        }

        // Mark as processed
        cache.insert(request_id.to_string(), now);

        // Periodically clean up expired entries (every 100 requests or so)
        if cache.len() > 100 {
            cache.retain(|_, &mut timestamp| {
                now.duration_since(timestamp).as_secs() < REQUEST_CACHE_TTL_SECS
            });
        }

        false
    } else {
        // Cache not initialized, allow request
        false
    }
}

/// Server state
#[derive(Clone)]
pub struct ServerState {
    pub device_id: String,
    pub device_name: String,
    pub port: i32,
}

/// Get the actual port the server is running on
/// 从全局配置获取端口
pub async fn get_actual_port() -> i32 {
    super::get_cluster_setting().await.port
}

/// Check if a port is available for binding (with exclusive access)
/// 检查端口是否可用于绑定（独占模式）
///
/// On Windows, uses SO_EXCLUSIVEADDRUSE to ensure exclusive port binding.
/// 在 Windows 上使用 SO_EXCLUSIVEADDRUSE 确保端口独占绑定。
pub async fn is_port_available(port: i32) -> bool {
    create_exclusive_socket(port).is_ok()
}

/// Create a socket with exclusive address use
/// 创建独占地址的 socket
fn create_exclusive_socket(port: i32) -> Result<tokio::net::TcpListener, std::io::Error> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port as u16));

    // Create socket with socket2
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    // On Windows, set SO_EXCLUSIVEADDRUSE to prevent other sockets from binding to the same port
    // 在 Windows 上设置 SO_EXCLUSIVEADDRUSE 防止其他 socket 绑定同一端口
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawSocket;
        use windows_sys::Win32::Networking::WinSock::{setsockopt, SO_EXCLUSIVEADDRUSE, SOL_SOCKET};

        socket.set_reuse_address(false)?;

        // Set SO_EXCLUSIVEADDRUSE via Windows API
        let raw_socket = socket.as_raw_socket() as usize;
        let exclusive: i32 = 1;
        let result = unsafe {
            setsockopt(
                raw_socket,
                SOL_SOCKET,
                SO_EXCLUSIVEADDRUSE,
                &exclusive as *const i32 as *const u8,
                std::mem::size_of::<i32>() as i32,
            )
        };

        if result != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to set SO_EXCLUSIVEADDRUSE",
            ));
        }
    }

    // Bind the socket
    socket.bind(&addr.into())?;
    socket.listen(128)?;

    // Convert to tokio TcpListener
    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    tokio::net::TcpListener::from_std(std_listener)
}

/// Start HTTP server with configured port
/// 启动 HTTP 服务器，使用配置的端口
///
/// Port is loaded from database (cluster_setting.port).
/// If binding fails, returns error and notifies frontend.
/// 端口从数据库加载（cluster_setting.port）。
/// 如果绑定失败，返回错误并通知前端。
///
/// Uses SO_EXCLUSIVEADDRUSE on Windows to ensure exclusive port binding.
/// 在 Windows 上使用 SO_EXCLUSIVEADDRUSE 确保端口独占绑定。
pub async fn start_http_server() -> Result<(), String> {
    // Check if server is already running
    // 检查服务器是否已经在运行
    if let Ok(guard) = SHUTDOWN_SENDER.read() {
        if guard.is_some() {
            log::info!("HTTP server is already running, skip start");
            return Ok(());
        }
    }

    // Load port from database
    let setting = super::get_cluster_setting().await;
    let port = if setting.port > 0 {
        setting.port
    } else {
        DEFAULT_PORT
    };

    let device_id = CLIENT_ID.read().await.clone();
    let device_name = if setting.device_name.is_empty() {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "MangoFinder".to_string())
    } else {
        setting.device_name
    };

    // Try binding to the configured port with exclusive access
    // 使用独占模式绑定端口
    let addr = SocketAddr::from(([0, 0, 0, 0], port as u16));
    let listener = match create_exclusive_socket(port) {
        Ok(l) => l,
        Err(e) => {
            let error_msg = format!("Port {} is already in use: {}", port, e);
            log::error!("Failed to bind port {}: {}", port, e);

            // Notify frontend about port binding failure (user-friendly message)
            // 通知前端端口绑定失败（用户友好的信息）
            if let Some(app_handle) = APP_HANDLE.get() {
                if let Err(emit_err) = app_handle.emit("cluster-port-error", &serde_json::json!({
                    "port": port
                })) {
                    log::error!("Failed to emit cluster-port-error event: {}", emit_err);
                }
            }

            return Err(error_msg);
        }
    };

    // Update cluster port in database and global state (ensures consistency)
    if let Err(e) = super::update_cluster_port(port).await {
        log::error!("Failed to update cluster port: {}", e);
    }

    log::info!("HTTP server listening on {}", addr);

    let state = Arc::new(ServerState {
        device_id,
        device_name,
        port,
    });

    let app = Router::new()
        .route("/ping", get(handle_ping))
        .route("/search", post(handle_search))
        .route("/chunks", post(handle_chunks))
        .route("/file/:file_id/cluster_similars", post(handle_find_similars))
        .route("/file/:file_id", get(handle_file_download))
        .route("/file/:file_id/content", get(handle_file_content))
        .route("/pairing/request", post(handle_pairing_request))
        .route("/pairing/respond", post(handle_pairing_respond))
        .route("/pairing/reset_notify", post(handle_reset_notify))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port as u16));
    log::info!("HTTP server listening on {}", addr);

    // Setup shutdown channel
    let (shutdown_sender, mut shutdown_receiver) = watch::channel(false);
    if let Ok(mut guard) = SHUTDOWN_SENDER.write() {
        *guard = Some(shutdown_sender);
    } else {
        log::error!("Failed to write SHUTDOWN_SENDER: lock poisoned");
    }

    // Spawn the server in a background task so this function returns immediately
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async move {
                shutdown_receiver.changed().await.ok();
                log::info!("HTTP server shutdown signal received");
            })
            .await
        {
            log::error!("HTTP server error: {}", e);
        }
    });

    Ok(())
}

/// Stop HTTP server
pub async fn stop_http_server() {
    // Send shutdown signal
    if let Ok(guard) = SHUTDOWN_SENDER.read() {
        if let Some(sender) = guard.as_ref() {
            let _ = sender.send(true);
        }
    }
    // Note: Don't clear SHUTDOWN_SENDER here, let it be cleared when server actually stops
    // 注意：不要在这里清空 SHUTDOWN_SENDER，让服务器实际停止时再清空
}

/// GET /ping - Health check and device info (merged with /device/info)
/// GET /ping - 健康检查和设备信息（合并了 /device/info）
async fn handle_ping(State(state): State<Arc<ServerState>>) -> CommonResponse<PingData> {
    log::info!("GET /ping");
    let index_count = file_info_repo::count().unwrap_or(0);
    let capabilities = DeviceCapabilities::default();

    CommonResponse::success(PingData {
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
        status: "online".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        index_count,
        last_index_time: None,
        capabilities,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

/// POST /search - Remote search request
async fn handle_search(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(req): Json<RemoteSearchRequest>,
) -> CommonResponse<SearchData> {
    log::info!("POST /search query={}, type={}, limit={}", req.query, req.search_type, req.limit);
    // Verify requester is paired
    let _requester_id: &str = match verify_paired_device(&headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Execute local search
    let results = match req.search_type.as_str() {
        "keyword" => searcher::keyword_search(&req.query).await,
        _ => {
            // Default to semantic search
            searcher::semantic_search(&req.query).await
        }
    };

    // Limit results
    let results: Vec<_> = results.into_iter().take(req.limit).collect();
    let total = results.len();

    CommonResponse::success(SearchData {
        results,
        total,
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
    })
}

/// POST /chunks - Get text chunks by IDs for remote devices
/// POST /chunks - 根据ID获取文本片段（供远程设备调用）
async fn handle_chunks(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Json(req): Json<ChunksRequest>,
) -> CommonResponse<ChunksData> {
    log::info!("POST /chunks ids={:?}", req.ids);
    // Verify requester is paired
    let _requester_id: &str = match verify_paired_device(&headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Get chunks by IDs
    let chunks = match file_content_embedding_repo::list_chunks_by_ids(&req.ids) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get chunks: {}", e);
            return CommonResponse::error(CODE_INTERNAL_ERROR, "Failed to get chunks");
        }
    };

    CommonResponse::success(ChunksData {
        chunks,
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
    })
}

/// POST /file/:file_id/cluster_similars - Find similar files across devices with dual loop prevention
/// POST /file/:file_id/cluster_similars - 跨设备查找相似文件（双重防循环机制）
///
/// This endpoint is called by remote devices to find similar files.
/// It searches locally and forwards the request to other devices.
///
/// Dual loop prevention (双重保证防止死循环):
/// 1. exclude_device_ids: Devices to skip when forwarding requests
///    排除列表：转发请求时跳过的设备
/// 2. request_id: Unique ID cached for 10 minutes to prevent reprocessing
///    请求ID：缓存10分钟，防止重复处理同一请求
async fn handle_find_similars(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Path(file_id): Path<i64>,
    Json(req): Json<RemoteFindSimilarsRequest>,
) -> CommonResponse<FindSimilarsData> {
    log::info!("POST /file/{}/cluster_similars request_id={}, limit={}", file_id, req.request_id, req.limit);

    // Verify requester is paired first
    let requester_id: &str = match verify_paired_device(&headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    // Check if this request has already been processed (prevent loops)
    // 检查此请求是否已被处理过（防止循环）
    if check_and_mark_request(&req.request_id) {
        log::warn!(
            "Request {} already processed, returning empty results to prevent loop",
            req.request_id
        );
        return CommonResponse::success(FindSimilarsData {
            results: vec![],
            total: 0,
            device_id: state.device_id.clone(),
            device_name: state.device_name.clone(),
        });
    }

    // Get file info
    let file_info = match file_info_repo::get_by_id(file_id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            log::warn!("File not found: {}", file_id);
            return CommonResponse::error(CODE_NOT_FOUND, &t!("message.file-not-found").to_string());
        }
        Err(e) => {
            log::error!("Failed to get file: {}", e);
            return CommonResponse::error(CODE_INTERNAL_ERROR, &t!("message.internal-error").to_string());
        }
    };

    let limit = req.limit;

    // Build exclude list: add ourselves and the requester
    let mut exclude_device_ids = req.exclude_device_ids.clone();
    exclude_device_ids.push(state.device_id.clone());
    if !requester_id.is_empty() && !exclude_device_ids.contains(&requester_id.to_string()) {
        exclude_device_ids.push(requester_id.to_string());
    }

    // 1. Search locally
    let local_results = match crate::similarity::local_similarity_service::find_similars_by_file_id(&file_info, limit).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to find local similar files: {}", e);
            return CommonResponse::error(CODE_INTERNAL_ERROR, &t!("message.internal-error").to_string());
        }
    };

    // 2. Search on other remote devices (with updated exclude list)
    //    Pass the same request_id to prevent loops
    let remote_results = crate::similarity::remote_similarity_service::remote_find_similars_with_exclude(
        &file_info,
        limit,
        &exclude_device_ids,
        &req.request_id,
    ).await;

    // 3. Merge results
    let mut all_results = local_results;
    all_results.extend(remote_results);
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(limit);
    let total = all_results.len();

    CommonResponse::success(FindSimilarsData {
        results: all_results,
        total,
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
    })
}

/// GET /file/:file_id - Download file from this device by file ID
/// Only indexed files can be downloaded (security measure)
async fn handle_file_download(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Path(file_id): Path<i64>,
) -> Result<Response<Body>, StatusCode> {
    log::info!("GET /file/{}", file_id);
    // Verify requester is paired
    let requester_id = verify_paired_device_status(&headers)?;

    // Get file info from database by file_id (only indexed files)
    let file_info = file_info_repo::get_by_id(file_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_path = &file_info.path;

    log::info!(
        "Remote file request from {}: {} (id: {})",
        requester_id,
        file_path,
        file_id
    );

    // Read file data
    let file_data = std::fs::read(file_path).map_err(|e| {
        log::error!("Failed to read file {}: {}", file_path, e);
        StatusCode::NOT_FOUND
    })?;

    // Determine content type based on extension
    let content_type = std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "json" => "application/json",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            _ => "application/octet-stream",
        })
        .unwrap_or("application/octet-stream");

    // Build response with file data
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_info.name),
        )
        .header("X-Device-Id", &state.device_id)
        .header("X-Device-Name", &state.device_name)
        .body(Body::from(file_data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

/// GET /file/:file_id/content - Get file content/parsed content by file ID for remote devices
/// GET /file/:file_id/content - 获取文件内容/解析内容（供远程设备调用）
async fn handle_file_content(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    Path(file_id): Path<i64>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /file/{}/content", file_id);
    // Verify requester is paired
    let requester_id = verify_paired_device_status(&headers)?;

    // Get file info from database
    let file_info = file_info_repo::get_by_id(file_id)
        .map_err(|e| {
            log::error!("Failed to get file info: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "code": 0,
        "data": {
            "content": file_info.content,
            "deviceId": state.device_id,
            "deviceName": state.device_name
        }
    })))
}

/// Notify frontend about pairing request handling result
fn notify_pairing_handled(payload: &PairingRequestPayload, status: &str) {
    if let Some(app_handle) = APP_HANDLE.get() {
        let _ = app_handle.emit(
            "pairing-request-handled",
            json!({
                "status": status,
                "device_id": payload.device_id,
                "device_name": payload.device_name,
                "ip_address": payload.ip_address,
                "port": payload.port
            }),
        );
    }
}

/// Check if device already exists and return its status
/// Returns Some(response) if device exists and should be handled early
fn check_existing_device(
    payload: &PairingRequestPayload,
) -> Option<Result<Json<Value>, StatusCode>> {
    let device = device_repo::get_by_device_id(&payload.device_id).ok()??;

    match device.pairing_status {
        PairingStatus::Paired => {
            // Log the request as already_paired
            let request = PairingRequest {
                device_id: payload.device_id.clone(),
                device_name: payload.device_name.clone(),
                ip_address: payload.ip_address.clone(),
                port: payload.port,
                direction: "in".to_string(),
                status: PairingRequestStatus::Accepted,
                remark: t!("pairing.remark.already-paired").to_string(),
                ..Default::default()
            };
            let _ = pairing_request_repo::insert(&request);

            notify_pairing_handled(payload, "already_paired");
            Some(Ok(Json(json!({
                "code": 0,
                "data": {
                    "status": PairingResponseStatus::AlreadyPaired,
                    "message": "Device is already paired"
                }
            }))))
        }
        PairingStatus::Rejected => {
            log::info!(
                "Pairing request from rejected device {}, auto-rejecting",
                payload.device_id
            );

            // Log the request as auto_rejected
            let request = PairingRequest {
                device_id: payload.device_id.clone(),
                device_name: payload.device_name.clone(),
                ip_address: payload.ip_address.clone(),
                port: payload.port,
                direction: "in".to_string(),
                status: PairingRequestStatus::AutoRejected,
                remark: t!("pairing.remark.auto-rejected").to_string(),
                ..Default::default()
            };
            let _ = pairing_request_repo::insert(&request);

            notify_pairing_handled(payload, "auto_rejected");
            Some(Ok(Json(json!({
                "code": 0,
                "data": {
                    "status": PairingResponseStatus::Rejected,
                    "message": "Device is rejected"
                }
            }))))
        }
        _ => None,
    }
}

/// Handle auto-pair scenario when both sides sent requests to each other
/// Returns Some(response) if auto-pair is applicable
fn handle_auto_pair(payload: &PairingRequestPayload) -> Option<Result<Json<Value>, StatusCode>> {
    let out_request =
        pairing_request_repo::get_pending_by_device_id(&payload.device_id, "out").ok()??;

    log::info!(
        "Auto-pairing: both sides sent requests to each other, accepting {}",
        payload.device_id
    );

    // Update device to paired status (automatic process)
    if let Ok(Some(device)) = device_repo::get_by_device_id(&payload.device_id) {
        let remark = t!("pairing.remark.auto-paired");
        let _ = device_repo::update_pairing_status_with_remark(device.id, PairingStatus::Paired, &remark, false);
    }

    // Update outgoing request to accepted
    let _ = pairing_request_repo::accept(out_request.id);

    // Log incoming request as accepted
    let request = PairingRequest {
        device_id: payload.device_id.clone(),
        device_name: payload.device_name.clone(),
        ip_address: payload.ip_address.clone(),
        port: payload.port,
        direction: "in".to_string(),
        status: PairingRequestStatus::Accepted,
        remark: t!("pairing.remark.auto-paired").to_string(),
        ..Default::default()
    };
    let _ = pairing_request_repo::insert(&request);

    notify_pairing_handled(payload, "auto_accepted");
    Some(Ok(Json(json!({
        "code": 0,
        "data": {
            "status": PairingResponseStatus::Approved,
            "message": "Auto-paired: both sides requested pairing"
        }
    }))))
}

/// Handle auto-accept scenario when auto_accept_pairing is enabled
/// Returns Some(response) if auto-accept is applicable
async fn handle_auto_accept(
    payload: &PairingRequestPayload,
) -> Option<Result<Json<Value>, StatusCode>> {
    let config = super::get_cluster_setting().await;
    if !config.auto_accept_pairing {
        return None;
    }

    // Auto-accept
    let device = Device {
        device_id: payload.device_id.clone(),
        name: payload.device_name.clone(),
        ip_address: payload.ip_address.clone(),
        port: payload.port,
        online_status: OnlineStatus::Online,
        pairing_status: PairingStatus::Paired,
        discovery_method: "mdns".to_string(),
        ..Default::default()
    };
    let inserted_device = device_repo::upsert(&device).ok()?;

    // Update pairing status with remark (upsert doesn't set pairing_remark)
    let remark = t!("pairing.remark.auto-accepted");
    let _ = device_repo::update_pairing_status_with_remark(inserted_device.id, PairingStatus::Paired, &remark, false);

    // Log the request
    let request = PairingRequest {
        device_id: payload.device_id.clone(),
        device_name: payload.device_name.clone(),
        ip_address: payload.ip_address.clone(),
        port: payload.port,
        direction: "in".to_string(),
        status: PairingRequestStatus::Accepted,
        remark: t!("pairing.remark.auto-accepted").to_string(),
        ..Default::default()
    };
    let _ = pairing_request_repo::insert(&request);

    notify_pairing_handled(payload, "auto_accepted");
    Some(Ok(Json(json!({
        "code": 0,
        "data": {
            "status": PairingResponseStatus::Approved,
            "message": "Pairing auto-approved"
        }
    }))))
}

/// Handle pending request scenario - create pending request and notify frontend
fn handle_pending_request(payload: &PairingRequestPayload) -> Result<Json<Value>, StatusCode> {
    // Log incoming pairing request
    let request = PairingRequest {
        device_id: payload.device_id.clone(),
        device_name: payload.device_name.clone(),
        ip_address: payload.ip_address.clone(),
        port: payload.port,
        direction: "in".to_string(),
        status: PairingRequestStatus::Pending,
        remark: t!("pairing.remark.waiting-approval").to_string(),
        ..Default::default()
    };
    pairing_request_repo::insert(&request).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create/update device record with pending_in status
    let device = Device {
        device_id: payload.device_id.clone(),
        name: payload.device_name.clone(),
        ip_address: payload.ip_address.clone(),
        port: payload.port,
        online_status: OnlineStatus::Online,
        pairing_status: PairingStatus::PendingIn,
        discovery_method: "mdns".to_string(),
        ..Default::default()
    };
    let upserted_device = device_repo::upsert(&device).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update pairing status with remark (upsert doesn't set pairing_remark)
    let remark = t!("pairing.remark.waiting-approval").to_string();
    let _ = device_repo::update_pairing_status_with_remark(upserted_device.id, PairingStatus::PendingIn, &remark, false);

    // Notify frontend about pending request
    notify_pairing_handled(payload, "pending");

    Ok(Json(json!({
        "code": 0,
        "data": {
            "status": PairingResponseStatus::Pending,
            "message": "Pairing request received, waiting for approval"
        }
    })))
}

/// POST /pairing/request - Handle incoming pairing request
async fn handle_pairing_request(
    State(_state): State<Arc<ServerState>>,
    Json(payload): Json<PairingRequestPayload>,
) -> Result<Json<Value>, StatusCode> {
    log::info!(
        "Received pairing request from {} ({})",
        payload.device_name,
        payload.ip_address
    );

    // 1. Check if device already exists (paired or rejected)
    if let Some(result) = check_existing_device(&payload) {
        return result;
    }

    // 2. Check for auto-pair scenario (both sides sent requests)
    if let Some(result) = handle_auto_pair(&payload) {
        return result;
    }

    // 3. Check if auto_accept_pairing is enabled
    if let Some(result) = handle_auto_accept(&payload).await {
        return result;
    }

    // 4. Default: create pending request
    handle_pending_request(&payload)
}

/// POST /pairing/respond - Handle pairing response (from remote device)
async fn handle_pairing_respond(
    Json(payload): Json<PairingResponsePayload>,
) -> Result<Json<Value>, StatusCode> {
    log::info!(
        "Received pairing response from {}: approved={}",
        payload.responder_id,
        payload.approved
    );

    // Update device pairing status - use responder_id to find the remote device
    if let Ok(Some(device)) = device_repo::get_by_device_id(&payload.responder_id) {
        let (new_status, remark) = if payload.approved {
            (PairingStatus::Paired, t!("pairing.remark.remote-accepted").to_string())
        } else {
            // Remote device rejected our request, mark as blocked
            // 对方拒绝了本机的请求，标记为 blocked
            (PairingStatus::Blocked, t!("pairing.remark.remote-rejected").to_string())
        };
        log::info!(
            "Updating device pairing status to {:?}",
            new_status
        );
        device_repo::update_pairing_status_with_remark(device.id, new_status, &remark, false)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Update the latest request status (any direction) - use responder_id to find the request
    // 更新最新的请求状态（不限方向）- 使用 responder_id 查找请求
    if let Ok(Some(latest_request)) =
        pairing_request_repo::get_latest_by_device_id_any(&payload.responder_id)
    {
        let (status, remark) = if payload.approved {
            (PairingRequestStatus::Accepted, t!("pairing.remark.remote-accepted").to_string())
        } else {
            (PairingRequestStatus::Rejected, t!("pairing.remark.remote-rejected").to_string())
        };
        let _ = pairing_request_repo::update_status_with_remark(latest_request.id, status, &remark);
    }

    // Notify frontend about pairing response
    if let Some(app_handle) = APP_HANDLE.get() {
        let _ = app_handle.emit("pairing-response-received", &payload);
    }

    Ok(Json(json!({
        "code": 0,
        "data": {
            "status": if payload.approved { "approved" } else { "rejected" }
        }
    })))
}

/// POST /pairing/reset_notify - Handle reset notification from remote device
/// POST /pairing/reset_notify - 处理远程设备的重置通知
///
/// When a remote device resets its pairing status, it notifies this device.
/// This device should also reset its pairing status for that device.
/// 当远程设备重置配对状态时，它会通知本设备。
/// 本设备也应该重置该设备的配对状态。
async fn handle_reset_notify(
    Json(payload): Json<ResetNotifyPayload>,
) -> Result<Json<Value>, StatusCode> {
    log::info!(
        "Received reset notify from {} (previous status: {})",
        payload.from_device_id,
        payload.previous_status
    );

    // Get local device record for this remote device
    // 获取该远程设备的本机记录
    if let Ok(Some(device)) = device_repo::get_by_device_id(&payload.from_device_id) {
        // Reset pairing status to None with remark
        // 重置配对状态为 None 并附带说明
        // Note: is_manual=true because remote reset is triggered by user action on remote device
        // 注意: is_manual=true 因为远程重置是由对方设备上的用户操作触发的
        let remark = t!("pairing.remark.remote-reset").to_string();
        device_repo::update_pairing_status_with_remark(device.id, PairingStatus::None, &remark, true)
            .map_err(|e| {
                log::error!("Failed to update pairing status for device {}: {:?}", payload.from_device_id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Update the latest pairing request record for this device
        // 更新该设备最新的配对请求记录
        if let Ok(Some(latest_request)) =
            pairing_request_repo::get_latest_by_device_id_any(&payload.from_device_id)
        {
            let _ = pairing_request_repo::update_status_with_remark(
                latest_request.id,
                PairingRequestStatus::Cancelled,
                &remark,
            );
        }

        // Notify frontend about pairing reset
        if let Some(app_handle) = APP_HANDLE.get() {
            let _ = app_handle.emit("pairing-reset-received", &payload);
        }

        log::info!(
            "Reset pairing status for device {} to None (was notified by remote)",
            payload.from_device_id
        );
    }

    Ok(Json(json!({
        "code": 0,
        "data": {
            "status": "ok"
        }
    })))
}
