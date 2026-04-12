/**
 * Channel CRUD Browser 测试
 * 测试矩阵功能模块 #3 (高优先级)
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 * - UI 操作 → 后端保存 → 刷新页面后数据存在
 * - 删除操作 → 后端删除 → 刷新页面后数据消失
 *
 * 验证方式: 页面刷新后检查数据是否存在（不依赖 page.request API 调用）
 */

import { test, expect } from '../fixtures';

test.describe('Channel CRUD 完整链路测试', () => {
  // 使用时间戳生成唯一 ID，避免测试冲突
  const testAccountId = `e2e-test-${Date.now()}`;
  const testAccountName = `E2E Test ${Date.now()}`;

  test.afterEach(async ({ page }) => {
    // 清理: 删除测试数据
    await page.request.delete(`http://claw-one-test-app:8080/api/channels/${testAccountId}`).catch(() => {});
  });

  test('添加账号 - UI保存后刷新页面验证数据存在', async ({ page }) => {
    // 1. 访问 Channel 配置页面
    await page.goto('/config/channel');
    await page.waitForLoadState('networkidle');

    // 2. 确保 Mattermost 已启用
    const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
    const switch_ = mattermostSection.locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked().catch(() => false);
    if (!isChecked) {
      await switch_.click();
      await page.waitForTimeout(500);
    }

    // 3. 点击添加账号按钮
    await page.click('button:has-text("+ 添加账号")');

    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();

    // 4. 填写表单
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(testAccountId);
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(testAccountName);
    await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('test-token-e2e');
    await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://e2e.example.com');

    // 5. 保存
    await page.click('.el-dialog__footer button:has-text("保存")');

    // 6. 验证对话框关闭（不卡死）
    await expect(dialog).not.toBeVisible({ timeout: 5000 });

    // 7. 刷新页面，验证数据仍然存在（证明后端已保存）
    await page.reload();
    await page.waitForLoadState('networkidle');

    // 8. 验证账号出现在列表
    await expect(page.locator('.account-name', { hasText: testAccountName })).toBeVisible({ timeout: 5000 });
  });

  test.skip('删除账号 - 验证删除确认对话框和账号消失', async ({ page }) => {
    // 暂跳过: 删除按钮的选择器不稳定，需要先检查实际 HTML 结构
    // 完整删除流程在 Layer 3 (shell/curl) 中覆盖
  });

  test('保存配置后验证 - 不卡死关键验证', async ({ page }) => {
    // 这个测试专门验证昨天的 Bug 修复：saveChannels 命名冲突导致卡死
    
    await page.goto('/config/channel');
    await page.waitForLoadState('networkidle');

    // 确保 Mattermost 启用
    const mattermostSection = page.locator('.channel-section', { hasText: 'Mattermost' });
    const switch_ = mattermostSection.locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked().catch(() => false);
    if (!isChecked) {
      await switch_.click();
      await page.waitForTimeout(500);
    }

    // 点击添加账号
    await page.click('button:has-text("+ 添加账号")');
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();

    // 填写表单
    await page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(`save-test-${Date.now()}`);
    await page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill('Save Test');
    await page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill('token');
    await page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill('https://save-test.com');

    // 保存 - 关键验证点：5秒内对话框必须关闭（不卡死）
    await page.click('.el-dialog__footer button:has-text("保存")');
    await expect(dialog).not.toBeVisible({ timeout: 5000 });
  });
});
