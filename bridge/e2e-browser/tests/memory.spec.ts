/**
 * Memory 配置 Browser 测试
 * 测试矩阵功能模块 #4
 */

import { test, expect, testData } from '../fixtures';

test.describe('Memory Configuration', () => {
  test.beforeEach(async ({ memoryPage }) => {
    await memoryPage.goto();
  });

  test('添加 File Memory', async ({ page, memoryPage }) => {
    const data = testData.memory.testMemory;
    
    await memoryPage.addMemory(data);
    
    await memoryPage.waitForToast('保存成功');
    await memoryPage.verifyMemoryExists(data.name);
  });

  test('Memory 类型选择', async ({ page }) => {
    await page.click('button:has-text("添加 Memory")');
    
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 点击类型下拉
    await page.click('.el-select');
    
    // 验证选项存在
    const dropdown = page.locator('.el-select-dropdown');
    await expect(dropdown).toBeVisible();
    
    // 应该有 file/memory 等选项
    const options = await page.locator('.el-select-dropdown__item').allTextContents();
    expect(options.length).toBeGreaterThan(0);
  });

  test('Memory 配置持久化', async ({ page, memoryPage }) => {
    // 添加 Memory
    const data = testData.memory.testMemory;
    await memoryPage.addMemory(data);
    await memoryPage.waitForToast('保存成功');
    
    // 刷新页面
    await page.reload();
    await page.waitForLoadState('networkidle');
    await memoryPage.clickTab('Memory');
    
    // 验证数据仍然存在
    await memoryPage.verifyMemoryExists(data.name);
  });
});
