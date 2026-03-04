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
