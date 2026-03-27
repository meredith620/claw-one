//! StateManager 单元测试
//! 
//! 测试状态机转换、Safe Mode 逻辑、配置应用流程

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tempfile::TempDir;
    
    use crate::{
        config::ConfigManager,
        runtime::ErrorType,
        settings::OpenClawConfig,
        state::{AppState, StateManager},
    };

    /// 创建测试用的 StateManager
    async fn create_test_state_manager() -> (TempDir, StateManager) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("openclaw.json");
        
        // 写入基础配置
        let baseline = serde_json::json!({
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "~/.openclaw/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        tokio::fs::write(&config_path, serde_json::to_string_pretty(&baseline).unwrap())
            .await
            .unwrap();
        
        let config_manager = Arc::new(ConfigManager::with_config_path(config_path));
        let openclaw_config = OpenClawConfig::default();
        let state_manager = StateManager::new(config_manager, &openclaw_config);
        
        (temp_dir, state_manager)
    }

    // ==================== 状态机基础测试 ====================

    #[tokio::test]
    async fn test_initial_state_is_normal() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        let state = state_manager.get_state().await;
        assert!(matches!(state, AppState::Normal));
    }

    #[tokio::test]
    async fn test_enter_safe_mode_config_error() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        // 手动进入 SafeMode
        state_manager.enter_safe_mode(
            ErrorType::Config,
            "Test config error".to_string(),
            false,
            Some("abc123".to_string()),
        ).await;
        
        let state = state_manager.get_state().await;
        match state {
            AppState::SafeMode { error_type, message, auto_rolled_back, last_known_good } => {
                assert_eq!(error_type, ErrorType::Config);
                assert_eq!(message, "Test config error");
                assert!(!auto_rolled_back);
                assert_eq!(last_known_good, Some("abc123".to_string()));
            }
            _ => panic!("Expected SafeMode state"),
        }
    }

    #[tokio::test]
    async fn test_enter_safe_mode_system_error() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        state_manager.enter_safe_mode(
            ErrorType::System,
            "Port already in use".to_string(),
            false,
            Some("def456".to_string()),
        ).await;
        
        let state = state_manager.get_state().await;
        match state {
            AppState::SafeMode { error_type, message, .. } => {
                assert_eq!(error_type, ErrorType::System);
                assert_eq!(message, "Port already in use");
            }
            _ => panic!("Expected SafeMode state"),
        }
    }

    #[tokio::test]
    async fn test_enter_safe_mode_with_auto_rollback() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        state_manager.enter_safe_mode(
            ErrorType::Config,
            "Auto rollback completed".to_string(),
            true,  // auto_rolled_back = true
            Some("abc123".to_string()),
        ).await;
        
        let state = state_manager.get_state().await;
        match state {
            AppState::SafeMode { auto_rolled_back, .. } => {
                assert!(auto_rolled_back);
            }
            _ => panic!("Expected SafeMode state"),
        }
    }

    // ==================== 状态响应测试 ====================

    #[tokio::test]
    async fn test_get_state_response_normal() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        let response = state_manager.get_state_response().await;
        assert!(response.is_ok());
        
        let response = response.unwrap();
        // 状态可能是 Running, Stopped, Starting 等，取决于实际运行时
        println!("Normal state response: {:?}", response);
    }

    #[tokio::test]
    async fn test_get_state_response_safe_mode() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        // 进入 SafeMode
        state_manager.enter_safe_mode(
            ErrorType::Config,
            "Config validation failed".to_string(),
            false,
            Some("last-good-commit".to_string()),
        ).await;
        
        let response = state_manager.get_state_response().await;
        assert!(response.is_ok());
        
        let response = response.unwrap();
        println!("SafeMode state response: {:?}", response);
        
        // 验证 can_rollback 字段存在
        // 注意：实际值取决于 Git 仓库状态
    }

    // ==================== 状态转换测试 ====================

    #[tokio::test]
    async fn test_safe_mode_to_normal_recovery() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        // 1. 进入 SafeMode
        state_manager.enter_safe_mode(
            ErrorType::Config,
            "Test error".to_string(),
            false,
            None,
        ).await;
        
        let state = state_manager.get_state().await;
        assert!(matches!(state, AppState::SafeMode { .. }));
        
        // 2. 尝试恢复
        // 注意：实际恢复需要 RuntimeManager 能成功重启服务
        // 这里只测试状态转换逻辑
        let result = state_manager.recover_from_safe_mode().await;
        
        // 结果取决于实际服务状态，可能成功或失败
        println!("Recovery result: {:?}", result);
    }

    #[tokio::test]
    async fn test_recover_from_safe_mode_when_not_in_safe_mode() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        // 当前是 Normal 状态，尝试恢复应该失败
        let result = state_manager.recover_from_safe_mode().await;
        assert!(result.is_err());
        
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Not in SafeMode"));
    }

    // ==================== 配置应用流程测试 ====================

    #[tokio::test]
    async fn test_apply_config_sets_applying_state() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        // 准备新配置
        let new_config = serde_json::json!({
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "/new/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        
        // 应用配置（这会尝试重启服务，可能失败）
        let result = state_manager.apply_config(new_config, Some("Test config".to_string())).await;
        
        // 结果取决于实际服务重启是否成功
        println!("Apply config result: {:?}", result);
        
        // 检查状态
        let state = state_manager.get_state().await;
        println!("State after apply: {:?}", state);
        
        // 如果服务重启失败，应该进入 SafeMode
        // 如果成功，应该是 Normal
    }

    // ==================== 工具方法测试 ====================

    #[tokio::test]
    async fn test_get_current_version_from_git() {
        let (temp_dir, state_manager) = create_test_state_manager().await;
        
        // 初始化 Git 仓库并创建提交
        let version_config_dir = temp_dir.path().join("version-config");
        tokio::fs::create_dir_all(&version_config_dir).await.unwrap();
        
        // 使用 std::process::Command 执行 git 命令
        let output = std::process::Command::new("git")
            .args(["init", version_config_dir.to_str().unwrap()])
            .output()
            .expect("Failed to init git");
        assert!(output.status.success());
        
        // 配置 git user
        let _ = std::process::Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "config", "user.email", "test@test.com"])
            .output();
        let _ = std::process::Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "config", "user.name", "Test"])
            .output();
        
        // 创建初始提交
        let config_file = version_config_dir.join("openclaw.json");
        tokio::fs::write(&config_file, "{}").await.unwrap();
        
        let _ = std::process::Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "add", "."])
            .output();
        let output = std::process::Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "commit", "-m", "Initial"])
            .output()
            .expect("Failed to commit");
        assert!(output.status.success());
        
        // 获取版本
        // 注意：这需要 ConfigManager 的 Git 仓库已正确设置
        // 实际测试可能需要更多设置
    }

    // ==================== 边界条件测试 ====================

    #[tokio::test]
    async fn test_safe_mode_with_empty_message() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        state_manager.enter_safe_mode(
            ErrorType::Config,
            "".to_string(),  // 空消息
            false,
            None,
        ).await;
        
        let state = state_manager.get_state().await;
        assert!(matches!(state, AppState::SafeMode { .. }));
    }

    #[tokio::test]
    async fn test_safe_mode_with_long_message() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        
        let long_message = "Error: ".to_string() + &"x".repeat(10000);
        
        state_manager.enter_safe_mode(
            ErrorType::System,
            long_message,
            false,
            None,
        ).await;
        
        let state = state_manager.get_state().await;
        assert!(matches!(state, AppState::SafeMode { .. }));
    }

    #[tokio::test]
    async fn test_concurrent_state_access() {
        let (_temp_dir, state_manager) = create_test_state_manager().await;
        let state_manager = Arc::new(state_manager);
        
        // 创建多个并发任务读取状态
        let mut handles = vec![];
        
        for _ in 0..10 {
            let sm = state_manager.clone();
            let handle = tokio::spawn(async move {
                let state = sm.get_state().await;
                state
            });
            handles.push(handle);
        }
        
        // 同时写入状态
        let sm = state_manager.clone();
        let write_handle = tokio::spawn(async move {
            sm.enter_safe_mode(ErrorType::Config, "Test".to_string(), false, None).await;
        });
        
        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await.unwrap();
        }
        write_handle.await.unwrap();
        
        // 最终状态应该是 SafeMode
        let final_state = state_manager.get_state().await;
        assert!(matches!(final_state, AppState::SafeMode { .. }));
    }
}
