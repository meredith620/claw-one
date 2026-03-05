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
mod settings;
mod state;
mod types;

use config::ConfigManager;
use settings::Settings;
use state::StateManager;

#[tokio::main]
async fn main() {
    // 加载配置（在初始化日志前，以便使用配置中的日志级别）
    let settings = match Settings::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            eprintln!("   请确保配置文件存在且格式正确");
            eprintln!("   配置文件路径可通过 CLAW_ONE_CONFIG 环境变量指定");
            std::process::exit(1);
        }
    };

    // 初始化日志
    let log_filter = format!(
        "claw_one={},tower_http=debug",
        settings.server.log_level
    );
    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .init();

    info!("Starting Claw One backend v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from: {:?}", std::env::var("CLAW_ONE_CONFIG").unwrap_or_else(|_| "default".to_string()));
    info!("Server will listen on {}:{}", settings.server.host, settings.server.port);
    info!("Managing OpenClaw service: {}", settings.openclaw.service_name);

    // 确保单实例
    if let Err(e) = ensure_single_instance() {
        eprintln!("Failed to start: {}", e);
        std::process::exit(1);
    }

    // 初始化配置管理器
    let config_manager = Arc::new(ConfigManager::new());
    
    // 初始化状态管理器（使用配置）
    let state_manager = Arc::new(StateManager::new(config_manager.clone(), &settings.openclaw));

    // 获取静态文件目录
    let static_dir = settings.static_dir();
    let static_dir_str = static_dir.to_string_lossy().to_string();
    info!("Static files directory: {}", static_dir_str);

    // 检查静态文件目录是否存在
    if !static_dir.exists() {
        tracing::warn!("Static files directory does not exist: {}", static_dir_str);
        tracing::warn!("Web UI will not be available. Please build the frontend first.");
    }

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
        .fallback_service(tower_http::services::ServeDir::new(static_dir))
        // 共享状态
        .layer(axum::extract::Extension(config_manager))
        .layer(axum::extract::Extension(state_manager));

    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>().expect("Invalid host address"),
        settings.server.port
    ));
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
