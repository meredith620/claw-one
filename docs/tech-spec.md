# 技术规范

## 接口定义

### 1. Claw Runtime Interface (CRI)

运行时适配器的统一接口定义。

```typescript
/**
 * Claw Runtime 接口
 * 所有 Runtime 适配器必须实现此接口
 */
interface ClawRuntime {
  /** 运行时名称，如 "openclaw", "picoclaw" */
  readonly name: string;
  
  /** 运行时版本 */
  readonly version: string;
  
  /** 运行时描述 */
  readonly description: string;
  
  /**
   * 启动 Runtime
   * @param configPath 配置文件路径
   * @throws RuntimeError 启动失败时抛出
   */
  start(configPath: string): Promise<void>;
  
  /**
   * 停止 Runtime
   * @param force 是否强制停止
   */
  stop(force?: boolean): Promise<void>;
  
  /**
   * 重启 Runtime
   */
  restart(): Promise<void>;
  
  /**
   * 获取运行状态
   */
  status(): Promise<RuntimeStatus>;
  
  /**
   * 健康检查
   * @returns 健康状态详情
   */
  health(): Promise<HealthStatus>;
  
  /**
   * 获取日志流
   * @param options 日志选项
   */
  logs(options?: LogOptions): AsyncIterable<LogEntry>;
  
  /**
   * 验证配置
   * @param config 配置对象或路径
   * @returns 验证结果
   */
  validate(config: Config | string): Promise<ValidationResult>;
  
  /**
   * 热重载配置（如支持）
   */
  reload(): Promise<void>;
  
  /**
   * 安装 Skill
   * @param skill Skill 规格
   */
  installSkill(skill: SkillSpec): Promise<void>;
  
  /**
   * 卸载 Skill
   * @param skillId Skill ID
   */
  uninstallSkill(skillId: string): Promise<void>;
  
  /**
   * 列出已安装 Skills
   */
  listSkills(): Promise<Skill[]>;
}

/**
 * 运行时状态
 */
interface RuntimeStatus {
  state: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  pid?: number;
  uptime?: number;
  startTime?: Date;
  error?: string;
}

/**
 * 健康状态
 */
interface HealthStatus {
  healthy: boolean;
  checks: HealthCheck[];
  timestamp: Date;
}

interface HealthCheck {
  name: string;
  status: 'pass' | 'fail' | 'warn';
  message?: string;
  duration: number;
}

/**
 * 日志选项
 */
interface LogOptions {
  /** 行数限制 */
  tail?: number;
  /** 是否跟随新日志 */
  follow?: boolean;
  /** 日志级别过滤 */
  level?: 'debug' | 'info' | 'warn' | 'error';
  /** 时间范围 */
  since?: Date;
  until?: Date;
}

interface LogEntry {
  timestamp: Date;
  level: string;
  message: string;
  source?: string;
}

/**
 * 验证结果
 */
interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

interface ValidationError {
  path: string;
  message: string;
  code: string;
}

interface ValidationWarning {
  path: string;
  message: string;
  suggestion?: string;
}

/**
 * Skill 规格
 */
interface SkillSpec {
  id: string;
  name: string;
  version: string;
  source: string; // npm 包名、git 仓库或本地路径
  config?: Record<string, unknown>;
}

interface Skill extends SkillSpec {
  installedAt: Date;
  status: 'active' | 'inactive' | 'error';
}
```

### 2. Config Guardian API

配置管理的核心 API。

