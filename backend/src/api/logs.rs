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

pub async fn handler(
    Query(query): Query<LogsQuery>,
) -> Result<Json<serde_json::Value>> {
    // TODO: 实现日志获取
    // 1. 尝试读取 ~/.openclaw/logs/gateway.log
    // 2. 如果失败，尝试 openclaw logs 命令
    
    let logs = get_logs(query.limit).await?;

    Ok(Json(serde_json::json!({
        "logs": logs,
    })))
}

async fn get_logs(limit: usize) -> Result<String> {
    use tokio::fs;
    
    // 优先尝试直接读文件
    let log_path = dirs::home_dir()
        .ok_or_else(|| crate::error::AppError::Internal("No home directory".to_string()))?
        .join(".openclaw")
        .join("logs")
        .join("gateway.log");
    
    match fs::read_to_string(&log_path).await {
        Ok(content) => {
            // 只返回最后 limit 行
            let lines: Vec<&str> = content.lines().collect();
            let start = lines.len().saturating_sub(limit);
            Ok(lines[start..].join("\n"))
        }
        Err(_) => {
            // 备用：通过 CLI 获取
            get_logs_via_cli(limit).await
        }
    }
}

async fn get_logs_via_cli(limit: usize) -> Result<String> {
    use tokio::process::Command;
    
    let output = Command::new("openclaw")
        .args([
            "logs",
            "--limit",
            &limit.to_string(),
        ])
        .output()
        .await
        .map_err(|e| crate::error::AppError::Io(e))?;
    
    if !output.status.success() {
        return Err(crate::error::AppError::Runtime(
            "Failed to get logs from CLI".to_string()
        ));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
