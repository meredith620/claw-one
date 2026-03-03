# Claw One 最终设计文档

**版本**: MVP 1.0  
**日期**: 2026-03-03  
**状态**: 设计完成，待开发

---

## 1. 产品定位

### 1.1 产品层级

| 层级 | 产品 | 形态 | 目标用户 | 说明 |
|------|------|------|---------|------|
| **L1** | Claw One toC | N100 小主机（标准配置）| 个人、家庭 | 开箱即用 |
| **L2** | Claw One toB | N100+ 小主机（高配）| 小微企业 | +企业功能（SSO、审计等）|

### 1.2 核心目标

让 OpenClaw **开箱即用**，解决：
1. 部署复杂 → 硬件预装，插电即用
2. 配置易错 → 事务性配置 + 自动回滚
3. 门槛高 → Web 可视化配置，无需命令行

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────┐
│  用户浏览器                              │
│  http://box-ip:8080                     │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│  N100 Box（Ubuntu Server）               │
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
│  │  - /api/status                   │  │
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
│  │  systemd                         │  │
│  │  - openclaw.service              │  │
│  │  - 自动重启、健康监控            │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### 2.2 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **前端** | Vue 3 + TypeScript + Vite | 纯 Web，无 Tauri |
| **后端** | Rust + Axum + Tokio | 单 crate |
| **进程管理** | systemd | 管理 OpenClaw 生命周期 |
| **快照存储** | Git | 本地仓库，自动 commit |
| **配置格式** | JSON | 与 OpenClaw 原生兼容 |

### 2.3 项目结构（单 crate）

```
claw-one/
├── Cargo.toml              # Rust 依赖
├── src/
│   ├── main.rs             # Axum 服务启动
│   ├── api.rs              # HTTP 路由
│   ├── config.rs           # 配置管理 + Git 操作
│   ├── runtime.rs          # CRI trait + OpenClaw 实现
│   ├── state.rs            # AppState (Normal/SafeMode)
│   └── systemd.rs          # systemd 交互
├── static/                 # Vue 构建产物（嵌入二进制）
│   └── dist/
├── frontend/               # Vue 源码（开发）
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
└── docs/
```

---

## 3. 核心功能设计

### 3.1 配置管理（D3）

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
└── 否 → 检查日志
         ├── 配置错误 → git checkout 回滚 → 重启 → 进入 Safe Mode（显示已回滚）
         └── 系统错误 → 进入 Safe Mode（不回滚，用户决定）
```

#### Git 快照
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
1. **配置错误** → 自动回滚 → Safe Mode（提示已回滚）
2. **系统错误** → 不回滚 → Safe Mode（让用户决定）
3. **手动触发** → Safe Mode

#### 页面按钮

| 场景 | 主按钮 | 二级菜单（更多选项）|
|------|--------|-------------------|
| 配置错误（已回滚）| [重新编辑] [查看日志] | [强制使用新配置] [恢复出厂设置] |
| 系统错误 | [重新编辑] [回滚到具体版本] [查看日志] | [恢复出厂设置] |
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

### 3.4 安全隔离（D4）

**无软件层隔离**，依赖：
- Box 硬件物理隔离
- OpenClaw 以普通用户权限运行

**理由**：
- Box 是独立设备，不影响用户电脑
- 简化实现，减少权限问题

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
http://box-ip:8080
├── /                     # 首页（重定向）
├── /config               # 配置向导（表单编辑 openclaw.json）
├── /status               # 状态监控（Runtime 状态 + 日志）
└── /safe-mode            # Safe Mode 恢复界面（错误时显示）
```

### 4.3 开发模式

```bash
# 开发时
frontend$ npm run dev      # Vite dev server localhost:5173
                           # 代理 API 到 Rust localhost:3000

# 构建
frontend$ npm run build    # 生成 dist/

# Rust 嵌入
cargo build --release      # 将 dist/ 嵌入二进制
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

### 5.2 状态响应

```json
// GET /api/state
{
  "state": "safe_mode",
  "reason": {
    "type": "config_error",
    "message": "Missing required field 'apiKey'",
    "auto_rolled_back": true,
    "current_version": "2026-03-03T16:30:00+08:00"
  }
}
```

---

## 6. 部署与安装

### 6.1 Box 首次启动

1. 插电开机
2. 等待 30-60 秒启动
3. 手机/电脑连接 Box WiFi / 网线
4. 浏览器访问 `http://claw-one.local` 或 `http://192.168.x.x`
5. 首次配置向导：设置管理员密码、配置模型 API Key、配置渠道
6. 完成，开始使用

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
| **D6** | Vue 3 + Vite（纯 Web）| 界面友好，Box 浏览器访问 |
| **D7** | JSON 格式 | OpenClaw 原生兼容 |
| **D8** | 仅 Box 形态，toC/toB 代码一致 | Desktop 是 Box 软件形态，仅 Linux |

---

## 8. MVP 范围与路线图

### MVP（6-8 周）

| 周 | 内容 |
|----|------|
| 1-2 | Rust 后端：API、Config Guardian、Git 快照、systemd 集成 |
| 2-3 | OpenClaw CRI 适配器 |
| 3-4 | Safe Mode 逻辑 |
| 4-5 | Vue 前端：配置向导、状态监控、Safe Mode 界面 |
| 5-6 | 集成测试、N100 部署测试 |
| 6-8 | Bug 修复、文档、发布 |

### V2 规划

- PicoClaw 支持
- 快照自动清理策略
- toB 企业功能（SSO、审计）
- 完整沙箱验证

---

*文档完成，待开发*
