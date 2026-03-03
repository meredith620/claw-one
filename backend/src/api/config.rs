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
    // TODO: 实现配置应用流程
    // 1. 保存配置
    // 2. 重启 OpenClaw
    // 3. 健康检查
    // 4. 成功则 git commit，失败则回滚

    config_manager.apply_config(request.config, request.message).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Config applied successfully",
    })))
}
