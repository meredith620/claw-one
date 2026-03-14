// Runtime 管理模块
// 负责与 OpenClaw 进程交互，通过 systemd 管理生命周期

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

use crate::error::{AppError, Result};
use crate::settings::OpenClawConfig;

/// OpenClaw 服务状态
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 启动中
    Starting,
    /// 停止中
    Stopping,
    /// 失败
    Failed(String),
    /// 未知
    Unknown,
}

/// Runtime 管理器
pub struct RuntimeManager {
    /// systemd 服务名称
    service_name: String,
    /// 健康检查端口
    health_port: u16,
    /// 健康检查超时时间
    health_timeout: Duration,
    /// OpenClaw 安装根目录
    openclaw_home: String,
}

impl RuntimeManager {
    /// 从配置创建 RuntimeManager
    pub fn from_config(config: &OpenClawConfig) -> Self {
        Self {
            service_name: config.service_name.clone(),
            health_port: config.health_port,
            health_timeout: Duration::from_secs(config.health_timeout),
            openclaw_home: config.openclaw_home.clone(),
        }
    }

    /// 使用默认配置创建（向后兼容）
    pub fn new() -> Self {
        Self {
            service_name: "openclaw-gateway".to_string(),
            health_port: 18790,
            health_timeout: Duration::from_secs(30),
            openclaw_home: "~/.openclaw".to_string(),
        }
    }

    /// 设置健康检查端口
    pub fn with_health_port(mut self, port: u16) -> Self {
        self.health_port = port;
        self
    }

    /// 设置健康检查超时
    pub fn with_health_timeout(mut self, timeout_secs: u64) -> Self {
        self.health_timeout = Duration::from_secs(timeout_secs);
        self
    }

    /// 启动 OpenClaw 服务
    pub async fn start(&self) -> Result<()> {
        self.systemctl("start").await
    }

    /// 停止 OpenClaw 服务
    pub async fn stop(&self) -> Result<()> {
        self.systemctl("stop").await
    }

    /// 重启 OpenClaw 服务
    pub async fn restart(&self) -> Result<()> {
        self.systemctl("restart").await
    }

    /// 获取服务状态（使用 systemctl --user）
    pub async fn status(&self) -> Result<ServiceStatus> {
        let output = Command::new("systemctl")
            .args([
                "--user",
                "is-active",
                &format!("{}.service", self.service_name),
            ])
            .output()
            .map_err(|e| AppError::Runtime(format!("Failed to check status: {}", e)))?;

        let status = String::from_utf8_lossy(&output.stdout);
        match status.trim() {
            "active" => Ok(ServiceStatus::Running),
            "inactive" => Ok(ServiceStatus::Stopped),
            "activating" => Ok(ServiceStatus::Starting),
            "deactivating" => Ok(ServiceStatus::Stopping),
            "failed" => {
                // 获取失败原因
                let reason = self.get_failure_reason().await?;
                Ok(ServiceStatus::Failed(reason))
            }
            _ => Ok(ServiceStatus::Unknown),
        }
    }

    /// 执行健康检查（带超时）
    pub async fn health_check(&self) -> Result<bool> {
        let result = timeout(
            self.health_timeout,
            self.do_health_check(),
        ).await;

        match result {
            Ok(Ok(healthy)) => Ok(healthy),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(false), // 超时视为不健康
        }
    }

    /// 等待服务健康（带超时）
    pub async fn wait_for_healthy(&self) -> Result<bool> {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(500);

        while start.elapsed() < self.health_timeout {
            if self.health_check().await? {
                return Ok(true);
            }
            tokio::time::sleep(check_interval).await;
        }

        Ok(false)
    }

