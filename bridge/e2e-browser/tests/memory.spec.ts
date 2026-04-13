/**
 * Memory 配置 Browser 测试
 * 测试矩阵功能模块 #4
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 *
 * API 路径: GET/PUT http://claw-one-test-app:8080/api/memory
 */

import { test, expect, ConfigVerifier } from '../fixtures';

const API_BASE = 'http://claw-one-test-app:8080';

async function getMemory(): Promise<any> {
  const response = await fetch(`${API_BASE}/api/memory`);
  if (!response.ok) throw new Error(`API failed: ${response.status}`);
  return response.json();
}

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
    
    // 等待保存完成
    await page.waitForTimeout(1000);

    // 5. API 详细验证（完整链路验证 - Issue 2）
    const response = await page.request.get(`${API_BASE}/api/memory`);
    expect(response.ok()).toBeTruthy();
    
    // 核心断言：验证具体字段值
    const memData = await response.json();
    console.log('[Memory] API 返回数据:', JSON.stringify(memData, null, 2));
    
    // 验证 enabled 字段
    expect(memData.enabled).toBe(true);
    
    // 验证 provider 字段
    expect(memData.provider).toBe('ollama');
    
    // 验证 baseUrl 字段
    expect(memData.baseUrl).toBe(baseUrl);
    
    console.log('[Memory] API 详细验证通过：enabled/provider/baseUrl 字段正确');
    
    // 文件层验证（ConfigVerifier 集成 - P1）
    const inFile = await ConfigVerifier.verifyMemoryExists({
      enabled: true,
      provider: 'ollama',
      baseUrl: baseUrl
    });
    expect(inFile).toBeTruthy();
    console.log('[Memory] ConfigVerifier 文件验证通过：enabled/provider/baseUrl 字段匹配');
  });
});
