// 应用配置模块
// 负责加载和解析 claw-one.toml 配置文件

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 应用配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    /// 服务器配置
    #[serde(default)]
    pub server: ServerConfig,
    /// OpenClaw 连接配置
    #[serde(default)]
    pub openclaw: OpenClawConfig,
    /// 路径配置
    #[serde(default)]
    pub paths: PathsConfig,
    /// 功能开关
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 监听地址
    #[serde(default = "default_host")]
    pub host: String,
    /// 监听端口
    #[serde(default = "default_port")]
    pub port: u16,
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenClawConfig {
    /// OpenClaw 安装根目录（包含 openclaw.json 配置文件）
    #[serde(default = "default_openclaw_home")]
    pub openclaw_home: String,
    /// OpenClaw 服务名称（用于 systemctl）
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// 健康检查端口
    #[serde(default = "default_health_port")]
    pub health_port: u16,
    /// 健康检查超时（秒）
    #[serde(default = "default_health_timeout")]
    pub health_timeout: u64,
    /// OpenClaw 配置文件路径（覆盖 openclaw_home 的默认）
    #[serde(default)]
    pub config_path: String,
    /// Git 仓库路径
    #[serde(default)]
    pub git_repo_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathsConfig {
    /// 数据目录
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    /// 静态文件目录
    #[serde(default)]
    pub static_dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeaturesConfig {
    /// 自动备份
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    /// 安全模式
    #[serde(default = "default_true")]
    pub safe_mode: bool,
    /// 首次运行向导
    #[serde(default = "default_true")]
    pub first_run_wizard: bool,
}

// 默认值函数
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8080
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_openclaw_home() -> String {
    "~/.openclaw".to_string()
}
fn default_service_name() -> String {
    "openclaw-gateway".to_string()
}
fn default_health_port() -> u16 {
    18790
}
fn default_health_timeout() -> u64 {
    30
}
fn default_data_dir() -> String {
    "~/.config/claw-one".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
        }
    }
}

impl Default for OpenClawConfig {
    fn default() -> Self {
        Self {
            openclaw_home: default_openclaw_home(),
            service_name: default_service_name(),
            health_port: default_health_port(),
            health_timeout: default_health_timeout(),
            config_path: String::new(),
            git_repo_path: String::new(),
        }
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            static_dir: String::new(),
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            safe_mode: true,
            first_run_wizard: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            openclaw: OpenClawConfig::default(),
            paths: PathsConfig::default(),
            features: FeaturesConfig::default(),
        }
    }
}

