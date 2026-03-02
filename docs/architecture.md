# 架构设计

## 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│  Layer 3: 产品形态层 (Product Forms)                             │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐  │
│  │  toC: Desktop (免费)     │  │  toC: Box / toB: Enterprise │  │
│  │  toC: Box (N100 小主机)  │  │  + 配套 toB 软件            │  │
│  └─────────────────────────┘  └─────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│  Layer 2: Claw Manager（共用内核）                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Config      │  │ Runtime     │  │ Security                │ │
│  │ Guardian    │  │ Adapter     │  │ Sandbox                 │ │
│  │ (事务配置)   │  │ (多分支)    │  │ (系统保护)              │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│  功能：快照回滚、Safe Mode、远程诊断（授权制）、权限隔离          │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│  Layer 1: Runtime Adapters（可插拔）                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────┐ │
│  │ OpenClaw    │  │ PicoClaw    │  │ Nanobot     │  │ Future │ │
│  │ (Node.js)   │  │ (Go)        │  │ (Rust)      │  │ ...    │ │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. Config Guardian（配置守护进程）

**职责**：解耦配置管理与 Gateway 运行，确保"配置错误不锁死"。

#### 状态机

```
┌─────────────┐     启动失败      ┌─────────────┐
│   Normal    │ ───────────────→ │  Safe Mode  │
│   (正常模式) │                  │  (安全模式)  │
└──────┬──────┘                  └──────┬──────┘
       │                                │
       │ 配置变更                        │ 回滚操作
       ▼                                ▼
┌─────────────┐                  ┌─────────────┐
│ Validating  │                  │  Rolling    │
│  (验证中)   │                  │  Back       │
└──────┬──────┘                  └─────────────┘
       │
       │ 验证通过
       ▼
┌─────────────┐
│  Applying   │
│  (应用中)   │
└──────┬──────┘
       │
       │ 健康检查通过
       ▼
┌─────────────┐
│   Normal    │
└─────────────┘
       │
       │ 健康检查失败
       ▼
┌─────────────┐
│ Safe Mode   │
│ (自动进入)   │
└─────────────┘
```

#### 事务性配置流程

```typescript
interface ConfigTransaction {
  id: string;
  timestamp: number;
  previousConfig: ConfigSnapshot;
  newConfig: Config;
  
  async execute(): Promise<void> {
    // 1. 创建快照
    await this.createSnapshot();
    
    // 2. 语法验证 (JSON Schema)
    const syntaxResult = await this.validateSyntax();
    if (!syntaxResult.valid) {
      throw new ConfigError('SYNTAX_ERROR', syntaxResult.errors);
    }
    
    // 3. 语义验证 (运行时预检)
    const semanticResult = await this.runtimeAdapter.validate(this.newConfig);
    if (!semanticResult.valid) {
      throw new ConfigError('SEMANTIC_ERROR', semanticResult.errors);
    }
    
    // 4. 应用配置
    await this.runtimeAdapter.applyConfig(this.newConfig);
    
    // 5. 健康检查 (超时 30s)
    const healthy = await this.waitForHealthy(30000);
    if (!healthy) {
      // 自动回滚
      await this.rollback();
      throw new ConfigError('HEALTH_CHECK_FAILED');
    }
    
    // 6. 提交（保留快照）
    await this.commit();
  }
}
```

#### Safe Mode（安全模式）

当 Gateway 启动失败时自动触发：

```
Safe Mode Web UI (http://localhost:18790)
├── 错误诊断
│   ├── 人话解释（非技术用户能看懂）
│   ├── 原始错误日志（技术用户查看）
│   └── 一键修复建议
├── 配置回滚
│   ├── 最近 10 个可用版本
│   ├── 一键回滚到上一版本
│   └── 重置为出厂配置
├── 诊断导出
│   └── 生成加密诊断包（用于远程支持）
└── 退出 Safe Mode
    └── 修复成功后手动退出
```

### 2. Runtime Adapter（运行时适配器）

