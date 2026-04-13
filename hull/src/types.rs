use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenClawState {
    Running,
    Starting,
    ConfigError {
        error: String,
        auto_rolled_back: bool,
    },
    SystemError {
        error: String,
    },
    Stopped,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub state: OpenClawState,
    pub current_version: Option<String>,
    pub last_error: Option<String>,
    pub can_rollback: bool,
}

// 配置类型 - 使用灵活的结构来兼容实际的 openclaw.json
pub type Config = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotsResponse {
    pub snapshots: Vec<Snapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyConfigRequest {
    pub config: Config,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    pub snapshot_id: String,
}

// 模块配置相关类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub module: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInstance {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPriority {
    pub primary: String,
    pub fallbacks: Vec<String>,
}

// Runtime 状态响应（用于 API）
// 注意：这与 runtime.rs 中的 ServiceStatus 枚举不同
// 这里是 API 响应的序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum RuntimeStatusResponse {
    Running,
    Stopped,
    Starting,
    Stopping,
    Failed { message: String },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub service: RuntimeStatusResponse,
    pub healthy: bool,
    pub pid: Option<u32>,
}
