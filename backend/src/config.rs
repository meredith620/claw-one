use std::path::PathBuf;
use std::process::Command;
use chrono::Utc;
use tokio::fs;

use crate::{
    error::{AppError, Result},
    types::{Config, Snapshot},
};

/// 出厂配置文件名
const FACTORY_CONFIG_FILE: &str = "factory-config.json";
/// 首次配置标记文件名
const INIT_FLAG_FILE: &str = ".initialized";

/// 配置管理器 - 负责配置文件的读写和 Git 版本控制
pub struct ConfigManager {
    /// 配置文件路径（如 ~/.openclaw/openclaw.json）
    config_path: PathBuf,
    /// Git 仓库目录（配置文件的父目录）
    git_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_path = std::env::var("CLAW_ONE_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("No home directory")
                    .join(".openclaw")
                    .join("openclaw.json")
            });
        
        let git_dir = config_path.parent()
            .expect("Invalid config path")
            .to_path_buf();
        
        Self {
            config_path,
            git_dir,
        }
    }

    /// 检查是否是首次配置（未初始化）
    pub async fn is_first_setup(&self) -> Result<bool> {
        // 检查初始化标记文件
        let init_flag = self.git_dir.join(INIT_FLAG_FILE);
        if init_flag.exists() {
            return Ok(false);
        }
        
        // 检查配置文件是否存在且有内容
        if !self.config_path.exists() {
            return Ok(true);
        }
        
        // 检查配置文件是否为空或只有基本结构
        let content = fs::read_to_string(&self.config_path).await?;
        let config: Config = serde_json::from_str(&content)?;
        
        // 如果配置为空对象或没有关键字段，认为是首次配置
        if let Some(obj) = config.as_object() {
            if obj.is_empty() {
                return Ok(true);
            }
            // 检查是否有模型或渠道配置
            let has_models = obj.get("models")
                .and_then(|m| m.as_object())
                .map(|m| !m.is_empty())
                .unwrap_or(false);
            let has_channels = obj.get("channels")
                .and_then(|c| c.as_array())
                .map(|c| !c.is_empty())
                .unwrap_or(false);
            
            if !has_models && !has_channels {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// 标记初始化完成
    pub async fn mark_initialized(&self) -> Result<()> {
        let init_flag = self.git_dir.join(INIT_FLAG_FILE);
        fs::write(&init_flag, Utc::now().to_rfc3339()).await?;
        Ok(())
    }

    /// 保存出厂配置
    pub async fn save_factory_config(&self, config: &Config) -> Result<()> {
        let factory_path = self.git_dir.join(FACTORY_CONFIG_FILE);
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&factory_path, content).await?;
        Ok(())
    }

    /// 恢复出厂设置
    pub async fn reset_to_factory(&self) -> Result<Config> {
        let factory_path = self.git_dir.join(FACTORY_CONFIG_FILE);
        
        if !factory_path.exists() {
            // 如果没有出厂配置，创建最小配置
            let minimal: Config = serde_json::json!({
                "version": "1.0",
                "gateway": {
                    "port": 18790,
                    "bind": "127.0.0.1"
                },
                "models": {},
                "channels": []
            });
            self.save_factory_config(&minimal).await?;
            return Ok(minimal);
        }
        
        let content = fs::read_to_string(&factory_path).await?;
        let config: Config = serde_json::from_str(&content)?;
        
        // 应用出厂配置
        self.save_config(&config).await?;
        
        // 创建 Git 提交记录
        self.ensure_git_repo().await?;
        self.git_add(".").await?;
        self.git_commit("Reset to factory settings").await?;
        
        Ok(config)
    }

    /// 确保 Git 仓库已初始化
    pub async fn ensure_git_repo(&self) -> Result<()> {
        let git_path = self.git_dir.join(".git");
        if !git_path.exists() {
            // 初始化 Git 仓库
            let output = Command::new("git")
                .args(["init", self.git_dir.to_str().unwrap()])
                .output()
                .map_err(|e| AppError::Git(format!("Failed to init git repo: {}", e)))?;
            
            if !output.status.success() {
                return Err(AppError::Git(
                    String::from_utf8_lossy(&output.stderr).to_string()
                ));
            }

            // 配置 Git 用户信息
            self.git_config("user.name", "Claw One").await?;
            self.git_config("user.email", "dev@claw.one").await?;

            // 如果配置文件存在，创建初始提交
            if self.config_path.exists() {
                self.git_add(".").await?;
                self.git_commit("Initial config").await?;
            }
        }
        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> Result<Config> {
        use tokio::fs;
        
        // 确保配置文件存在
        if !self.config_path.exists() {
            return Err(AppError::ConfigNotFound);
        }
        
        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| AppError::Io(e))?;
        
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置（不创建 Git 提交）
    pub async fn save_config(&self, config: &Config) -> Result<()> {
        use tokio::fs;
        
        // 确保父目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| AppError::Io(e))?;
        }
        
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content).await.map_err(|e| AppError::Io(e))?;
        
        Ok(())
    }

    /// 应用配置并创建 Git 提交
    pub async fn apply_config(
        &self,
        config: Config,
        message: Option<String>,
    ) -> Result<String> {
        // 1. 确保 Git 仓库已初始化
        self.ensure_git_repo().await?;
        
        // 2. 保存配置到文件
        self.save_config(&config).await?;
        
        // 3. 添加到 Git
        self.git_add(".").await?;
        
        // 4. 检查是否有变更
        if !self.has_changes().await? {
            return Err(AppError::Runtime("No changes to commit".to_string()));
        }
        
        // 5. 创建提交
        let commit_msg = message.unwrap_or_else(|| {
            format!("Config update at {}", Utc::now().to_rfc3339())
        });
        let commit_id = self.git_commit(&commit_msg).await?;
        
        Ok(commit_id)
    }

    /// 获取快照列表（最近的 Git 提交）
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        // 如果 Git 仓库不存在，返回空列表
        let git_path = self.git_dir.join(".git");
        if !git_path.exists() {
            return Ok(vec![]);
        }
        
        // 使用 git log 获取提交历史
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "log",
                "--pretty=format:%H|%ci|%s",
                "-n", "100",  // 最近 100 条
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to list snapshots: {}", e)))?;
        
        if !output.status.success() {
            // 可能是空仓库
            return Ok(vec![]);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let snapshots: Vec<Snapshot> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                Snapshot {
                    id: parts.get(0).unwrap_or(&"").to_string(),
                    timestamp: parts.get(1).unwrap_or(&"").to_string(),
                    message: parts.get(2).unwrap_or(&"").to_string(),
                }
            })
            .collect();
        
        Ok(snapshots)
    }

    /// 回滚到指定版本
    pub async fn rollback(&self, snapshot_id: &str) -> Result<()> {
        // 1. 检查 Git 仓库
        let git_path = self.git_dir.join(".git");
        if !git_path.exists() {
            return Err(AppError::Git("Git repository not initialized".to_string()));
        }
        
        // 2. 检查快照是否存在
        let snapshots = self.list_snapshots().await?;
        if !snapshots.iter().any(|s| s.id == snapshot_id) {
            return Err(AppError::Git(format!("Snapshot {} not found", snapshot_id)));
        }
        
        // 3. 执行 git checkout
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "checkout",
                snapshot_id,
                "--",
                self.config_path.file_name().unwrap().to_str().unwrap(),
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to checkout: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // 4. 创建回滚提交记录
        self.git_add(".").await?;
        let msg = format!("Rollback to {}", &snapshot_id[..8]);
        self.git_commit(&msg).await?;
        
        Ok(())
    }

    /// 检查是否可以回滚
    pub async fn can_rollback(&self) -> Result<bool> {
        let snapshots = self.list_snapshots().await?;
        Ok(snapshots.len() > 1)
    }

    // 私有辅助方法
    
    async fn git_config(&self, key: &str, value: &str) -> Result<()> {
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "config",
                key,
                value,
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to set git config: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        Ok(())
    }

    async fn git_add(&self, path: &str) -> Result<()> {
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "add",
                path,
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to git add: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        Ok(())
    }

    async fn git_commit(&self, message: &str) -> Result<String> {
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "commit",
                "-m", message,
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to git commit: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // 获取最新提交的 hash
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to get commit id: {}", e)))?;
        
        let commit_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(commit_id)
    }

    async fn has_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "status",
                "--porcelain",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to check status: {}", e)))?;
        
        Ok(!output.stdout.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_ensure_git_repo() {
        // 创建临时目录
        let temp_dir = std::env::temp_dir().join("claw-one-test");
        let _ = fs::remove_dir_all(&temp_dir).await;
        fs::create_dir_all(&temp_dir).await.unwrap();
        
        // 创建配置管理器
        let config_path = temp_dir.join("openclaw.json");
        std::env::set_var("CLAW_ONE_CONFIG", config_path.to_str().unwrap());
        
        let manager = ConfigManager::new();
        manager.ensure_git_repo().await.unwrap();
        
        // 验证 .git 目录存在
        assert!(temp_dir.join(".git").exists());
        
        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}