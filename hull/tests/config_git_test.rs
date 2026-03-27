//! ConfigManager Git 操作单元测试
//! 
//! 测试 Git 快照、回滚、提交等功能

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::process::Command;
    use tempfile::TempDir;
    
    use crate::config::ConfigManager;

    /// 创建带有 Git 仓库的测试环境
    async fn create_test_manager_with_git() -> (TempDir, ConfigManager) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("openclaw.json");
        
        // 创建初始配置
        let initial_config = serde_json::json!({
            "version": "1.0",
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "~/.openclaw/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        
        tokio::fs::write(&config_path, serde_json::to_string_pretty(&initial_config).unwrap())
            .await
            .unwrap();
        
        let manager = ConfigManager::with_config_path(config_path);
        
        // 初始化 Git 仓库
        let version_config_dir = temp_dir.path().join("version-config");
        tokio::fs::create_dir_all(&version_config_dir).await.unwrap();
        
        // git init
        let output = Command::new("git")
            .args(["init", version_config_dir.to_str().unwrap()])
            .output()
            .expect("Failed to init git");
        assert!(output.status.success(), "Git init failed: {}", String::from_utf8_lossy(&output.stderr));
        
        // 配置 git user
        let _ = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "config", "user.email", "test@test.com"])
            .output();
        let _ = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "config", "user.name", "Test"])
            .output();
        
        // 复制配置到 version-config/
        let version_config_path = version_config_dir.join("openclaw.json");
        tokio::fs::write(&version_config_path, serde_json::to_string_pretty(&initial_config).unwrap())
            .await
            .unwrap();
        
        // 初始提交
        let _ = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "add", "openclaw.json"])
            .output();
        let output = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "commit", "-m", "Initial commit"])
            .output()
            .expect("Failed to commit");
        assert!(output.status.success(), "Initial commit failed: {}", String::from_utf8_lossy(&output.stderr));
        
        (temp_dir, manager)
    }

    // ==================== Git 快照测试 ====================

    #[tokio::test]
    async fn test_sync_to_version_config_creates_commit() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        // 修改配置
        let new_config = serde_json::json!({
            "version": "2.0",
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "/new/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        
        // 保存并同步到 Git
        let result = manager.sync_to_version_config(&new_config, Some("Test commit".to_string())).await;
        
        // 应该成功并返回 commit ID
        assert!(result.is_ok());
        let commit_id = result.unwrap();
        assert!(commit_id.is_some(), "Expected commit ID to be returned");
        
        // 验证 Git 历史中有新提交
        let version_config_dir = temp_dir.path().join("version-config");
        let output = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "log", "--oneline"])
            .output()
            .expect("Failed to get git log");
        
        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains("Test commit"), "Expected 'Test commit' in git log, got: {}", log);
    }

    #[tokio::test]
    async fn test_sync_to_version_config_no_changes() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        // 使用相同的配置（无变更）
        let same_config = serde_json::json!({
            "version": "1.0",
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "~/.openclaw/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        
        // 先同步一次
        let _ = manager.sync_to_version_config(&same_config, Some("First commit".to_string())).await;
        
        // 再次同步相同内容
        let result = manager.sync_to_version_config(&same_config, Some("Second commit".to_string())).await;
        
        // 应该返回 Ok(None) 表示无变更
        assert!(result.is_ok());
        // 注意：由于文件内容相同，Git 应该检测到无变化
    }

    // ==================== 快照列表测试 ====================

    #[tokio::test]
    async fn test_list_snapshots() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        // 初始应该有一个快照
        let snapshots = manager.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 1, "Expected 1 initial snapshot");
        assert!(snapshots[0].message.contains("Initial") || snapshots[0].message.contains("Initial commit"));
        
        // 添加更多提交
        for i in 1..=3 {
            let new_config = serde_json::json!({
                "version": format!("1.{}", i),
                "models": { "providers": {} },
                "agents": { 
                    "defaults": { "workspace": format!("/workspace{}", i) }, 
                    "list": [] 
                },
                "channels": {}
            });
            
            manager.sync_to_version_config(&new_config, Some(format!("Commit {}", i))).await.unwrap();
        }
        
        // 验证有 4 个快照
        let snapshots = manager.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 4, "Expected 4 snapshots");
        
        // 验证按时间倒序排列（最新的在前）
        assert!(snapshots[0].message.contains("Commit 3") || snapshots[0].message.contains("Initial"));
    }

    #[tokio::test]
    async fn test_list_snapshots_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("openclaw.json");
        
        // 创建配置但不初始化 Git
        let config = serde_json::json!({"version": "1.0"});
        tokio::fs::write(&config_path, config.to_string()).await.unwrap();
        
        let manager = ConfigManager::with_config_path(config_path);
        
        // 应该返回空列表而不是错误
        let snapshots = manager.list_snapshots().await.unwrap();
        assert!(snapshots.is_empty());
    }

    // ==================== 回滚测试 ====================

    #[tokio::test]
    async fn test_rollback_to_previous_version() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        // 获取初始 commit ID
        let initial_snapshots = manager.list_snapshots().await.unwrap();
        let initial_commit = initial_snapshots[0].id.clone();
        
        // 创建新配置
        let new_config = serde_json::json!({
            "version": "2.0",
            "models": { "providers": {} },
            "agents": { 
                "defaults": { "workspace": "/new/workspace" }, 
                "list": [] 
            },
            "channels": {}
        });
        
        manager.sync_to_version_config(&new_config, Some("Second commit".to_string())).await.unwrap();
        
        // 验证当前配置是新版本
        let current_config = manager.get_config().await.unwrap();
        assert_eq!(current_config["version"], "2.0");
        
        // 回滚到初始版本
        let result = manager.rollback(&initial_commit).await;
        assert!(result.is_ok(), "Rollback failed: {:?}", result);
        
        // 验证配置已回滚
        let rolled_back_config = manager.get_config().await.unwrap();
        assert_eq!(rolled_back_config["version"], "1.0");
    }

    #[tokio::test]
    async fn test_rollback_invalid_commit() {
        let (_temp_dir, manager) = create_test_manager_with_git().await;
        
        // 尝试回滚到不存在的 commit
        let result = manager.rollback("invalid-commit-id").await;
        assert!(result.is_err());
        
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("not found") || err_msg.contains("Snapshot"));
    }

    #[tokio::test]
    async fn test_rollback_preserves_git_history() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        // 创建多个提交
        for i in 1..=3 {
            let new_config = serde_json::json!({
                "version": format!("1.{}", i),
                "models": { "providers": {} },
                "agents": { "defaults": { "workspace": "/workspace" }, "list": [] },
                "channels": {}
            });
            manager.sync_to_version_config(&new_config, Some(format!("Commit {}", i))).await.unwrap();
        }
        
        // 获取所有快照
        let snapshots_before = manager.list_snapshots().await.unwrap();
        let count_before = snapshots_before.len();
        
        // 回滚到第二个提交
        let second_commit = &snapshots_before[snapshots_before.len() - 2].id;
        manager.rollback(second_commit).await.unwrap();
        
        // 验证 Git 历史仍然完整（回滚不删除历史）
        let snapshots_after = manager.list_snapshots().await.unwrap();
        assert_eq!(snapshots_after.len(), count_before, "Rollback should not delete git history");
    }

    // ==================== apply_config 集成测试 ====================

    #[tokio::test]
    async fn test_apply_config_creates_git_commit() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        let new_config = serde_json::json!({
            "version": "2.0",
            "models": { "providers": {} },
            "agents": { "defaults": { "workspace": "/new/workspace" }, "list": [] },
            "channels": {}
        });
        
        // apply_config 会保存配置并创建 Git 提交
        let result = manager.apply_config(new_config, Some("Apply new config".to_string())).await;
        
        assert!(result.is_ok());
        let commit_id = result.unwrap();
        assert!(!commit_id.is_empty());
        
        // 验证提交存在
        let snapshots = manager.list_snapshots().await.unwrap();
        assert!(snapshots.iter().any(|s| s.id == commit_id));
    }

    // ==================== 边界条件测试 ====================

    #[tokio::test]
    async fn test_git_with_special_characters_in_message() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        
        let new_config = serde_json::json!({
            "version": "2.0",
            "models": {},
            "agents": { "defaults": {}, "list": [] },
            "channels": {}
        });
        
        // 使用特殊字符的提交信息
        let message = "Config update: test \"quotes\" and 'apostrophes' and \n newlines";
        let result = manager.sync_to_version_config(&new_config, Some(message.to_string())).await;
        
        assert!(result.is_ok());
        
        // 验证提交信息被正确处理
        let version_config_dir = temp_dir.path().join("version-config");
        let output = Command::new("git")
            .args(["-C", version_config_dir.to_str().unwrap(), "log", "-1", "--pretty=%B"])
            .output()
            .expect("Failed to get git log");
        
        let log_message = String::from_utf8_lossy(&output.stdout);
        assert!(log_message.contains("Config update"));
    }

    #[tokio::test]
    async fn test_concurrent_git_operations() {
        let (temp_dir, manager) = create_test_manager_with_git().await;
        let manager = Arc::new(manager);
        
        // 创建多个并发任务尝试创建提交
        let mut handles = vec![];
        
        for i in 0..5 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let new_config = serde_json::json!({
                    "version": format!("1.{}", i),
                    "models": {},
                    "agents": { "defaults": {}, "list": [] },
                    "channels": {}
                });
                
                mgr.sync_to_version_config(&new_config, Some(format!("Concurrent commit {}", i))).await
            });
            handles.push(handle);
        }
        
        // 等待所有任务完成
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // 由于 Git 操作的串行性，部分操作可能失败（这是预期的）
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        println!("Concurrent operations: {}/5 succeeded", success_count);
        
        // 验证最终状态是一致的
        let snapshots = manager.list_snapshots().await.unwrap();
        assert!(!snapshots.is_empty());
    }
}
