# Provider 增强功能设计

**版本**: v1.5
**日期**: 2026-04-14
**状态**: 待开发（P0 问题已全部修复）

---

## 1. 概述

本文档描述三项 Provider 增强功能：
1. **完善 Moonshot Provider** - 基于 OpenClaw 源码补充完整信息
2. **新增 MiniMax Provider** - 内置支持的模型服务商
3. **Provider Verify 功能** - 保存前验证凭证有效性

---

## 2. MiniMax Provider

### 2.1 背景

MiniMax 是继 Moonshot 之后又一个需要内置支持的模型服务商。根据 OpenClaw 文档，MiniMax 提供：
- MiniMax-M2.7（默认推理模型）
- MiniMax-M2.7-highspeed（更快推理层）
- 图像理解（MiniMax-VL-01）
- 图像生成（image-01）
- 音乐生成（music-2.5+）
- 视频生成（Hailuo-2.3）

### 2.2 设计方案

#### Provider Type 定义

在 `providerTypes` 中新增 `minimax` 类型：

```typescript
const providerTypes = [
  { id: 'moonshot', name: 'Moonshot', icon: '🌙' },
  { id: 'openai', name: 'OpenAI', icon: '🤖' },
  { id: 'anthropic', name: 'Anthropic', icon: '🧠' },
  { id: 'minimax', name: 'MiniMax', icon: '🔵' },  // 新增
  { id: 'custom', name: '其他 Provider', icon: '🔧' },
]
```

#### Base URL 配置

MiniMax 有两个端点：
- 国际版: `https://api.minimax.io/anthropic`
- 中国版: `https://api.minimaxi.com/anthropic`

```typescript
const minimaxBaseUrls = {
  global: 'https://api.minimax.io/anthropic',
  cn: 'https://api.minimaxi.com/anthropic',
}
```

> **注意**: 还有 `https://api.minimax.io/v1` 端点，用于非 Anthropic 兼容场景。

#### API 类型

MiniMax 使用 Anthropic 兼容的 API：
- API 类型: `anthropic-messages`
- authHeader: true

#### 模型目录（基于 OpenClaw 源码）

```typescript
const MINIMAX_TEXT_MODEL_CATALOG = {
  "MiniMax-M2.7": {
    name: "MiniMax M2.7",
    reasoning: true,
    input: ["text", "image"],  // M2.7 支持图像理解
  },
  "MiniMax-M2.7-highspeed": {
    name: "MiniMax M2.7 Highspeed",
    reasoning: true,
    input: ["text"],
  },
};

const DEFAULT_MINIMAX_CONTEXT_WINDOW = 204800;
const DEFAULT_MINIMAX_MAX_TOKENS = 131072;
const MINIMAX_DEFAULT_MODEL_ID = "MiniMax-M2.7";
```

### 2.3 前端实现

#### 1. Provider 类型列表

```typescript
// ProviderModule.vue
const providerTypes = [
  { id: 'moonshot', name: 'Moonshot', icon: '🌙' },
  { id: 'openai', name: 'OpenAI', icon: '🤖' },
  { id: 'anthropic', name: 'Anthropic', icon: '🧠' },
  { id: 'minimax', name: 'MiniMax', icon: '🔵' },  // 新增
  { id: 'custom', name: '其他 Provider', icon: '🔧' },
]
```

#### 2. 自动检测 MiniMax Provider

> **P2 修复**: 原检测逻辑 `id.startsWith('minimax-')` 可能误匹配第三方 provider。改用更精确的 URL 检测。

```typescript
// loadData() 中新增判断
// 只通过 baseUrl 检测，避免 id 冲突
const isMinimax = baseUrl.includes('minimax.io') ||
                  baseUrl.includes('minimaxi.com')

if (isMinimax) {
  instances.minimax.push(p)
}
```

#### 3. MiniMax 表单特殊处理

