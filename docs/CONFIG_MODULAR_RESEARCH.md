# Claw One 模块化配置系统 - 调研分析报告

**日期**: 2026-03-06  
**目的**: 评估配置模块化方案的可行性，探索简化设计

---

## 1. Schema 定义方式对比

### 1.1 方案对比

| 方案 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **YAML Schema** (当前设计) | 人类可读、易手写、注释友好 | 无标准验证机制、类型弱 | 简单配置、快速迭代 |
| **JSON Schema** | 标准成熟、验证工具多、生态好 | 冗长、手写痛苦、无注释 | 复杂验证、严格类型 |
| **CUE Lang** | 专为配置设计、类型安全、验证强大 | 学习曲线陡、小众、工具链不成熟 | 复杂配置合并、云原生 |
| **TypeScript DSL** | 类型安全、IDE 支持好 | 需要编译、重量级 | 开发时配置生成 |
| **纯 JSON + 代码** | 简单直接、无学习成本 | 扩展需改代码 | 内置固定模块 |

### 1.2 推荐方案

**阶段 1 (MVP)**: YAML Schema + 轻量验证
- 原因：Claw One 用户是开发者，熟悉 YAML
- 足够表达 Provider/Channel 等常见配置
- 易于调试和手动修复

**阶段 2 (成熟)**: YAML + JSON Schema 验证
- YAML 作为用户编写格式
- 转换为 JSON Schema 进行严格验证
- 兼顾可读性和可靠性

### 1.3 简化 Schema 设计

**最小必要字段**:
```yaml
# 最简 schema 示例
schema:
  fields:
    - id: apiKey
      type: password
      required: true
    
    - id: baseUrl
      type: string
      default: "https://api.xxx.com"

# 复杂条件显示可以省略，用帮助文本替代
```

**删除复杂功能**:
- ❌ 条件显示 (visible_when)
- ❌ 动态选项 (从 API 获取)
- ❌ 嵌套数组
- ✅ 保留：基本类型、默认值、必填验证

---

## 2. 脚本语言选择对比

### 2.1 方案对比

| 方案 | 优点 | 缺点 | 安全考虑 |
|------|------|------|----------|
| **Lua** | 轻量、成熟、易嵌入 | 需额外库、小众 | 沙箱需自行实现 |
| **QuickJS** | ES2020 支持、单文件 C 库 | 体积稍大 (~1MB) | 同样需沙箱 |
| **WASM** | 多语言支持、标准沙箱 | 复杂、过重 | 内存安全但可能 DoS |
| **Starlark** | Google 设计、确定性执行、沙箱原生 | Python 子集、小众 | 官方安全设计 |
| **无脚本** | 最安全、最简单 | 灵活性差 | 100% 安全 |
| **外部命令** | 简单、任意语言 | 性能差、安全难控 | 需限制执行权限 |

### 2.2 关键发现

**关于 Lua**:
- 标准 Lua 的 `os.execute` 和 `io` 库需要手动禁用
- 网络访问需要额外限制
- 调试支持一般

**关于 Starlark**:
- 为 Bazel 设计，专为不可信代码设计
- 无递归、无全局状态、确定性执行
- 原生禁止文件/网络访问
- **可能是更好的选择**

**关于无脚本**:
- 如果只做配置映射和简单验证，完全可以不用脚本
- 复杂验证交给后端硬编码
- **最简单的方案**

### 2.3 推荐方案

**方案 A (极简)**: 无脚本 + 后端硬编码验证
- Schema 只做字段定义
- 验证逻辑按模块类型硬编码（如所有 Provider 统一验证方式）
- 优点：安全、简单、可维护
- 缺点：新增验证类型需改代码

**方案 B (平衡)**: Starlark 脚本
- 确定性执行，天生沙箱
- 适合复杂验证逻辑
- 学习成本低于 Lua

---

## 3. 简化设计方案

### 3.1 极度简化版

**核心思想**: 不做通用插件系统，只做"配置模板"

```
config-modules/
├── moonshot.yaml       # 纯静态配置模板
├── deepseek.yaml       # 用户复制后修改
└── mattermost.yaml
```

**moonshot.yaml 内容**:
```yaml
# 这是一个配置模板，不是插件
name: Moonshot Provider
fields:
  - name: apiKey
    type: password
  - name: defaultModel
    type: select
    options: [kimi-k2.5, kimi-k2-turbo]

# 固定的 JSON 映射模板
mapping: |
  models.providers.moonshot.apiKey = {{apiKey}}
  models.providers.moonshot.models[0].id = {{defaultModel}}
  agents.defaults.model.primary = "moonshot/{{defaultModel}}"
```

**特点**:
- 无脚本、无验证、无复杂逻辑
- 模板引擎只做简单变量替换
- 新增 Provider = 复制模板 + 改字段
- 优点：极度简单、安全、易理解
- 缺点：不灵活，特殊逻辑需改代码