**Claw Runtime Interface (CRI)**

```typescript
interface ClawRuntime {
  readonly name: string;        // "openclaw" | "picoclaw" | "nanobot"
  readonly version: string;
  
  // 生命周期
  start(configPath: string): Promise<void>;
  stop(): Promise<void>;
  restart(): Promise<void>;
  
  // 状态查询
  health(): Promise<HealthStatus>;
  logs(options: LogOptions): AsyncIterable<LogEntry>;
  
  // 配置管理
  validate(config: Config): Promise<ValidationResult>;
  reload(): Promise<void>;
  
  // 扩展能力
  installSkill(skill: SkillSpec): Promise<void>;
  uninstallSkill(skillId: string): Promise<void>;
  listSkills(): Promise<Skill[]>;
}

// 适配器注册表
class RuntimeRegistry {
  private adapters: Map<string, ClawRuntime> = new Map();
  
  register(name: string, adapter: ClawRuntime): void;
  get(name: string): ClawRuntime | undefined;
  listAvailable(): string[];
  
  // 自动检测系统已安装的 runtime
  async autoDetect(): Promise<string[]>;
}
```

**适配器实现**

| Runtime | 实现方式 | 特点 |
|---------|---------|------|
| **OpenClaw** | 调用官方 CLI (`openclaw gateway`) | 功能最全，生态最成熟 |
| **PicoClaw** | 直接执行二进制 | 轻量，适合低配硬件 |
| **Nanobot** | 调用 Rust 二进制 | 安全加固，适合 toB |

**运行时切换**

用户在 UI 中可一键切换：
1. 导出当前配置
2. 停止当前 runtime
3. 启动新 runtime
4. 导入并验证配置
5. 如失败自动回滚

### 3. Security Sandbox（安全沙盒）

针对"原生部署保护系统"的核心设计。

#### 四层隔离策略

```
┌─────────────────────────────────────────────────────────────┐
│ Level 3: 硬件隔离（Claw Box）                                │
│ 独立 N100 小主机，物理隔离，完全不影响用户电脑                 │
└─────────────────────────────────────────────────────────────┘
                              ↓ 软件版使用下方层级
┌─────────────────────────────────────────────────────────────┐
│ Level 2: 系统容器隔离（可选）                                 │
│ Systemd-nspawn / LXC，轻量级系统容器                          │
│ • 共享宿主内核，开销极低                                       │
│ • 独立文件系统视图                                             │
│ • 网络隔离可选                                                 │
└─────────────────────────────────────────────────────────────┘
                              ↓ 默认使用下方层级
┌─────────────────────────────────────────────────────────────┐
│ Level 1: 用户沙盒（默认）                                     │
│ • 专用用户 clawuser                                           │
│ • 受限文件访问（chroot / landlock）                           │
│ • 进程命名空间隔离                                             │
│ • cgroup 资源限制                                              │
└─────────────────────────────────────────────────────────────┘
                              ↓ 高级用户可选
┌─────────────────────────────────────────────────────────────┐
│ Level 0: 裸机运行                                             │
│ 直接在用户环境运行，无隔离                                     │
│ ⚠️ 需用户明确确认风险                                          │
└─────────────────────────────────────────────────────────────┘
```

#### 用户沙盒（Level 1）实现细节

```bash
# 创建隔离用户
useradd -r -s /bin/false -d /var/lib/claw -M clawuser

# 目录权限
/var/lib/claw/
├── .openclaw/          # OpenClaw 配置和数据（owner: clawuser）
├── workspace/          # 工作目录（用户可访问）
├── tools/              # 已安装工具（白名单机制）
└── logs/               # 日志文件

# 受限能力（Linux capabilities）
# 移除：CAP_SYS_ADMIN, CAP_NET_ADMIN, CAP_SYS_PTRACE 等
# 保留：CAP_NET_BIND_SERVICE（绑定低端口，如需）

# 资源限制（cgroup v2）
memory.max = 4G          # 内存上限
cpu.max = 200000 1000000 # CPU 限制 20%
io.max = ...             # IO 限制
```

