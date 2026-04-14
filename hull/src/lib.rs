pub mod api;
pub mod config;
pub mod error;
pub mod runtime;
pub mod settings;
pub mod state;
pub mod types;
pub mod validation;

// Re-export for integration tests
pub use config::ConfigManager;
pub use settings::Settings;
pub use state::StateManager;

use axum::{
    response::Json,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

/// Health check handler
pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Build API router (without static file service)
/// Used for both testing and production
pub fn build_api_router(
    config_manager: Arc<ConfigManager>,
    state_manager: Arc<StateManager>,
) -> Router {
    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/state", get(api::state::handler))
        .route("/api/status", get(api::status::handler))
        // Config API
        .route(
            "/api/config",
            get(api::config::get_handler).post(api::config::post_handler),
        )
        .route(
            "/api/config/:module",
            get(api::config::get_module_handler).post(api::config::save_module_handler),
        )
        // Provider config API
        .route("/api/providers", get(api::providers::list_providers))
        .route(
            "/api/providers/verify",
            post(api::providers::verify_provider),
        )
        .route(
            "/api/providers/github-copilot/init",
            post(api::providers::github_copilot_init),
        )
        .route(
            "/api/providers/github-copilot/status",
            get(api::providers::github_copilot_status),
        )
        .route(
            "/api/providers/:id",
            get(api::providers::get_provider)
                .post(api::providers::save_provider)
                .delete(api::providers::delete_provider),
        )
        .route(
            "/api/model-priority",
            get(api::providers::get_model_priority).post(api::providers::save_model_priority),
        )
        // Agent config API
        .route(
            "/api/agents",
            get(api::agents::get_agents).post(api::agents::save_agents),
        )
        .route(
            "/api/agents/:id",
            delete(api::agents::delete_agent),
        )
        // Memory config API
        .route(
            "/api/memory",
            get(api::memory::get_memory).post(api::memory::save_memory),
        )
        // Channel config API
        .route(
            "/api/channels",
            get(api::channels::get_channels).post(api::channels::save_channels),
        )
        .route(
            "/api/channels/:channel_type/:account_id",
            delete(api::channels::delete_channel),
        )
        // Config validation API
        .route("/api/config/validate", post(api::config::validate_handler))
        .route("/api/snapshots", get(api::snapshots::handler))
        .route("/api/rollback", post(api::rollback::handler))
        .route("/api/logs", get(api::logs::handler))
        .route("/api/restart", post(api::restart::handler))
        .route("/api/setup/check", get(api::setup::check_handler))
        .route("/api/setup/complete", post(api::setup::complete_handler))
        .route("/api/setup/reset", post(api::setup::reset_handler))
        .layer(axum::extract::Extension(config_manager))
        .layer(axum::extract::Extension(state_manager))
}
