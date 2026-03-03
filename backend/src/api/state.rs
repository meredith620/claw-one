use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    types::{OpenClawState, StateResponse},
};

pub async fn handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<StateResponse>> {
    // TODO: 实现真实的状态检查
    // 1. 检查 OpenClaw 进程
    // 2. 检查健康状态
    // 3. 返回当前状态

    let response = StateResponse {
        state: OpenClawState::Running,
        current_version: Some("2024-01-01T00:00:00Z".to_string()),
        last_error: None,
        can_rollback: config_manager.can_rollback().await?,
    };

    Ok(Json(response))
}
