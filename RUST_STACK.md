# Claw One - Rust 项目结构

## 技术栈确认

| 层级 | 技术 | 说明 |
|------|------|------|
| 后端 | Rust + Tokio + Axum | 高性能异步服务 |
| 前端 | React + TypeScript + Vite | 现代前端栈 |
| Desktop | Tauri | Rust 原生封装 |

---

## 项目目录结构

```
claw-one/
├── Cargo.toml              # Workspace 根配置
├── crates/                 # 多 crate 工作区
│   ├── claw-manager/       # 主服务（Config Guardian + Runtime Adapter）
│   ├── claw-runtime/       # CRI 接口定义 + 适配器实现
│   ├── claw-sandbox/       # 安全沙盒（用户隔离、权限管理）
│   ├── claw-api/           # HTTP API 定义（前后端共享）
│   └── claw-cli/           # 命令行工具
├── desktop/                # Tauri Desktop 应用
│   ├── src-tauri/          # Rust 端（调用 claw-manager）
│   └── src/                # React 前端
├── web/                    # Web UI（独立部署用，与 desktop/src 共享组件）
├── docs/                   # 文档
└── scripts/                # 构建脚本
```

---

## Crate 详细设计

### 1. claw-runtime (CRI 接口层)

```rust
// crates/claw-runtime/src/lib.rs
pub trait ClawRuntime: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    async fn start(&self, config_path: &Path) -> Result<(), RuntimeError>;
    async fn stop(&self, force: bool) -> Result<(), RuntimeError>;
    async fn status(&self) -> Result<RuntimeStatus, RuntimeError>;
    async fn health(&self) -> Result<HealthStatus, RuntimeError>;
    async fn validate(&self, config: &Config) -> Result<ValidationResult, RuntimeError>;
    
    // 日志流
    fn logs(&self, options: LogOptions) -> BoxStream<'static, Result<LogEntry, RuntimeError>>;
}

// 适配器注册表
pub struct RuntimeRegistry {
    adapters: HashMap<String, Box<dyn ClawRuntime>>,
}

// OpenClaw 适配器实现
pub struct OpenClawAdapter {
    install_path: PathBuf,
    process: Option<Child>,
}
```

**依赖**:
```toml
[dependencies]
async-trait = "0.1"
tokio = { version = "1", features = ["process", "io-util"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
futures = "0.3"
```

### 2. claw-manager (核心服务)

```rust
// crates/claw-manager/src/lib.rs
pub struct ConfigGuardian {
    runtime: Arc<dyn ClawRuntime>,
    snapshot_store: Arc<dyn SnapshotStore>,
    state: RwLock<GuardianState>,
}

impl ConfigGuardian {
    /// 事务性配置应用
    pub async fn apply_config(&self, config: Config) -> Result<ApplyResult, ConfigError> {
        // 1. 创建快照
        let snapshot = self.snapshot_store.create(&config).await?;
        
        // 2. 验证
        self.runtime.validate(&config).await?;
        
        // 3. 持久化事务状态（用于崩溃恢复）
        self.persist_transaction(&snapshot).await?;
        
        // 4. 应用配置
        self.runtime.apply(&config).await?;
        
        // 5. 健康检查
        if !self.wait_for_healthy(Duration::from_secs(30)).await {
            self.rollback(&snapshot).await?;
            return Err(ConfigError::HealthCheckFailed);
        }
        
        // 6. 提交
        self.commit_transaction().await?;
        Ok(ApplyResult::success(snapshot.id))
    }
    
    /// 崩溃恢复
    pub async fn recover(&self) -> Result<(), ConfigError> {
        if let Some(txn) = self.load_pending_transaction().await? {
            // 有未完成事务，回滚到上一版本
            self.rollback(&txn.previous_snapshot).await?;
        }
        Ok(())
    }
}
```

**依赖**:
```toml
[dependencies]
claw-runtime = { path = "../claw-runtime" }
claw-sandbox = { path = "../claw-sandbox" }
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }
sled = "0.34"              # 嵌入式 KV 存储
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
config = "0.14"
thiserror = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 3. claw-sandbox (安全沙盒)

```rust
// crates/claw-sandbox/src/lib.rs
pub struct Sandbox {
    level: IsolationLevel,
    user: Option<User>,
}

pub enum IsolationLevel {
    L0,  // 裸机（无隔离）
    L1,  // 用户沙盒（默认）
    L2,  // 系统容器
    L3,  // 硬件隔离
}

