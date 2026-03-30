use serde::{Deserialize, Serialize};

/// Cluster configuration for multi-device connectivity
/// 集群配置 - 多机互联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSetting {
    /// Whether cluster feature is enabled
    /// 是否启用集群功能
    pub enabled: bool,
    /// HTTP service port
    /// HTTP 服务端口
    pub port: i32,
    /// Device display name (empty = use hostname)
    /// 设备显示名称（空=使用主机名）
    pub device_name: String,
    /// Whether to allow other devices to discover this device
    /// 是否允许其他设备发现自己
    pub allow_to_be_discovered: bool,
    /// Whether to auto send pairing request to newly discovered devices
    /// 是否自动向新发现的设备发送配对请求
    pub auto_request_pairing: bool,
    /// Whether to auto accept pairing requests
    /// 是否自动接受配对请求
    pub auto_accept_pairing: bool,
    /// Interval for checking device online status (in seconds)
    /// 设备在线状态检测间隔（秒）
    pub online_check_interval: i32,
}

impl Default for ClusterSetting {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 7890,
            device_name: "".to_string(),
            allow_to_be_discovered: true,
            auto_request_pairing: false,
            auto_accept_pairing: false,
            online_check_interval: 30,
        }
    }
}

impl ClusterSetting {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json_string(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

/// Device capabilities - what types of search the device supports
/// 设备能力 - 设备支持的搜索类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// Supports text/document search
    /// 支持文本/文档搜索
    pub text: bool,
    /// Supports image search
    /// 支持图片搜索
    pub image: bool,
    /// Supports audio search
    /// 支持音频搜索
    pub audio: bool,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            text: true,
            image: true,
            audio: true,
        }
    }
}

impl DeviceCapabilities {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json_string(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

/// Remote search request
/// 远程搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchRequest {
    /// Search query
    /// 搜索查询
    pub query: String,
    /// Search type: "keyword", "semantic", "hybrid"
    /// 搜索类型
    #[serde(default = "default_search_type")]
    pub search_type: String,
    /// Maximum results to return
    /// 最大返回结果数
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_search_type() -> String {
    "semantic".to_string()
}

fn default_limit() -> usize {
    20
}

/// Remote search response
/// 远程搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchResponse {
    /// Search results
    /// 搜索结果
    pub results: Vec<crate::structs::search_result::SearchResult>,
    /// Source device ID
    /// 来源设备 ID
    pub source_device_id: String,
    /// Source device name
    /// 来源设备名称
    pub source_device_name: String,
}

/// Device info response for /ping endpoint
/// /ping 端点的设备信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoResponse {
    /// Device ID (UUID)
    /// 设备 ID
    pub device_id: String,
    /// Device name
    /// 设备名称
    pub device_name: String,
    /// Application version
    /// 应用版本
    pub version: String,
    /// Number of indexed files
    /// 索引文件数量
    pub index_count: i64,
    /// Last indexing time
    /// 最后索引时间
    pub last_index_time: Option<String>,
    /// Device capabilities
    /// 设备能力
    pub capabilities: DeviceCapabilities,
}

/// Pairing request payload
/// 配对请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequestPayload {
    /// Requester's device ID
    /// 请求者设备 ID
    pub device_id: String,
    /// Requester's device name
    /// 请求者设备名称
    pub device_name: String,
    /// Requester's IP address
    /// 请求者 IP 地址
    pub ip_address: String,
    /// Requester's service port
    /// 请求者服务端口
    pub port: i32,
}

/// Pairing response payload
/// 配对响应载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingResponsePayload {
    /// Requester's device ID (who sent the original request)
    /// 请求者设备 ID（发起请求的设备）
    pub requester_id: String,
    /// Responder's device ID (who is responding)
    /// 响应者设备 ID（回复的设备）
    pub responder_id: String,
    /// Whether approved
    /// 是否批准
    pub approved: bool,
}

/// Reset notify payload
/// 重置通知载荷
///
/// Sent when a device resets its pairing status to notify the other party
/// 当设备重置配对状态时发送，通知对方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetNotifyPayload {
    /// Device ID of the sender
    /// 发送方设备 ID
    pub from_device_id: String,
    /// Previous pairing status before reset
    /// 重置前的配对状态
    pub previous_status: String,
}

/// Remote find similars request
/// 远程相似文件查找请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFindSimilarsRequest {
    /// Unique request ID to prevent loop (UUID)
    /// 唯一请求ID，防止循环（UUID）
    pub request_id: String,
    /// Device IDs to exclude from search (to prevent loops)
    /// 排除的设备 ID 列表（防止循环）
    #[serde(default)]
    pub exclude_device_ids: Vec<String>,
    /// Maximum results to return
    /// 最大返回结果数
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Remote find similars response
/// 远程相似文件查找响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteFindSimilarsResponse {
    /// Search results
    /// 搜索结果
    pub results: Vec<crate::structs::search_result::SearchResult>,
    /// Total count
    /// 总数
    pub total: usize,
}
