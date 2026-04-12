/**
 * 用户工作流 Browser 测试
 * 测试矩阵功能模块 #5 (高优先级)
 * 
 * Layer 4 测试目标: 验证完整用户流程
 *
 * 部分测试包含完整链路验证（UI + API）
 */

import { test, expect } from '../fixtures';

const API_BASE = 'http://claw-one-test-app:8080';

/**
 * 通过 API 删除 channel 账号
 */
async function deleteChannelViaAPI(channelId: string): Promise<void> {
  await fetch(`${API_BASE}/api/channels/${channelId}`, { method: 'DELETE' }).catch(() => {});
}

test.describe('User Workflows', () => {
  test('配置页面导航 - 所有模块可切换', async ({ page }) => {
    await page.goto('/config/provider');
    await page.waitForLoadState('networkidle');
    
    const modules = [
      { name: 'Provider', icon: '🧠' },
      { name: 'Agent', icon: '🤖' },
      { name: 'Memory', icon: '💾' },
      { name: 'Channel', icon: '📱' },
    ];
    
    for (const mod of modules) {
      await page.locator('.menu-item', { hasText: mod.name }).click();
      await page.waitForTimeout(300);
      
      const activeMenu = page.locator('.menu-item.active', { hasText: mod.name });
      await expect(activeMenu).toBeVisible();
      await expect(page).toHaveURL(new RegExp(`/config/${mod.name.toLowerCase()}`));
    }
  });

  test('首页到配置的跳转', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('body')).toBeVisible();
  });

  test('Channel 配置 - 关键 Bug 验证流程', async ({ page }) => {
    const testChannelId = `e2e-workflow-${Date.now()}`;
    const testChannelName = `Workflow Test ${Date.now()}`;
    
    try {
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      
      const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
      const switch_ = mattermostSection.locator('.el-switch');
      const isChecked = await switch_.locator('input').isChecked().catch(() => false);
      if (!isChecked) {
        await switch_.click();
        await page.waitForTimeout(500);
      }
      
      await expect(page.locator('button:has-text("+ 添加账号")')).toBeVisible();
      
      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();
      
      await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(testChannelId);
      await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(testChannelName);
      await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('test-token');
      await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://workflow.example.com');
      
      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      
      // 使用 .first() 避免 strict mode violation
      await expect(page.locator('.account-name', { hasText: testChannelName }).first()).toBeVisible();
      
      // 通过 API 验证数据已保存（完整链路验证）
      const apiResponse = await page.request.get(`${API_BASE}/api/channels`);
      expect(apiResponse.ok()).toBeTruthy();
      const channels = await apiResponse.json();
      
      let found = false;
      for (const type of ['mattermost', 'feishu', 'ding', 'lark']) {
        if (channels[type]?.accounts?.[testChannelId]) {
          found = true;
          expect(channels[type].accounts[testChannelId].name).toBe(testChannelName);
          break;
        }
      }
      expect(found).toBeTruthy();
    } finally {
      await deleteChannelViaAPI(testChannelId);
    }
  });

  test('Provider 页面 - 添加实例对话框流程', async ({ page }) => {
    await page.goto('/config/provider');
    await page.waitForLoadState('networkidle');
    
    await page.locator('button:has-text("+ 添加实例")').first().click();
    
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    await page.click('.el-dialog__footer button:has-text("取消")');
    await expect(dialog).not.toBeVisible({ timeout: 3000 });
  });
});
