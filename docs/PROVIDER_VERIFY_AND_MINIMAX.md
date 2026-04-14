# Provider 增强功能设计

**版本**: v1.1
**日期**: 2026-04-14
**状态**: 待开发（P0 问题已修复）

---

## 1. 概述

本文档描述两项 Provider 增强功能：
1. **新增 MiniMax Provider** - 内置支持的模型服务商
2. **Provider Verify 功能** - 保存前验证凭证有效性

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

#### API 类型

MiniMax 使用 Anthropic 兼容的 API：
- API 类型: `anthropic-messages`

#### 默认模型

```typescript
const minimaxModels = [
  { id: 'MiniMax-M2.7', name: 'MiniMax M2.7', reasoning: true },
  { id: 'MiniMax-M2.7-highspeed', name: 'MiniMax M2.7 Highspeed', reasoning: true },
]
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

## 3. Provider Verify 功能

### 3.1 背景

当前保存 Provider 时，直接写入配置，如果凭证无效要等到实际使用才能发现。Verify 功能在保存前验证凭证有效性，提升用户体验。

### 3.2 P0 问题修复汇总

| 问题 | 修复方案 |
|------|----------|
| API 路径不一致 | 统一用 `/api/providers/verify`（预保存验证，不带 `:id`） |
| Anthropic 验证逻辑 bug | `content` 是数组，需检查 `as_array().map(\|a\| !a.is_empty())` |
| openai-completions 漏实现 | 添加 `verify_openai_compatible` 分支 |

### 3.3 前端 P1 问题修复

#### region 切换时清除验证状态

```typescript
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

#### API Key 格式校验（前置检查）

```typescript
const validateApiKeyFormat = (apiKey: string, apiType: string): { valid: boolean; message: string } => {
  if (!apiKey || apiKey.trim().length === 0) {
    return { valid: false, message: '请输入 API Key' }
  }

  // MiniMax: 长度校验（通常 50+ 字符）
  if (apiType === 'anthropic-messages' && apiKey.length < 30) {
    return { valid: false, message: 'API Key 格式不正确（长度过短）' }
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

### 3.4 安全建议（实现时需确认）

> ⚠️ 以下安全措施需要在实现阶段确认和落实：

1. **日志脱敏**: 后端 request logging 应排除 `/providers/verify` 路径，避免 API Key 泄露
2. **错误处理**: 避免使用 `unwrap()`，改用 `?` + 错误处理
3. **频率限制**: 考虑添加验证请求频率限制（如每分钟最多 10 次）

### 3.5 设计方案

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
pub async fn verify_provider(
    Extension(config_manager): Extension<Arc<ConfigManager>>,
    Json(data): Json<serde_json::Value>,
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
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    Ok(response.status().is_success())
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
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    // P0 修复: Anthropic 返回 200 即使 key 无效，content 是数组需要检查非空
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await?;
        // content 是 MessageContent 数组，需要检查数组非空
        let valid = body.get("content")
            .and_then(|c| c.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        Ok(valid)
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
      <el-button @click="verifyCredentials" :loading="verifying">
        验证
      </el-button>
    </template>
  </el-input>
</el-form-item>
<div v-if="verifyStatus" class="verify-result" :class="verifyStatus.valid ? 'success' : 'error'">
  {{ verifyStatus.message }}
</div>
```

##### 2. Verify 状态

```typescript
const verifying = ref(false)
const verifyStatus = ref<{ valid: boolean; message: string } | null>(null)
```

##### 3. Verify 方法

```typescript
const verifyCredentials = async () => {
  if (!formData.apiKey) {
    ElMessage.warning('请输入 API Key')
    return
  }

  // P1 修复: 前置格式校验
  const formatCheck = validateApiKeyFormat(formData.apiKey, formData.api)
  if (!formatCheck.valid) {
    ElMessage.warning(formatCheck.message)
    return
  }

  verifying.value = true
  verifyStatus.value = null

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

## 4. API 变更汇总

### 4.1 新增端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/providers/verify | 验证 Provider 凭证 |

### 4.2 请求/响应示例

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

## 5. 测试设计

### 5.1 单元测试（Layer 1）

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
```

### 5.2 集成测试（Layer 2）

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

### 5.3 E2E 测试（Layer 3）

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

## 6. 实现计划

### Phase 1: MiniMax Provider
- [ ] 后端: 添加 minimax 到 provider 类型识别
- [ ] 前端: 添加 minimax 类型和表单支持
- [ ] 测试: 单元测试 + 集成测试

### Phase 2: Provider Verify
- [ ] 后端: 实现 /api/providers/verify 端点
- [ ] 前端: 添加验证按钮和状态显示
- [ ] 测试: 单元测试 + 集成测试 + E2E

### Phase 3: 完整链路验证
- [ ] 测试: 从前端表单到后端 API 到配置文件完整路径
- [ ] 文档: 更新 FINAL_DESIGN.md

---

## 7. 风险与注意事项

1. **网络隔离**: Verify 需要访问外部 API，测试环境可能无法访问
2. **超时处理**: API 响应慢时需要合理超时
3. **敏感信息**: API Key 不应在前端日志中打印
4. **重试限制**: 避免频繁验证请求
5. **日志脱敏**: 后端需排除 /providers/verify 路径的详细日志
