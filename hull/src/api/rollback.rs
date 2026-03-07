use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    types::RollbackRequest,
};

pub async fn handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(request): Json<RollbackRequest>,
) -> Result<Json<serde_json::Value>> {
    config_manager.rollback(&request.snapshot_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Rolled back successfully",
    })))
}
