use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    error::Result,
    state::{AppState, StateManager},
};

pub async fn handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
) -> Result<Json<serde_json::Value>> {
    let current_state = state_manager.get_state().await;

    match current_state {
        AppState::SafeMode { .. } => {
            // 在 SafeMode 下，执行恢复逻辑
            state_manager.recover_from_safe_mode().await?;
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Service recovered from SafeMode and restarted",
            })))
        }
        _ => {
            // 正常运行时，执行普通重启
            state_manager.restart_service().await?;
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Service restarted successfully",
            })))
        }
    }
}
