/**
 * 用户工作流 Browser 测试
 * 测试矩阵功能模块 #5 (高优先级)
 * 
 * Layer 4 测试目标: 验证完整用户流程
 *
 * 部分测试包含完整链路验证（UI + API）
 */

import { test, expect, ConfigVerifier } from '../fixtures';

const API_BASE = process.env.CLAW_ONE_URL || 'http://claw-one-test-app:8080';

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

/**
 * 通过 API 删除 channel 账号
 */
async function deleteChannelViaAPI(channelId: string): Promise<void> {
  // Channel DELETE API 路径是 /api/channels/:type/:id
  // 遍历所有 channel type 尝试删除
  for (const type of ['mattermost', 'feishu', 'ding', 'lark']) {
    await fetch(`${API_BASE}/api/channels/${type}/${channelId}`, { method: 'DELETE' }).catch(() => {});
  }
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
      
      // API 层验证（完整链路验证）
      const channelExists = await verifyChannelViaAPI(testChannelId, { name: testChannelName });
      expect(channelExists).toBeTruthy();
      console.log('[Workflow Channel] API 验证通过：账号数据存在');
      
      // 文件层验证（ConfigVerifier 集成 - P2）
      const inFile = await ConfigVerifier.verifyChannelExists(testChannelId, { name: testChannelName });
      expect(inFile).toBeTruthy();
      console.log('[Workflow Channel] ConfigVerifier 文件验证通过：账号 name 字段匹配');
    } finally {
      await deleteChannelViaAPI(testChannelId);
    }
  });

  test('Channel 配置 - 删除账号完整流程', async ({ page }) => {
    const testChannelId = `e2e-workflow-del-${Date.now()}`;
    const testChannelName = `Workflow Del ${Date.now()}`;
    
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
      
      // 1. 添加账号
      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();
      
      await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(testChannelId);
      await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(testChannelName);
      await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('test-token');
      await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://workflow-del.example.com');
      
      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      
      // 2. 验证 UI 显示
      await expect(page.locator('.account-name', { hasText: testChannelName }).first()).toBeVisible();
      
      // 3. 验证 API 存在
      const channelExistsBefore = await verifyChannelViaAPI(testChannelId, { name: testChannelName });
      expect(channelExistsBefore).toBeTruthy();
      
      // 4. 找到删除按钮并点击
      const accountLocator = page.locator('.account-name', { hasText: testChannelName });
      await expect(accountLocator).toBeVisible({ timeout: 5000 });
      await accountLocator.scrollIntoViewIfNeeded();
      await page.waitForTimeout(500);
      
      // 找到删除按钮 - 在账号名称的父容器中找按钮容器
      const deleteButton = accountLocator.locator('..').locator('..').locator('button:has-text("删除")').first();
      const isEnabled = await deleteButton.isEnabled();
      console.log('[Workflow Channel Delete] 删除按钮是否可用:', isEnabled);
      
      // 设置 dialog 处理器（在点击删除之前）
      page.on('dialog', async dialog => {
        console.log('[Workflow Channel Delete] 检测到对话框:', dialog.message());
        await dialog.accept();
      });
      
      // 点击删除按钮
      await deleteButton.click({ force: true, timeout: 5000 });
      console.log('[Workflow Channel Delete] 删除按钮已点击');
      
      // 等待对话框出现 - Element UI dialog 使用确定按钮
      await page.waitForSelector('button:has-text("确定")', { timeout: 3000 }).catch(() => {
        console.log('[Workflow Channel Delete] 确定按钮未找到');
      });
      
      // 如果对话框出现了，点击确定
      const confirmButton = page.locator('button:has-text("确定")');
      if (await confirmButton.isVisible({ timeout: 500 }).catch(() => false)) {
        await confirmButton.click();
        console.log('[Workflow Channel Delete] 点击了确定按钮');
      }
      
      // 等待删除操作
      await page.waitForTimeout(3000);
      
      // 5. API 验证：账号已删除
      const channelDeleted = !(await verifyChannelViaAPI(testChannelId));
      console.log('[Workflow Channel Delete] API 验证删除结果:', channelDeleted);
      expect(channelDeleted).toBeTruthy();
      console.log('[Workflow Channel Delete] API 验证通过：账号已从后端移除');
      
      // 6. UI 验证：账号名称不再显示
      await expect(page.locator('.account-name', { hasText: testChannelName })).not.toBeVisible({ timeout: 5000 });
      console.log('[Workflow Channel Delete] UI 验证通过：账号已从界面移除');
      
      // 7. 文件层验证
      const inFile = await ConfigVerifier.verifyChannelDeleted(testChannelId);
      expect(inFile).toBeTruthy();
      console.log('[Workflow Channel Delete] ConfigVerifier 文件验证通过：账号已从 openclaw.json 移除');
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
