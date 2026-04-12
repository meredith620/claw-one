/**
 * Provider CRUD Browser 测试
 * 测试矩阵功能模块 #1
 */

import { test, expect } from '../fixtures';

test.describe('Provider CRUD', () => {
  test.beforeEach(async ({ providerPage }) => {
    await providerPage.goto();
  });

  // ─── 基础 UI 测试 ───────────────────────────────────────────

  test('Provider 页面加载正常', async ({ page }) => {
    // 验证 Provider 分类存在
    await expect(page.locator('.provider-section', { hasText: 'Moonshot' })).toBeVisible();
    await expect(page.locator('.provider-section', { hasText: 'OpenAI' })).toBeVisible();
    await expect(page.locator('.provider-section', { hasText: 'Anthropic' })).toBeVisible();

    // 验证添加实例按钮存在
    await expect(page.locator('button:has-text("+ 添加实例")').first()).toBeVisible();
  });

  test('添加 Provider 实例 - 打开对话框', async ({ page }) => {
    // 点击添加实例按钮
    await page.locator('button:has-text("+ 添加实例")').first().click();

    // 验证对话框打开
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();

    // 验证表单字段存在
    await expect(page.locator('.el-form-item', { hasText: '实例名称' })).toBeVisible();
    await expect(page.locator('.el-form-item', { hasText: 'API Key' })).toBeVisible();
  });

  test('模型优先级设置区域存在', async ({ page }) => {
    // 验证模型优先级区域
    await expect(page.locator('.priority-section')).toBeVisible();
    await expect(page.locator('.priority-section', { hasText: '模型优先级设置' })).toBeVisible();
  });

  // ─── CRUD 测试说明 ─────────────────────────────────────────
  // Provider CRUD 完整测试在 Layer 3 (API E2E) 中已覆盖
  // Layer 4 只测试纯 UI 层面的交互
  //
  // 问题: 从 Playwright 容器内调用 API (http://claw-one-test-app:8080)
  // 存在网络隔离问题，page.request 无法正常获取 JSON 响应
  //
  // 解决方案: Layer 4 专注于测试 UI 交互逻辑，不依赖数据准备
  // - 对话框打开/关闭
  // - 表单字段存在
  // - 按钮响应
  // 完整 CRUD 数据流由 Layer 3 (shell + curl) 测试覆盖
});