```vue
<el-form-item v-if="currentType === 'minimax'" label="地区">
  <el-radio-group v-model="formData.region">
    <el-radio-button label="global">国际版</el-radio-button>
    <el-radio-button label="cn">中国版</el-radio-button>
  </el-radio-group>
</el-form-item>
```

```typescript
// MiniMax 表单默认值（基于 OpenClaw 源码）
const getDefaultFormData = (type: string) => {
  if (type === 'minimax') {
    return {
      region: 'global',  // P1 修复: 明确默认值
      baseUrl: 'https://api.minimax.io/anthropic',
      api: 'anthropic-messages',
      defaultModel: 'MiniMax-M2.7',
      contextWindow: 204800,
      maxTokens: 131072,
    }
  }
  // ...
}

// MiniMax 自动设置 baseUrl
watch(() => formData.region, (region) => {
  if (currentType.value === 'minimax') {
    formData.baseUrl = region === 'cn'
      ? 'https://api.minimaxi.com/anthropic'
      : 'https://api.minimax.io/anthropic'
    formData.api = 'anthropic-messages'
    // P1 修复: region 切换时重置验证状态
    verifyStatus.value = null
  }
})
```

---

## 3. Moonshot Provider（基于 OpenClaw 源码补充）

### 3.1 OpenClaw 源码参考

根据 OpenClaw 源码 `provider-catalog-zY8QJDj_.js` 和 `onboard.ts`：

#### Base URL 配置

```typescript
const MOONSHOT_BASE_URL = "https://api.moonshot.ai/v1";
const MOONSHOT_CN_BASE_URL = "https://api.moonshot.cn/v1";
```

#### API 类型

```typescript
// Moonshot 使用 OpenAI 兼容 API
const api = "openai-completions";
```

> **Moonshot Verify 说明**: Moonshot 使用 `openai-completions`，Verify API 用 `GET /models` 验证。Moonshot API 是 OpenAI 兼容的，`/models` 端点应正常工作。

#### 模型目录（Model Catalog）

```typescript
const MOONSHOT_MODEL_CATALOG = [
  {
    id: "kimi-k2.5",
    name: "Kimi K2.5",
    reasoning: false,
    input: ["text", "image"],
    contextWindow: 262144,
    maxTokens: 262144,
  },
  {
    id: "kimi-k2-thinking",
    name: "Kimi K2 Thinking",
    reasoning: true,
    input: ["text"],
    contextWindow: 262144,
    maxTokens: 262144,
  },
  {
    id: "kimi-k2-thinking-turbo",
    name: "Kimi K2 Thinking Turbo",
    reasoning: true,
    input: ["text"],
    contextWindow: 262144,
    maxTokens: 262144,
  },
  {
    id: "kimi-k2-turbo",
    name: "Kimi K2 Turbo",
    reasoning: false,
    input: ["text"],
    contextWindow: 256000,
    maxTokens: 16384,
  },
];
```

#### 默认模型

```typescript
const MOONSHOT_DEFAULT_MODEL_ID = "kimi-k2.5";
const MOONSHOT_DEFAULT_MODEL_REF = "moonshot/kimi-k2.5";
```

### 3.2 自动检测 Moonshot Provider

```typescript
// loadData() 中判断
const isMoonshot = baseUrl.includes('moonshot.ai') ||
                   baseUrl.includes('moonshot.cn')

if (isMoonshot) {
  instances.moonshot.push(p)
}
```

### 3.3 Moonshot 表单特殊处理

```vue
<el-form-item v-if="currentType === 'moonshot'" label="地区">
  <el-radio-group v-model="formData.region">
    <el-radio-button label="global">国际版</el-radio-button>
    <el-radio-button label="cn">中国版</el-radio-button>
  </el-radio-group>
</el-form-item>
```

