# Claw One 最终设计文档

**版本**: MVP 1.0  
**日期**: 2026-03-27  
**状态**: 后端实现完成，前端开发中

---

## 1. 产品定位

### 1.1 产品形态

Claw One 是 OpenClaw 的 Web 管理工具，提供配置管理、状态监控、版本回滚等功能。

### 1.2 核心目标

便捷管理 OpenClaw 常用功能，解决：
1. 配置复杂 → 可视化表单，无需手写 JSON
2. 配置易错 → 事务性配置 + 自动回滚
3. 门槛高 → Web 界面操作，无需命令行

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────┐
│  用户浏览器                              │
│  http://localhost:8080                  │
└─────────────────┬───────────────────────┘
                   │
┌─────────────────▼───────────────────────┐
│  Claw One Server                        │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │  Vue 3 前端（静态文件）          │   │
│  │  - 配置向导                      │   │
│  │  - 状态监控                      │   │
│  │  - Safe Mode 恢复界面            │   │
│  └─────────────────────────────────┘   │
│                  │                      │
│  ┌───────────────▼──────────────────┐  │
│  │  Axum HTTP API                   │  │
│  │  - /api/config (GET/POST)        │  │
│  │  - /api/state                    │  │
│  │  - /api/snapshots                │  │
│  │  - /api/rollback                 │  │
│  └───────────────┬──────────────────┘  │
│                  │                      │
│  ┌───────────────▼──────────────────┐  │
│  │  Config Guardian（核心）         │  │
│  │  - 配置验证                      │  │
│  │  - Git 快照管理                  │  │
│  │  - 自动回滚逻辑                  │  │
│  │  - Safe Mode 状态管理            │  │
│  └───────────────┬──────────────────┘  │
│                  │                      │
│  ┌───────────────▼──────────────────┐  │
│  │  Runtime Adapter (CRI)           │  │
│  │  - OpenClaw 适配器（当前）       │  │
│  │  - PicoClaw 适配器（预留）       │  │
│  └───────────────┬──────────────────┘  │
│                  │                      │
│  ┌───────────────▼──────────────────┐  │
│  │  OpenClaw 进程管理               │  │
│  │  - 启动/停止/重启                │  │
│  │  - 健康检查                      │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### 2.2 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **前端** | Vue 3 + TypeScript + Vite | 纯 Web，无 Tauri |
| **后端** | Rust + Axum + Tokio | 单 crate |
| **进程管理** | 命令行/进程管理 | 启动/停止/重启 OpenClaw |
| **快照存储** | Git | 本地仓库，自动 commit |
| **配置格式** | JSON | 与 OpenClaw 原生兼容 |

### 2.3 项目结构（单 crate）

```
claw-one/
├── hull/                   # Rust 后端
│   ├── src/
│   │   ├── main.rs         # Axum 服务启动
│   │   ├── api/            # HTTP 路由
│   │   ├── config.rs       # 配置管理 + Git 操作
│   │   ├── runtime.rs      # CRI trait
│   │   ├── state.rs        # AppState
│   │   └── validation.rs   # 配置验证
│   └── tests/              # 集成测试
├── static/                 # Vue 构建产物
├── bridge/                 # Vue 前端
├── e2e/                    # E2E 测试
└── docs/                   # 文档
```

### 2.4 实现状态

| 模块 | 状态 | 说明 |
|------|------|------|
| HTTP API | ✅ | Provider/Agent/Memory/Channel CRUD |
| Config Guardian | ✅ | Git 快照、回滚、验证 |
| StateManager | ✅ | Safe Mode 状态机 |
| 测试框架 | ✅ | 90 个测试覆盖三层架构 |
| Vue 3 前端 | ⏳ | 待开发 |
│   ├── package.json
│   └── vite.config.ts
└── docs/
```

---

## 3. 核心功能设计

### 3.1 配置验证（新增）

使用 **JSON Schema** 进行配置验证：

```rust
// src/validation.rs
const CONFIG_SCHEMA: &str = include_str!("../schemas/openclaw.schema.json");

