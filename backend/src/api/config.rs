use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    state::StateManager,
    types::{ApplyConfigRequest, Config},
};

pub async fn get_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<Config>> {
    let config = config_manager.get_config().await?;
    Ok(Json(config))
}

pub async fn post_handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
    Json(request): Json<ApplyConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    // 使用 StateManager 进行事务性配置应用
    state_manager.apply_config(request.config, request.message).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Config applied successfully",
    })))
}