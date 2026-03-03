use axum::{
    routing::get,
    Router,
    response::Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{info, warn};

mod api;
mod config;
mod runtime;
mod state;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("claw_one=debug,tower_http=debug")
        .init();

    info!("Starting Claw One backend...");

    // 确保单实例
    if let Err(e) = ensure_single_instance() {
        eprintln!("Failed to start: {}", e);
        std::process::exit(1);
    }

    // 构建路由
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/state", get(api::state::handler))
        .route("/api/config", get(api::config::get_handler).post(api::config::post_handler))
        .route("/api/snapshots", get(api::snapshots::handler))
        .route("/api/rollback", post(api::rollback::handler))
        .route("/api/logs", get(api::logs::handler))
        // 静态文件服务（Vue 构建产物）
        .fallback_service(tower_http::services::ServeDir::new("../static/dist"));

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