    /// 检查进程是否运行（快速检查，只检查当前用户的进程）
    pub async fn is_process_running(&self) -> bool {
        // 获取当前用户 UID
        let uid = match std::env::var("UID") {
            Ok(uid_str) => uid_str,
            Err(_) => match Command::new("id").args(["-u"]).output() {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).trim().to_string()
                }
                _ => return false,
            },
        };

        let output = Command::new("pgrep")
            .args(["-u", &uid, "-f", "openclaw gateway"])
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// 获取服务日志（最近 n 行）- 使用 journalctl --user
    pub async fn get_logs(&self, lines: usize) -> Result<String> {
        let output = Command::new("journalctl")
            .args([
                "--user",
                "-u",
                &format!("{}.service", self.service_name),
                "-n",
                &lines.to_string(),
                "--no-pager",
            ])
            .output()
            .map_err(|e| AppError::Runtime(format!("Failed to get logs: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::Runtime(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 从日志中分类错误类型
    pub fn classify_error(log_content: &str) -> Option<ErrorType> {
        // 配置错误关键词
        let config_errors = [
            "Config validation failed",
            "Invalid API key",
            "Missing required field",
            "Cannot parse config",
            "SyntaxError",
            "ValidationError",
            "invalid configuration",
        ];

        // 系统错误关键词
        let system_errors = [
            "EADDRINUSE",
            "Permission denied",
            "Out of memory",
            "ENOSPC",
            "port already in use",
        ];

        for pattern in &config_errors {
            if log_content.contains(pattern) {
                return Some(ErrorType::Config);
            }
        }

        for pattern in &system_errors {
            if log_content.contains(pattern) {
                return Some(ErrorType::System);
            }
        }

        None
    }

    // 私有辅助方法 - 使用 systemctl --user

    async fn systemctl(&self, action: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args([
                "--user",
                action,
                &format!("{}.service", self.service_name),
            ])
            .output()
            .map_err(|e| AppError::Runtime(format!("Failed to {} service: {}", action, e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Runtime(format!(
                "systemctl --user {} failed: {}",
                action, stderr
            )));
        }

        Ok(())
    }

    async fn do_health_check(&self) -> Result<bool> {
        // 首先检查进程是否存在
        if !self.is_process_running().await {
            return Ok(false);
        }

        // 然后检查 HTTP 健康端点
        match self.http_health_check().await {
            Ok(healthy) => Ok(healthy),
            Err(_) => {
                // HTTP 检查失败，尝试 CLI 检查作为备用
                self.cli_health_check().await
            }
        }
    }

    /// 从 openclaw.json 配置文件中读取 gateway.port
    fn get_gateway_port_from_config(&self) -> Option<u16> {
        // 展开 ~ 为用户主目录
        let config_path = if self.openclaw_home.starts_with("~/") {
            dirs::home_dir()
                .map(|home| home.join(&self.openclaw_home[2..]))
                .map(|p| p.join("openclaw.json"))?
        } else {
            PathBuf::from(&self.openclaw_home).join("openclaw.json")
        };

        let content = std::fs::read_to_string(&config_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        
        json.get("gateway")
            .and_then(|g| g.get("port"))
            .and_then(|p| p.as_u64())
            .map(|p| p as u16)
    }

    async fn http_health_check(&self) -> Result<bool> {
        // 优先从 openclaw.json 读取端口，否则使用配置的 health_port
        let port = self.get_gateway_port_from_config()
            .unwrap_or(self.health_port);
        
        let url = format!("http://127.0.0.1:{}/health", port);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::Runtime(format!("Failed to build HTTP client: {}", e)))?;

        match client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn cli_health_check(&self) -> Result<bool> {
        let output = Command::new("openclaw")
            .args(["health", "--json"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                // 尝试解析 JSON 输出
                let stdout = String::from_utf8_lossy(&output.stdout);
                match serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(json) => Ok(json.get("healthy").and_then(|v| v.as_bool()).unwrap_or(false)),
                    Err(_) => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    async fn get_failure_reason(&self) -> Result<String> {
        let output = Command::new("systemctl")
            .args([
                "--user",
                "status",
                &format!("{}.service", self.service_name),
                "--no-pager",
            ])
            .output()
            .map_err(|e| AppError::Runtime(format!("Failed to get status: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(format!("{}", if stderr.is_empty() { stdout } else { stderr }))
    }
}

/// 错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    /// 配置错误
    Config,
    /// 系统错误
    System,
}

/// 运行时适配器 trait（预留多 Runtime 支持）
#[async_trait::async_trait]
pub trait Runtime: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    async fn restart(&self) -> Result<()>;
    async fn status(&self) -> Result<ServiceStatus>;
    async fn health_check(&self) -> Result<bool>;
}

/// OpenClaw 运行时适配器
pub struct OpenClawRuntime {
    manager: RuntimeManager,
}

impl OpenClawRuntime {
    pub fn new() -> Self {
        Self {
            manager: RuntimeManager::new(),
        }
    }
}

#[async_trait::async_trait]
impl Runtime for OpenClawRuntime {
    fn name(&self) -> &str {
        "openclaw"
    }

    async fn start(&self) -> Result<()> {
        self.manager.start().await
    }

    async fn stop(&self) -> Result<()> {
        self.manager.stop().await
    }

    async fn restart(&self) -> Result<()> {
        self.manager.restart().await
    }

    async fn status(&self) -> Result<ServiceStatus> {
        self.manager.status().await
    }

    async fn health_check(&self) -> Result<bool> {
        self.manager.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_error() {
        // 配置错误
        assert_eq!(
            RuntimeManager::classify_error("Config validation failed: missing field"),
            Some(ErrorType::Config)
        );
        assert_eq!(
            RuntimeManager::classify_error("Cannot parse config at line 10"),
            Some(ErrorType::Config)
        );

        // 系统错误
        assert_eq!(
            RuntimeManager::classify_error("Error: EADDRINUSE, port 8080 already in use"),
            Some(ErrorType::System)
        );
        assert_eq!(
            RuntimeManager::classify_error("Permission denied when reading /etc/openclaw"),
            Some(ErrorType::System)
        );

        // 未知错误
        assert_eq!(
            RuntimeManager::classify_error("Some random error message"),
            None
        );
    }
}