```typescript
// Moonshot 表单默认值
const getDefaultFormData = (type: string) => {
  if (type === 'moonshot') {
    return {
      region: 'global',
      baseUrl: 'https://api.moonshot.ai/v1',
      api: 'openai-completions',
      defaultModel: 'kimi-k2.5',
    }
  }
  // ...
}

// region 切换时自动设置 baseUrl
watch(() => formData.region, (region) => {
  if (currentType.value === 'moonshot') {
    formData.baseUrl = region === 'cn'
      ? 'https://api.moonshot.cn/v1'
      : 'https://api.moonshot.ai/v1'
    formData.api = 'openai-completions'
    verifyStatus.value = null
  }
})
```

### 3.4 模型选择器

```vue
<el-form-item v-if="currentType === 'moonshot'" label="模型">
  <el-select v-model="formData.model" placeholder="选择模型">
    <el-option label="Kimi K2.5" value="kimi-k2.5" />
    <el-option label="Kimi K2 Thinking" value="kimi-k2-thinking" />
    <el-option label="Kimi K2 Thinking Turbo" value="kimi-k2-thinking-turbo" />
    <el-option label="Kimi K2 Turbo" value="kimi-k2-turbo" />
  </el-select>
</el-form-item>
```

---

## 4. Provider Verify 功能

### 4.1 背景

当前保存 Provider 时，直接写入配置，如果凭证无效要等到实际使用才能发现。Verify 功能在保存前验证凭证有效性，提升用户体验。

### 4.2 Review 问题修复汇总

| 优先级 | 问题 | 修复方案 |
|--------|------|----------|
| **P0** | API 路径不一致 | 统一用 `/api/providers/verify`（预保存验证，不带 `:id`） |
| **P0** | Anthropic 验证逻辑 bug | `content` 是数组，需检查 `as_array().map(\|a\| !a.is_empty())` |
| **P0** | openai-completions 漏实现 | 添加 `verify_openai_compatible` 分支 |
| **P0** | 无用 Extension(config_manager) | 移除未使用的 Extension |
| **P0** | OpenAI 兼容验证只检查 HTTP 状态 | 需检查 `data` 数组非空 |
| **P0** | Anthropic 验证未排除 error 响应 | 需检查 `error` 字段不存在 |
| **P1** | region 切换时未清除验证状态 | 重置 `verifyStatus` |
| **P1** | 格式校验失败无 UI 反馈 | 格式错误时设置 `verifyStatus` 显示错误 |
| **P1** | MiniMax region 默认值未定义 | 明确默认值为 `global` |
| **P2** | 验证按钮 loading 时可重复点击 | 添加 `:disabled="verifying"` |
| **P2** | 无超时反馈 | 添加超时检测，10s 后提示用户 |
| **P2** | MiniMax API Key 格式校验不精确 | 改用正则校验：`^eyJ[A-Za-z0-9-_]+$`（JWT 格式） |

### 4.3 前端实现

#### API Key 格式校验（前置检查）

```typescript
const validateApiKeyFormat = (apiKey: string, apiType: string): { valid: boolean; message: string } => {
  if (!apiKey || apiKey.trim().length === 0) {
    return { valid: false, message: '请输入 API Key' }
  }

  // MiniMax: JWT 格式校验（eyJ 开头）
  if (apiType === 'anthropic-messages') {
    if (!/^eyJ[A-Za-z0-9-_]+$/.test(apiKey)) {
      return { valid: false, message: 'MiniMax API Key 格式不正确（应为 JWT 格式）' }
    }
  }

  // OpenAI: 应以 sk- 开头
  if (apiType === 'openai-chat' && !apiKey.startsWith('sk-')) {
    return { valid: false, message: 'OpenAI API Key 应以 sk- 开头' }
  }

  return { valid: true, message: '' }
}
```

#### P2 优化：添加 debounce 防抖

```typescript
import { debounce } from 'lodash-es'

const debouncedVerify = debounce(() => {
  verifyCredentials()
}, 500)

// 在输入框上使用
@input="debouncedVerify"
```

