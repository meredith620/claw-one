/**
 * Memory 配置 Browser 测试
 * 测试矩阵功能模块 #4
 */

import { test, expect } from '../fixtures';

test.describe('Memory Configuration', () => {
  test.beforeEach(async ({ memoryPage }) => {
    await memoryPage.goto();
  });

  test('Memory 页面加载正常', async ({ page }) => {
    // 验证基础配置区域
    await expect(page.locator('.memory-section', { hasText: 'Memory 基础配置' })).toBeVisible();
    
    // 验证启用开关
    await expect(page.locator('.el-form-item', { hasText: '启用 Memory' })).toBeVisible();
  });

  test('Provider 选择存在', async ({ page }) => {
    // 验证 Provider 选项
    await expect(page.locator('.el-radio-button', { hasText: 'Ollama' })).toBeVisible();
    await expect(page.locator('.el-radio-button', { hasText: 'OpenAI' })).toBeVisible();
  });

  test('保存 Memory 配置按钮存在', async ({ page }) => {
    await expect(page.locator('button:has-text("保存 Memory 配置")')).toBeVisible();
  });

  test('高级功能区域存在', async ({ page }) => {
    await expect(page.locator('.memory-section', { hasText: '高级功能' })).toBeVisible();
  });
});