#### 技能安装安全

```typescript
interface SkillSecurityPolicy {
  // 安装前扫描
  async preInstallCheck(skill: SkillSpec): Promise<SecurityReport> {
    return {
      permissions: ['fs.read', 'fs.write', 'net.request'], // 所需权限
      riskLevel: 'medium',  // low | medium | high
      warnings: ['需要访问 ~/.ssh 目录'],
      sandboxRequired: true
    };
  }
  
  // 用户确认流程
  async promptUser(report: SecurityReport): Promise<boolean>;
  
  // 运行时隔离
  async installInSandbox(skill: SkillSpec): Promise<void> {
    // 高风险 skill 在临时容器内测试运行
    // 通过后才正式安装
  }
}
```

### 4. 远程支持系统（用户授权制）

**核心原则**：用户完全控制，无后门。

```
用户端                              支持端
  │                                   │
  │  1. 遇到问题                      │
  │  2. 点击"请求远程协助"            │
  ▼                                   │
┌─────────────┐                       │
│ 生成诊断包   │                       │
│ • 日志脱敏   │                       │
│ • 配置摘要   │                       │
│ • 系统信息   │                       │
└──────┬──────┘                       │
       │                              │
       │  3. 用户审核诊断包内容        │
       │  4. 确认发送                  │
       ▼                              ▼
┌─────────────┐                 ┌─────────────┐
│ 加密上传    │ ──────────────→ │ 支持平台    │
│ (可选邮件)  │                 │ 技术支持    │
└─────────────┘                 └─────────────┘
                                        │
  ◄─────────────────────────────────────┘
  5. 技术支持分析后提供解决方案
  6. 用户自行操作修复（或授权远程）

【可选实时远程】
  7. 用户点击"允许远程连接"
  8. 生成一次性连接码（5分钟有效）
  9. 技术支持输入连接码建立加密隧道
  10. 用户可随时中断连接
```

**技术实现**

```typescript
interface RemoteSupport {
  // 诊断包生成
  async generateDiagnosticPackage(
    options: PackageOptions
  ): Promise<EncryptedPackage>;
  
  // 实时远程（需用户主动授权）
  async requestRemoteSession(): Promise<SessionCode>;
  async approveRemoteSession(code: string): Promise<SecureTunnel>;
  
  // 连接类型
  connectType: 'tailscale' | 'wireguard' | 'webrtc';
  
  // 安全保证
  sessionTimeout: 30 * 60 * 1000; // 30分钟超时
  canBeTerminatedByUser: true;
  auditLog: true; // 记录所有操作
}
```

## 数据流图

### 配置变更流程

```
┌────────┐    ┌─────────────┐    ┌────────────────┐    ┌──────────┐
│  User  │───→│  Web UI     │───→│ Config Guardian│───→│ Runtime  │
└────────┘    └─────────────┘    └────────────────┘    └──────────┘
                                        │
                                        ↓
                                ┌───────────────┐
                                │  Snapshot DB  │
                                │  (Git-like)   │
                                └───────────────┘
                                        │
                    ┌───────────────────┴───────────────────┐
                    ↓                                           ↓
            ┌─────────────┐                             ┌─────────────┐
            │  Success    │                             │  Failure    │
            │  (Normal)   │                             │ (Safe Mode) │
            └─────────────┘                             └─────────────┘
```

### 运行时启动流程

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Start Cmd  │───→│  Pre-check  │───→│  Launch     │───→│  Health     │
│             │    │  (validate) │    │  Process    │    │  Check      │
└─────────────┘    └─────────────┘    └─────────────┘    └──────┬──────┘
                                                                  │
                                            ┌────────────────────┘
                                            ↓
                                    ┌─────────────┐
                                    │  Ready      │
                                    └─────────────┘
```

---

*最后更新：2026-03-02*
