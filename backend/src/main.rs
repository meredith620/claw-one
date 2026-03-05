use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs;
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

/// Claw One - OpenClaw 配置守护程序
#[derive(Parser)]
#[command(name = "claw-one")]
#[command(about = "Claw One - OpenClaw Configuration Guardian")]
#[command(version)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 前台运行服务（默认）
    Run,
    /// 后台启动服务
    Start {
        /// 守护进程模式
        #[arg(short, long)]
        daemon: bool,
    },
    /// 停止后台服务
    Stop,
    /// 重启服务
    Restart,
    /// 查看服务状态
    Status,
    /// 配置开机自启（systemd user）
    Enable,
    /// 取消开机自启
    Disable,
    /// 查看配置
    Config,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 设置配置文件环境变量（如果指定）
    if let Some(config_path) = cli.config {
        std::env::set_var("CLAW_ONE_CONFIG", config_path);
    }

    // 执行子命令或默认运行
    match cli.command {
        Some(Commands::Run) | None => {
            run_server().await;
        }
        Some(Commands::Start { daemon }) => {
            start_service(daemon).await;
        }
        Some(Commands::Stop) => {
            stop_service().await;
        }
        Some(Commands::Restart) => {
            restart_service().await;
        }
        Some(Commands::Status) => {
            show_status().await;
        }
        Some(Commands::Enable) => {
            enable_autostart().await;
        }
        Some(Commands::Disable) => {
            disable_autostart().await;
        }
        Some(Commands::Config) => {
            show_config();
        }
    }
}

