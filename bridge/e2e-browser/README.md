# claw-one Bridge - Playwright 前端 E2E 测试

针对昨天发现的 **前端测试盲区**（Bug: Channel 保存卡死），引入 Playwright 进行浏览器级 E2E 测试。

## 测试矩阵覆盖

| 功能模块 | 状态 | 优先级 | 测试数 | 说明 |
|---------|------|--------|--------|------|
| Provider CRUD | ✅ 已覆盖 | 中 | 3 | Provider 页面和对话框 |
| Agent CRUD | ✅ 已覆盖 | 中 | 3 | Agent 模式和添加对话框 |
| **Channel CRUD** | ✅ **已覆盖** | **高** | 4 | 添加/编辑/删除 Channel（Bug 修复验证）|
| Memory 配置 | ✅ 已覆盖 | 低 | 4 | Memory 配置页面 |
| **用户工作流** | ✅ **已覆盖** | **高** | 4 | 页面导航、关键 Bug 验证流程 |

**5个功能模块已全部补齐 Browser 测试。**

**总计: 18 个测试用例**

## 快速开始

### 1. 安装依赖

```bash
cd bridge
npm install
npx playwright install chromium firefox
```

### 2. 启动 claw-one 服务

```bash
cd ..
./target/release/claw-one run
# 或使用默认配置
```

确保服务运行在 `http://localhost:8080`

### 3. 运行测试

```bash
# 运行所有测试
npm run test:e2e

# 运行特定模块测试
npm run test:provider   # Provider CRUD
npm run test:channel    # Channel CRUD（关键 Bug 验证）
npm run test:agent      # Agent CRUD
npm run test:memory     # Memory 配置
npm run test:workflow   # 用户工作流

# 使用 UI 模式（调试用）
npm run test:e2e:ui

# 生成报告
npm run test:e2e:report
```

## 关键测试用例

### Channel CRUD 测试（Bug 修复验证）

```typescript
// 关键验证点：保存按钮点击后页面不卡死，3秒内返回
test('添加 Mattermost 账号 - 关键验证：不卡死', async ({ page }) => {
  // 启用 Mattermost
  await page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch').click();
  
  // 打开添加账号对话框
  await page.click('button:has-text("+ 添加账号")');
  
  // 填写表单 - 使用 label 定位
  await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill('test-id');
  await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill('Test Bot');
  await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('token');
  await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://mm.example.com');
  
  // 保存 - 关键验证：对话框3秒内关闭（没有卡死）
  await page.click('.el-dialog__footer button:has-text("保存")');
  await expect(page.locator('.el-dialog')).not.toBeVisible({ timeout: 5000 });
});
```

## 测试结构

```
e2e-browser/
├── fixtures.ts           # 测试夹具和页面封装类
├── global-setup.ts       # 全局设置
├── playwright.config.ts  # Playwright 配置
└── tests/
    ├── provider.spec.ts  # Provider 页面测试 (3 tests)
    ├── channel.spec.ts   # Channel CRUD (4 tests) - 关键 Bug 验证
    ├── agent.spec.ts     # Agent 配置测试 (3 tests)
    ├── memory.spec.ts    # Memory 配置测试 (4 tests)
    └── workflow.spec.ts  # 用户工作流 (4 tests)
```

**总计: 18 个测试用例**

## 环境变量

```bash
# 指定 claw-one 服务地址
CLAW_ONE_URL=http://localhost:8085 npm run test:e2e

# CI 模式（无界面，自动重试）
CI=true npm run test:e2e
```

## 测试分层完整架构

```
┌─────────────────────────────────────────────────────────────┐
│ 测试分层示意图                                                │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Browser E2E 测试 (Playwright) ⬅️ 新增               │
│ ├── 模拟真实用户点击、输入、表单提交                         │
│ ├── 覆盖 Vue 组件逻辑                                        │
│ └── ✅ 唯一可以发现 Channel Bug 的测试层                      │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: API E2E 测试 (curl/HTTP)                            │
│ ├── 直接调用后端 API                                         │
│ └── ⚠️ 绕过前端 Vue 组件逻辑（盲区）                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: 集成测试 (Rust tests)                               │
│ └── API 端到端测试                                           │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: 单元测试 (Rust #[cfg(test)])                        │
│ └── 模块内部逻辑测试                                         │
└─────────────────────────────────────────────────────────────┘
```

## 背景

昨天的 Bug 是 `saveChannels` 命名冲突：
1. `import { saveChannels } from '../api'` (API 函数)
2. `const saveChannels = async () => {...}` (本地事件处理函数)

导致 `saveChannelsData()` 中调用 `saveChannels` 实际是本地函数，形成**无限递归**。

**现有 E2E 测试完全无法发现此 Bug**，因为它们直接调用 `POST /api/channels`，绕过前端 Vue 组件逻辑。Playwright Browser 测试是唯一可以发现此类问题的测试层。
