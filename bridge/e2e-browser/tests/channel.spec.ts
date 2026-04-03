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

  test('添加 Mattermost 账号 - 关键验证：不卡死', async ({ page, channelPage }) => {
    const data = testData.channel.mattermost;
    
    // 添加 Channel - 关键验证点：3秒内完成，不卡死
    await channelPage.addChannel(data);
    
    // 验证保存成功提示
    await channelPage.waitForToast('保存成功');
    
    // 验证账号出现在列表中
    await channelPage.verifyChannelExists(data.name);
  });

  test('添加 Telegram 账号', async ({ page, channelPage }) => {
    const data = testData.channel.telegram;
    
    await channelPage.addChannel(data);
    await channelPage.waitForToast('保存成功');
    await channelPage.verifyChannelExists(data.name);
  });

  test('删除 Channel', async ({ page, channelPage }) => {
    // 先添加一个 Channel
    const data = testData.channel.mattermost;
    await channelPage.addChannel(data);
    await channelPage.waitForToast('保存成功');
    
    // 删除它
    await channelPage.deleteChannel(data.name);
    await channelPage.waitForToast('删除成功');
    
    // 验证已删除
    await expect(page.locator('text=' + data.name)).not.toBeVisible();
  });

  test('编辑 Channel', async ({ page, channelPage }) => {
    // 先添加一个 Channel
    const data = testData.channel.mattermost;
    await channelPage.addChannel(data);
    await channelPage.waitForToast('保存成功');
    
    // 点击编辑按钮
    const row = page.locator('.el-table__row', { hasText: data.name });
    await row.locator('button:has-text("编辑")').click();
    
    // 修改名称
    const newName = data.name + ' - Edited';
    await page.fill('input[placeholder*="Bot"]', newName);
    
    // 保存 - 同样验证不卡死
    await page.click('button:has-text("保存")');
    await expect(page.locator('.el-dialog')).not.toBeVisible({ timeout: 5000 });
    
    await channelPage.waitForToast('保存成功');
    await channelPage.verifyChannelExists(newName);
  });

  test('表单验证 - Token 必填', async ({ page }) => {
    await page.click('button:has-text("+ 添加账号")');
    
    // 只填写部分字段
    await page.fill('input[placeholder*="default"]', 'test-id');
    await page.fill('input[placeholder*="Bot"]', 'Test Bot');
    // 不填 Token
    
    await page.click('button:has-text("保存")');
    
    // 验证验证错误
    await expect(page.locator('.el-form-item__error')).toBeVisible();
  });

  test('批量添加多个 Channel', async ({ page, channelPage }) => {
    const channels = [
      testData.channel.mattermost,
      testData.channel.telegram,
    ];
    
    for (const data of channels) {
      await channelPage.addChannel(data);
      await channelPage.waitForToast('保存成功');
    }
    
    // 验证所有 Channel 都在列表中
    for (const data of channels) {
      await channelPage.verifyChannelExists(data.name);
    }
  });
});
