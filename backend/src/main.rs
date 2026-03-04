use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

mod api;
mod config;
mod error;
mod runtime;
mod state;
mod types;

use config::ConfigManager;
use state::StateManager;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("claw_one=debug,tower_http=debug")
        .init();

    info!("Starting Claw One backend v{}", env!("CARGO_PKG_VERSION"));

    // 确保单实例
    if let Err(e) = ensure_single_instance() {
        eprintln!("Failed to start: {}", e);
        std::process::exit(1);
    }

    // 初始化配置管理器
    let config_manager = Arc::new(ConfigManager::new());
    
    // 初始化状态管理器
    let state_manager = Arc::new(StateManager::new(config_manager.clone()));

    // 构建路由
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/state", get(api::state::handler))
        .route("/api/config", get(api::config::get_handler).post(api::config::post_handler))
        .route("/api/snapshots", get(api::snapshots::handler))
        .route("/api/rollback", post(api::rollback::handler))
        .route("/api/logs", get(api::logs::handler))
        .route("/api/restart", post(api::restart::handler))
        // 首次启动向导相关 API
        .route("/api/setup/check", get(api::setup::check_handler))
        .route("/api/setup/complete", post(api::setup::complete_handler))
        .route("/api/setup/reset", post(api::setup::reset_handler))
        // 静态文件服务（Vue 构建产物）
        .fallback_service(tower_http::services::ServeDir::new("../static/dist"))
        // 共享状态
        .layer(axum::extract::Extension(config_manager))
        .layer(axum::extract::Extension(state_manager));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

fn ensure_single_instance() -> anyhow::Result<()> {
    use std::fs::OpenOptions;
    use fs2::FileExt;

    let lock_file = std::env::temp_dir().join("claw-one.lock");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&lock_file)?;

    match file.try_lock_exclusive() {
        Ok(()) => {
            // 保持文件句柄，进程结束时自动释放
            Box::leak(Box::new(file));
            Ok(())
        }
        Err(_) => Err(anyhow::anyhow!("Claw One is already running")),
    }
}