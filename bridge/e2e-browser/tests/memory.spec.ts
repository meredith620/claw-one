/**
 * Memory 配置 Browser 测试
 * 测试矩阵功能模块 #4
 *
 * Layer 4 测试目标: 验证从前端 UI 保存成功后，配置文件内容正确
 * 验证方式: 检查保存成功消息 + ConfigVerifier 文件层验证
 */

import { test, expect, ConfigVerifier } from '../fixtures';

test.describe('Memory Configuration', () => {
  test.beforeEach(async ({ memoryPage }) => {
    await memoryPage.goto();
  });

  test('Memory 页面加载正常', async ({ page }) => {
    await expect(page.locator('.memory-section', { hasText: 'Memory 基础配置' })).toBeVisible();
    await expect(page.locator('.el-form-item', { hasText: '启用 Memory' })).toBeVisible();
  });

  test('Provider 选择存在', async ({ page }) => {
    await expect(page.locator('.el-radio-button', { hasText: 'Ollama' })).toBeVisible();
    await expect(page.locator('.el-radio-button', { hasText: 'OpenAI' })).toBeVisible();
  });

  test('保存 Memory 配置按钮存在', async ({ page }) => {
    await expect(page.locator('button:has-text("保存 Memory 配置")')).toBeVisible();
  });

  test('高级功能区域存在', async ({ page }) => {
    await expect(page.locator('.memory-section', { hasText: '高级功能' })).toBeVisible();
  });

  test('启用 Memory 并保存 - 完整链路验证', async ({ page }) => {
    // 1. 启用 Memory 开关
    const switch_ = page.locator('.el-form-item', { hasText: '启用 Memory' }).locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked();
    if (!isChecked) {
      await switch_.click();
      await page.waitForTimeout(300);
    }

    // 2. 选择 Ollama provider
    await page.locator('.el-radio-button', { hasText: 'Ollama' }).click();
    await page.waitForTimeout(300);

    // 3. 填写 Base URL
    const baseUrl = 'http://localhost:11434';
    const baseUrlInput = page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').first();
    await baseUrlInput.fill(baseUrl);

    // 4. 保存
    await page.click('button:has-text("保存 Memory 配置")');
    
    // 5. 验证保存成功消息（方案A：验证 UI 反馈而非直接调用 API）
    await expect(page.locator('.el-message:has-text("保存成功")')).toBeVisible({ timeout: 5000 });
    console.log('[Memory] UI 保存成功消息已显示');
    
    // 6. 文件层验证（ConfigVerifier 集成 - P1）
    const inFile = await ConfigVerifier.verifyMemoryExists({
      enabled: true,
      provider: 'ollama',
      baseUrl: baseUrl
    });
    expect(inFile).toBeTruthy();
    console.log('[Memory] ConfigVerifier 文件验证通过：enabled/provider/baseUrl 字段匹配');
  });

  test('禁用 Memory - 验证 UI 关闭后 API 和文件层数据已更新', async ({ page }) => {
    // 1. 先启用 Memory（作为前置条件）
    const switch_ = page.locator('.el-form-item', { hasText: '启用 Memory' }).locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked();
    if (!isChecked) {
      await switch_.click();
      await page.waitForTimeout(300);
    }

    // 2. 选择 Ollama provider
    await page.locator('.el-radio-button', { hasText: 'Ollama' }).click();
    await page.waitForTimeout(300);

    // 3. 填写 Base URL
    const baseUrl = 'http://localhost:11434';
    const baseUrlInput = page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').first();
    await baseUrlInput.fill(baseUrl);

    // 4. 保存（启用状态）
    await page.click('button:has-text("保存 Memory 配置")');
    
    // 5. 验证启用保存成功
    await expect(page.locator('.el-message:has-text("保存成功")')).toBeVisible({ timeout: 5000 });
    console.log('[Memory] 启用保存成功');
    
    // 6. 禁用 Memory 开关
    await switch_.click();
    await page.waitForTimeout(300);

    // 7. 保存（禁用状态）
    await page.click('button:has-text("保存 Memory 配置")');
    
    // 8. 验证禁用保存成功消息
    await expect(page.locator('.el-message:has-text("保存成功")')).toBeVisible({ timeout: 5000 });
    console.log('[Memory Delete] UI 保存成功消息已显示');
    
    // 9. 文件层验证：Memory 配置中 enabled 已为 false
    const inFile = await ConfigVerifier.verifyMemoryExists({
      enabled: false,
      provider: 'ollama',
      baseUrl: baseUrl
    });
    expect(inFile).toBeTruthy();
    console.log('[Memory Delete] ConfigVerifier 文件验证通过：enabled 字段已更新为 false');
  });
});
