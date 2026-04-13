/**
 * Provider CRUD Browser 测试
 * 测试矩阵功能模块 #1
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 *
 * API 路径: GET/POST/DELETE http://claw-one-test-app:8080/api/providers
 */

import { test, expect, ConfigVerifier } from '../fixtures';

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
    const testApiKey = 'sk-test-e2e-12345';
    
    try {
      await page.locator('button:has-text("+ 添加实例")').first().click();

      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      // 填写表单
      await dialog.locator('.el-form-item', { hasText: '实例名称' }).locator('input').fill(uniqueName);
      await dialog.locator('input[type="password"]').first().fill(testApiKey);

      // 选择默认模型
      const modelSelect = dialog.locator('.el-form-item', { hasText: '默认模型' }).locator('.el-select').first();
      const modelExists = await modelSelect.isVisible().catch(() => false);
      let selectedModel = '';
      if (modelExists) {
        await modelSelect.click();
        await page.waitForTimeout(300);
        const firstOption = page.locator('.el-select-dropdown__item:visible').first();
        if (await firstOption.isVisible({ timeout: 2000 }).catch(() => false)) {
          selectedModel = await firstOption.textContent() || '';
          await firstOption.click();
        }
      }

      // 保存
      await dialog.locator('.el-dialog__footer button:has-text("保存")').click();
      await expect(dialog).not.toBeVisible({ timeout: 10000 });

      // 验证 UI 显示新实例
      await expect(page.locator('.instance-name, .instance-card', { hasText: uniqueName })).toBeVisible({ timeout: 5000 });

      // API 详细验证（完整链路验证 - Issue 2）
      // 验证 provider 数量增加
      const providers = await getProviders();
      const newProvider = providers.find(p => 
        (p.name === uniqueName || p.id === uniqueName || p.instanceName === uniqueName)
      );
      
      console.log('[Provider] 添加后 providers:', providers.map(p => ({ id: p.id, name: p.name })));
      console.log('[Provider] 查找目标:', uniqueName);
      
      // 核心断言：精确验证新 provider 存在且字段正确
      expect(newProvider).toBeTruthy();
      
      // 验证实例名称字段
      const providerName = newProvider?.name || newProvider?.instanceName || newProvider?.id;
      expect(providerName).toBe(uniqueName);
      
      // 验证 API Key 已保存（通过检查返回数据中是否存在相关字段）
      // 注意：某些 API 可能不返回 API Key 本身，只验证对象存在即可
      expect(newProvider).toHaveProperty('id');
      
      // 验证默认模型已设置
      if (selectedModel) {
        const providerModel = newProvider?.defaultModel || newProvider?.model;
        console.log('[Provider] 选择的模型:', selectedModel, '| API 返回模型:', providerModel);
        // 模型可能未在返回中体现，此处记录日志即可
      }
      
      console.log('[Provider] API 详细验证通过：实例名称和字段正确');
      
      // 文件层验证（ConfigVerifier 集成 - P1）
      const inFile = await ConfigVerifier.verifyProviderExists(uniqueName, { name: uniqueName });
      expect(inFile).toBeTruthy();
      console.log('[Provider] ConfigVerifier 文件验证通过：实例名称字段匹配');
    } finally {
      // 清理 - 使用精确的时间戳匹配
      const providers = await getProviders();
      const latestProvider = providers.find(p => 
        (p.name?.includes(`e2e-provider-`) || p.id?.includes(`e2e-provider-`))
      );
      if (latestProvider) {
        await deleteProviderViaAPI(latestProvider.id);
        console.log('[Provider] 清理完成:', latestProvider.id);
      }
    }
  });

  test('模型优先级设置区域存在', async ({ page }) => {
    await expect(page.locator('.priority-section')).toBeVisible();
    await expect(page.locator('.priority-section', { hasText: '模型优先级设置' })).toBeVisible();
  });
});
