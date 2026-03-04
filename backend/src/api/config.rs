use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    types::{ApplyConfigRequest, Config},
};

pub async fn get_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<Config>> {
    let config = config_manager.get_config().await?;
    Ok(Json(config))
}

pub async fn post_handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(request): Json<ApplyConfigRequest>,
) -> Result<Json<serde_json::Value>> {
    // 应用配置并创建 Git 提交
    let commit_id = config_manager.apply_config(request.config, request.message).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Config applied successfully",
        "commit_id": commit_id,
    })))
}
