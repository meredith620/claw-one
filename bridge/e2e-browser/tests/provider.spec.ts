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

  // ─── CRUD 测试 ────────────────────────────────────────────

  test('添加 Provider 实例 - 完整流程', async ({ page, providerPage }) => {
    const uniqueName = `test-openai-${Date.now()}`;
    const providerData = { name: uniqueName, apiKey: 'sk-test-key-12345' };

    // 调用 addProvider 添加实例
    await providerPage.addProvider(providerData, 'openai');

    // 验证出现在列表中
    await providerPage.verifyProviderExists(uniqueName);

    // 清理：删除该 Provider（测试结束后不保留脏数据）
    await providerPage.deleteProvider(uniqueName);
  });

  test('重复创建同名 Provider 报错', async ({ page, providerPage }) => {
    const uniqueName = `test-dup-${Date.now()}`;
    const providerData = { name: uniqueName, apiKey: 'sk-test-key-dup' };

    // 创建第一个 Provider
    await providerPage.addProvider(providerData, 'openai');
    await providerPage.verifyProviderExists(uniqueName);

    // 再创建同名 Provider，预期后端拒绝（弹窗错误或表单校验）
    await providerPage.addProvider(providerData, 'openai');

    // 期望出现错误提示（ElMessage 错误类型 或 表单内联报错）
    const hasToastError = await page.locator('.el-message--error').count();
    const hasFormError = await page.locator('.el-form-item__error').count();

    // 至少有一种错误反馈
    expect(hasToastError > 0 || hasFormError > 0).toBeTruthy();

    // 清理
    await providerPage.deleteProvider(uniqueName);
  });

  test('编辑 Provider 实例', async ({ page, providerPage }) => {
    const originalName = `test-edit-src-${Date.now()}`;
    const editedName = `test-edit-dst-${Date.now()}`;
    const providerData = { name: originalName, apiKey: 'sk-edit-original' };

    // 创建 Provider
    await providerPage.addProvider(providerData, 'openai');
    await providerPage.verifyProviderExists(originalName);

    // 点击编辑按钮
    const card = page.locator('.instance-card', { hasText: originalName });
    await card.locator('button:has-text("编辑")').click();

    // 验证对话框打开
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();

    // 修改实例名称
    const nameInput = dialog.locator('.el-form-item', { hasText: '实例名称' }).locator('input');
    await nameInput.fill('');
    await nameInput.fill(editedName);

    // 修改 API Key
    const apiKeyInput = dialog.locator('input[type="password"]');
    await apiKeyInput.fill('sk-edit-modified');

    // 保存
    await dialog.locator('button:has-text("保存")').click();
    await expect(dialog).not.toBeVisible({ timeout: 5000 });

    // 验证新名称出现在列表，旧名称消失
    await providerPage.verifyProviderExists(editedName);
    await expect(page.locator('.instance-id', { hasText: originalName })).not.toBeVisible();

    // 清理
    await providerPage.deleteProvider(editedName);
  });

  test('删除 Provider 实例', async ({ page, providerPage }) => {
    const uniqueName = `test-delete-${Date.now()}`;
    const providerData = { name: uniqueName, apiKey: 'sk-delete-key' };

    // 创建 Provider
    await providerPage.addProvider(providerData, 'openai');
    await providerPage.verifyProviderExists(uniqueName);

    // 删除 Provider
    await providerPage.deleteProvider(uniqueName);

    // 验证从列表消失
    await expect(page.locator('.instance-id', { hasText: uniqueName })).not.toBeVisible();
  });
});
