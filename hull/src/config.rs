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
/// 版本控制目录名
const VERSION_CONFIG_DIR: &str = "version-config";

/// 配置管理器 - 负责配置文件的读写和 Git 版本控制
/// 设计原则：
/// 1. 主配置文件 (~/.openclaw/openclaw.json) 是运行时读写的主文件
/// 2. version-config/ 目录是 Git 版本控制的隔离区域
/// 3. 保存时复制到 version-config/，diff 验证后再 Git 提交
pub struct ConfigManager {
    /// 主配置文件路径（如 ~/.openclaw/openclaw.json）
    config_path: PathBuf,
    /// 主目录（配置文件的父目录，如 ~/.openclaw/）
    git_dir: PathBuf,
    /// 版本控制目录（如 ~/.openclaw/version-config/）
    version_config_dir: PathBuf,
    /// 版本控制中的配置文件路径（如 ~/.openclaw/version-config/openclaw.json）
    version_config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        // 首先尝试从 CLAW_OPENCLAW_CONFIG 获取（专门用于 openclaw.json）
        let config_path = std::env::var("CLAW_OPENCLAW_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // 尝试从 claw-one.toml 读取 openclaw_home
                let openclaw_home = Self::get_openclaw_home_from_settings();
                PathBuf::from(openclaw_home).join("openclaw.json")
            });
        
        Self::with_config_path(config_path)
    }

    /// 使用指定的配置路径创建 ConfigManager（用于测试）
    pub fn with_config_path(config_path: PathBuf) -> Self {
        let git_dir = config_path.parent()
            .expect("Invalid config path")
            .to_path_buf();
        
        // 版本控制目录：~/.openclaw/version-config/
        let version_config_dir = git_dir.join(VERSION_CONFIG_DIR);
        let version_config_path = version_config_dir.join("openclaw.json");
        
        Self {
            config_path,
            git_dir,
            version_config_dir,
            version_config_path,
        }
    }
    
    /// 从 claw-one.toml 读取 openclaw_home 配置
    fn get_openclaw_home_from_settings() -> String {
        // 尝试从环境变量获取配置文件路径
        let config_file = std::env::var("CLAW_ONE_CONFIG")
            .ok()
            .or_else(|| {
                // 尝试从可执行文件路径推导: bin/../config/claw-one.toml
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| {
                        let parent = exe.parent()?;  // bin/
                        let grandparent = parent.parent()?;  // 安装目录
                        Some(grandparent.join("config").join("claw-one.toml").to_string_lossy().to_string())
                    })
            })
            .or_else(|| {
                // 使用默认路径
                dirs::home_dir()
                    .map(|h| h.join("claw-one").join("config").join("claw-one.toml").to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "~/.claw-one/config/claw-one.toml".to_string());
        
        eprintln!("[DEBUG] ConfigManager: 尝试读取 claw-one.toml: {}", config_file);
        
        // 解析 TOML 获取 openclaw_home
        if let Ok(content) = std::fs::read_to_string(&config_file) {
            eprintln!("[DEBUG] ConfigManager: 成功读取 claw-one.toml");
            // 简单解析 openclaw_home 字段
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("openclaw_home") {
                    if let Some(value) = line.split('=').nth(1) {
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        eprintln!("[DEBUG] ConfigManager: 找到 openclaw_home = {}", value);
                        // 展开 ~ 为 home 目录
                        if value.starts_with("~/") {
                            if let Some(home) = dirs::home_dir() {
                                let expanded = home.join(&value[2..]).to_string_lossy().to_string();
                                eprintln!("[DEBUG] ConfigManager: 展开后路径 = {}", expanded);
                                return expanded;
                            }
                        }
                        return value.to_string();
                    }
                }
            }
            eprintln!("[DEBUG] ConfigManager: 未在 claw-one.toml 中找到 openclaw_home 字段");
        } else {
            eprintln!("[DEBUG] ConfigManager: 无法读取 claw-one.toml，使用默认路径");
        }
        
        // 默认回退
        let default_path = dirs::home_dir()
            .map(|h| h.join(".openclaw").to_string_lossy().to_string())
            .unwrap_or_else(|| "~/.openclaw".to_string());
        eprintln!("[DEBUG] ConfigManager: 使用默认 openclaw_home = {}", default_path);
        default_path
    }

    /// 自动迁移旧版 Git 仓库到 version-config/ 目录
    /// 
    /// 检测逻辑：
    /// - 如果 ~/.openclaw/.git 存在且 ~/.openclaw/version-config/.git 不存在 → 需要迁移
    /// 
    /// 迁移步骤：
    /// 1. 创建 version-config/ 目录
    /// 2. 复制当前 openclaw.json 到 version-config/
    /// 3. 移动 .git 目录到 version-config/
    /// 4. 更新 Git 工作目录配置
    pub async fn migrate_legacy_git_repo(&self) -> Result<bool> {
        let old_git_dir = self.git_dir.join(".git");
        let new_git_dir = self.version_config_dir.join(".git");
        
        // 检查是否需要迁移
        if !old_git_dir.exists() {
            tracing::info!("No legacy Git repo found, skip migration");
            return Ok(false);
        }
        
        if new_git_dir.exists() {
            tracing::info!("version-config/.git already exists, skip migration");
            return Ok(false);
        }
        
        tracing::info!("Migrating legacy Git repo to version-config/");
        
        // 1. 确保 version-config/ 目录存在
        if !self.version_config_dir.exists() {
            fs::create_dir_all(&self.version_config_dir).await
                .map_err(|e| AppError::Runtime(format!("Failed to create version-config dir: {}", e)))?;
        }
        
        // 2. 复制当前配置文件到 version-config/（如果不存在）
        if self.config_path.exists() && !self.version_config_path.exists() {
            fs::copy(&self.config_path, &self.version_config_path).await
                .map_err(|e| AppError::Runtime(format!("Failed to copy config: {}", e)))?;
            tracing::info!("Copied openclaw.json to version-config/");
        }
        
        // 3. 移动 .git 目录
        // 使用 tokio::fs::rename 是原子操作，但如果跨设备可能失败，这时需要手动复制
        match fs::rename(&old_git_dir, &new_git_dir).await {
            Ok(_) => {
                tracing::info!("Moved .git to version-config/");
            }
            Err(e) => {
                // 可能是跨设备，尝试复制后删除
                tracing::warn!("Rename failed (cross-device?), trying copy+remove: {}", e);
                Self::copy_dir_recursive(&old_git_dir, &new_git_dir).await?;
                fs::remove_dir_all(&old_git_dir).await
                    .map_err(|e| AppError::Runtime(format!("Failed to remove old .git: {}", e)))?;
                tracing::info!("Copied .git to version-config/ and removed old one");
            }
        }
        
        // 4. 更新 Git 工作目录配置（指向新的路径）
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "config", "--local", "core.worktree",
                self.version_config_dir.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to update git worktree: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // 5. 更新 Git 索引（重新索引新位置的文件）
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "add", "-A",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to update git index: {}", e)))?;
        
        if !output.status.success() {
            tracing::warn!("Git add -A output: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // 6. 验证迁移后的仓库状态
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "status", "--short",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to check git status: {}", e)))?;
        
        let status_output = String::from_utf8_lossy(&output.stdout);
        if !status_output.is_empty() {
            tracing::warn!("Git status after migration:\n{}", status_output);
            // 如果有未提交的变更，创建一个迁移提交
            let output = Command::new("git")
                .args([
                    "-C", self.version_config_dir.to_str().unwrap(),
                    "commit", "-m", "Migrate to version-config layout",
                ])
                .output()
                .map_err(|e| AppError::Git(format!("Failed to create migration commit: {}", e)))?;
            
            if output.status.success() {
                tracing::info!("Created migration commit");
            }
        }
        
        tracing::info!("Migration completed successfully");
        Ok(true)
    }
    
    /// 递归复制目录（用于跨设备迁移 .git）
    async fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<()> {
        fs::create_dir_all(dst).await
            .map_err(|e| AppError::Runtime(format!("Failed to create dir {:?}: {}", dst, e)))?;
        
        let mut entries = fs::read_dir(src).await
            .map_err(|e| AppError::Runtime(format!("Failed to read dir {:?}: {}", src, e)))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::Runtime(format!("Failed to read entry: {}", e)))? 
        {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if entry.file_type().await.map_err(|e| AppError::Runtime(format!("Failed to get file type: {}", e)))?.is_dir() {
                Box::pin(Self::copy_dir_recursive(&src_path, &dst_path)).await?;
            } else {
                fs::copy(&src_path, &dst_path).await
                    .map_err(|e| AppError::Runtime(format!("Failed to copy file: {}", e)))?;
            }
        }
        
        Ok(())
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
    /// 
    /// 流程：
    /// 1. 加载 factory-config.json（如果不存在则创建最小配置）
    /// 2. 保存到主配置文件
    /// 3. 同步到 version-config/ 并创建 Git 提交
    pub async fn reset_to_factory(&self) -> Result<Config> {
        let factory_path = self.git_dir.join(FACTORY_CONFIG_FILE);
        
        let config = if !factory_path.exists() {
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
            minimal
        } else {
            let content = fs::read_to_string(&factory_path).await?;
            serde_json::from_str(&content)?
        };
        
        // 使用 sync_to_version_config 完成：保存 + 复制 + diff + Git 提交
        self.sync_to_version_config(&config, Some("Reset to factory settings".to_string())).await?;
        
        Ok(config)
    }

    /// 确保 Git 仓库已初始化
    /// 在 version-config/ 目录中初始化 Git，实现运行时数据与版本控制完全隔离
    pub async fn ensure_git_repo(&self) -> Result<()> {
        // 确保 version-config/ 目录存在
        if !self.version_config_dir.exists() {
            fs::create_dir_all(&self.version_config_dir).await
                .map_err(|e| AppError::Git(format!("Failed to create version-config dir: {}", e)))?;
        }
        
        let git_path = self.version_config_dir.join(".git");
        if !git_path.exists() {
            // 在 version-config/ 中初始化 Git 仓库
            let output = Command::new("git")
                .args(["init", self.version_config_dir.to_str().unwrap()])
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

            tracing::info!("Git repo initialized in version-config/");
        }

        Ok(())
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> Result<Config> {
        use tokio::fs;
        
        eprintln!("[DEBUG] ConfigManager: 尝试读取配置 from {:?}", self.config_path);
        
        // 确保配置文件存在
        if !self.config_path.exists() {
            eprintln!("[DEBUG] ConfigManager: 配置文件不存在!");
            return Err(AppError::ConfigNotFound);
        }
        
        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| {
                eprintln!("[DEBUG] ConfigManager: 读取文件失败: {}", e);
                AppError::Io(e)
            })?;
        
        eprintln!("[DEBUG] ConfigManager: 文件内容长度: {} bytes", content.len());
        
        let config: Config = serde_json::from_str(&content).map_err(|e| {
            eprintln!("[DEBUG] ConfigManager: JSON 解析失败: {}", e);
            AppError::Json(e)
        })?;
        
        // 检查是否有 providers
        if let Some(models) = config.get("models") {
            if let Some(providers) = models.get("providers") {
                if let Some(obj) = providers.as_object() {
                    eprintln!("[DEBUG] ConfigManager: 找到 {} 个 providers", obj.len());
                }
            } else {
                eprintln!("[DEBUG] ConfigManager: models 中无 providers");
            }
        } else {
            eprintln!("[DEBUG] ConfigManager: 配置中无 models 字段");
        }
        
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
    /// 
    /// 流程：
    /// 1. 保存配置到主文件 (~/.openclaw/openclaw.json)
    /// 2. 复制到 version-config/ 目录
    /// 3. diff 验证两个文件一致
    /// 4. 在 version-config/ 中执行 Git 提交
    pub async fn apply_config(
        &self,
        config: Config,
        message: Option<String>,
    ) -> Result<String> {
        // 使用 sync_to_version_config 完成完整流程
        let commit_msg = message.unwrap_or_else(|| {
            format!("Config update at {}", Utc::now().to_rfc3339())
        });
        
        match self.sync_to_version_config(&config, Some(commit_msg)).await? {
            Some(commit_id) => Ok(commit_id),
            None => Err(AppError::Runtime("No changes to commit".to_string())),
        }
    }

    /// 获取快照列表（最近的 Git 提交）
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        // 如果 Git 仓库不存在，返回空列表
        let git_path = self.version_config_dir.join(".git");
        if !git_path.exists() {
            return Ok(vec![]);
        }
        
        // 使用 git log 获取提交历史
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
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
    /// 
    /// 流程：
    /// 1. 从 version-config/ Git 仓库检出指定版本的 openclaw.json
    /// 2. 将检出后的文件复制回主配置路径
    /// 3. 创建回滚提交记录
    pub async fn rollback(&self, snapshot_id: &str) -> Result<()> {
        // 1. 检查 Git 仓库
        let git_path = self.version_config_dir.join(".git");
        if !git_path.exists() {
            return Err(AppError::Git("Git repository not initialized".to_string()));
        }
        
        // 2. 检查快照是否存在
        let snapshots = self.list_snapshots().await?;
        if !snapshots.iter().any(|s| s.id == snapshot_id) {
            return Err(AppError::Git(format!("Snapshot {} not found", snapshot_id)));
        }
        
        // 3. 执行 git checkout 到 version-config/openclaw.json
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "checkout",
                snapshot_id,
                "--",
                "openclaw.json",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to checkout: {}", e)))?;
        
        if !output.status.success() {
            return Err(AppError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        
        // 4. 将 version-config/openclaw.json 复制回主配置路径
        fs::copy(&self.version_config_path, &self.config_path).await
            .map_err(|e| AppError::Io(e))?;
        
        // 5. diff 验证
        let version_content = fs::read_to_string(&self.version_config_path).await?;
        let main_content = fs::read_to_string(&self.config_path).await?;
        if version_content != main_content {
            return Err(AppError::Runtime(
                "Rollback failed: config file mismatch after copy".to_string()
            ));
        }
        
        // 6. 创建回滚提交记录
        self.git_add("openclaw.json").await?;
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
                "-C", self.version_config_dir.to_str().unwrap(),
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

    /// Git add 操作
    /// 设计原则：只在 version-config/ 目录中操作，管理 version-config/openclaw.json
    pub async fn git_add(&self, _path: &str) -> Result<()> {
        // 在 version-config/ 目录中添加 openclaw.json
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "add",
                "openclaw.json",
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

    pub async fn git_commit(&self, message: &str) -> Result<String> {
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
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
                "-C", self.version_config_dir.to_str().unwrap(),
                "rev-parse",
                "HEAD",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to get commit id: {}", e)))?;
        
        let commit_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(commit_id)
    }

    pub async fn has_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args([
                "-C", self.version_config_dir.to_str().unwrap(),
                "status",
                "--porcelain",
            ])
            .output()
            .map_err(|e| AppError::Git(format!("Failed to check status: {}", e)))?;
        
        Ok(!output.stdout.is_empty())
    }

    /// 同步配置到 version-config/ 目录并执行 Git 提交
    /// 核心逻辑：
    /// 1. 将主配置复制到 version-config/openclaw.json
    /// 2. diff 验证两个文件内容一致
    /// 3. 在 version-config/ 中执行 Git add & commit
    pub async fn sync_to_version_config(&self,
        config: &Config,
        commit_msg: Option<String>,
    ) -> Result<Option<String>> {
        // 1. 确保 version-config/ 目录存在
        if !self.version_config_dir.exists() {
            fs::create_dir_all(&self.version_config_dir).await
                .map_err(|e| AppError::Git(format!("Failed to create version-config dir: {}", e)))?;
        }

        // 2. 确保 Git 仓库已初始化
        self.ensure_git_repo().await?;

        // 3. 将配置序列化为 JSON
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Runtime(format!("Failed to serialize config: {}", e)))?;

        // 4. 写入主配置文件
        fs::write(&self.config_path, &config_json).await?;

        // 5. 复制到 version-config/
        fs::write(&self.version_config_path, &config_json).await?;

        // 6. diff 验证：读取两个文件并比较
        let original_content = fs::read_to_string(&self.config_path).await?;
        let copied_content = fs::read_to_string(&self.version_config_path).await?;

        if original_content != copied_content {
            return Err(AppError::Runtime(
                "Diff验证失败：version-config/openclaw.json 与主配置文件内容不一致".to_string()
            ));
        }

        tracing::info!("Config synced to version-config/ and diff verified");

        // 7. 在 version-config/ 中执行 Git 操作
        self.git_add("openclaw.json").await?;

        // 8. 检查是否有变更需要提交
        if !self.has_changes().await? {
            tracing::info!("No changes to commit in version-config/");
            return Ok(None);
        }

        // 9. 提交
        let msg = commit_msg.unwrap_or_else(|| "Update config".to_string());
        let commit_id = self.git_commit(&msg).await?;

        tracing::info!("Config committed to Git with id: {}", commit_id);
        Ok(Some(commit_id))
    }

    /// 获取 Provider 模块配置
    pub async fn get_providers(&self) -> Result<Vec<serde_json::Value>> {
        let config = self.get_config().await?;
        
        // 尝试从 models.providers 获取
        if let Some(providers_obj) = config
            .get("models")
            .and_then(|m| m.get("providers"))
            .and_then(|p| p.as_object())
        {
            let providers: Vec<serde_json::Value> = providers_obj
                .iter()
                .map(|(id, value)| {
                    let version = if id.starts_with("moonshot-") {
                        value.get("baseUrl")
                            .and_then(|u| u.as_str())
                            .map(|url| {
                                if url.contains("kimi.com") {
                                    "coding"
                                } else if url.contains("moonshot.cn") {
                                    "cn"
                                } else {
                                    "ai"
                                }
                            })
                            .map(|s| s.to_string())
                    } else {
                        None
                    };

                    serde_json::json!({
                        "id": id,
                        "version": version,
                        "enabled": value.get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        "api": value.get("api")
                            .and_then(|v| v.as_str())
                            .unwrap_or("anthropic-messages"),
                        "apiKey": value.get("apiKey").cloned().unwrap_or(serde_json::Value::Null),
                        "baseUrl": value.get("baseUrl").cloned().unwrap_or(serde_json::Value::Null),
                        "defaultModel": value.get("models")
                            .and_then(|m| m.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|m| m.get("id"))
                            .cloned()
                            .or_else(|| value.get("defaultModel").cloned()),
                    })
                })
                .collect();
            
            return Ok(providers);
        }
        
        // 返回空数组
        Ok(vec![])
    }

    /// 保存 Provider 实例
    pub async fn save_provider(
        &self,
        provider_id: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        let mut config = self.get_config().await?;
        
        // 确保 models.providers 路径存在
        if config.get("models").is_none() {
            config["models"] = serde_json::json!({});
        }
        if config["models"].get("providers").is_none() {
            config["models"]["providers"] = serde_json::json!({});
        }
        
        // 构造 OpenClaw 需要的 provider 数据（只保留必要字段）
        let default_model = data.get("defaultModel").and_then(|v| v.as_str());
        let model_name = data
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| default_model.unwrap_or(provider_id).to_string());
        
        // 构造完整的模型配置
        let model_config = if let Some(model_id) = default_model {
            serde_json::json!([{
                "id": model_id,
                "name": model_name,
                "contextWindow": 262144,
                "maxTokens": 32768,
                "input": ["text", "image"],
                "reasoning": true,
                "cost": {
                    "input": 0,
                    "output": 0,
                    "cacheRead": 0,
                    "cacheWrite": 0
                }
            }])
        } else {
            data.get("models").cloned().unwrap_or_else(|| serde_json::json!([]))
        };
        
        // 验证并修正 API 类型（必须符合 OpenClaw 支持的类型）
        let valid_api_types = vec![
            "openai-responses",
            "openai-completions",
            "openai-codex-responses",
            "anthropic-messages",
            "google-generative-ai",
            "github-copilot",
            "bedrock-converse-stream",
            "ollama",
        ];
        
        let api_type = data
            .get("api")
            .and_then(|v| v.as_str())
            .map(|s| {
                // 如果是旧版 "openai-chat"，自动转换为 "openai-responses"
                if s == "openai-chat" {
                    "openai-responses"
                } else if valid_api_types.contains(&s) {
                    s
                } else {
                    // 默认回退
                    "openai-responses"
                }
            })
            .unwrap_or("openai-responses");
        
        // 只保留 OpenClaw 需要的字段
        // 注意：OpenClaw 的 provider 配置不支持 enabled 字段
        let provider_data = serde_json::json!({
            "api": api_type,
            "apiKey": data.get("apiKey"),
            "baseUrl": data.get("baseUrl"),
            "models": model_config,
        });
        
        // 更新指定 provider
        config["models"]["providers"][provider_id] = provider_data;
        
        // 同时更新 agents.models，使模型对 agents 可用
        let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        if enabled {
            if let Some(model_id) = default_model {
                // 确保 agents.defaults.models 路径存在
                if config.get("agents").is_none() {
                    config["agents"] = serde_json::json!({});
                }
                if config["agents"].get("defaults").is_none() {
                    config["agents"]["defaults"] = serde_json::json!({});
                }
                if config["agents"]["defaults"].get("models").is_none() {
                    config["agents"]["defaults"]["models"] = serde_json::json!({});
                }
                
                // 构建模型引用键: provider_id/model_id
                let model_key = format!("{}/{}", provider_id, model_id);
                
                // 添加到 agents.models
                config["agents"]["defaults"]["models"][&model_key] = serde_json::json!({
                    "alias": model_name
                });
            }
        }
        
        self.save_config(&config).await?;
        Ok(())
    }

    /// 删除 Provider 实例
    pub async fn delete_provider(
        &self,
        provider_id: &str,
    ) -> Result<()> {
        let mut config = self.get_config().await?;
        
        if let Some(providers) = config
            .get_mut("models")
            .and_then(|m| m.get_mut("providers"))
            .and_then(|p| p.as_object_mut())
        {
            providers.remove(provider_id);
        }
        
        self.save_config(&config).await?;
        Ok(())
    }

    /// 获取模型优先级
    pub async fn get_model_priority(&self) -> Result<ModelPriority> {
        let config = self.get_config().await?;
        
        let primary = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("model"))
            .and_then(|m| m.get("primary"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let fallbacks: Vec<String> = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("model"))
            .and_then(|m| m.get("fallbacks"))
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(ModelPriority { primary, fallbacks })
    }

    /// 保存模型优先级
    pub async fn save_model_priority(
        &self,
        primary: &str,
        fallbacks: &[String],
    ) -> Result<()> {
        let mut config = self.get_config().await?;
        
        // 确保 agents.defaults.model 路径存在
        if config.get("agents").is_none() {
            config["agents"] = serde_json::json!({});
        }
        if config["agents"].get("defaults").is_none() {
            config["agents"]["defaults"] = serde_json::json!({});
        }
        if config["agents"]["defaults"].get("model").is_none() {
            config["agents"]["defaults"]["model"] = serde_json::json!({});
        }
        
        config["agents"]["defaults"]["model"]["primary"] = serde_json::Value::String(primary.to_string());
        config["agents"]["defaults"]["model"]["fallbacks"] = 
            serde_json::Value::Array(fallbacks.iter().map(|s| serde_json::Value::String(s.clone())).collect());
        
        self.save_config(&config).await?;
        Ok(())
    }

    // ==================== Agent 模块配置方法 ====================

    /// 获取 Agent 模块配置
    pub async fn get_agents(&self) -> Result<serde_json::Value> {
        let config = self.get_config().await?;
        
        let agents = config
            .get("agents")
            .cloned()
            .unwrap_or_else(|| {
                serde_json::json!({
                    "defaults": {
                        "workspace": "~/.openclaw/workspace",
                        "compaction": {
                            "mode": "safeguard"
                        },
                        "maxConcurrent": 4,
                        "subagents": {
                            "maxConcurrent": 8
                        },
                        "memorySearch": {
                            "enabled": true,
                            "provider": "ollama",
                            "remote": {
                                "baseUrl": "http://localhost:11434"
                            },
                            "model": "qwen3-embedding:0.6b",
                            "fallback": "none",
                            "sources": ["memory", "sessions"],
                            "query": {
                                "hybrid": {
                                    "enabled": false,
                                    "vectorWeight": 0.7,
                                    "textWeight": 0.3,
                                    "mmr": { "enabled": false },
                                    "temporalDecay": { "enabled": false, "halfLifeDays": 30 }
                                }
                            },
                            "store": {
                                "vector": {
                                    "enabled": false,
                                    "extensionPath": "~/.openclaw/extensions/vec0.so"
                                }
                            },
                            "sync": {
                                "sessions": {
                                    "deltaBytes": 100000,
                                    "deltaMessages": 50
                                }
                            },
                            "experimental": {
                                "sessionMemory": true
                            }
                        },
                        "model": {
                            "primary": "",
                            "fallbacks": []
                        },
                        "models": {}
                    },
                    "list": [],
                })
            });
        
        // 过滤掉 OpenClaw 不支持的字段（mode 和 agentDir）
        let mut agents_filtered = agents.clone();
        if let Some(obj) = agents_filtered.as_object_mut() {
            // 移除 mode 字段
            obj.remove("mode");
            // 移除 defaults.agentDir
            if let Some(defaults) = obj.get_mut("defaults") {
                if let Some(def_obj) = defaults.as_object_mut() {
                    def_obj.remove("agentDir");
                }
            }
        }
        
        Ok(agents_filtered)
    }

    /// 保存 Agent 模块配置
    pub async fn save_agents(&self, agents: &serde_json::Value) -> Result<()> {
        let mut config = self.get_config().await?;
        
        // 获取现有 agents 配置（如果存在）
        let existing_agents = config.get("agents").cloned().unwrap_or_else(|| {
            serde_json::json!({
                "defaults": {
                    "workspace": "~/.openclaw/workspace",
                    "compaction": {"mode": "safeguard"},
                    "maxConcurrent": 4,
                    "subagents": {"maxConcurrent": 8},
                    "memorySearch": {
                        "enabled": true,
                        "provider": "ollama",
                        "remote": {"baseUrl": "http://localhost:11434"},
                        "model": "qwen3-embedding:0.6b",
                        "fallback": "none",
                        "sources": ["memory", "sessions"],
                        "query": {"hybrid": {"enabled": false}},
                        "store": {"vector": {"enabled": false}},
                        "sync": {"sessions": {"deltaBytes": 100000, "deltaMessages": 50}},
                        "experimental": {"sessionMemory": true}
                    },
                    "model": {"primary": "", "fallbacks": []},
                    "models": {}
                },
                "list": [],
            })
        });
        
        // 合并配置：保留现有 defaults 中前端未提供的字段
        let merged = if let (Some(existing_obj), Some(new_obj)) = (existing_agents.as_object(), agents.as_object()) {
            let mut result = existing_obj.clone();
            
            // 注意：OpenClaw 不支持 agents.mode 字段，不保存 mode
            // 只保存 list
            if let Some(list) = new_obj.get("list") {
                result.insert("list".to_string(), list.clone());
            }
            
            // 合并 defaults：前端提供的字段覆盖，未提供的保留原值
            // 注意：OpenClaw 不支持 defaults.agentDir，需要过滤掉
            if let Some(new_defaults) = new_obj.get("defaults") {
                if let Some(new_def_obj) = new_defaults.as_object() {
                    // 如果有现有的 defaults，合并；否则直接使用新的
                    let mut merged_defaults = if let Some(existing_defaults) = existing_obj.get("defaults") {
                        if let Some(existing_def_obj) = existing_defaults.as_object() {
                            existing_def_obj.clone()
                        } else {
                            serde_json::Map::new()
                        }
                    } else {
                        serde_json::Map::new()
                    };
                    
                    for (key, value) in new_def_obj.iter() {
                        // 过滤掉 OpenClaw 不支持的字段
                        if key != "agentDir" {
                            eprintln!("[DEBUG] Merging key: {}, value type: {:?}", key, value);
                            merged_defaults.insert(key.clone(), value.clone());
                        }
                    }
                    result.insert("defaults".to_string(), serde_json::Value::Object(merged_defaults));
                }
            }
            
            serde_json::Value::Object(result)
        } else {
            agents.clone()
        };
        
        // 确保 config 是 Object 类型
        if let Some(config_obj) = config.as_object_mut() {
            config_obj.insert("agents".to_string(), merged);
        } else {
            return Err(AppError::Internal("配置根节点必须是对象".to_string()));
        }
        self.save_config(&config).await?;
        Ok(())
    }

    // ==================== Memory 模块配置方法 ====================

    /// 获取 Memory 模块配置
    /// 返回 None 表示配置不存在，由前端决定默认值
    pub async fn get_memory(&self) -> Result<Option<serde_json::Value>> {
        let config = self.get_config().await?;
        
        let memory = config
            .get("agents")
            .and_then(|a| a.get("defaults"))
            .and_then(|d| d.get("memorySearch"))
            .cloned();
        
        Ok(memory)
    }

    /// 保存 Memory 模块配置
    pub async fn save_memory(&self, memory: &serde_json::Value) -> Result<()> {
        let mut config = self.get_config().await?;
        
        // 确保 agents.defaults 路径存在
        if config.get("agents").is_none() {
            config["agents"] = serde_json::json!({});
        }
        if config["agents"].get("defaults").is_none() {
            config["agents"]["defaults"] = serde_json::json!({});
        }
        
        config["agents"]["defaults"]["memorySearch"] = memory.clone();
        self.save_config(&config).await?;
        Ok(())
    }

    // ==================== Channel 模块配置方法 ====================

    /// 获取 Channel 模块配置
    pub async fn get_channels(&self) -> Result<serde_json::Value> {
        let config = self.get_config().await?;
        
        let channels = config
            .get("channels")
            .cloned()
            .unwrap_or_else(|| {
                serde_json::json!({
                    "mattermost": {
                        "enabled": false,
                        "dmPolicy": "pairing",
                        "groupPolicy": "allowlist",
                        "accounts": {},
                        "allowFrom": [],
                        "mediaLocalRoots": []
                    }
                })
            });
        
        Ok(channels)
    }

    /// 保存 Channel 模块配置
    pub async fn save_channels(&self, channels: &serde_json::Value) -> Result<()> {
        let mut config = self.get_config().await?;
        config["channels"] = channels.clone();
        self.save_config(&config).await?;
        Ok(())
    }
}

/// 模型优先级
#[derive(Debug)]
pub struct ModelPriority {
    pub primary: String,
    pub fallbacks: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    /// 创建临时配置目录和 ConfigManager
    async fn create_test_manager() -> (tempfile::TempDir, ConfigManager) {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("openclaw.json");
        let manager = ConfigManager::with_config_path(config_path);
        (temp_dir, manager)
    }

    #[tokio::test]
    async fn test_get_config_valid() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入合法的 openclaw.json
        let config = serde_json::json!({
            "version": "1.0",
            "models": {},
            "channels": []
        });
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证 get_config 返回正确
        let result = manager.get_config().await.unwrap();
        assert_eq!(result.get("version").unwrap().as_str().unwrap(), "1.0");
    }

    #[tokio::test]
    async fn test_get_config_missing_file() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 确保文件不存在
        assert!(!manager.config_path.exists());
        
        // 验证返回 ConfigNotFound 错误
        let result = manager.get_config().await;
        assert!(matches!(result, Err(AppError::ConfigNotFound)));
    }

    #[tokio::test]
    async fn test_get_config_invalid_json() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入不合法的 JSON
        fs::write(&manager.config_path, "{ invalid json").await.unwrap();
        
        // 验证返回错误
        let result = manager.get_config().await;
        assert!(matches!(result, Err(AppError::Json(_))));
    }

    #[tokio::test]
    async fn test_get_config_empty_object() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入空对象
        fs::write(&manager.config_path, "{}").await.unwrap();
        
        // 验证能正常解析
        let result = manager.get_config().await.unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_save_config() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 保存配置
        let config = serde_json::json!({
            "version": "2.0",
            "test": "data"
        });
        manager.save_config(&config).await.unwrap();
        
        // 重新读取并验证内容一致
        let result = manager.get_config().await.unwrap();
        assert_eq!(result.get("version").unwrap().as_str().unwrap(), "2.0");
        assert_eq!(result.get("test").unwrap().as_str().unwrap(), "data");
    }

    #[tokio::test]
    async fn test_save_config_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        // 创建一个深层嵌套的路径，父目录不存在
        let config_path = temp_dir.path().join("a/b/c/openclaw.json");
        let manager = ConfigManager::with_config_path(config_path);
        
        // 确保父目录不存在
        assert!(!manager.config_path.parent().unwrap().exists());
        
        // 保存配置
        let config = serde_json::json!({"test": true});
        manager.save_config(&config).await.unwrap();
        
        // 验证父目录被自动创建且文件存在
        assert!(manager.config_path.parent().unwrap().exists());
        assert!(manager.config_path.exists());
    }

    #[tokio::test]
    async fn test_get_providers_empty() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入没有 providers 的配置
        let config = serde_json::json!({"models": {}});
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证返回空数组
        let providers = manager.get_providers().await.unwrap();
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn test_get_providers_with_data() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入包含 providers 的配置
        let config = serde_json::json!({
            "models": {
                "providers": {
                    "test-provider": {
                        "api": "openai-responses",
                        "apiKey": "sk-test",
                        "baseUrl": "https://api.test.com",
                        "enabled": true,
                        "models": [{"id": "model-1", "name": "Test Model"}]
                    }
                }
            }
        });
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证返回正确的结构
        let providers = manager.get_providers().await.unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].get("id").unwrap().as_str().unwrap(), "test-provider");
    }

    #[tokio::test]
    async fn test_save_provider_new() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 先创建基础配置
        let config = serde_json::json!({"models": {"providers": {}}});
        manager.save_config(&config).await.unwrap();
        
        // 保存新的 provider
        let provider_data = serde_json::json!({
            "api": "openai-responses",
            "apiKey": "sk-new",
            "baseUrl": "https://new.provider.com",
            "defaultModel": "gpt-4"
        });
        manager.save_provider("new-provider", &provider_data).await.unwrap();
        
        // 验证配置已保存
        let config = manager.get_config().await.unwrap();
        let providers = config["models"]["providers"].as_object().unwrap();
        assert!(providers.contains_key("new-provider"));
        assert_eq!(
            providers["new-provider"]["apiKey"].as_str().unwrap(),
            "sk-new"
        );
    }

    #[tokio::test]
    async fn test_save_provider_update() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 创建包含现有 provider 的配置
        let config = serde_json::json!({
            "models": {
                "providers": {
                    "existing": {
                        "api": "openai-responses",
                        "apiKey": "sk-old",
                        "baseUrl": "https://old.com"
                    }
                }
            }
        });
        manager.save_config(&config).await.unwrap();
        
        // 更新 provider
        let provider_data = serde_json::json!({
            "api": "anthropic-messages",
            "apiKey": "sk-new",
            "baseUrl": "https://new.com"
        });
        manager.save_provider("existing", &provider_data).await.unwrap();
        
        // 验证已更新
        let config = manager.get_config().await.unwrap();
        assert_eq!(
            config["models"]["providers"]["existing"]["apiKey"].as_str().unwrap(),
            "sk-new"
        );
        assert_eq!(
            config["models"]["providers"]["existing"]["api"].as_str().unwrap(),
            "anthropic-messages"
        );
    }

    #[tokio::test]
    async fn test_delete_provider() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 创建包含 provider 的配置
        let config = serde_json::json!({
            "models": {
                "providers": {
                    "to-delete": {"apiKey": "sk-test"},
                    "keep": {"apiKey": "sk-keep"}
                }
            }
        });
        manager.save_config(&config).await.unwrap();
        
        // 删除 provider
        manager.delete_provider("to-delete").await.unwrap();
        
        // 验证已删除
        let config = manager.get_config().await.unwrap();
        let providers = config["models"]["providers"].as_object().unwrap();
        assert!(!providers.contains_key("to-delete"));
        assert!(providers.contains_key("keep"));
    }

    #[tokio::test]
    async fn test_get_agents_default() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入没有 agents 的配置
        let config = serde_json::json!({});
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证返回默认值
        let agents = manager.get_agents().await.unwrap();
        assert!(agents.get("defaults").is_some());
        assert!(agents.get("list").is_some());
    }

    #[tokio::test]
    async fn test_save_agents() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 先创建基础配置
        let config = serde_json::json!({"agents": {"defaults": {}, "list": []}});
        manager.save_config(&config).await.unwrap();
        
        // 保存 agents 配置
        let agents = serde_json::json!({
            "defaults": {
                "workspace": "/custom/workspace"
            },
            "list": [{"id": "agent-1", "name": "Test Agent"}]
        });
        manager.save_agents(&agents).await.unwrap();
        
        // 验证已保存
        let config = manager.get_config().await.unwrap();
        assert_eq!(
            config["agents"]["defaults"]["workspace"].as_str().unwrap(),
            "/custom/workspace"
        );
    }

    #[tokio::test]
    async fn test_get_memory_none() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入没有 memory 配置的配置
        let config = serde_json::json!({"agents": {"defaults": {}}});
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证返回 None
        let memory = manager.get_memory().await.unwrap();
        assert!(memory.is_none());
    }

    #[tokio::test]
    async fn test_save_memory() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 先创建基础配置
        let config = serde_json::json!({"agents": {"defaults": {}}});
        manager.save_config(&config).await.unwrap();
        
        // 保存 memory 配置
        let memory = serde_json::json!({
            "enabled": true,
            "provider": "ollama",
            "model": "test-model"
        });
        manager.save_memory(&memory).await.unwrap();
        
        // 验证已保存
        let config = manager.get_config().await.unwrap();
        assert_eq!(
            config["agents"]["defaults"]["memorySearch"]["model"].as_str().unwrap(),
            "test-model"
        );
    }

    #[tokio::test]
    async fn test_get_channels_default() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 写入没有 channels 的配置
        let config = serde_json::json!({});
        fs::write(&manager.config_path, config.to_string()).await.unwrap();
        
        // 验证返回默认值
        let channels = manager.get_channels().await.unwrap();
        assert!(channels.get("mattermost").is_some());
        assert_eq!(
            channels["mattermost"]["enabled"].as_bool().unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn test_save_channels() {
        let (_temp_dir, manager) = create_test_manager().await;
        
        // 先创建基础配置
        let config = serde_json::json!({});
        manager.save_config(&config).await.unwrap();
        
        // 保存 channels 配置
        let channels = serde_json::json!({
            "mattermost": {
                "enabled": true,
                "dmPolicy": "allow"
            }
        });
        manager.save_channels(&channels).await.unwrap();
        
        // 验证已保存
        let config = manager.get_config().await.unwrap();
        assert_eq!(
            config["channels"]["mattermost"]["enabled"].as_bool().unwrap(),
            true
        );
    }
}