impl Sandbox {
    /// 创建隔离用户
    pub async fn create_user(&self, username: &str) -> Result<User, SandboxError> {
        // 调用 nix 库或系统命令创建用户
    }
    
    /// 设置 chroot
    pub async fn setup_chroot(&self, root: &Path) -> Result<(), SandboxError> {
        // 使用 unshare + chroot
    }
    
    /// 设置 cgroup 资源限制
    pub async fn setup_cgroup(&self, pid: u32, limits: ResourceLimits) -> Result<(), SandboxError> {
        // cgroup v2
    }
}
```

**依赖**:
```toml
[dependencies]
nix = { version = "0.28", features = ["user", "process", "mount"] }
caps = "0.5"               # Linux capabilities
sysinfo = "0.30"           # 系统信息
walkdir = "2"              # 目录遍历
```

### 4. claw-api (共享 API 定义)

```rust
// crates/claw-api/src/lib.rs
use serde::{Deserialize, Serialize};

// 请求/响应类型定义
// 前后端共享，确保类型一致

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyConfigRequest {
    pub config: Config,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyConfigResponse {
    pub success: bool,
    pub snapshot_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeStatusResponse {
    pub state: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}
```

### 5. claw-cli (命令行工具)

```rust
// crates/claw-cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动服务
    Start {
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// 停止服务
    Stop,
    /// 查看状态
    Status,
    /// 应用配置
    Apply {
        #[arg(short, long)]
        file: PathBuf,
    },
    /// 回滚配置
    Rollback {
        #[arg(short, long)]
        snapshot: Option<String>,
    },
    /// 进入 Safe Mode
    SafeMode,
}
```

**依赖**:
```toml
[dependencies]
claw-manager = { path = "../claw-manager" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

---

## Workspace Cargo.toml

```toml
# Cargo.toml (根)
[workspace]
members = [
    "crates/claw-runtime",
    "crates/claw-manager",
    "crates/claw-sandbox",
    "crates/claw-api",
    "crates/claw-cli",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

---

## Desktop (Tauri)

```
desktop/
├── src/                    # React 前端代码
│   ├── components/
│   ├── pages/
│   ├── api/               # 调用 Tauri command
│   └── App.tsx
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── main.rs        # 入口
│   │   └── commands.rs    # Tauri command 定义
│   ├── Cargo.toml         # 依赖 claw-manager
│   └── tauri.conf.json
└── package.json
```

**src-tauri/src/commands.rs**:
```rust
use claw_manager::ConfigGuardian;
use tauri::State;

#[tauri::command]
async fn apply_config(
    config: String,
    guardian: State<'_, ConfigGuardian>,
) -> Result<String, String> {
    let config: Config = serde_json::from_str(&config).map_err(|e| e.to_string())?;
    let result = guardian.apply_config(config).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_string(&result).unwrap())
}

#[tauri::command]
async fn get_runtime_status(guardian: State<'_, ConfigGuardian>) -> Result<String, String> {
    let status = guardian.runtime.status().await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_string(&status).unwrap())
}
```

---

## 开发流程建议

### Week 1-2: 基础设施
1. 搭建 Workspace 结构
2. 实现 `claw-api`（共享类型定义）
3. 实现 `claw-runtime` 基础接口
4. 实现 OpenClaw 适配器（进程管理）

### Week 3-4: 核心功能
1. 实现 `claw-sandbox`（L1 用户沙盒）
2. 实现 `claw-manager` Config Guardian
3. 实现事务性配置 + 快照系统
4. 实现 Safe Mode 基础版本

### Week 5-6: Web UI
1. 搭建 React 项目
2. 配置表单组件
3. 对接后端 API
4. Tauri 封装

### Week 7-8: 集成测试
1. 端到端测试
2. 性能优化
3. 文档编写

---

## 风险缓解

| 风险 | 缓解措施 |
|------|---------|
| Rust 开发速度慢 | 严格限制 MVP 范围，边缘功能延后 |
| 借用检查学习成本 | 先用 `Arc<Mutex<T>>` 绕过复杂生命周期，后期优化 |
| 异步生态复杂 | 只用 tokio，避免混用其他运行时 |
| 与 OpenClaw 集成 | 前期用 HTTP API 桥接，后期可考虑直接集成 |

---

## 下一步行动

1. 初始化 Git 仓库
2. 创建 Workspace 结构
3. 添加基础依赖
4. 从 `claw-api` 和 `claw-runtime` 开始实现
