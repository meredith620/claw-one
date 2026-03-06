# Claw One 可视化配置设计文档

**版本**: v2.4  
**日期**: 2026-03-06  
**状态**: ✅ 设计确认完成，进入实现阶段

---

## 1. 设计目标与核心要求

将 Claw One 的前端配置界面从 JSON 编辑器改造为**模块化的可视化表单**，实现对 OpenClaw `openclaw.json` 配置文件的直观管理。

### 核心要求

| 编号 | 要求 | 说明 |
|------|------|------|
| R1 | 直接读写 openclaw.json | 不引入额外存储，所有配置变更直接反映到 OpenClaw 配置 |
| R2 | 字段级定向修改 | 表单只修改对应配置路径，保留其他现有字段不变 |
| R3 | 纯可视化界面 | 禁止用户在页面内手工编辑 JSON |
| R4 | 渐进式 Provider 支持 | 先实现 Moonshot 验证可行性，再扩展到 OpenAI/Anthropic，最后 Custom Provider |
| R5 | 手动重启机制 | 配置保存后提示需要重启，用户手动触发重启 |
| R6 | 模块化设计 | 按功能模块划分：Provider、Agent、Memory、Channel |
| R7 | 轻量动态表单 | 表单渲染基于 Schema 动态生成，避免硬编码绑定前后端 |

---

## 2. 已确认设计决策

| 问题 | 决策 |
|------|------|
| **模块范围** | 只保留 Provider、Agent、Memory、Channel **4个核心模块** |
| **布局方案** | **两栏布局**（左侧导航 + 右侧内容） |
| **导入/导出** | ❌ **不需要** |
| **Agent 工作区创建** | 自动提供建议值，用户不修改就用建议值 |
| **模型列表** | 固定列表 + 支持自定义输入 |
| **Memory Provider** | 先 Ollama，后期可扩展 |
| **飞书多账号** | ✅ 支持多账号，参考 Mattermost 设计 |
| **API Key 验证** | 保存时验证有效性 |
| **版本管理** | 以 openclaw.json 整体做 Git 版本管理 |
| **默认模型设置** | Provider 设置时可设为默认模型 |
| **内置 Provider** | Moonshot、OpenAI、Anthropic、MiniMax、Custom |
| **Provider 多实例** | ✅ 支持，每个类型可创建多个不同 API Key 的实例 |
| **实例 ID 规则** | `{type}-{name}`，如 `moonshot-work` |
| **表单渲染** | 轻量动态生成，基于 Schema 而非硬编码 |

---

## 3. 内置 Provider 列表

| 内置 Provider | 阶段 | 说明 |
|--------------|------|------|
| **Moonshot** | Phase 1 | 含3个子选项：Kimi Coding(默认)/.ai/.cn |
| **OpenAI** | Phase 1 | API Key 认证 |
| **Anthropic** | Phase 1 | API Key 认证 |
| **MiniMax** | Phase 2 | OAuth/M2.5/M2.5(CN)/M2.5 Highspeed |
| **Custom** | Phase 2 | 用户自定义，支持多种 API 格式 |

---

## 4. 系统架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                          前端 (Vue3)                                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐ │
│  │ Provider配置 │ │  Agent配置   │ │  Memory配置  │ │ Channel配置│ │
│  │ (动态表单)   │ │  (动态表单)  │ │  (动态表单)  │ │ (动态表单) │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ API
┌─────────────────────────────────────────────────────────────────────┐
│                          后端 (Rust)                                 │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  ConfigManager                                                 │ │
│  │  ├── read_module(module: &str) -> ModuleConfig                │ │
│  │  ├── write_module(module: &str, data: Value) -> Result       │ │
│  │  ├── validate_module(module: &str, data: Value) -> Result    │ │
│  │  └── restart_openclaw() -> Result                            │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│                              ▼                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  SchemaRegistry (轻量动态表单)                                  │ │
│  │  ├── provider_schemas() -> Vec<Schema>                         │ │
│  │  ├── render_form(schema_id) -> FormConfig                     │ │
│  │  └── validate_data(schema_id, data) -> Result                 │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                      │
│                              ▼                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  GitManager (版本控制)                                         │ │
│  │  ├── commit_changes(message: &str) -> CommitHash              │ │
│  │  └── rollback_to(commit: &str) -> Result                     │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        openclaw.json                                │
│                   (OpenClaw 实际运行配置)                            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 5. 轻量动态表单设计

### 5.1 核心原则

