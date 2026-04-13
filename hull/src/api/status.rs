use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{error::Result, state::StateManager, types::StatusResponse};

/// GET /api/status - 获取 Runtime 状态
///
/// 返回 OpenClaw Runtime 的运行状态、健康状态和进程信息
pub async fn handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
) -> Result<Json<StatusResponse>> {
    let response = state_manager.get_runtime_status_response().await?;
    Ok(Json(response))
}