### 3.2 中等复杂度版

**核心思想**: 分类处理，不同类型不同处理

```rust
// 后端代码结构
enum ModuleType {
    Provider,    // 统一 Provider 处理逻辑
    Channel,     // 统一 Channel 处理逻辑
    Memory,      // 统一 Memory 处理逻辑
}

// 每个类型有硬编码的验证和映射逻辑
// Schema 只描述字段，不描述行为
```

**优点**:
- 代码可控、安全
- 新增同类型模块只需配置
- 不同类型扩展需改代码（可接受）

---

## 4. OpenClaw 配置复杂度分析

### 4.1 配置路径统计

分析 `openclaw.json` 关键配置项：

```
models.providers.{id}                    # Provider 配置
agents.defaults.model.primary            # 默认模型
agents.defaults.memorySearch             # Memory 配置
channels.{id}.accounts.{id}              # 渠道配置
agents.list[]                            # Agent 列表
bindings[]                               # Agent 绑定
plugins.entries.{id}                     # 插件启停
```

### 4.2 复杂度评估

**低复杂度** (适合配置化):
- Provider API Key、Base URL
- 渠道 Token、URL
- 插件启停开关

**中复杂度** (需要逻辑):
- 模型列表生成（依赖 Provider）
- Agent 工作区创建
- 渠道账号多实例

**高复杂度** (可能不适合):
- Memory Search 的嵌套配置
- Agent 的完整配置（workspace, agentDir）
- 复杂的 bindings 规则

### 4.3 关键发现

**发现 1**: OpenClaw 配置有大量**交叉引用**
- Provider 定义模型 → Agent 引用模型
- Agent 定义 ID → Bindings 引用 ID
- 这种引用关系难以用简单配置表达

**发现 2**: 配置结构**不规则**
- 有的路径是对象（`providers.moonshot`）
- 有的是数组（`agents.list[]`）
- 有的是嵌套对象（`memorySearch.store.vector`）
- 统一映射规则难以设计

**发现 3**: 需要**运行时验证**
- API Key 有效性
- 模型可用性
- 工作区目录可写
- 纯配置无法完成

---

## 5. 可行性结论

### 5.1 可能过于复杂的场景

❌ **完全通用插件系统**:
- 任意字段映射
- 复杂条件逻辑
- 自定义验证脚本
- 原因：OpenClaw 配置结构不规则，通用方案过于复杂

❌ **用户任意扩展新类型**:
- 比如用户想加一个全新的"Scheduler"类型
- 需要 UI、验证、映射全套支持
- 原因：需要前后端协调，难以动态支持

### 5.2 可行的简化方案

✅ **同类型模块配置化** (推荐):
- Provider: 统一表单，不同 Provider 填不同值
- Channel: 统一表单，不同渠道填不同值
- 新增 Moonshot/DeepSeek = 添加配置项，不改代码

✅ **配置模板系统**:
- 预定义模板（Provider 模板、Channel 模板）
- 用户基于模板创建实例
- 硬编码处理逻辑，配置只描述字段值

✅ **硬编码模块 + 配置启用**:
- 后端硬编码支持 10-20 个常见 Provider/Channel
- 通过配置启用/禁用
- 新增需改代码，但实现简单可靠

---

## 6. 最终建议

### 推荐方案: "分类配置模板"

**核心设计**:
1. **后端硬编码模块类型**: Provider、Channel、Memory、Agent
2. **每种类型统一处理逻辑**: 表单生成、验证、映射
3. **配置描述字段**: YAML 描述字段、默认值、选项
4. **不支持自定义类型**: 新增类型需改代码

**示例**:
```yaml
# modules/moonshot.yaml
name: Moonshot
type: provider
fields:
  - id: apiKey
    type: password
    required: true
  - id: defaultModel
    type: select
    options: [kimi-k2.5, kimi-k2-turbo, ...]
    default: kimi-k2.5

# 后端硬编码 mapping 逻辑
# 所有 Provider 统一映射到 models.providers.{id}
```

**优点**:
- 实现简单（1-2 周可完成）
- 安全可靠（无用户代码执行）
- 易于维护（逻辑集中）
- 足够灵活（覆盖 90% 场景）

**缺点**:
- 新增类型需改代码（可接受）
- 不支持复杂自定义逻辑（可用其他方式解决）

---

## 7. 待讨论问题

1. 是否接受"新增类型需改代码"的限制？
2. 优先实现哪种类型？（Provider / Channel / Memory）
3. 是否需要支持自定义 Provider（通过通用模板）？
4. 验证逻辑放在后端还是允许简单脚本？

---

**结论**: OpenClaw 配置结构复杂且不规则，**完全通用插件系统可能过于复杂**。建议采用"分类配置模板"方案，在简单性和灵活性之间取得平衡。