```typescript
interface ConfigGuardian {
  /**
   * 获取当前配置
   */
  getCurrentConfig(): Promise<Config>;
  
  /**
   * 获取配置快照列表
   */
  listSnapshots(): Promise<ConfigSnapshot[]>;
  
  /**
   * 获取指定快照
   */
  getSnapshot(id: string): Promise<ConfigSnapshot>;
  
  /**
   * 应用新配置（事务性）
   */
  applyConfig(
    config: Config,
    options?: ApplyOptions
  ): Promise<ApplyResult>;
  
  /**
   * 回滚到指定快照
   */
  rollback(snapshotId: string): Promise<void>;
  
  /**
   * 验证配置（不应用）
   */
  validate(config: Config): Promise<ValidationResult>;
  
  /**
   * 导出配置
   */
  exportConfig(): Promise<string>; // JSON 字符串
  
  /**
   * 导入配置
   */
  importConfig(configJson: string): Promise<ValidationResult>;
  
  /**
   * 进入 Safe Mode
   */
  enterSafeMode(reason: SafeModeReason): Promise<void>;
  
  /**
   * 退出 Safe Mode
   */
  exitSafeMode(): Promise<void>;
  
  /**
   * 生成诊断包
   */
  generateDiagnosticPackage(): Promise<Buffer>;
}

interface ConfigSnapshot {
  id: string;
  timestamp: Date;
  config: Config;
  message?: string;
  author?: string;
  runtimeVersion: string;
}

interface ApplyOptions {
  /** 是否跳过验证 */
  skipValidation?: boolean;
  /** 健康检查超时（毫秒） */
  healthCheckTimeout?: number;
  /** 是否自动回滚 */
  autoRollback?: boolean;
  /** 提交信息 */
  message?: string;
}

interface ApplyResult {
  success: boolean;
  snapshotId?: string;
  error?: ConfigError;
  warnings?: ValidationWarning[];
}

interface SafeModeReason {
  type: 'startup_failed' | 'health_check_failed' | 'manual';
  message: string;
  error?: Error;
}
```

### 3. Remote Support API

远程支持系统 API。

```typescript
interface RemoteSupport {
  /**
   * 生成诊断包
   */
  generateDiagnosticPackage(
    options?: DiagnosticOptions
  ): Promise<DiagnosticPackage>;
  
  /**
   * 请求远程会话（用户发起）
   */
  requestSession(): Promise<SessionRequest>;
  
  /**
   * 批准远程会话（用户确认）
   */
  approveSession(requestId: string): Promise<SessionCode>;
  
  /**
   * 拒绝远程会话
   */
  rejectSession(requestId: string): Promise<void>;
  
  /**
   * 终止当前会话
   */
  terminateSession(): Promise<void>;
  
  /**
   * 获取当前会话状态
   */
  getSessionStatus(): Promise<SessionStatus | null>;
  
  /**
   * 获取审计日志
   */
  getAuditLogs(): Promise<AuditLogEntry[]>;
}

interface DiagnosticPackage {
  id: string;
  createdAt: Date;
  encrypted: boolean;
  size: number;
  contents: DiagnosticContent[];
  export(): Promise<Buffer>;
}

interface DiagnosticContent {
  type: 'config' | 'logs' | 'system_info' | 'runtime_status';
  sensitive: boolean;
  preview: string;
}

interface DiagnosticOptions {
  /** 是否包含敏感信息 */
  includeSensitive?: boolean;
  /** 日志时间范围 */
  logRange?: { since: Date; until: Date };
  /** 是否包含配置文件 */
  includeConfig?: boolean;
}

interface SessionRequest {
  id: string;
  createdAt: Date;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
  expiresAt: Date;
}

interface SessionCode {
  code: string;
  expiresAt: Date;
  maxDuration: number; // 秒
}

interface SessionStatus {
  id: string;
  startedAt: Date;
  expiresAt: Date;
  connectionType: 'tailscale' | 'wireguard' | 'webrtc';
  canTerminate: boolean;
}

interface AuditLogEntry {
  timestamp: Date;
  action: string;
  user?: string;
  details?: Record<string, unknown>;
}
```

## 数据模型

### 配置模型

