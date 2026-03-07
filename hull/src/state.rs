// 应用状态管理模块
// 负责管理 Normal/SafeMode 状态，以及配置应用的事务性流程

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    config::ConfigManager,
    error::{AppError, Result},
    runtime::{ErrorType, RuntimeManager, ServiceStatus},
    settings::OpenClawConfig,
    types::{OpenClawState, StateResponse},
};

/// 应用状态
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// 正常运行
    Normal,
    /// 配置应用进行中
    ApplyingConfig {
        /// 目标配置的 Git 提交 ID
        target_commit: String,
        /// 开始时间
        start_time: std::time::Instant,
    },
    /// Safe Mode（配置错误或系统错误）
    SafeMode {
        /// 错误类型
        error_type: ErrorType,
        /// 错误消息
        message: String,
        /// 是否已自动回滚
        auto_rolled_back: bool,
        /// 上一个成功的提交 ID
        last_known_good: Option<String>,
    },
}

/// 状态管理器
pub struct StateManager {
    /// 当前状态
    state: Arc<RwLock<AppState>>,
    /// 配置管理器
    config_manager: Arc<ConfigManager>,
    /// 运行时管理器
    runtime_manager: RuntimeManager,
}

impl StateManager {
    pub fn new(config_manager: Arc<ConfigManager>, openclaw_config: &OpenClawConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState::Normal)),
            config_manager,
            runtime_manager: RuntimeManager::from_config(openclaw_config),
        }
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }

    /// 获取状态响应（用于 API）
    pub async fn get_state_response(&self) -> Result<StateResponse> {
        let state = self.get_state().await;
        let can_rollback = self.config_manager.can_rollback().await?;

        match state {
            AppState::Normal => {
                // 检查 OpenClaw 实际运行状态
                let runtime_status = self.runtime_manager.status().await?;
                let openclaw_state = match runtime_status {
                    ServiceStatus::Running => OpenClawState::Running,
                    ServiceStatus::Stopped => OpenClawState::Stopped,
                    ServiceStatus::Starting => OpenClawState::Starting,
                    ServiceStatus::Failed(msg) => {
                        // 从失败消息分类错误
                        if let Some(error_type) = RuntimeManager::classify_error(&msg) {
                            match error_type {
                                ErrorType::Config => OpenClawState::ConfigError {
                                    error: msg,
                                    auto_rolled_back: false,
                                },
                                ErrorType::System => OpenClawState::SystemError { error: msg },
                            }
                        } else {
                            OpenClawState::SystemError { error: msg }
                        }
                    }
                    _ => OpenClawState::Unknown,
                };

                Ok(StateResponse {
                    state: openclaw_state,
                    current_version: self.get_current_version().await.ok(),
                    last_error: None,
                    can_rollback,
                })
            }
            AppState::ApplyingConfig { target_commit, .. } => {
                Ok(StateResponse {
                    state: OpenClawState::Starting,
                    current_version: Some(target_commit),
                    last_error: None,
                    can_rollback,
                })
            }
            AppState::SafeMode {
                error_type,
                ref message,
                auto_rolled_back,
                last_known_good,
            } => {
                let openclaw_state = match error_type {
                    ErrorType::Config => OpenClawState::ConfigError {
                        error: message.clone(),
                        auto_rolled_back,
                    },
                    ErrorType::System => OpenClawState::SystemError {
                        error: message.clone(),
                    },
                };

                Ok(StateResponse {
                    state: openclaw_state,
                    current_version: last_known_good,
                    last_error: Some(message.clone()),
                    can_rollback,
                })
            }
        }
    }

    /// 事务性配置应用
    /// 
    /// 流程：
    /// 1. 保存配置并创建 Git 提交
    /// 2. 设置状态为 ApplyingConfig
    /// 3. 重启 OpenClaw 服务
    /// 4. 等待健康检查（超时 30s）
    /// 5. 健康检查通过 → 状态恢复 Normal
    /// 6. 健康检查失败 → 分类错误 → 自动回滚（配置错误）或进入 SafeMode（系统错误）
    pub async fn apply_config(
        &self,
        config: crate::types::Config,
        message: Option<String>,
    ) -> Result<()> {
        // 1. 记录当前版本（用于回滚）
        let current_version = self.get_current_version().await.ok();

        // 2. 保存配置并创建 Git 提交
        let commit_id = self.config_manager.apply_config(config, message).await?;

        // 3. 设置状态为 ApplyingConfig
        {
            let mut state = self.state.write().await;
            *state = AppState::ApplyingConfig {
                target_commit: commit_id.clone(),
                start_time: std::time::Instant::now(),
            };
        }

        // 4. 重启 OpenClaw 服务
        if let Err(e) = self.runtime_manager.restart().await {
            // 重启失败，进入 SafeMode
            self.enter_safe_mode(
                ErrorType::System,
                format!("Failed to restart service: {}", e),
                false,
                current_version,
            ).await;
            return Err(e);
        }

        // 5. 等待健康检查
        let healthy = self.runtime_manager.wait_for_healthy().await?;

        if healthy {
            // 健康检查通过，恢复 Normal 状态
            let mut state = self.state.write().await;
            *state = AppState::Normal;
            Ok(())
        } else {
            // 健康检查失败，需要诊断问题
            self.handle_health_check_failure(current_version).await
        }
    }

    /// 手动触发 SafeMode
    pub async fn enter_safe_mode(
        &self,
        error_type: ErrorType,
        message: String,
        auto_rolled_back: bool,
        last_known_good: Option<String>,
    ) {
        let mut state = self.state.write().await;
        *state = AppState::SafeMode {
            error_type,
            message,
            auto_rolled_back,
            last_known_good,
        };
    }

    /// 从 SafeMode 恢复
    pub async fn recover_from_safe_mode(&self) -> Result<()> {
        let current_state = self.get_state().await;
        
        if !matches!(current_state, AppState::SafeMode { .. }) {
            return Err(AppError::Runtime("Not in SafeMode".to_string()));
        }

        // 重启服务尝试恢复
        self.runtime_manager.restart().await?;

        // 检查健康状态
        if self.runtime_manager.wait_for_healthy().await? {
            let mut state = self.state.write().await;
            *state = AppState::Normal;
            Ok(())
        } else {
            Err(AppError::Runtime("Service still unhealthy after recovery attempt".to_string()))
        }
    }

    /// 恢复出厂设置
    pub async fn reset_to_factory(&self) -> Result<()> {
        // TODO: 实现恢复出厂设置逻辑
        // 1. 加载 factory-config.json
        // 2. 应用配置
        // 3. 重启服务
        
        Err(AppError::Runtime("Factory reset not implemented yet".to_string()))
    }

    // 私有辅助方法

    async fn get_current_version(&self) -> Result<String> {
        let snapshots = self.config_manager.list_snapshots().await?;
        snapshots
            .first()
            .map(|s| s.id.clone())
            .ok_or_else(|| AppError::Git("No commits found".to_string()))
    }

    async fn handle_health_check_failure(
        &self,
        last_known_good: Option<String>,
    ) -> Result<()> {
        // 1. 获取服务日志
        let logs = self.runtime_manager.get_logs(50).await?;

        // 2. 从日志分类错误
        let error_type = RuntimeManager::classify_error(&logs)
            .unwrap_or(ErrorType::System);

        match error_type {
            ErrorType::Config => {
                // 配置错误：自动回滚
                if let Some(ref version) = last_known_good {
                    match self.config_manager.rollback(version).await {
                        Ok(_) => {
                            // 回滚成功，重启服务
                            let _ = self.runtime_manager.restart().await;
                            
                            // 进入 SafeMode，标记已回滚
                            self.enter_safe_mode(
                                ErrorType::Config,
                                "Configuration error detected. Automatically rolled back to last known good version.".to_string(),
                                true,
                                last_known_good,
                            ).await;
                        }
                        Err(e) => {
                            // 回滚失败，进入 SafeMode
                            self.enter_safe_mode(
                                ErrorType::Config,
                                format!("Configuration error detected. Auto-rollback failed: {}", e),
                                false,
                                last_known_good,
                            ).await;
                        }
                    }
                } else {
                    // 没有历史版本，无法回滚
                    self.enter_safe_mode(
                        ErrorType::Config,
                        "Configuration error detected. No previous version available for rollback.".to_string(),
                        false,
                        None,
                    ).await;
                }
            }
            ErrorType::System => {
                // 系统错误：不回滚，让用户决定
                self.enter_safe_mode(
                    ErrorType::System,
                    "System error detected (e.g., port conflict, permission denied). Please check the logs and fix the issue.".to_string(),
                    false,
                    last_known_good,
                ).await;
            }
        }

        Err(AppError::Runtime("Health check failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 这些测试需要异步运行时，使用 tokio::test
    // 实际测试将在集成测试中进行
}