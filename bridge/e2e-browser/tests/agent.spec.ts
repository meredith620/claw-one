/**
 * Agent CRUD Browser 测试
 * 测试矩阵功能模块 #2
 */

import { test, expect } from '../fixtures';

test.describe('Agent Configuration', () => {
  test.beforeEach(async ({ agentPage }) => {
    await agentPage.goto();
  });

  test('Agent 页面加载正常', async ({ page }) => {
    // 验证模式选择存在
    await expect(page.locator('.agent-section', { hasText: 'Agent 模式' })).toBeVisible();
    
    // 验证单 Agent 和多 Agent 模式选项
    await expect(page.locator('.el-radio-button', { hasText: '单 Agent 模式' })).toBeVisible();
    await expect(page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' })).toBeVisible();
  });

  test('切换到 Multi-Agent 模式并添加 Agent', async ({ page }) => {
    // 切换到 Multi-Agent 模式
    await page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' }).click();
    await page.waitForTimeout(300);
    
    // 验证添加 Agent 按钮出现
    await expect(page.locator('button:has-text("+ 添加 Agent")')).toBeVisible();
    
    // 点击添加 Agent
    await page.click('button:has-text("+ 添加 Agent")');
    
    // 验证对话框
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    await expect(page.locator('.el-form-item', { hasText: 'Agent ID' })).toBeVisible();
    await expect(page.locator('.el-form-item', { hasText: '显示名称' })).toBeVisible();
  });

  test('保存 Agent 配置按钮存在', async ({ page }) => {
    await expect(page.locator('button:has-text("保存 Agent 配置")')).toBeVisible();
  });
});