```typescript
/**
 * OpenClaw 配置结构（简化）
 */
interface Config {
  /** 配置版本 */
  version: string;
  
  /** Gateway 配置 */
  gateway: GatewayConfig;
  
  /** 模型配置 */
  models: ModelConfig[];
  
  /** 渠道配置 */
  channels: ChannelConfig[];
  
  /** Skills 配置 */
  skills?: SkillConfig[];
  
  /** 插件配置 */
  plugins?: PluginConfig[];
}

interface GatewayConfig {
  port: number;
  bind: string;
  auth?: AuthConfig;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
}

interface AuthConfig {
  type: 'token' | 'jwt' | 'oauth';
  secret?: string;
  providers?: OAuthProvider[];
}

interface ModelConfig {
  id: string;
  provider: 'openai' | 'anthropic' | 'azure' | 'custom';
  apiKey?: string;
  baseUrl?: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
}

interface ChannelConfig {
  id: string;
  type: 'telegram' | 'wechat' | 'feishu' | 'dingtalk' | 'webhook';
  enabled: boolean;
  config: Record<string, unknown>; // 各渠道特有配置
}

interface SkillConfig {
  id: string;
  enabled: boolean;
  config?: Record<string, unknown>;
}

interface PluginConfig {
  id: string;
  enabled: boolean;
  config?: Record<string, unknown>;
}
```

### JSON Schema 示例

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OpenClaw Config",
  "type": "object",
  "required": ["version", "gateway", "models", "channels"],
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    },
    "gateway": {
      "type": "object",
      "required": ["port", "bind"],
      "properties": {
        "port": {
          "type": "integer",
          "minimum": 1,
          "maximum": 65535,
          "default": 3000
        },
        "bind": {
          "type": "string",
          "default": "127.0.0.1"
        },
        "logLevel": {
          "type": "string",
          "enum": ["debug", "info", "warn", "error"],
          "default": "info"
        }
      }
    },
    "models": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "provider", "model"],
        "properties": {
          "id": { "type": "string" },
          "provider": {
            "type": "string",
            "enum": ["openai", "anthropic", "azure", "custom"]
          },
          "apiKey": {
            "type": "string",
            "minLength": 1
          },
          "model": { "type": "string" },
          "temperature": {
            "type": "number",
            "minimum": 0,
            "maximum": 2
          },
          "maxTokens": {
            "type": "integer",
            "minimum": 1
          }
        }
      }
    }
  }
}
```

## 安全策略

### 1. 配置安全

| 策略 | 实现 |
|------|------|
| API Key 加密存储 | 使用系统 keychain 或文件加密 |
| 配置快照加密 | 敏感字段脱敏后存储 |
| 传输加密 | HTTPS/TLS |
| 访问控制 | 本地访问默认绑定 127.0.0.1 |

### 2. 运行时安全

| 策略 | 实现 |
|------|------|
| 用户沙盒 | 专用用户 + chroot |
| 资源限制 | cgroup v2 |
| 能力限制 | Linux capabilities |
| 网络隔离 | 可选 namespace |

### 3. 技能安全

```typescript
interface SkillSecurityPolicy {
  // 安装前静态分析
  staticAnalysis(skill: SkillSpec): SecurityReport;
  
  // 动态沙箱测试
  sandboxTest(skill: SkillSpec): Promise<SandboxResult>;
  
  // 权限分级
  permissionLevels: {
    'low': ['fs.read'],
    'medium': ['fs.read', 'fs.write', 'net.request'],
    'high': ['fs.*', 'net.*', 'process.exec']
  };
  
  // 用户确认
  promptUser(report: SecurityReport): Promise<boolean>;
}
```

## 错误处理

### 错误码规范

| 错误码 | 说明 | 用户提示 |
|--------|------|---------|
| `CONFIG_SYNTAX_ERROR` | JSON 语法错误 | "配置格式有误，请检查括号、引号" |
| `CONFIG_SEMANTIC_ERROR` | 语义错误 | "API Key 格式不正确" |
| `CONFIG_VALIDATION_FAILED` | 验证失败 | "端口已被占用，请更换端口" |
| `RUNTIME_START_FAILED` | 启动失败 | "服务启动失败，已进入 Safe Mode" |
| `HEALTH_CHECK_FAILED` | 健康检查失败 | "服务未正常运行，已自动回滚" |
| `ROLLBACK_FAILED` | 回滚失败 | "自动恢复失败，请导出诊断包联系支持" |

### Safe Mode 触发条件

1. Gateway 启动失败（exit code != 0）
2. 健康检查连续 3 次失败
3. 配置应用后 30 秒内未就绪
4. 用户手动触发

---

*最后更新：2026-03-02*
