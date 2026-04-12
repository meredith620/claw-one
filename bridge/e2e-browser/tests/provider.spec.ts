/**
 * Provider CRUD Browser 测试
 * 测试矩阵功能模块 #1
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 *
 * API 路径: GET/POST/DELETE http://claw-one-test-app:8080/api/providers
 */

import { test, expect } from '../fixtures';

const API_BASE = 'http://claw-one-test-app:8080';

async function getProviders(): Promise<any[]> {
  const response = await fetch(`${API_BASE}/api/providers`);
  if (!response.ok) throw new Error(`API failed: ${response.status}`);
  return response.json();
}

async function deleteProviderViaAPI(providerId: string): Promise<void> {
  await fetch(`${API_BASE}/api/providers/${providerId}`, { method: 'DELETE' }).catch(() => {});
}

test.describe('Provider CRUD', () => {
  test.beforeEach(async ({ providerPage }) => {
    await providerPage.goto();
  });

  test('Provider 页面加载正常', async ({ page }) => {
    await expect(page.locator('.provider-section', { hasText: 'Moonshot' })).toBeVisible();
    await expect(page.locator('.provider-section', { hasText: 'OpenAI' })).toBeVisible();
    await expect(page.locator('.provider-section', { hasText: 'Anthropic' })).toBeVisible();
    await expect(page.locator('button:has-text("+ 添加实例")').first()).toBeVisible();
  });

  test('添加 Provider 实例 - 完整链路验证', async ({ page }) => {
    const uniqueName = `e2e-provider-${Date.now()}`;
    
    // 记录添加前的 provider 数量
    const providersBefore = await getProviders();
    const countBefore = providersBefore.length;
    
    try {
      await page.locator('button:has-text("+ 添加实例")').first().click();

      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      // 填写表单
      await dialog.locator('.el-form-item', { hasText: '实例名称' }).locator('input').fill(uniqueName);
      await dialog.locator('input[type="password"]').first().fill('sk-test-e2e-12345');

      // 选择默认模型
      const modelSelect = dialog.locator('.el-form-item', { hasText: '默认模型' }).locator('.el-select').first();
      const modelExists = await modelSelect.isVisible().catch(() => false);
      if (modelExists) {
        await modelSelect.click();
        await page.waitForTimeout(300);
        const firstOption = page.locator('.el-select-dropdown__item:visible').first();
        if (await firstOption.isVisible({ timeout: 2000 }).catch(() => false)) {
          await firstOption.click();
        }
      }

      // 保存
      await dialog.locator('.el-dialog__footer button:has-text("保存")').click();
      await expect(dialog).not.toBeVisible({ timeout: 10000 });

      // 验证 UI 显示新实例
      await expect(page.locator('.instance-name, .instance-card', { hasText: uniqueName })).toBeVisible({ timeout: 5000 });

      // 通过 API 验证数据已保存（完整链路验证）
      // 验证 provider 数量增加
      const providersAfter = await getProviders();
      const countAfter = providersAfter.length;
      console.log(`[Provider] 添加前数量: ${countBefore}, 添加后数量: ${countAfter}`);
      expect(countAfter).toBeGreaterThan(countBefore);
    } finally {
      // 清理 - 删除最新添加的 provider
      const providers = await getProviders();
      const latestProvider = providers.find(p => p.id.includes('e2e-provider'));
      if (latestProvider) {
        await deleteProviderViaAPI(latestProvider.id);
      }
    }
  });

  test('模型优先级设置区域存在', async ({ page }) => {
    await expect(page.locator('.priority-section')).toBeVisible();
    await expect(page.locator('.priority-section', { hasText: '模型优先级设置' })).toBeVisible();
  });
});
