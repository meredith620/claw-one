use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    error::Result,
    state::StateManager,
};

pub async fn handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
) -> Result<Json<serde_json::Value>> {
    // 手动触发服务重启
    state_manager.recover_from_safe_mode().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Service restarted successfully",
    })))
}