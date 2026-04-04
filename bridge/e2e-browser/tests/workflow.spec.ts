/**
 * 用户工作流 Browser 测试
 * 测试矩阵功能模块 #5 (高优先级)
 * 
 * 测试完整的用户配置流程
 */

import { test, expect } from '../fixtures';

test.describe('User Workflows', () => {
  test('配置页面导航 - 所有模块可切换', async ({ page }) => {
    await page.goto('/config/provider');
    await page.waitForLoadState('networkidle');
    
    // 验证所有侧边栏菜单可以正常切换
    const modules = [
      { name: 'Provider', icon: '🧠' },
      { name: 'Agent', icon: '🤖' },
      { name: 'Memory', icon: '💾' },
      { name: 'Channel', icon: '📱' },
    ];
    
    for (const mod of modules) {
      await page.locator('.menu-item', { hasText: mod.name }).click();
      await page.waitForTimeout(300);
      
      // 验证菜单被激活
      const activeMenu = page.locator('.menu-item.active', { hasText: mod.name });
      await expect(activeMenu).toBeVisible();
      
      // 验证 URL 变化
      await expect(page).toHaveURL(new RegExp(`/config/${mod.name.toLowerCase()}`));
    }
  });

  test('首页到配置的跳转', async ({ page }) => {
    // 访问首页
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // 验证首页加载成功
    await expect(page.locator('body')).toBeVisible();
  });

  test('Channel 配置 - 关键 Bug 验证流程', async ({ page }) => {
    await page.goto('/config/channel');
    await page.waitForLoadState('networkidle');
    
    // 启用 Mattermost（如果未启用）
    const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
    const switch_ = mattermostSection.locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked().catch(() => false);
    if (!isChecked) {
      await switch_.click();
      await page.waitForTimeout(500);
    }
    
    // 等待添加账号按钮出现
    await expect(page.locator('button:has-text("+ 添加账号")')).toBeVisible();
    
    // 点击添加账号
    await page.click('button:has-text("+ 添加账号")');
    
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 填写表单
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill('test-bug-fix');
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill('Bug Fix Test');
    await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('test-token-123');
    await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://test.example.com');
    
    // 保存 - 关键验证：不卡死
    await page.click('.el-dialog__footer button:has-text("保存")');
    
    // 验证对话框关闭（没有卡死）
    await expect(dialog).not.toBeVisible({ timeout: 5000 });
    
    // 验证账号出现在列表
    await expect(page.locator('.account-name', { hasText: 'Bug Fix Test' })).toBeVisible();
  });

  test('Provider 页面 - 添加实例对话框流程', async ({ page }) => {
    await page.goto('/config/provider');
    await page.waitForLoadState('networkidle');
    
    // 点击添加实例
    await page.locator('button:has-text("+ 添加实例")').first().click();
    
    // 验证对话框打开
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 取消
    await page.click('.el-dialog__footer button:has-text("取消")');
    await expect(dialog).not.toBeVisible({ timeout: 3000 });
  });
});