### 4.4 安全建议（实现时需确认）

> ⚠️ 以下安全措施需要在实现阶段确认和落实：

1. **日志脱敏**: 后端 request logging 应排除 `/providers/verify` 路径，避免 API Key 泄露
2. **错误处理**: 避免使用 `unwrap()`，改用 `?` + 错误处理
3. **频率限制**: 考虑添加验证请求频率限制（如每分钟最多 10 次）

### 4.5 设计方案

#### API 设计

**新增 API 端点:**

```
POST /api/providers/verify
```

> **P0 修复**: 原设计使用 `/api/providers/:id/verify`，但这是"预保存验证"接口，此时 provider 还未创建，没有 `:id`。统一使用 `/api/providers/verify`（不带 `:id`）。

**请求体:**
```json
{
  "apiKey": "sk-xxx",
  "baseUrl": "https://api.openai.com",
  "api": "openai-chat"
}
```

**响应:**
```json
// 成功
{
  "success": true,
  "valid": true,
  "message": "凭证验证通过"
}

// 失败
{
  "success": true,
  "valid": false,
  "message": "Invalid API key: API key invalid"
}

// 网络错误
{
  "success": false,
  "error": "无法连接到 API 端点"
}
```

**HTTP 状态码说明:**
- `200`: 请求成功（无论验证通过与否）
- `400`: 请求参数缺失或格式错误
- `408`: 请求超时
- `500`: 服务器内部错误

#### 验证策略

不同 API 类型使用不同的验证方式：

| API 类型 | 验证方法 |
|----------|----------|
| `openai-chat` | GET /models 列出模型 |
| `anthropic-messages` | POST /v1/messages 发送测试消息 |
| `openai-completions` | GET /models 列出模型 |

#### 后端实现

```rust
// hull/src/api/providers.rs

/// 验证 Provider 凭证（预保存验证，不依赖 provider id）
/// POST /api/providers/verify
pub async fn verify_provider(
    Json(data): Json<serde_json::Value>,  // P0 修复: 移除未使用的 Extension(config_manager)
) -> Result<Json<serde_json::Value>> {
    let api_key = data.get("apiKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing apiKey".to_string()))?;

    let base_url = data.get("baseUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing baseUrl".to_string()))?;

    let api_type = data.get("api")
        .and_then(|v| v.as_str())
        .unwrap_or("openai-chat");

    // 根据 API 类型选择验证方法
    match verify_by_api_type(api_type, base_url, api_key).await {
        Ok(valid) => Ok(Json(serde_json::json!({
            "success": true,
            "valid": valid,
            "message": if valid { "凭证验证通过" } else { "凭证无效" }
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })))
    }
}

async fn verify_by_api_type(api_type: &str, base_url: &str, api_key: &str) -> Result<bool> {
    match api_type {
        "openai-chat" => {
            verify_openai_compatible(base_url, api_key).await
        }
        // P0 修复: openai-completions 之前漏实现，落入默认分支 Ok(true)
        "openai-completions" => {
            verify_openai_compatible(base_url, api_key).await
        }
        "anthropic-messages" => {
            verify_anthropic_compatible(base_url, api_key).await
        }
        _ => Ok(true) // 未知类型跳过验证
    }
}

async fn verify_openai_compatible(base_url: &str, api_key: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}/models", base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(Duration::from_secs(10))  // P2 修复: 明确超时时间
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::BadRequest("验证请求超时，请检查网络连接".to_string())
            } else {
                AppError::BadRequest(format!("验证请求失败: {}", e))
            }
        })?;

    // P0 修复: 仅检查 HTTP 200 不够，需验证响应体中 models 数组非空
    // OpenAI 兼容 API 返回 { "data": [...models...] }
    if response.status().is_success() {
        let body: serde_json::Value = response.json().await
            .map_err(|e| AppError::BadRequest(format!("解析响应失败: {}", e)))?;
        let valid = body.get("data")
            .and_then(|d| d.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        Ok(valid)
    } else {
        Ok(false)
    }
}

async fn verify_anthropic_compatible(base_url: &str, api_key: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .timeout(Duration::from_secs(10))  // P2 修复: 明确超时时间
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::BadRequest("验证请求超时，请检查网络连接".to_string())
            } else {
                AppError::BadRequest(format!("验证请求失败: {}", e))
            }
        })?;

    // P0 修复: Anthropic 返回 200 即使 key 无效
    // 1. content 数组需要非空
    // 2. 需要排除 error 响应（如 { "error": { "type": "authentication_error" } }）
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await?;
        // 先检查是否有 error 字段，有 error 则验证失败
        if body.get("error").is_some() {
            return Ok(false);
        }
        // content 是 MessageContent 数组，需要检查数组非空
        let content_valid = body.get("content")
            .and_then(|c| c.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        Ok(content_valid)
    } else {
        Ok(false)
    }
}
```

