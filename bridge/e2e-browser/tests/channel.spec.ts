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
 * 不依赖文件系统验证，而是通过 claw-one API 验证配置状态
 * API 路径: GET/POST/DELETE http://claw-one-test-app:8080/api/channels
 *
 * 关键 Bug 修复验证:
 * - Bug: saveChannels 命名冲突导致保存卡死
 * - 验证: 保存按钮点击后页面不卡死,5秒内返回
 */

import { test, expect } from '../fixtures';

const API_BASE = 'http://claw-one-test-app:8080';

/**
 * 通过 claw-one API 获取 channel 配置
 */
async function getChannelConfig(): Promise<any> {
  const response = await fetch(`${API_BASE}/api/channels`);
  if (!response.ok) {
    throw new Error(`API request failed: ${response.status}`);
  }
  return response.json();
}

/**
 * 通过 API 验证 channel 账号是否存在
 */
async function verifyChannelViaAPI(channelId: string, expectedData?: { name?: string; botToken?: string; baseUrl?: string }): Promise<boolean> {
  const config = await getChannelConfig();
  
  // 遍历所有 channel 类型查找账号
  for (const channelType of ['mattermost', 'feishu', 'ding', 'lark']) {
    const channelConfig = config[channelType];
    if (!channelConfig?.accounts) continue;
    
    if (channelConfig.accounts[channelId]) {
      if (!expectedData) return true;
      
      const account = channelConfig.accounts[channelId];
      if (expectedData.name && account.name !== expectedData.name) return false;
      if (expectedData.botToken && account.botToken !== expectedData.botToken) return false;
      if (expectedData.baseUrl && account.baseUrl !== expectedData.baseUrl) return false;
      return true;
    }
  }
  return false;
}

/**
 * 通过 API 删除 channel 账号
 */
async function deleteChannelViaAPI(channelId: string): Promise<void> {
  await fetch(`${API_BASE}/api/channels/${channelId}`, { method: 'DELETE' });
}

/**
 * 启用 Mattermost 开关
 */
async function enableMattermost(page: any): Promise<void> {
  const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
  const switch_ = mattermostSection.locator('.el-switch');
  const isChecked = await switch_.locator('input').isChecked().catch(() => false);
  if (!isChecked) {
    await switch_.click();
    await page.waitForTimeout(500);
  }
}

/**
 * 填写添加账号表单
 */
async function fillAccountForm(page: any, accountId: string, accountName: string, token: string, baseUrl: string): Promise<void> {
  await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(accountId);
  await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(accountName);
  await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(token);
  await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(baseUrl);
}

test.describe('Channel CRUD 完整链路测试', () => {

  test('添加账号 - 验证 UI 保存后 API 返回数据正确', async ({ page }) => {
    // 使用时间戳生成唯一 ID，避免测试冲突
    const testAccountId = `e2e-ui-add-${Date.now()}`;
    const testAccountName = `E2E UI Add ${Date.now()}`;
    const testToken = `token-${Date.now()}`;
    
    try {
      // 1. 访问 Channel 配置页面
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');

      // 2. 确保 Mattermost 已启用
      await enableMattermost(page);

      // 3. 点击添加账号按钮
      await page.click('button:has-text("+ 添加账号")');

      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      // 4. 填写表单
      await fillAccountForm(page, testAccountId, testAccountName, testToken, 'https://e2e-ui-add.example.com');

      // 5. 保存
      await page.click('.el-dialog__footer button:has-text("保存")');

      // 6. 验证对话框关闭（不卡死）
      await expect(dialog).not.toBeVisible({ timeout: 5000 });

      // 7. 验证账号出现在列表
      await expect(page.locator('.account-name', { hasText: testAccountName })).toBeVisible({ timeout: 5000 });

      // 8. 通过 API 验证 openclaw.json 中存在该账号（完整链路验证）
      const channelExists = await verifyChannelViaAPI(testAccountId, {
        name: testAccountName,
        botToken: testToken,
        baseUrl: 'https://e2e-ui-add.example.com'
      });
      expect(channelExists).toBeTruthy();
    } finally {
      // 清理
      await deleteChannelViaAPI(testAccountId);
    }
  });

  test('删除账号 - 验证 UI 删除后 API 返回数据已移除', async ({ page }) => {
    // TODO: 修复删除确认对话框选择器问题后再启用
    test.skip();
    // 使用时间戳生成唯一 ID
    const testAccountId = `e2e-ui-del-${Date.now()}`;
    const testAccountName = `E2E UI Del ${Date.now()}`;
    const testToken = `token-del-${Date.now()}`;
    
    try {
      // 1. 先通过 UI 创建测试账号（确保数据格式正确）
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      await enableMattermost(page);

      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      await fillAccountForm(page, testAccountId, testAccountName, testToken, 'https://delete-ui.example.com');

      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });

      // 验证账号存在
      await expect(page.locator('.account-name', { hasText: testAccountName })).toBeVisible();

      // 2. 通过 API 确认数据已创建
      const channelExistsBefore = await verifyChannelViaAPI(testAccountId);
      expect(channelExistsBefore).toBeTruthy();

      // 3. 点击删除按钮
      const accountCard = page.locator('.channel-account-item, .account-card', { hasText: testAccountName });
      await accountCard.locator('button:has-text(\"删除\")').click();

      // 4. 确认删除
      await page.locator('.el-message-box__btns button:has-text(\"确定\")').click();
      await page.waitForTimeout(500);

      // 5. 验证账号从列表消失
      await expect(page.locator('.account-name', { hasText: testAccountName })).not.toBeVisible();

      // 6. 通过 API 验证 openclaw.json 中该账号已删除（完整链路验证）
      const channelDeleted = !(await verifyChannelViaAPI(testAccountId));
      expect(channelDeleted).toBeTruthy();
    } finally {
      // 确保清理（如果删除失败）
      await deleteChannelViaAPI(testAccountId);
    }
  });

  test('保存配置 - 验证不卡死且 API 数据已更新', async ({ page }) => {
    // 使用时间戳生成唯一 ID
    const uniqueId = `e2e-ui-save-${Date.now()}`;
    const uniqueName = `E2E UI Save ${Date.now()}`;
    
    try {
      await page.goto('/config/channel');
      await page.waitForLoadState('networkidle');
      await enableMattermost(page);

      // 点击添加账号
      await page.click('button:has-text("+ 添加账号")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();

      await fillAccountForm(page, uniqueId, uniqueName, 'token', 'https://save-ui.example.com');

      // 保存 - 关键验证点：5秒内对话框必须关闭（不卡死）
      const startTime = Date.now();
      await page.click('.el-dialog__footer button:has-text("保存")');
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      const elapsed = Date.now() - startTime;
      
      console.log(`[Channel] 保存操作耗时: ${elapsed}ms`);
      expect(elapsed).toBeLessThan(5000);

      // 通过 API 验证 openclaw.json 中存在该配置
      const exists = await verifyChannelViaAPI(uniqueId, { name: uniqueName });
      expect(exists).toBeTruthy();
    } finally {
      // 清理
      await deleteChannelViaAPI(uniqueId);
    }
  });
});
