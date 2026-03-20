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
        // 首先尝试从 CLAW_OPENCLAW_CONFIG 获取（专门用于 openclaw.json）
        let config_path = std::env::var("CLAW_OPENCLAW_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // 尝试从 claw-one.toml 读取 openclaw_home
                let openclaw_home = Self::get_openclaw_home_from_settings();
                PathBuf::from(openclaw_home).join("openclaw.json")
            });
        
        let git_dir = config_path.parent()
            .expect("Invalid config path")
            .to_path_buf();
        
        Self {
            config_path,
            git_dir,
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

        // 确保 .gitignore 存在（无论 Git 仓库是否新创建）
        let gitignore_path = self.git_dir.join(".gitignore");
        let gitignore_content = "# Auto-generated by Claw One\n\
                                # Exclude nested git repositories\n\
                                agents/*/workspace*/\n\
                                workspace-*/\n\
                                .git/\n";
        
        // 检查 .gitignore 是否需要创建或更新
        // 支持旧格式 agents/*/workspace* 和新格式 agents/*/workspace*/
        let need_update = match tokio::fs::read_to_string(&gitignore_path).await {
            Ok(content) => !content.contains("agents/*/workspace"),
            Err(_) => true,
        };

        if need_update {
            tokio::fs::write(&gitignore_path, gitignore_content).await
                .map_err(|e| AppError::Git(format!("Failed to create/update .gitignore: {}", e)))?;
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

    pub async fn git_add(&self, path: &str) -> Result<()> {
        // 如果 path 是 "."，改为只添加配置文件，避免添加子目录中的其他 git 仓库
        let target = if path == "." {
            self.config_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("openclaw.json")
        } else {
            path
        };
        
        let output = Command::new("git")
            .args([
                "-C", self.git_dir.to_str().unwrap(),
                "add",
                target,
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

    pub async fn has_changes(&self) -> Result<bool> {
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

    // ==================== 模块级配置方法 ====================

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