#### 前端实现

##### 1. 新增 Verify 按钮

```vue
<el-form-item label="API Key" required>
  <el-input v-model="formData.apiKey" type="password" placeholder="输入 API Key" show-password>
    <template #append>
      <!-- P2 修复: loading 时禁用按钮 -->
      <el-button @click="verifyCredentials" :loading="verifying" :disabled="verifying">
        {{ verifying ? '验证中...' : '验证' }}
      </el-button>
    </template>
  </el-input>
</el-form-item>
<!-- 格式校验错误也显示在这里 -->
<div v-if="verifyStatus" class="verify-result" :class="verifyStatus.valid ? 'success' : 'error'">
  {{ verifyStatus.message }}
</div>
```

##### 2. Verify 状态

```typescript
const verifying = ref(false)
const verifyStatus = ref<{ valid: boolean; message: string } | null>(null)
const verifyTimeout = ref<number | null>(null)
```

##### 3. Verify 方法

```typescript
const verifyCredentials = async () => {
  if (!formData.apiKey) {
    verifyStatus.value = { valid: false, message: '请输入 API Key' }
    return
  }

  // P1 修复: 格式校验失败时设置 verifyStatus，UI 显示错误
  const formatCheck = validateApiKeyFormat(formData.apiKey, formData.api)
  if (!formatCheck.valid) {
    verifyStatus.value = { valid: false, message: formatCheck.message }
    return
  }

  verifying.value = true
  verifyStatus.value = null

  // P2 修复: 设置 15s 超时（大于后端 10s 超时）
  const timeoutId = setTimeout(() => {
    verifying.value = false
    verifyStatus.value = { valid: false, message: '验证超时，请检查网络连接' }
    ElMessage.error('验证请求超时')
  }, 15000)
  verifyTimeout.value = timeoutId as any

  try {
    const response = await fetch(`/api/providers/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        apiKey: formData.apiKey,
        baseUrl: formData.baseUrl,
        api: formData.api,
      }),
    })

    // 清除超时计时器
    if (verifyTimeout.value) {
      clearTimeout(verifyTimeout.value)
      verifyTimeout.value = null
    }

    const result = await response.json()

    if (result.success) {
      verifyStatus.value = {
        valid: result.valid,
        message: result.message || (result.valid ? '验证通过' : '验证失败')
      }
      if (!result.valid) {
        ElMessage.error('API Key 无效，请检查')
      }
    } else {
      verifyStatus.value = {
        valid: false,
        message: result.error || '验证失败'
      }
      ElMessage.error(result.error || '验证请求失败')
    }
  } catch (e) {
    if (verifyTimeout.value) {
      clearTimeout(verifyTimeout.value)
      verifyTimeout.value = null
    }
    verifyStatus.value = {
      valid: false,
      message: '网络错误，请检查连接'
    }
    ElMessage.error('验证请求失败')
  } finally {
    verifying.value = false
  }
}
```

##### 4. 保存前自动验证（可选）

```typescript
const saveInstance = async () => {
  // 如果有验证结果且无效，阻止保存
  if (verifyStatus.value && !verifyStatus.value.valid) {
    const confirmed = await ElMessageBox.confirm(
      'API Key 验证失败，是否仍要保存？',
      '验证失败',
      { confirmButtonText: '仍要保存', cancelButtonText: '取消' }
    ).catch(() => false)

    if (!confirmed) return
  }

  // ... 保存逻辑
}
```

---

## 5. API 变更汇总

### 5.1 新增端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/providers/verify | 验证 Provider 凭证 |

### 5.2 请求/响应示例

```bash
# 验证 Provider
curl -X POST http://localhost:8080/api/providers/verify \
  -H "Content-Type: application/json" \
  -d '{
    "apiKey": "sk-xxx",
    "baseUrl": "https://api.minimax.io/anthropic",
    "api": "anthropic-messages"
  }'