**避免硬编码**，采用 Schema 驱动的前端表单渲染：

```yaml
# 后端提供 Schema 定义
schema:
  fields:
    - id: "apiKey"
      type: "password"
      label: "API Key"
      required: true
      
    - id: "defaultModel"
      type: "select"
      label: "默认模型"
      options:
        - value: "kimi-k2.5"
          label: "Kimi K2.5"
        - value: "custom"
          label: "自定义 (手动输入)"
      allow_custom: true
```

**前端根据 Schema 动态渲染**：
- `type: password` → 密码输入框
- `type: select` → 下拉选择框
- `type: string` → 文本输入框
- `type: boolean` → 开关组件

### 5.2 优势

| 场景 | 硬编码方案 | 轻量动态方案 |
|------|-----------|-------------|
| 新增 Provider | 改前端代码 + 发布 | 后端添加 Schema 即可 |
| 修改字段 | 改前后端代码 | 改 Schema 定义 |
| 前端升级 | 需要重新构建 | 无需改动 |
| 部署灵活性 | 绑定版本 | 后端热更新 Schema |

### 5.3 前端组件映射表

```typescript
const componentMap: Record<string, Component> = {
  'string': InputText,
  'password': InputPassword,
  'number': InputNumber,
  'boolean': Switch,
  'select': Select,
  'textarea': TextArea,
  'array': ArrayInput,
}
```

---

## 6. 模块详细设计

### 6.1 OpenClaw 配置结构分析

```json
{
  "models": {
    "mode": "merge",
    "providers": {
      "moonshot": {
        "baseUrl": "https://api.moonshot.ai/v1",
        "apiKey": "${MOONSHOT_API_KEY}",
        "api": "openai-completions",
        "models": [...]
      }
    }
  },
  "agents": {
    "defaults": {
      "model": { "primary": "moonshot/kimi-k2.5", "fallbacks": [...] },
      "memorySearch": { ... }
    },
    "list": [
      { "id": "main", "name": "Main", "workspace": "...", "agentDir": "..." }
    ]
  },
  "bindings": [
    { "agentId": "main", "match": { "channel": "mattermost", "accountId": "default" }}
  ],
  "channels": {
    "mattermost": {
      "enabled": true,
      "dmPolicy": "pairing",
      "accounts": {
        "default": { "name": "...", "botToken": "...", "baseUrl": "..." }
      }
    }
  },
  "plugins": {
    "entries": { "memory-core": { "enabled": true } },
    "slots": { "memory": "memory-core" }
  }
}
```

### 6.2 Provider / Model 配置

**核心设计：Provider 类型 + 多实例**

每个 Provider 类型（Moonshot/OpenAI/Anthropic/MiniMax/Custom）可以创建多个实例，每个实例有独立的 API Key 和配置。

**实例 ID 生成规则**: `{type}-{name}`，如 `moonshot-work`、`openai-personal`

**UI 设计：两栏布局**

```
┌─────────────────────────────────────────────────────────────────┐
│  Claw One 配置                                 [保存] [重启]    │
├──────────────────┬──────────────────────────────────────────────┤
│  🧠 Provider     │  AI Provider 配置                             │
│  🤖 Agent        │  ───────────────────────────────────────────  │
│  🧠 Memory       │                                               │
│  📱 Channel      │  ── Moonshot ───────────────────────────────  │
│                  │  ┌─ moonshot-work ─────────────────────────┐  │
│                  │  │  版本: .ai  模型: kimi-k2.5              │  │
│                  │  │  状态: ✅ 已启用            [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  ┌─ moonshot-china ────────────────────────┐  │
│                  │  │  版本: .cn  模型: kimi-k2.5              │  │
│                  │  │  状态: ⚪ 未启用            [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加 Moonshot 实例]                         │
│                  │                                               │
│                  │  ── OpenAI ─────────────────────────────────  │
│                  │  ┌─ openai-personal ───────────────────────┐  │
│                  │  │  模型: gpt-4o               [配置] [删除]│  │
│                  │  │  状态: ✅ 已启用                          │  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加 OpenAI 实例]                           │
│                  │                                               │
│                  │  ── Anthropic ──────────────────────────────  │
│                  │  [+ 添加 Anthropic 实例]                        │
│                  │                                               │
│                  │  ── MiniMax ────────────────────────────────  │
│                  │  [+ 添加 MiniMax 实例]                          │
│                  │                                               │
│                  │  ── Custom ─────────────────────────────────  │
│                  │  ┌─ deepseek-team ─────────────────────────┐  │
│                  │  │  Type: deepseek  模型: deepseek-chat     │  │
│                  │  │  状态: ✅ 已启用            [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加 Custom Provider]                       │
│                  │                                               │
│                  │  ═══════════════════════════════════════════  │
│                  │  模型优先级设置（从已启用实例中选择）            │
│                  │  ═══════════════════════════════════════════  │
│                  │                                               │
│                  │  Primary:    [moonshot-work/kimi-k2.5 ▼]      │
│                  │  Fallback 1: [openai-personal/gpt-4o ▼]       │
│                  │  [+ 添加 Fallback]                              │
│                  │                                               │
└──────────────────┴──────────────────────────────────────────────┘
```

