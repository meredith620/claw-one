use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
};

/// 获取 Memory 配置
pub async fn get_memory(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let memory = config_manager.get_memory().await?;
    Ok(Json(memory))
}

/// 保存 Memory 配置
pub async fn save_memory(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    config_manager.save_memory(&data).await?;
    
    // 创建 Git 提交
    config_manager.ensure_git_repo().await?;
    config_manager.git_add(".").await?;
    if config_manager.has_changes().await? {
        let commit_id = config_manager.git_commit("Update memory config").await?;
        return Ok(Json(serde_json::json!({
            "success": true,
            "commit": commit_id,
        })));
    }
    
    Ok(Json(serde_json::json!({"success": true})))
}