pub fn validate_config(config: &Value) -> Result<(), Vec<String>> {
    let schema: Value = serde_json::from_str(CONFIG_SCHEMA)?;
    // 使用 jsonschema 或手动验证
    // 1. 类型检查（port 必须是 number）
    // 2. 必填项检查（models, channels）
    // 3. 范围检查（port 1-65535）
    // 4. 交叉引用检查（channels 引用的 model 必须存在）
}
```

**Schema 来源**: 参考 OpenClaw 源码中的类型定义自行维护

#### 流程
```
用户提交配置
    ↓
保存到 openclaw.json
    ↓
systemctl restart openclaw
    ↓
等待健康检查（30s 超时，可配置）
    ↓
成功？
├── 是 → git commit → 返回成功
└── 否 → 进入 Safe Mode
         ├── 配置错误 → 用户选择：
         │              1) 继续编辑配置（推荐）
         │              2) 回滚到历史版本
         │              3) 恢复出厂设置
         └── 系统错误 → 用户选择：
                        1) 尝试恢复
                        2) 查看日志
                        3) 恢复出厂设置
```

> **设计决策**: 采用手工处理而非自动回滚，理由：
> 1. 用户可快速干预，减少误操作
> 2. 保留用户配置进度，可在Safe Mode下修复
> 3. 避免意外自动回滚导致用户困惑
> 4. 便于调试，保留错误现场

#### Git 快照（更新）

```
/var/lib/claw/
├── openclaw.json           # 当前配置（~/.openclaw/openclaw.json 的链接或复制）
├── .git/                   # Git 仓库
├── factory-config.json     # 出厂配置（openclaw 初始化后的配置）
└── .applying               # 标记文件
```

**Git 提交**: 
```rust
Command::new("git")
    .args([
        "-C", "/var/lib/claw",
        "commit", 
        "-m", &format!("Config update at {}", timestamp()),
        "--author", "Claw One <dev@claw.one>"
    ])
```

**快照保留**: 近 3 个月（约 90 天），MVP 先保留最近 100 个提交
```
/var/lib/claw/
├── openclaw.json           # 当前配置
├── .git/                   # Git 仓库
│   └── 自动管理提交历史
├── factory-config.json     # 出厂配置
└── .applying               # 标记文件（应用中时存在）
```

#### 关键参数
- **健康检查超时**: 默认 30s，可配到 60s+
- **保留快照数**: 不限制，Git 自动管理
- **自动清理**: MVP 不做，预留 V2 接口

### 3.2 Safe Mode（D5）

#### 触发条件
1. **配置错误** → Safe Mode → 用户选择：继续编辑 / 回滚 / 恢复出厂
2. **系统错误** → Safe Mode → 用户选择：尝试恢复 / 查看日志 / 恢复出厂
3. **手动触发** → Safe Mode

#### 页面按钮

| 场景 | 主按钮 | 二级菜单（更多选项）|
|------|--------|-------------------|
| 配置错误 | [继续编辑配置] [回滚到历史版本] [查看日志] | [恢复出厂设置] |
| 系统错误 | [尝试恢复] [查看日志] | [恢复出厂设置] |
| 手动触发 | 同系统错误 | [恢复出厂设置] |

#### 出厂设置
- **位置**: Safe Mode 二级菜单
- **配置内容**: 空模型、空渠道（最小配置）
- **存储**: `/var/lib/claw/factory-config.json`
- **确认弹窗**: 是（防误触）

### 3.3 CRI 接口（D2）

```rust
// src/runtime.rs
#[async_trait]
pub trait ClawRuntime: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&mut self) -> Result<(), Error>;
    async fn stop(&mut self) -> Result<(), Error>;
    fn status(&self) -> RuntimeStatus;
    async fn health(&self) -> bool;
}

// MVP 只实现 OpenClaw
pub struct OpenClawAdapter { ... }
impl ClawRuntime for OpenClawAdapter { ... }