/// 运行服务器（前台）
async fn run_server() {
    // 加载配置
    let settings = match Settings::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            eprintln!("   请确保配置文件存在且格式正确");
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
    info!("Configuration: {:?}", std::env::var("CLAW_ONE_CONFIG").unwrap_or_else(|_| "default".to_string()));
    info!("Server: {}:{}", settings.server.host, settings.server.port);
    info!("OpenClaw service: {}", settings.openclaw.service_name);

    // 确保单实例
    if let Err(e) = ensure_single_instance() {
        eprintln!("Failed to start: {}", e);
        std::process::exit(1);
    }

    // 初始化配置管理器
    let config_manager = std::sync::Arc::new(ConfigManager::new());
    
    // 初始化状态管理器
    let state_manager = std::sync::Arc::new(StateManager::new(config_manager.clone(), &settings.openclaw));

    // 获取静态文件目录
    let static_dir = settings.static_dir();
    let static_dir_str = static_dir.to_string_lossy().to_string();
    info!("Static files: {}", static_dir_str);

    // 检查静态文件目录
    if !static_dir.exists() {
        tracing::warn!("Static files not found: {}", static_dir_str);
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
        .route("/api/setup/check", get(api::setup::check_handler))
        .route("/api/setup/complete", post(api::setup::complete_handler))
        .route("/api/setup/reset", post(api::setup::reset_handler))
        .fallback_service(tower_http::services::ServeDir::new(static_dir))
        .layer(axum::extract::Extension(config_manager))
        .layer(axum::extract::Extension(state_manager));

    let addr = SocketAddr::from((
        settings.server.host.parse::<std::net::IpAddr>().expect("Invalid host"),
        settings.server.port
    ));
    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 后台启动服务
async fn start_service(daemon: bool) {
    // 检查是否已在运行
    if is_running().await {
        println!("⚠️  Claw One 已在运行");
        std::process::exit(0);
    }

    // 获取当前可执行文件路径
    let exe_path = std::env::current_exe().expect("Failed to get executable path");

    println!("🚀 启动 Claw One...");

    if daemon {
        start_daemon_mode(&exe_path).await;
    } else {
        // 使用 systemd user 服务启动（如果可用）
        match start_systemd_service().await {
            Ok(_) => {
                println!("✅ Claw One 已通过 systemd 启动");
                println!("   查看状态: claw-one-backend status");
            }
            Err(e) => {
                println!("⚠️  systemd 启动失败: {}", e);
                println!("   尝试使用守护进程模式...");
                start_daemon_mode(&exe_path).await;
            }
        }
    }
}

/// 守护进程模式启动
async fn start_daemon_mode(exe_path: &std::path::Path) {
    let log_file = dirs::home_dir()
        .map(|h| h.join(".local/share/claw-one/claw-one.log"))
        .unwrap_or_else(|| PathBuf::from("/tmp/claw-one.log"));
    
    fs::create_dir_all(log_file.parent().unwrap()).ok();
    
    let _child = Command::new("nohup")
        .arg(exe_path)
        .arg("run")
        .stdout(Stdio::from(fs::File::create(&log_file).expect("Failed to create log file")))
        .stderr(Stdio::from(fs::File::create(&log_file).expect("Failed to create log file")))
        .spawn()
        .expect("Failed to start daemon");
    
    println!("✅ Claw One 已在后台启动");
    println!("   日志文件: {}", log_file.display());
}

/// 停止服务
async fn stop_service() {
    // 先尝试停止 systemd 服务
    let systemd_result = Command::new("systemctl")
        .args(["--user", "stop", "claw-one"])
        .output();

    if systemd_result.is_ok() && systemd_result.unwrap().status.success() {
        println!("✅ Claw One 已停止（systemd）");
        return;
    }

    // 否则查找并终止进程
    let output = Command::new("pkill")
        .args(["-f", "claw-one-backend"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("✅ Claw One 已停止");
        }
        _ => {
            println!("⚠️  Claw One 未在运行");
        }
    }
}

/// 重启服务
async fn restart_service() {
    stop_service().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    start_service(false).await;
}

/// 查看状态
async fn show_status() {
    // 检查进程是否在运行
    let output = Command::new("pgrep")
        .args(["-f", "claw-one-backend"])
        .output();

    let is_running = matches!(output, Ok(o) if o.status.success());

    // 检查 systemd 状态
    let systemd_status = Command::new("systemctl")
        .args(["--user", "is-active", "claw-one"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 检查开机自启状态
    let enabled = Command::new("systemctl")
        .args(["--user", "is-enabled", "claw-one"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    println!("========================================");
    println!("  Claw One 状态");
    println!("========================================");
    println!();

    if is_running {
        println!("运行状态: ✅ 运行中");
    } else {
        println!("运行状态: ❌ 未运行");
    }

    println!("Systemd 状态: {}", systemd_status);
    println!("开机自启: {}", if enabled { "✅ 已启用" } else { "❌ 未启用" });

    // 检查配置文件
    let config_path = std::env::var("CLAW_ONE_CONFIG")
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join("claw-one/config/claw-one.toml").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/.claw-one/config/claw-one.toml".to_string())
        });

    println!("配置文件: {}", config_path);

    if !PathBuf::from(shellexpand::tilde(&config_path).as_ref()).exists() {
        println!("⚠️  配置文件不存在！");
    }

    println!();
    println!("常用命令:");
    println!("  claw-one-backend start     # 启动服务");
    println!("  claw-one-backend stop      # 停止服务");
    println!("  claw-one-backend restart   # 重启服务");
    println!("  claw-one-backend enable    # 配置开机自启");
    println!();
}

/// 配置开机自启
async fn enable_autostart() {
    println!("🔧 配置开机自启...");

    // 检查 systemd 用户服务文件是否存在
    let service_file = dirs::home_dir()
        .map(|h| h.join(".config/systemd/user/claw-one.service"))
        .unwrap_or_else(|| PathBuf::from("~/.config/systemd/user/claw-one.service"));

    if !service_file.exists() {
        println!("⚠️  未找到 systemd 服务文件");
        println!("   请先运行 'make install' 安装服务文件");
        return;
    }

    // 重新加载 systemd
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    // 启用服务
    match Command::new("systemctl")
        .args(["--user", "enable", "claw-one"])
        .output()
    {
        Ok(o) if o.status.success() => {
            println!("✅ 开机自启已配置");
            println!("   服务将在登录时自动启动");
        }
        Ok(o) => {
            println!("❌ 配置失败: {}", String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => {
            println!("❌ 配置失败: {}", e);
        }
    }
}

/// 取消开机自启
async fn disable_autostart() {
    println!("🔧 取消开机自启...");

    match Command::new("systemctl")
        .args(["--user", "disable", "claw-one"])
        .output()
    {
        Ok(o) if o.status.success() => {
            println!("✅ 开机自启已取消");
        }
        Ok(o) => {
            println!("❌ 取消失败: {}", String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => {
            println!("❌ 取消失败: {}", e);
        }
    }
}

/// 显示配置
fn show_config() {
    let settings = match Settings::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            std::process::exit(1);
        }
    };

    println!("========================================");
    println!("  Claw One 配置");
    println!("========================================");
    println!();
    println!("[服务器]");
    println!("  监听地址: {}:{}", settings.server.host, settings.server.port);
    println!("  日志级别: {}", settings.server.log_level);
    println!();
    println!("[OpenClaw]");
    println!("  安装目录: {}", settings.openclaw_home().display());
    println!("  服务名称: {}", settings.openclaw.service_name);
    println!("  配置文件: {}", settings.openclaw_config_path().display());
    println!("  健康端口: {}", settings.openclaw.health_port);
    println!();
    println!("[路径]");
    println!("  数据目录: {}", settings.data_dir().display());
    println!("  静态文件: {}", settings.static_dir().display());
    println!();
    println!("[功能]");
    println!("  自动备份: {}", if settings.features.auto_backup { "开启" } else { "关闭" });
    println!("  安全模式: {}", if settings.features.safe_mode { "开启" } else { "关闭" });
    println!();
}

/// 检查服务是否运行
async fn is_running() -> bool {
    Command::new("pgrep")
        .args(["-f", "claw-one-backend"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 使用 systemd 启动服务
async fn start_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("systemctl")
        .args(["--user", "start", "claw-one"])
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string().into())
    }
}

/// 确保单实例运行
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
            Box::leak(Box::new(file));
            Ok(())
        }
        Err(_) => Err(anyhow::anyhow!("Claw One is already running")),
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
