# Claw One 可视化配置设计文档

**版本**: v2.2  
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
| **内置 Provider** | Moonshot、OpenAI、Anthropic、Custom |
| **内置 Provider API 格式** | **固定格式，不需要用户选择** |
| **Custom Provider API 格式** | **用户选择**（openai-completions/responses/anthropic-messages） |
| **表单渲染** | 轻量动态生成，基于 Schema 而非硬编码 |

---

## 3. 内置 Provider 列表

| 内置 Provider | 阶段 | 固定 API 格式 | 说明 |
|--------------|------|--------------|------|
| **Moonshot AI** | Phase 1 | `openai-completions` | 优先实现，验证可行性 |
| **OpenAI** | Phase 2 | `openai-completions` 或 `openai-responses` | OpenAI 支持两种 API 格式 |
| **Anthropic** | Phase 2 | `anthropic-messages` | Claude 系列模型 |
| **Custom** | Phase 3 | 用户选择 | 支持 `openai-completions` / `openai-responses` / `anthropic-messages` |

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

**布局设计：两栏布局**

采用**左侧 Provider 列表 + 右侧配置详情**的两栏布局：

```
┌─────────────────────────────────────────────────────────────────┐
│  Claw One 配置                                 [保存] [重启]    │
├──────────────────┬──────────────────────────────────────────────┤
│  🧠 Provider     │  AI Provider 配置                             │
│  🤖 Agent        │  ───────────────────────────────────────────  │
│  🧠 Memory       │                                               │
│  📱 Channel      │  ┌─────────────────────────────────────────┐ │
│                  │  │ 🌙 Moonshot    [开关]  [配置▼] [删除]  │ │
│                  │  │    状态: ✅ 已启用                       │ │
│                  │  │    模型: kimi-k2.5                       │ │
│                  │  └─────────────────────────────────────────┘ │
│                  │  ┌─────────────────────────────────────────┐ │
│                  │  │ 🤖 OpenAI      [开关]  [配置▼] [删除]  │ │
│                  │  │    状态: ⚪ 未启用                       │ │
│                  │  └─────────────────────────────────────────┘ │
│                  │  ┌─────────────────────────────────────────┐ │
│                  │  │ 🧠 Anthropic   [开关]  [配置▼] [删除]  │ │
│                  │  │    状态: ⚪ 未启用                       │ │
│                  │  └─────────────────────────────────────────┘ │
│                  │                                               │
│                  │  [+ 添加 Custom Provider]                     │
│                  │                                               │
└──────────────────┴──────────────────────────────────────────────┘
```

**点击展开 Provider 详情表单：**

```
┌─────────────────────────────────────────────────────────────────┐
│  Claw One 配置                                 [保存] [重启]    │
├──────────────────┬──────────────────────────────────────────────┤
│  🧠 Provider     │  🌙 Moonshot 配置                             │
│  🤖 Agent        │  ───────────────────────────────────────────  │
│  🧠 Memory       │                                               │
│  📱 Channel      │  基础配置                                      │
│                  │  ───────────────────────────────────────────  │
│                  │  启用 Provider          [✓ 开启]              │
│                  │  API Key                [sk-***    ] [显示]   │
│                  │  Base URL               [https://api...]      │
│                  │                                               │
│                  │  模型配置                                      │
│                  │  ───────────────────────────────────────────  │
│                  │  默认模型               [Kimi K2.5 ▼]         │
│                  │                                               │
│                  │  [取消]  [保存配置]                             │
│                  │                                               │
└──────────────────┴──────────────────────────────────────────────┘
```

**布局说明：**

| 布局 | 适用性 | 决策 |
|------|--------|------|
| **两栏** | 模块少(4个)，内容区宽敞 | ✅ **采用** |
| 三栏 | 模块多，需要二级导航 | 不适用 |

**设计理由：**
1. 只有 4 个核心模块，左侧导航足够简洁
2. Provider 列表本身就是二级导航，点击后在右侧展开详情
3. 两栏布局给表单内容区更多空间
4. 不需要导入/导出功能，界面更简洁

---

**内置 Provider Schema 定义**

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

### 6.3 Multi-Agent 配置

涉及配置路径:
- `agents.list[].{id,name,workspace,agentDir}`
- `bindings[].{agentId,match.channel,match.accountId}`

### 6.4 Memory 配置

涉及配置路径:
- `agents.defaults.memorySearch`

### 6.5 Channel 配置

涉及配置路径:
- `channels.feishu`（支持多账号）
- `channels.mattermost`（支持多账号）

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

### Phase 1: 基础框架
- [ ] 后端 SchemaRegistry 实现
- [ ] Provider Schema 定义（Moonshot）
- [ ] 前端动态表单渲染
- [ ] 基础 API 实现

### Phase 2: 核心功能
- [ ] OpenAI / Anthropic Provider Schema
- [ ] Agent 配置模块
- [ ] Memory 配置模块
- [ ] Channel 配置模块

### Phase 3: 完善
- [ ] Custom Provider 支持
- [ ] 配置验证（API Key 校验）
- [ ] Git 版本管理
- [ ] 重启机制

---

## 9. 历史文档

- `VISUAL_CONFIG_DESIGN_v2.md` - 本合并版本的前身文档
- `CONFIG_MODULAR_RESEARCH.md` - 方案调研分析