```

---

## 6. 测试设计

### 6.1 单元测试（Layer 1）

```rust
// hull/tests/api_providers.rs

#[tokio::test]
async fn test_verify_provider_success() {
    // Mock 一个成功的 API 响应
}

#[tokio::test]
async fn test_verify_provider_invalid_key() {
    // Mock 一个失败的 API 响应
}

#[tokio::test]
async fn test_verify_provider_network_error() {
    // Mock 一个网络错误
}

#[tokio::test]
async fn test_verify_provider_timeout() {
    // Mock 一个超时响应
}
```

### 6.2 集成测试（Layer 2）

```rust
#[tokio::test]
async fn test_verify_minimax_provider() {
    let server = TestServer::new().await;

    let response = server
        .client
        .post(server.url("/api/providers/verify"))
        .json(&serde_json::json!({
            "apiKey": "test-key",
            "baseUrl": "https://api.minimax.io/anthropic",
            "api": "anthropic-messages"
        }))
        .send()
        .await
        .unwrap();

    // 验证响应结构
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert!(body.get("success").is_some());
}
```

### 6.3 E2E 测试（Layer 3）

```bash
# e2e/tests/test_provider_verify.sh
#!/bin/bash
set -e

echo "测试: Provider 验证功能"

# 1. 访问 Provider 配置页面
# 2. 填写表单
# 3. 点击验证按钮
# 4. 验证结果显示
# 5. 确认保存
```

---

## 7. 实现计划

### Phase 1: Moonshot Provider 完善
- [ ] 前端: 补充 Moonshot region 切换（global/cn）
- [ ] 前端: 补充模型选择器
- [ ] 验证: 与 OpenClaw 源码保持一致

### Phase 2: MiniMax Provider
- [ ] 后端: 添加 minimax 到 provider 类型识别
- [ ] 前端: 添加 minimax 类型和表单支持
- [ ] 测试: 单元测试 + 集成测试

### Phase 3: Provider Verify
- [ ] 后端: 实现 /api/providers/verify 端点
- [ ] 前端: 添加验证按钮和状态显示
- [ ] 测试: 单元测试 + 集成测试 + E2E

### Phase 4: 完整链路验证
- [ ] 测试: 从前端表单到后端 API 到配置文件完整路径
- [ ] 文档: 更新 FINAL_DESIGN.md

---

## 8. 风险与注意事项

1. **网络隔离**: Verify 需要访问外部 API，测试环境可能无法访问
2. **超时处理**: API 响应慢时需要合理超时（后端 10s，前端 15s）
3. **敏感信息**: API Key 不应在前端日志中打印
4. **重试限制**: 避免频繁验证请求
5. **日志脱敏**: 后端需排除 /providers/verify 路径的详细日志
6. **HTTP 状态码**: 验证结果在 body 中返回，HTTP 状态码用于表示请求是否成功
