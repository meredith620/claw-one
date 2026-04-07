/**
 * Channel CRUD Browser 测试
 * 测试矩阵功能模块 #3 (高优先级)
 *
 * 这是昨天修复的关键 Bug 所在模块:
 * - Bug: saveChannels 命名冲突导致保存卡死
 * - 验证: 保存按钮点击后页面不卡死,3秒内返回
 */

import { test, expect, testData } from '../fixtures';

test('添加 Mattermost 账号 - 关键验证:不卡死', async ({ page }) => {
  // 直接访问 Channel 配置页面
  await page.goto('/config/channel');
  await page.waitForLoadState('networkidle');

  // 点击 Mattermost 开关启用(无论当前状态,都点击两次确保启用)
  const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
  await mattermostSection.locator('.el-switch').click();
  await page.waitForTimeout(300);
  await mattermostSection.locator('.el-switch').click();
  await page.waitForTimeout(500);

  // 等待账号区域渲染完成
  await page.waitForSelector('.subsection-title:has-text("账号列表")', { timeout: 5000 });

  // 点击添加账号按钮
  await page.click('button:has-text("+ 添加账号")');

  const dialog = page.locator('.el-dialog');
  await expect(dialog).toBeVisible();

  // 填写表单 - 固定账号 ID 避免冲突
  const accountIdInput = page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input');
  await accountIdInput.fill('test-account-001');

  const nameInput = page.locator('.el-form-item', { hasText: '显示名称' }).locator('input');
  await nameInput.fill(testData.channel.mattermost.name);

  const tokenInput = page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input');
  await tokenInput.fill(testData.channel.mattermost.token);

  const urlInput = page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input');
  await urlInput.fill(testData.channel.mattermost.url);

  // 强制触发 Vue 绑定更新(Element Plus v-model 在 fill 后可能延迟)
  await accountIdInput.dispatchEvent('input');
  await nameInput.dispatchEvent('input');
  await tokenInput.dispatchEvent('input');
  await urlInput.dispatchEvent('input');
  await page.waitForTimeout(200);

  // 保存 - 关键验证点:页面不卡死,5秒内对话框必须关闭
  await page.click('.el-dialog__footer button:has-text("保存")');

  // 等待对话框关闭(证明 save 没有卡死)
  await expect(dialog).not.toBeVisible({ timeout: 5000 });

  // 注意:这里不验证账号出现在列表中,因为那属于"账号 CRUD 列表"测试的范围
  // 当前测试只验证 saveMattermostAccount 函数不卡死
});
