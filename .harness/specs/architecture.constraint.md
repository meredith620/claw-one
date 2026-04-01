---
# HARNESS METADATA
# type: constraint
# part-of: harness-architecture
# scope: architecture
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
---

# 架构约束

> **适用于:** 任何涉及架构变更的任务
> 
> ⚠️ **HARNESS 文件**: 本文档属于 Harness 架构，修改需谨慎

## 核心架构

```
┌──────────────────────────────────────────────┐
│  Layer 1: 表示层 (Axum HTTP API)              │
│  - 路由定义、参数解析、响应序列化              │
│  - 无业务逻辑，只负责 HTTP 协议转换            │
├──────────────────────────────────────────────┤
│  Layer 2: 应用层 (Config Guardian)            │
│  - ConfigManager: 配置 CRUD + Git 操作        │
│  - StateManager: Safe Mode 状态机             │
│  - Validation: 配置验证                       │
├──────────────────────────────────────────────┤
│  Layer 3: 基础设施层 (CRI Adapter)            │
│  - OpenClawAdapter: OpenClaw 进程管理         │
│  - 预留: PicoClawAdapter                      │
└──────────────────────────────────────────────┘
```

## 分层约束

### 1. API 层 (hull/src/api/)

**职责:**
- 定义路由和 handler
- 请求/响应序列化
- 错误转换为 HTTP 状态码

**禁止:**
- ❌ 直接调用 Git 操作
- ❌ 直接读写配置文件
- ❌ 包含业务逻辑

**正确:**
- ✅ 调用 ConfigManager/StateManager 方法
- ✅ 使用 `?` 传播错误

```rust
// ✅ api/config.rs - 只负责 HTTP
pub async fn apply_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<Value>,
) -> Result<Json<ConfigResponse>, ApiError> {
    state.config_manager.apply(config).await?;
    Ok(Json(ConfigResponse::success()))
}
```

### 2. 应用层 (hull/src/config.rs, state.rs)

**职责:**
- 业务逻辑编排
- 事务管理
- 状态机维护

**ConfigManager 职责:**
```rust
impl ConfigManager {
    // ✅ 配置 CRUD（带 Git 快照）
    pub async fn read_config(&self) -> Result<Value, Error>;
    pub async fn save_config(&self, config: &Value) -> Result<(), Error>;
    pub async fn apply(&self, config: Value) -> Result<(), Error>;
    
    // ✅ Git 操作封装
    pub async fn commit_snapshot(&self, msg: &str) -> Result<(), Error>;
    pub async fn rollback(&self, version: &str) -> Result<(), Error>;
}
```

**StateManager 职责:**
```rust
impl StateManager {
    // ✅ Safe Mode 状态机
    pub async fn enter_safe_mode(&self, reason: Error) -> Result<(), Error>;
    pub async fn exit_safe_mode(&self) -> Result<(), Error>;
    pub fn current_state(&self) -> AppState;
}
```

### 3. 运行时层 (hull/src/runtime.rs)

**职责:**
- OpenClaw 进程生命周期管理
- 健康检查

```rust
#[async_trait]
pub trait ClawRuntime: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    fn status(&self) -> RuntimeStatus;
    async fn health(&self) -> bool;
}
```

## 依赖方向

```
api/ → config.rs, state.rs → runtime.rs
  ↓         ↓                    ↓
外部    业务逻辑             系统调用
```

**禁止循环依赖!**

## 模块组织

### 文件命名

```
src/
├── main.rs           # 服务入口
├── lib.rs            # 库导出（测试用）
├── api/
│   ├── mod.rs        # 路由聚合
│   ├── config.rs     # /api/config 端点
│   ├── health.rs     # /api/health 端点
│   └── ...
├── config.rs         # ConfigManager
├── state.rs          # StateManager
├── validation.rs     # 配置验证
├── runtime.rs        # ClawRuntime trait
└── error.rs          # 错误类型定义
```

### Crate 组织

**单 crate 设计（MVP）:**
- `hull/` 一个 crate 包含全部后端代码
- 测试使用 `#[cfg(test)]` 和 `tests/` 目录

**未来拆分（V2）:**
```
crates/
├── claw-one-api/     # HTTP API
├── claw-one-core/    # Config + State + Runtime
└── claw-one-cli/     # 命令行工具
```

## 并发约束

### 单实例保证

```rust
// 使用文件锁
pub struct InstanceLock { file: File }

impl InstanceLock {
    pub fn acquire() -> Result<Self, Error> {
        let file = File::create("/tmp/claw-one.lock")?;
        file.try_lock_exclusive()?;  // fs2 crate
        Ok(InstanceLock { file })
    }
}
```

### 共享状态

```rust
// AppState 使用 Arc 共享
pub struct AppState {
    pub config_manager: ConfigManager,
    pub state_manager: StateManager,
    pub runtime: Arc<Mutex<dyn ClawRuntime>>,
}

// Axum 中传递
let app = Router::new()
    .with_state(Arc::new(app_state));
```

## 错误传播

```
底层错误 → thiserror 定义 → anyhow 上下文 → HTTP 响应
```

**每层添加上下文:**
```rust
// runtime.rs
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("进程启动失败: {0}")]
    StartFailed(String),
    #[error("健康检查失败: {0}")]
    HealthCheckFailed(String),
}

// config.rs
pub async fn apply(&self) -> Result<(), Error> {
    self.runtime.start().await
        .context("启动 OpenClaw 失败")?;  // anyhow
    // ...
}

// api/config.rs
async fn handler() -> Result<..., ApiError> {
    config_manager.apply().await
        .map_err(|e| ApiError::internal(e.to_string()))?;
}
```

## 测试架构约束

### 测试分层

| 测试类型 | 范围 | 依赖 |
|----------|------|------|
| 单元测试 | 单个模块 | 无外部依赖，mock |
| 集成测试 | API 端点 | 真实 ConfigManager，mock Runtime |
| E2E 测试 | 完整链路 | Docker，完整 OpenClaw |

### 测试隔离

```rust
// 每个测试独立临时目录
pub struct TestServer {
    temp_dir: TempDir,
    // ...
}

impl TestServer {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        // 在临时目录初始化 Git 仓库
        // ...
    }
}
```