impl Settings {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }

    /// 从环境变量获取配置文件路径并加载
    pub fn from_env() -> anyhow::Result<Self> {
        // 1. 首先检查环境变量
        if let Ok(config_path) = std::env::var("CLAW_ONE_CONFIG") {
            eprintln!("[DEBUG] 使用 CLAW_ONE_CONFIG 环境变量: {}", config_path);
            if std::path::Path::new(&config_path).exists() {
                return Self::from_file(&config_path);
            } else {
                eprintln!("⚠️  CLAW_ONE_CONFIG 指向的文件不存在: {}", config_path);
            }
        }

        // 2. 尝试从可执行文件路径推导（适用于安装后的运行）
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(install_dir) = exe_path.parent().and_then(|p| p.parent()) {
                let derived_config = install_dir.join("config").join("claw-one.toml");
                eprintln!(
                    "[DEBUG] 尝试从可执行文件路径推导配置: {}",
                    derived_config.display()
                );
                if derived_config.exists() {
                    eprintln!("[DEBUG] 从 {} 加载配置", derived_config.display());
                    return Self::from_file(&derived_config);
                }
            }
        }

        // 3. 尝试用户主目录
        let home_config =
            dirs::home_dir().map(|h| h.join("claw-one").join("config").join("claw-one.toml"));

        if let Some(ref config_path) = home_config {
            eprintln!("[DEBUG] 尝试用户主目录配置: {}", config_path.display());
            if config_path.exists() {
                eprintln!("[DEBUG] 从 {} 加载配置", config_path.display());
                return Self::from_file(config_path);
            }
        }

        // 4. 尝试当前目录（开发环境）
        let cwd_config = std::env::current_dir()
            .ok()
            .map(|d| d.join("config").join("claw-one.toml"));

        if let Some(ref config_path) = cwd_config {
            eprintln!("[DEBUG] 尝试当前目录配置: {}", config_path.display());
            if config_path.exists() {
                eprintln!("[DEBUG] 从 {} 加载配置", config_path.display());
                return Self::from_file(config_path);
            }
        }

        eprintln!("⚠️  未找到配置文件，使用默认配置");
        eprintln!("   尝试过的路径:");
        if let Some(ref p) = home_config {
            eprintln!("     - {}", p.display());
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent().and_then(|p| p.parent()) {
                eprintln!("     - {}", dir.join("config/claw-one.toml").display());
            }
        }
        if let Some(ref p) = cwd_config {
            eprintln!("     - {}", p.display());
        }
        Ok(Self::default())
    }

    /// 获取数据目录（展开 ~）
    pub fn data_dir(&self) -> PathBuf {
        expand_path(&self.paths.data_dir)
    }

    /// 获取静态文件目录
    pub fn static_dir(&self) -> PathBuf {
        if !self.paths.static_dir.is_empty() {
            return expand_path(&self.paths.static_dir);
        }

        // 从可执行文件路径推导
        if let Ok(exe) = std::env::current_exe() {
            if let Some(install_dir) = exe.parent().and_then(|p| p.parent()) {
                // 优先检查 share/static (安装后的标准路径)
                let share_static = install_dir.join("share").join("static");
                if share_static.exists() {
                    return share_static;
                }
                // 兼容旧路径 static/dist (开发环境)
                let old_static = install_dir.join("static").join("dist");
                if old_static.exists() {
                    return old_static;
                }
            }
        }

        // 默认回退
        PathBuf::from("../static/dist")
    }

    /// 获取 OpenClaw 安装根目录
    pub fn openclaw_home(&self) -> PathBuf {
        expand_path(&self.openclaw.openclaw_home)
    }

    /// 获取 OpenClaw 配置文件路径
    /// 优先级: config_path > openclaw_home/openclaw.json
    pub fn openclaw_config_path(&self) -> PathBuf {
        if !self.openclaw.config_path.is_empty() {
            expand_path(&self.openclaw.config_path)
        } else {
            self.openclaw_home().join("openclaw.json")
        }
    }

    /// 获取 Git 仓库路径
    pub fn git_repo_path(&self) -> PathBuf {
        if self.openclaw.git_repo_path.is_empty() {
            self.openclaw_config_path()
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.data_dir())
        } else {
            expand_path(&self.openclaw.git_repo_path)
        }
    }
}

/// 展开路径中的 ~ 为用户主目录
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        dirs::home_dir()
            .map(|home| home.join(&path[2..]))
            .unwrap_or_else(|| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

/// 检查配置文件是否存在，不存在则提示用户
pub fn check_config_or_exit() {
    match Settings::from_env() {
        Ok(settings) => {
            // 检查是否是默认配置
            if settings.openclaw.service_name == "openclaw-gateway"
                && settings.openclaw.openclaw_home == default_openclaw_home()
            {
                println!("⚠️  警告: 您正在使用默认配置");
                println!("   OpenClaw 根目录: {}", settings.openclaw_home().display());
                println!(
                    "   配置文件路径: {}",
                    settings.openclaw_config_path().display()
                );
                println!("   如需修改，请编辑配置文件设置正确的 OpenClaw 连接信息");
            }
        }
        Err(e) => {
            eprintln!("❌ 配置文件错误: {}", e);
            eprintln!("   请检查 TOML 配置文件格式");
            std::process::exit(1);
        }
    }
}
