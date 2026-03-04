use axum::{extract::Extension, Json};
use std::sync::Arc;

use crate::{
    error::Result,
    state::StateManager,
    types::StateResponse,
};

pub async fn handler(
    Extension(state_manager): Extension<Arc<StateManager>>,
) -> Result<Json<StateResponse>> {
    let response = state_manager.get_state_response().await?;
    Ok(Json(response))
}