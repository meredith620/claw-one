/**
 * Agent CRUD Browser 测试
 * 测试矩阵功能模块 #2
 */

import { test, expect, testData } from '../fixtures';

test.describe('Agent CRUD', () => {
  test.beforeEach(async ({ agentPage }) => {
    await agentPage.goto();
  });

  test('添加 Agent', async ({ page, agentPage }) => {
    const data = testData.agent.testAgent;
    
    await agentPage.addAgent(data);
    
    await agentPage.waitForToast('保存成功');
    await agentPage.verifyAgentExists(data.name);
  });

  test('Agent 表单完整填写', async ({ page }) => {
    await page.click('button:has-text("添加 Agent")');
    
    const dialog = page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 填写所有字段
    await page.fill('input[placeholder*="ID"]', 'full-test-agent');
    await page.fill('input[placeholder*="名称"]', 'Full Test Agent');
    
    // 选择 Provider
    await page.click('.el-select');
    await page.click('.el-select-dropdown__item', { hasText: /openai|anthropic/i });
    
    await page.fill('input[placeholder*="模型"]', 'gpt-4-turbo');
    await page.fill('textarea', '你是一个专业的助手，请用中文回复。');
    
    await page.click('.el-dialog__footer button:has-text("保存")');
    await agentPage.waitForToast('保存成功');
  });

  test('Agent 列表显示', async ({ page, agentPage }) => {
    // 添加测试数据
    await agentPage.addAgent(testData.agent.testAgent);
    await agentPage.waitForToast('保存成功');
    
    // 验证表格列显示
    const table = page.locator('.el-table');
    await expect(table).toBeVisible();
    
    // 验证列头
    await expect(table.locator('th', { hasText: 'ID' })).toBeVisible();
    await expect(table.locator('th', { hasText: '名称' })).toBeVisible();
    await expect(table.locator('th', { hasText: 'Provider' })).toBeVisible();
    await expect(table.locator('th', { hasText: '模型' })).toBeVisible();
  });
});
