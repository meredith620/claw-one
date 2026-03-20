use axum::{
    extract::Extension,
    Json,
};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
};

/// 获取 Agent 配置
pub async fn get_agents(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<serde_json::Value>> {
    let agents = config_manager.get_agents().await?;
    Ok(Json(agents))
}

/// 保存 Agent 配置
pub async fn save_agents(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    config_manager.save_agents(&data).await?;
    
    // 创建 Git 提交
    if let Err(e) = config_manager.ensure_git_repo().await {
        tracing::error!("Git ensure_git_repo failed: {}", e);
        return Err(e);
    }
    
    if let Err(e) = config_manager.git_add(".").await {
        tracing::error!("Git git_add failed: {}", e);
        return Err(e);
    }
    
    match config_manager.has_changes().await {
        Ok(true) => {
            match config_manager.git_commit("Update agent config").await {
                Ok(commit_id) => {
                    return Ok(Json(serde_json::json!({
                        "success": true,
                        "commit": commit_id,
                    })));
                }
                Err(e) => {
                    tracing::error!("Git git_commit failed: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(false) => {
            return Ok(Json(serde_json::json!({"success": true})));
        }
        Err(e) => {
            tracing::error!("Git has_changes failed: {}", e);
            return Err(e);
        }
    }
}