// V2 预留 PicoClaw
pub struct PicoClawAdapter { ... }
```

### 日志获取（新增）

**OpenClaw 日志位置**:
```
~/.openclaw/logs/
├── commands.log           # 命令日志
└── config-audit.jsonl     # 配置审计日志
```

**Rust 实现**:
```rust
pub async fn get_logs(tail: usize) -> Result<String, Error> {
    // 直接读取日志文件（不依赖 OpenClaw 运行）
    let log_path = dirs::home_dir()
        .unwrap()
        .join(".openclaw")
        .join("logs")
        .join("commands.log");
    
    fs::read_to_string(&log_path).await
}

// 或使用 openclaw logs 命令（如果 OpenClaw 运行中）
pub async fn get_logs_via_cli(limit: usize) -> Result<String, Error> {
    let output = Command::new("openclaw")
        .args(["logs", "--limit", &limit.to_string()])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

**无软件层隔离**，依赖：
- OpenClaw 以普通用户权限运行
- 系统进程管理

**理由**：
- 简化实现，减少权限问题
- 作为管理工具而非核心服务

---

## 4. 前端设计（D6）

### 4.1 技术栈

| 技术 | 用途 |
|------|------|
| Vue 3 | 框架 |
| TypeScript | 类型安全 |
| Vite | 构建工具 |
| Element Plus | UI 组件库 |
| Axios | HTTP 请求 |

### 4.2 页面结构

```
http://localhost:8080
├── /                     # 首页（重定向）
├── /config               # 配置向导（表单编辑 openclaw.json）
├── /status               # 状态监控（Runtime 状态 + 日志）
└── /safe-mode            # Safe Mode 恢复界面（错误时显示）
```

### 4.3 开发模式

```bash
# 开发时
bridge$ npm run dev        # Vite dev server localhost:5173
                           # 代理 API 到 Rust localhost:3000

# 构建
bridge$ npm run build      # 生成 dist/

# Rust 嵌入
cd hull && cargo build --release  # 将 dist/ 嵌入二进制
```

---

## 5. API 设计

### 5.1 接口列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /api/state | 获取当前状态（Normal/SafeMode）|
| GET | /api/config | 获取当前配置 |
| POST | /api/config | 应用新配置 |
| GET | /api/status | 获取 Runtime 状态 |
| POST | /api/restart | 重启 Runtime |
| GET | /api/snapshots | 获取快照列表（git log）|
| POST | /api/rollback | 回滚到指定版本 |
| POST | /api/reset | 恢复出厂设置 |
| GET | /api/logs | 获取日志 |
| GET | / | 静态文件（Vue 应用）|

### 5.2 模块配置 API

| 资源 | GET | POST | DELETE | 说明 |
|------|-----|------|--------|------|
| Provider | /api/providers | /api/providers | /api/providers/:id | 完整 CRUD |
| Agent | /api/agents | /api/agents | /api/agents/:id | 完整 CRUD |
| Channel | /api/channels | /api/channels | /api/channels/:type/:id | 完整 CRUD |
| Memory | /api/memory | /api/memory | - | 只读/保存，无删除 |

> **注意**: Memory 配置通过 POST /api/memory 整体更新，不支持单独删除字段。

### 5.2 状态响应

```json
// GET /api/state
{
  "state": "safe_mode",
  "reason": {
    "type": "config_error",
    "message": "Missing required field 'apiKey'",
    "current_version": "2026-03-03T16:30:00+08:00"
  }
}
```

> **注意**: 手工处理方案下，配置错误不会自动回滚，用户可在 Safe Mode 页面选择继续编辑或手动回滚。

---

## 6. 部署与安装

### 6.1 首次启动

1. 确保 OpenClaw 已安装并可运行
2. 安装 Claw One（详见安装文档）
3. 浏览器访问 `http://localhost:8080`
4. 首次配置向导：设置管理员密码、配置模型 API Key、配置渠道
5. 完成，开始使用

### 6.2 系统服务

```ini
# /etc/systemd/system/openclaw.service
[Unit]
Description=OpenClaw Gateway
After=network.target

[Service]
Type=simple
User=clawuser
ExecStart=/usr/local/bin/openclaw gateway start
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```ini
# /etc/systemd/system/claw-one.service
[Unit]
Description=Claw One Manager
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/claw-one
Restart=always

[Install]
WantedBy=multi-user.target
```

---

## 7. 决策汇总（D1-D8）

| 决策 | 选择 | 理由 |
|------|------|------|
| **D1** | 单 crate | 快速启动，MVP 后易拆分 |
| **D2** | 抽象 CRI | 预留 PicoClaw 扩展，MVP 只实现 OpenClaw |
| **D3** | systemd + git 快照 | 简化进程管理，Git 天然版本控制 |
| **D3-超时** | 30s 默认，可配 60s+ | 兼容慢速机器 |
| **D3-回滚** | 配置错误自动回滚，系统错误不回滚 | 减少误操作，给用户控制权 |
| **D4** | 无隔离 | 硬件隔离足够，简化实现 |
| **D5** | Safe Mode 状态标记 | 无独立进程，简化实现 |
| **D5-按钮** | 三场景不同按钮布局 | 精准匹配用户需求 |
| **D5-出厂设置** | 二级菜单，确认弹窗 | 防误触 |
| **D6** | Vue 3 + Vite（纯 Web）| 界面友好，浏览器访问 |
| **D7** | JSON 格式 | OpenClaw 原生兼容 |
| **D8** | 纯软件形态，toC/toB 代码一致 | 跨平台 Web 服务 |

---

## 8. MVP 范围与路线图

### MVP（6-8 周）

| 周 | 内容 |
|----|------|
| 1-2 | Rust 后端：API、Config Guardian、Git 快照、systemd 集成 |
| 2-3 | OpenClaw CRI 适配器 |
| 3-4 | Safe Mode 逻辑 |
| 4-5 | Vue 前端：配置向导、状态监控、Safe Mode 界面 |
| 5-6 | 集成测试、跨平台测试 |
| 6-8 | Bug 修复、文档、发布 |

### V2 规划

- PicoClaw 支持
- 快照自动清理策略
- toB 企业功能（SSO、审计）
- 完整沙箱验证

---

## 9. 相关文档

| 文档 | 内容 |
|------|------|
| [README.md](./README.md) | 项目概述、快速开始 |
| [roadmap.md](./roadmap.md) | 实施路线图与待办清单 |
| [VISUAL_CONFIG_DESIGN.md](./VISUAL_CONFIG_DESIGN.md) | 前端配置界面设计 |
| [BUILD_STRATEGY.md](./BUILD_STRATEGY.md) | 构建策略 |
| [TEST_FRAMEWORK.md](../TEST_FRAMEWORK.md) | 测试框架文档（项目根目录）|

---

*文档已更新至当前实现状态*

**日期**: 2026-03-03  
**来源**: 本地 CLI 调研 + Agent 搜索

---

## 1. Rust `include_str!` 宏

### 说明
- **类型**: Rust 标准库内置宏（非 OpenClaw 提供）
- **功能**: 编译时将文件内容作为字符串常量嵌入二进制
- **用法**:
```rust
const SCHEMA: &str = include_str!("../schemas/openclaw.schema.json");
```

### Schema 来源
- 需自行维护 `schemas/openclaw.schema.json` 文件
- 可参考 OpenClaw 源码中的类型定义
- 或直接复制 OpenClaw 的 schema（如果公开）

---

## 2. OpenClaw 日志位置

### 日志目录
```
~/.openclaw/logs/
├── commands.log          # 命令执行日志
├── gateway.log           # Gateway 运行日志
└── config-audit.jsonl    # 配置变更审计日志
```

### 获取方式

**方式 1: CLI 命令（RPC 方式）**
```bash
openclaw logs [--follow] [--limit 200] [--json]
```
- 需要 OpenClaw Gateway 正在运行
- 通过 RPC 获取日志

**方式 2: 直接读取文件**
```rust
pub async fn get_logs() -> Result<String, Error> {
    let log_path = dirs::home_dir()
        .unwrap()
        .join(".openclaw")
        .join("logs")
        .join("gateway.log");
    
    fs::read_to_string(&log_path).await
}
```
- 不需要 OpenClaw 运行
- 直接读取本地文件

### 推荐方案
```rust
pub async fn get_logs(tail: usize) -> Result<String, Error> {
    // 优先尝试直接读文件（更可靠）
    let log_path = get_logs_dir().join("gateway.log");
    
    match fs::read_to_string(&log_path).await {
        Ok(content) => Ok(content),
        Err(_) => {
            // 备用：通过 CLI 获取
            get_logs_via_cli(tail).await
        }
    }
}
```

---

## 3. OpenClaw 健康检查

### 检查方式对比

| 方式 | 命令/端点 | 速度 | 可靠性 | 适用场景 |
|------|----------|------|--------|---------|
| **HTTP 健康端点** | `GET http://127.0.0.1:18790/health` | 快 | 高 | 首选方式 |
| **health 命令** | `openclaw health --json` | 中 | 高 | 备用方式 |
| **gateway status** | `openclaw gateway status --json` | 中 | 高 | 详细状态 |
| **进程检查** | `pgrep -f "openclaw gateway"` | 快 | 中 | 快速判断 |

### 推荐实现

```rust
pub enum OpenClawState {
    Running,           // 正常运行
    Starting,          // 启动中（进程在，但未就绪）
    ConfigError {      // 配置错误导致启动失败
        error: String,
        auto_rolled_back: bool,
    },
    SystemError {      // 系统错误（端口占用等）
        error: String,
    },
    Stopped,           // 已停止
    Unknown,           // 未知状态
}

pub async fn check_state(&self) -> OpenClawState {
    // 1. 快速进程检查
    if !self.process_running().await {
        return OpenClawState::Stopped;
    }
    
    // 2. HTTP 健康检查（默认端口 18790）
    let port = self.get_gateway_port().await.unwrap_or(18790);
    match self.http_health_check(port).await {
        Ok(true) => return OpenClawState::Running,
        Ok(false) => {
            // 服务未就绪，可能在启动中
            if self.is_starting_recently().await {
                return OpenClawState::Starting;
            }
        }
        Err(_) => {}
    }
    
    // 3. CLI health 命令（备用）
    match self.cli_health_check().await {
        Ok(status) if status.healthy => return OpenClawState::Running,
        Ok(status) => {
            // 根据错误信息判断类型
            return self.classify_error(&status.error).await;
        }
        Err(_) => {}
    }
    
    // 4. 检查日志判断错误类型
    self.determine_error_from_logs().await
}

// HTTP 健康检查
async fn http_health_check(&self, port: u16) -> Result<bool, Error> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/health", port);
    
    match client.get(&url).timeout(Duration::from_secs(5)).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

// CLI health 检查
async fn cli_health_check(&self) -> Result<HealthStatus, Error> {
    let output = Command::new("openclaw")
        .args(["health", "--json"])
        .output()
        .await?;
    
    if !output.status.success() {
        return Err(Error::CommandFailed);
    }
    
    let status: HealthStatus = serde_json::from_slice(&output.stdout)?;
    Ok(status)
}
```

### 错误分类关键词

```rust
fn classify_error(&self, log_content: &str) -> OpenClawState {
    // 配置错误关键词
    let config_errors = [
        "Config validation failed",
        "Invalid API key",
        "Missing required field",
        "Cannot parse config",
        "SyntaxError",
    ];
    
    // 系统错误关键词
    let system_errors = [
        "EADDRINUSE",           // 端口占用
        "Permission denied",    // 权限不足
        "Out of memory",        // 内存不足
        "ENOSPC",               // 磁盘满
    ];
    
    for pattern in &config_errors {
        if log_content.contains(pattern) {
            return OpenClawState::ConfigError {
                error: log_content.to_string(),
                auto_rolled_back: false,
            };
        }
    }
    
    for pattern in &system_errors {
        if log_content.contains(pattern) {
            return OpenClawState::SystemError {
                error: log_content.to_string(),
            };
        }
    }
    
    OpenClawState::Unknown
}
```

---

## 4. 单实例控制

### 实现方案

```rust
use fs2::FileExt;

pub struct InstanceLock {
    file: File,
}

impl InstanceLock {
    pub fn acquire() -> Result<Self, Error> {
        let lock_path = std::env::temp_dir().join("claw-one.lock");
        
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)
            .map_err(|e| Error::LockFailed(e.to_string()))?;
        
        // 尝试获取独占锁
        match file.try_lock_exclusive() {
            Ok(()) => Ok(InstanceLock { file }),
            Err(_) => Err(Error::AlreadyRunning),
        }
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
```

### 使用方式

```rust
fn main() -> Result<(), Error> {
    // 确保单实例
    let _lock = InstanceLock::acquire()
        .map_err(|_| {
            eprintln!("Claw One is already running!");
            std::process::exit(1);
        })?;
    
    // 继续启动...
}
```

---

## 5. 配置验证 Schema

### 建议的 Schema 结构

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OpenClaw Config",
  "type": "object",
  "required": ["gateway", "models", "channels"],
  "properties": {
    "gateway": {
      "type": "object",
      "required": ["port"],
      "properties": {
        "port": {
          "type": "integer",
          "minimum": 1,
          "maximum": 65535
        },
        "bind": {
          "type": "string",
          "default": "127.0.0.1"
        }
      }
    },
    "models": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "provider", "apiKey"],
        "properties": {
          "id": { "type": "string" },
          "provider": { "type": "string" },
          "apiKey": { "type": "string", "minLength": 1 }
        }
      }
    },
    "channels": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "type"],
        "properties": {
          "id": { "type": "string" },
          "type": { "type": "string" }
        }
      }
    }
  }
}
```

### Rust 验证代码

```rust
use serde_json::Value;