**添加 Moonshot 实例：**

```
┌─────────────────────────────────────────────────────────────┐
│  添加 Moonshot 实例                                [保存]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  实例名称: [work                ]  → 生成: moonshot-work    │
│                                                              │
│  版本选择:                                                   │
│  ● Kimi Coding (默认)                                        │
│    Base: https://api.kimi.com/coding/  API: anthropic-msg   │
│  ○ Kimi API (.ai)                                            │
│    Base: https://api.moonshot.ai/v1    API: openai-comp     │
│  ○ Kimi API (.cn)                                            │
│    Base: https://api.moonshot.cn/v1    API: openai-comp     │
│                                                              │
│  API Key: [sk-***                           ]                │
│  默认模型: [k2p5 ▼]  (根据版本动态变化)                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**添加 Custom Provider：**

```
┌─────────────────────────────────────────────────────────────┐
│  添加 Custom Provider                              [保存]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  类型: [deepseek         ]  (可输入已有类型或新类型)          │
│  实例名称: [team           ]                                  │
│                                                              │
│  生成 Provider ID: deepseek-team                             │
│                                                              │
│  API Key: [sk-***                           ]                │
│  Base URL: [https://api.deepseek.com/v1     ]                │
│  API 格式: [openai-completions ▼]                            │
│  默认模型: [deepseek-chat ▼]  (支持自定义输入)                │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**关键规则：**

| 规则 | 说明 |
|------|------|
| 实例 ID 唯一 | 全局唯一，不能与内置 provider ID 冲突 |
| 实例可删除 | 删除后从 openclaw.json 移除该 provider |
| 内置类型限制 | Moonshot/OpenAI/Anthropic/MiniMax/Custom 为固定类型 |
| Custom 类型 | 可重复创建实例，同 type 不同 name |
| 模型优先级 | 从所有已启用实例的模型中选择，格式：`{instance-id}/{model-id}` |

> **注意**: 内置 Provider 使用固定 `api` 字段，**只有 Custom Provider 需要用户选择 API 格式**。

```yaml
# Moonshot
name: Moonshot
type: provider
api: openai-completions  # 固定，不需要用户选择
fields:
  - id: enabled
    type: boolean
    label: 启用 Provider
    default: true
  - id: apiKey
    type: password
    label: API Key
    required: true
  - id: baseUrl
    type: string
    label: Base URL
    default: https://api.moonshot.ai/v1
  - id: defaultModel
    type: select
    label: 默认模型
    options:
      - value: kimi-k2.5
        label: Kimi K2.5
      - value: kimi-k2-thinking
        label: Kimi K2 Thinking
    allow_custom: true

# OpenAI

> **API 格式说明**: `openai-completions` = 标准 `/chat/completions` 端点（兼容性好）；`openai-responses` = 新版 `/responses` 端点（OpenAI 新功能）。一般第三方 Provider 使用 `openai-completions` 即可。

name: OpenAI
type: provider
api: openai-completions  # 默认，可切换为 openai-responses
fields:
  - id: enabled
    type: boolean
    label: 启用 Provider
    default: true
  - id: apiKey
    type: password
    label: API Key
    required: true
  - id: baseUrl
    type: string
    label: Base URL
    default: https://api.openai.com/v1
  - id: apiVariant
    type: select
    label: API 类型
    default: openai-completions
    options:
      - value: openai-completions
        label: Completions API
      - value: openai-responses
        label: Responses API
  - id: defaultModel
    type: select
    label: 默认模型
    options:
      - value: gpt-4o
        label: GPT-4o
      - value: gpt-4o-mini
        label: GPT-4o Mini
      - value: gpt-3.5-turbo
        label: GPT-3.5 Turbo
    allow_custom: true

# Anthropic
name: Anthropic
type: provider
api: anthropic-messages  # 固定，不需要用户选择
fields:
  - id: enabled
    type: boolean
    label: 启用 Provider
    default: true
  - id: apiKey
    type: password
    label: API Key
    required: true
  - id: baseUrl
    type: string
    label: Base URL
    default: https://api.anthropic.com/v1
  - id: defaultModel
    type: select
    label: 默认模型
    options:
      - value: claude-3-opus
        label: Claude 3 Opus
      - value: claude-3-sonnet
        label: Claude 3 Sonnet
      - value: claude-3-haiku
        label: Claude 3 Haiku
    allow_custom: true

# Custom (用户自定义 Provider)
name: Custom
type: provider
fields:
  - id: enabled
    type: boolean
    label: 启用 Provider
    default: true
  - id: providerId
    type: string
    label: Provider ID
    required: true
    placeholder: 如 deepseek、qwen、volcengine 等
  - id: name
    type: string
    label: 显示名称
    required: true
  - id: apiKey
    type: password
    label: API Key
    required: true
  - id: baseUrl
    type: string
    label: Base URL
    required: true
    placeholder: https://api.example.com/v1
  - id: apiFormat  # <-- 只有 Custom 需要选择 API 格式
    type: select
    label: API 格式
    required: true
    options:
      - value: openai-completions
        label: OpenAI Completions (兼容 OpenAI 格式)
      - value: openai-responses
        label: OpenAI Responses (OpenAI Responses API)
      - value: anthropic-messages
        label: Anthropic Messages (兼容 Claude 格式)
  - id: defaultModel
    type: string
    label: 默认模型
    required: true
    placeholder: 输入模型 ID，如 deepseek-chat
```

### 6.3 Agent 配置

**设计：支持单 Agent / Multi-Agent 模式切换**

| 模式 | 说明 | 配置内容 |
|------|------|---------|
| **单 Agent** (默认) | 使用 `agents.defaults` 配置 | 仅配置默认 Agent 参数 |
| **Multi-Agent** | 启用 `agents.list` 多 Agent | 配置多个自定义 Agent + 默认 Agent |

**UI 设计：**

```
┌─────────────────────────────────────────────────────────────────┐
│  🤖 Agent 配置                                   [保存] [重启]  │
├──────────────────┬──────────────────────────────────────────────┤
│  🧠 Provider     │                                               │
│  🤖 Agent        │  Agent 模式                                    │
│  🧠 Memory       │  ───────────────────────────────────────────  │
│  📱 Channel      │  ○ 单 Agent 模式 (默认)                        │
│                  │  ● Multi-Agent 模式                            │
│                  │                                               │
│                  │  ═══════════════════════════════════════════  │
│                  │  默认 Agent 配置                               │
│                  │  ═══════════════════════════════════════════  │
│                  │  工作区目录: [~/.openclaw/workspace    ] [浏览]│
│                  │  Agent 目录: [~/.openclaw/agent        ] [浏览]│
│                  │  [高级设置...]                                 │
│                  │                                               │
│                  │  ═══════════════════════════════════════════  │
│                  │  自定义 Agent 列表 (仅 Multi-Agent 模式显示)    │
│                  │  ═══════════════════════════════════════════  │
│                  │  ┌─ architecturer ─────────────────────────┐  │
│                  │  │  名称: 架构师助手                          │  │
│                  │  │  工作区: ~/workspace-architecturer         │  │
│                  │  │  专用模型: 使用默认           [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  ┌─ developer ─────────────────────────────┐  │
│                  │  │  名称: 开发助手                            │  │
│                  │  │  工作区: ~/workspace-developer             │  │
│                  │  │  专用模型: openai/gpt-4o      [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加 Agent]                                 │
│                  │                                               │
└──────────────────┴──────────────────────────────────────────────┘
```

**添加 Agent：**

```
┌─────────────────────────────────────────────────────────────┐
│  添加 Agent                                        [保存]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Agent ID: [developer        ]  (自动生成，唯一标识)         │
│                                                              │
│  显示名称: [开发助手         ]                                │
│                                                              │
│  工作区目录: [~/.openclaw/workspace-developer] [浏览]        │
│  ├─ 默认生成规则: ~/.openclaw/workspace-{id}                 │
│  └─ 用户可修改                                               │
│                                                              │
│  Agent 目录: [~/.openclaw/agents/developer/agent] [浏览]     │
│  ├─ 默认生成规则: ~/.openclaw/agents/{id}/agent              │
│  └─ 用户可修改                                               │
│                                                              │
│  专用模型 (可选): [使用默认 ▼]                                │
│  ├─ 使用默认 (跟随全局模型优先级)                             │
│  ├─ moonshot-work/kimi-k2.5                                  │
│  ├─ openai-personal/gpt-4o                                   │
│  └─ ... (从已启用的 Provider 实例中选择)                      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**配置映射：**

```json
{
  "agents": {
    "defaults": {
      "workspace": "~/.openclaw/workspace",
      "agentDir": "~/.openclaw/agent"
    },
    "list": [
      {
        "id": "architecturer",
        "name": "架构师助手",
        "workspace": "~/.openclaw/workspace-architecturer",
        "agentDir": "~/.openclaw/agents/architecturer/agent"
      },
      {
        "id": "developer",
        "name": "开发助手",
        "workspace": "~/.openclaw/workspace-developer",
        "agentDir": "~/.openclaw/agents/developer/agent",
        "model": { "primary": "openai-personal/gpt-4o" }  // 专用模型
      }
    ]
  }
}
```

**关键规则：**

| 规则 | 说明 |
|------|------|
| Agent ID | 自动生成（基于 name 或用户输入），全局唯一 |
| 目录默认值 | 按规则自动生成，用户可修改 |
| 专用模型 | 可选，默认使用全局模型优先级设置 |
| 绑定关系 | Agent 与 Channel 的绑定在 **Channel 模块** 中配置 |

### 6.4 Memory 配置

涉及配置路径:
- `agents.defaults.memorySearch`

### 6.5 Channel 配置

**设计：类型 + 账号列表（Mattermost）/ 单实例（飞书）**

| Channel | 结构 | 多账号 | 绑定粒度 |
|---------|------|--------|---------|
| **Mattermost** | `accounts.{id}` | ✅ 支持 | 账号级别（`accountId`） |
| **飞书** | 单实例 | ❌ 不支持 | 用户/群级别（`peer.id`） |

**UI 设计：**

```
┌─────────────────────────────────────────────────────────────────┐
│  📱 Channel 配置                                 [保存] [重启]  │
├──────────────────┬──────────────────────────────────────────────┤
│  🧠 Provider     │                                               │
│  🤖 Agent        │  ── Mattermost ─────────────────────────────  │
│  🧠 Memory       │  全局配置:                                     │
│  📱 Channel      │  ├─ enabled: [✓]                              │
│                  │  ├─ dmPolicy: [pairing ▼]                     │
│                  │  └─ groupPolicy: [allowlist ▼]                │
│                  │                                               │
│                  │  账号列表:                                     │
│                  │  ┌─ default ──────────────────────────────┐  │
│                  │  │  name: Main Bot                         │  │
│                  │  │  botToken: ***                          │  │
│                  │  │  baseUrl: https://mm.hengshi.com        │  │
│                  │  │  绑定 Agent: [main ▼]      [配置] [删除]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  ┌─ architecturer ─────────────────────────┐  │
│                  │  │  name: Architecturer Bot                │  │
│                  │  │  botToken: ***                          │  │
│                  │  │  baseUrl: https://mm.hengshi.com        │  │
│                  │  │  绑定 Agent: [architecturer ▼] [配置][删]│  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加 Mattermost 账号]                       │
│                  │                                               │
│                  │  ── 飞书 ───────────────────────────────────  │
│                  │  全局配置:                                     │
│                  │  ├─ enabled: [✓]                              │
│                  │  ├─ dmPolicy: [pairing ▼]                     │
│                  │  ├─ connectionMode: [websocket ▼]             │
│                  │  └─ renderMode: [raw ▼]                       │
│                  │                                               │
│                  │  认证配置:                                     │
│                  │  ├─ appId: [cli_xxxx        ]                 │
│                  │  └─ appSecret: [*****       ]                 │
│                  │                                               │
│                  │  绑定列表 (Multi-Agent 模式):                  │
│                  │  ┌─ ou_xxx ───────────────────────────────┐  │
│                  │  │  类型: 用户  绑定 Agent: [agent-1 ▼]   │  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  ┌─ ou_yyy ───────────────────────────────┐  │
│                  │  │  类型: 群    绑定 Agent: [agent-2 ▼]   │  │
│                  │  └─────────────────────────────────────────┘  │
│                  │  [+ 添加绑定]                                   │
│                  │                                               │
└──────────────────┴──────────────────────────────────────────────┘
```

**添加 Mattermost 账号：**

```
┌─────────────────────────────────────────────────────────────┐
│  添加 Mattermost 账号                              [保存]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  账号 ID: [work              ]  (唯一标识，如 default)       │
│  显示名称: [Work Bot         ]                                │
│  Bot Token: [*****          ]                                 │
│  Base URL: [https://mm.hengshi.com]                          │
│                                                              │
│  绑定 Agent: [architecturer ▼]  (从已创建 Agent 中选择)      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**添加飞书绑定（Multi-Agent 模式）：**

```
┌─────────────────────────────────────────────────────────────┐
│  添加飞书绑定                                      [保存]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  用户/群 ID: [ou_zzz         ]  (飞书用户的 Open ID)         │
│  类型: [用户 ▼]  (用户/群)                                   │
│  绑定 Agent: [下拉选择 ▼]  (从已创建的 Agent 中选择)          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**配置映射：**

```json
{
  "channels": {
    "mattermost": {
      "enabled": true,
      "dmPolicy": "pairing",
      "groupPolicy": "allowlist",
      "accounts": {
        "default": {
          "name": "Main Bot",
          "botToken": "...",
          "baseUrl": "https://mm.hengshi.com"
        },
        "architecturer": {
          "name": "Architecturer Bot",
          "botToken": "...",
          "baseUrl": "https://mm.hengshi.com"
        }
      }
    },
    "feishu": {
      "enabled": true,
      "appId": "cli_xxx",
      "appSecret": "xxx",
      "connectionMode": "websocket",
      "dmPolicy": "pairing",
      "renderMode": "raw"
    }
  },
  "bindings": [
    {
      "agentId": "main",
      "match": { "channel": "mattermost", "accountId": "default" }
    },
    {
      "agentId": "architecturer",
      "match": { "channel": "mattermost", "accountId": "architecturer" }
    },
    {
      "agentId": "qiangxianfei",
      "match": { "channel": "feishu", "peer": { "kind": "dm", "id": "ou_xxx" } }
    }
  ]
}
```

**关键规则：**

| 规则 | 说明 |
|------|------|
| Mattermost | 支持多账号，每个账号独立配置 + 绑定 Agent |
| 飞书 | 单实例，通过绑定列表管理不同用户/群到不同 Agent |
| Default Agent 模式 | 飞书绑定列表可为空（使用 default agent） |
| Multi-Agent 模式 | 必须配置绑定关系，否则消息无法路由 |

---

## 7. API 设计

### 获取模块配置

```http
GET /api/config/modules/:module
Response: {
  "module": "models",
  "data": { ... },
  "schema": { ... },
  "restartRequired": false
}
```

### 保存模块配置

```http
POST /api/config/modules/:module
Body: { ... }
Response: {
  "success": true,
  "changes": ["moonshot.apiKey", "agents.defaults.model.primary"],
  "restartRequired": true,
  "commit": "abc123"
}
```

### 重启 OpenClaw

```http
POST /api/openclaw/restart
Response: {
  "success": true,
  "logs": [...]
}
```

### 获取 Provider Schema 列表

```http
GET /api/config/schemas/providers
Response: {
  "schemas": [
    { "id": "moonshot", "name": "Moonshot", "icon": "🌙" },
    { "id": "openai", "name": "OpenAI", "icon": "🤖" },
    { "id": "anthropic", "name": "Anthropic", "icon": "🧠" },
    { "id": "custom", "name": "Custom", "icon": "⚙️" }
  ]
}
```

---

## 8. 实现阶段规划

### Phase 1: 基础框架 + Provider 模块
- [ ] 后端 SchemaRegistry 实现
- [ ] 前端动态表单渲染
- [ ] Moonshot Provider 多实例支持
- [ ] OpenAI / Anthropic Provider
- [ ] 模型优先级设置
- [ ] 基础 API 实现

### Phase 2: Agent + Channel 模块
- [ ] Agent 单/多模式切换
- [ ] Agent 创建与配置
- [ ] Channel 多账号配置
- [ ] Agent-Channel 绑定

### Phase 3: Memory + 完善
- [ ] Memory 配置模块
- [ ] MiniMax Provider
- [ ] Custom Provider
- [ ] 配置验证（API Key 校验）
- [ ] Git 版本管理
- [ ] 重启机制

---

## 9. 历史文档

- `VISUAL_CONFIG_DESIGN_v2.md` - 本合并版本的前身文档
- `CONFIG_MODULAR_RESEARCH.md` - 方案调研分析
