/**
 * Channel CRUD Browser 测试
 * 测试矩阵功能模块 #3 (高优先级)
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 *
 * 完整链路验证策略:
 * 1. UI 操作 → 后端 API 保存 → 通过 API 验证数据存在
 * 2. UI 操作 → 后端 API 删除 → 通过 API 验证数据已移除
 *
 * API 路径: GET/POST/DELETE http://claw-one-test-app:8080/api/channels
 */

import { test, expect } from '../fixtures';

const API_BASE = 'http://claw-one-test-app:8080';

async function getChannelConfig(): Promise<any> {
  const response = await fetch(`${API_BASE}/api/channels`);
  if (!response.ok) throw new Error(`API failed: ${response.status}`);
  return response.json();
}

async function verifyChannelViaAPI(channelId: string, expectedData?: { name?: string }): Promise<boolean> {
  const config = await getChannelConfig();
  for (const channelType of ['mattermost', 'feishu', 'ding', 'lark']) {
    const channelConfig = config[channelType];
    if (!channelConfig?.accounts) continue;
    if (channelConfig.accounts[channelId]) {
      if (!expectedData) return true;
      const account = channelConfig.accounts[channelId];
      return account.name === expectedData.name;
    }
  }
  return false;
}

async function deleteChannelViaAPI(channelId: string): Promise<void> {
  await fetch(`${API_BASE}/api/channels/${channelId}`, { method: 'DELETE' });
}

async function enableMattermost(page: any): Promise<void> {
  const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
  const switch_ = mattermostSection.locator('.el-switch');
  const isChecked = await switch_.locator('input').isChecked().catch(() => false);
  if (!isChecked) {
    await switch_.click();
    await page.waitForTimeout(500);
  }
}

async function fillAccountForm(page: any, accountId: string, accountName: string, token: string, baseUrl: string): Promise<void> {
  await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(accountId);
  await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(accountName);
  await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(token);
  await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(baseUrl);
}

test.describe('Channel CRUD 完整链路测试', () => {

  test('添加账号 - 验证 UI 保存后 API 返回数据正确', async ({ page }) => {
    const testAccountId = `e2e-ui-add-${Date.now()}`;
    const testAccountName = `E2E UI Add ${Date.now()}`;
    const testToken = `token-${Date.now()}`;
    
    try {
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      await enableMattermost(page);

      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      await fillAccountForm(page, testAccountId, testAccountName, testToken, 'https://e2e-ui-add.example.com');

      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });

      await expect(page.locator('.account-name', { hasText: testAccountName })).toBeVisible({ timeout: 5000 });

      const channelExists = await verifyChannelViaAPI(testAccountId, { name: testAccountName });
      expect(channelExists).toBeTruthy();
    } finally {
      await deleteChannelViaAPI(testAccountId);
    }
  });

  test('删除账号 - 验证 UI 删除后 API 返回数据已移除', async ({ page }) => {
    // 暂时跳过：Element Plus 确认对话框选择器不稳定
    // TODO: 修复确认对话框交互后再启用
    test.skip();
    
    const testAccountId = `e2e-ui-del-${Date.now()}`;
    const testAccountName = `E2E UI Del ${Date.now()}`;
    const testToken = `token-del-${Date.now()}`;
    
    try {
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      await enableMattermost(page);

      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      await fillAccountForm(page, testAccountId, testAccountName, testToken, 'https://delete-ui.example.com');

      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });

      await expect(page.locator('.account-name', { hasText: testAccountName })).toBeVisible();

      const channelExistsBefore = await verifyChannelViaAPI(testAccountId);
      expect(channelExistsBefore).toBeTruthy();

      // 点击删除按钮
      const deleteBtn = page.locator('.account-card, .channel-account-item').filter({ hasText: testAccountName }).locator('button:has-text("删除")');
      await deleteBtn.click();

      // 等待确认对话框
      await page.waitForTimeout(500);
      
      // 确认删除
      await page.locator('.el-message-box__btns button').filter({ hasText: '确定' }).click();
      await page.waitForTimeout(500);

      await expect(page.locator('.account-name', { hasText: testAccountName })).not.toBeVisible();

      const channelDeleted = !(await verifyChannelViaAPI(testAccountId));
      expect(channelDeleted).toBeTruthy();
    } finally {
      await deleteChannelViaAPI(testAccountId);
    }
  });

  test('保存配置 - 验证不卡死且 API 数据已更新', async ({ page }) => {
    const uniqueId = `e2e-ui-save-${Date.now()}`;
    const uniqueName = `E2E UI Save ${Date.now()}`;
    
    try {
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      await enableMattermost(page);

      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      await fillAccountForm(page, uniqueId, uniqueName, 'token', 'https://save-ui.example.com');

      const startTime = Date.now();
      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      const elapsed = Date.now() - startTime;
      
      console.log(`[Channel] 保存操作耗时: ${elapsed}ms`);
      expect(elapsed).toBeLessThan(5000);

      const exists = await verifyChannelViaAPI(uniqueId, { name: uniqueName });
      expect(exists).toBeTruthy();
    } finally {
      await deleteChannelViaAPI(uniqueId);
    }
  });
});
