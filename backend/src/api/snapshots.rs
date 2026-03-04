use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    config::ConfigManager,
    error::Result,
    types::SnapshotsResponse,
};

pub async fn handler(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
) -> Result<Json<SnapshotsResponse>> {
    let snapshots = config_manager.list_snapshots().await?;
    
    Ok(Json(SnapshotsResponse { snapshots }))
}
