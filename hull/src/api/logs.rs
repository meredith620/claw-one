use axum::{extract::Query, Json};
use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

/// 日志处理器
/// 
/// 注意：日志获取逻辑已移至 RuntimeManager
/// API 层只负责 HTTP 协议转换，不直接执行系统命令
pub async fn handler(
    Query(query): Query<LogsQuery>,
) -> Result<Json<serde_json::Value>> {
    // 优先尝试直接读文件（无需 Runtime）
    let logs = get_logs_from_file(query.limit).await
        .or_else(|_| get_logs_from_runtime(query.limit).await)?;

    Ok(Json(serde_json::json!({
        "logs": logs,
    })))
}

/// 从日志文件读取（优先方式）
async fn get_logs_from_file(limit: usize) -> Result<String> {
    use tokio::fs;
    
    let log_path = dirs::home_dir()
        .ok_or_else(|| crate::error::AppError::Internal("No home directory".to_string()))?
        .join(".openclaw")
        .join("logs")
        .join("gateway.log");
    
    let content = fs::read_to_string(&log_path).await
        .map_err(|e| crate::error::AppError::Io(e))?;
    
    // 只返回最后 limit 行
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(limit);
    Ok(lines[start..].join("\n"))
}

/// 通过 Runtime 获取日志（备用方式）
/// 
/// 架构原则：API 层不直接执行系统命令，
/// 所有系统调用都通过 RuntimeManager 处理
async fn get_logs_from_runtime(limit: usize) -> Result<String> {
    use crate::runtime::RuntimeManager;
    
    let runtime = RuntimeManager::new();
    
    // 首先尝试 journalctl 方式
    match runtime.get_logs(limit).await {
        Ok(logs) if !logs.is_empty() => Ok(logs),
        _ => {
            // 备用：通过 openclaw CLI
            runtime.get_logs_via_cli(limit).await
        }
    }
}
