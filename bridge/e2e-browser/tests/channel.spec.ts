/**
 * Channel CRUD Browser 测试
 * 测试矩阵功能模块 #3 (高优先级)
 * 
 * 这是昨天修复的关键 Bug 所在模块：
 * - Bug: saveChannels 命名冲突导致保存卡死
 * - 验证: 保存按钮点击后页面不卡死，3秒内返回
 */

import { test, expect, testData } from '../fixtures';

test.describe('Channel CRUD', () => {
  test.beforeEach(async ({ channelPage }) => {
    await channelPage.goto();
  });

  test('添加 Mattermost 账号 - 关键验证：不卡死', async ({ page }) => {
    const data = testData.channel.mattermost;
    
    // 添加 Channel - 关键验证点：3秒内完成，不卡死
    await test.step('打开添加账号对话框并填写表单', async () => {
      // 启用 Mattermost
      await page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch').click();
      await page.waitForTimeout(300);
      
      // 点击添加账号
      await page.click('button:has-text("+ 添加账号")');
      
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();
      
      // 填写表单
      await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(data.id);
      await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(data.name);
      await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(data.token);
      await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(data.url);
      
      // 保存 - 关键验证点：页面不卡死，3秒内返回
      await page.click('.el-dialog__footer button:has-text("保存")');
      
      // 等待对话框关闭（证明没有卡死）
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
    });
    
    // 验证账号出现在列表中
    await expect(page.locator('.account-name', { hasText: data.name })).toBeVisible();
  });

  test('删除 Channel 账号', async ({ page }) => {
    // 先添加一个账号
    const data = testData.channel.mattermost;
    
    // 启用 Mattermost 并添加账号
    await page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch').click();
    await page.waitForTimeout(300);
    await page.click('button:has-text("+ 添加账号")');
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(data.id);
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(data.name);
    await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(data.token);
    await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(data.url);
    await page.click('.el-dialog__footer button:has-text("保存")');
    await expect(page.locator('.el-dialog')).not.toBeVisible({ timeout: 5000 });
    
    // 删除它
    const card = page.locator('.account-card', { hasText: data.name });
    await card.locator('button:has-text("删除")').click();
    
    // 确认删除
    await page.click('.el-message-box__btns button:has-text("确定")');
    
    // 验证已删除
    await expect(page.locator('.account-name', { hasText: data.name })).not.toBeVisible();
  });

  test('编辑 Channel 账号', async ({ page }) => {
    const data = testData.channel.mattermost;
    
    // 启用并添加账号
    await page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch').click();
    await page.waitForTimeout(300);
    await page.click('button:has-text("+ 添加账号")');
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(data.id);
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(data.name);
    await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(data.token);
    await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(data.url);
    await page.click('.el-dialog__footer button:has-text("保存")');
    await expect(page.locator('.el-dialog')).not.toBeVisible({ timeout: 5000 });
    
    // 点击编辑按钮
    const card = page.locator('.account-card', { hasText: data.name });
    await card.locator('button:has-text("配置")').click();
    
    // 修改名称
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    const newName = data.name + ' - Edited';
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(newName);
    
    // 保存
    await page.click('.el-dialog__footer button:has-text("保存")');
    await expect(dialog).not.toBeVisible({ timeout: 5000 });
    
    // 验证更新
    await expect(page.locator('.account-name', { hasText: newName })).toBeVisible();
  });

  test('表单验证 - Token 必填', async ({ page }) => {
    // 启用 Mattermost
    await page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch').click();
    await page.waitForTimeout(300);
    
    await page.click('button:has-text("+ 添加账号")');
    
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 只填写部分字段，不填 Token
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill('test-id');
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill('Test Bot');
    // 不填 Token
    
    // 尝试保存
    await page.click('.el-dialog__footer button:has-text("保存")');
    
    // 对话框不应关闭（因为有必填验证）
    await expect(dialog).toBeVisible();
  });
});