pub struct ConfigValidator {
    schema: Value,
}

impl ConfigValidator {
    pub fn new() -> Result<Self, Error> {
        let schema = serde_json::from_str(CONFIG_SCHEMA)?;
        Ok(Self { schema })
    }
    
    pub fn validate(&self, config: &Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 1. 基本结构验证
        if !config.is_object() {
            errors.push("Config must be an object".to_string());
            return Err(errors);
        }
        
        // 2. 必填字段
        let required = ["gateway", "models", "channels"];
        for field in &required {
            if config.get(field).is_none() {
                errors.push(format!("Missing required field: {}", field));
            }
        }
        
        // 3. 类型验证
        if let Some(gateway) = config.get("gateway") {
            if let Some(port) = gateway.get("port") {
                if let Some(num) = port.as_u64() {
                    if num < 1 || num > 65535 {
                        errors.push(format!("Port {} out of range (1-65535)", num));
                    }
                } else {
                    errors.push("Port must be a number".to_string());
                }
            }
        }
        
        // 4. 交叉引用验证
        if let (Some(models), Some(channels)) = 
            (config.get("models"), config.get("channels")) {
            // 验证 channels 引用的 models 存在
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## 6. 其他确认项

### Git 提交作者
```rust
Command::new("git")
    .args([
        "commit",
        "-m", message,
        "--author", "Claw One <dev@claw.one>"
    ])
```

### 快照保留策略
- 保留近 3 个月（约 90 天）
- MVP 简化：保留最近 100 个提交
- V2 实现完整清理策略

### 配置文件路径
- 默认: `~/.openclaw/openclaw.json`
- 环境变量覆盖: `CLAW_ONE_CONFIG`

### 出厂配置来源
- OpenClaw 初始化后的配置
- 位置: `/var/lib/claw/factory-config.json`

---

*调研完成，待合并到 FINAL_DESIGN.md*

---

*RESEARCH_FINDINGS.md 已合并到本文档*
