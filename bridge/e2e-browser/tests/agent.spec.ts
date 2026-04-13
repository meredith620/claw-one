/**
 * Agent CRUD Browser 测试
 * 测试矩阵功能模块 #2
 *
 * Layer 4 测试目标: 验证从前端 UI 到后端 API 的完整链路
 *
 * 注意: /api/agents 返回模型别名，agent 配置通过 /api/config 验证
 */

import { test, expect, ConfigVerifier } from '../fixtures';

const API_BASE = process.env.CLAW_ONE_URL || 'http://claw-one-test-app:8080';

/**
 * 通过 API 获取完整配置
 */
async function getConfig(): Promise<any> {
  const response = await fetch(`${API_BASE}/api/config`);
  if (!response.ok) throw new Error(`API failed: ${response.status}`);
  return response.json();
}

test.describe('Agent Configuration', () => {
  test.beforeEach(async ({ agentPage }) => {
    await agentPage.goto();
  });

  test('Agent 页面加载正常', async ({ page }) => {
    await expect(page.locator('.agent-section', { hasText: 'Agent 模式' })).toBeVisible();
    await expect(page.locator('.el-radio-button', { hasText: '单 Agent 模式' })).toBeVisible();
    await expect(page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' })).toBeVisible();
  });

  test('切换到 Multi-Agent 模式并添加 Agent - 完整链路验证', async ({ page }) => {
    const agentId = `e2e-agent-${Date.now()}`;
    const agentName = `E2E Agent ${Date.now()}`;
    
    try {
      // 1. 切换到 Multi-Agent 模式
      await page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' }).click();
      await page.waitForTimeout(300);
      
      await expect(page.locator('button:has-text("+ 添加 Agent")')).toBeVisible();
      
      // 2. 打开添加对话框
      await page.click('button:has-text("+ 添加 Agent")');
      
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();
      await expect(page.locator('.el-form-item', { hasText: 'Agent ID' })).toBeVisible();
      await expect(page.locator('.el-form-item', { hasText: '显示名称' })).toBeVisible();
      
      // 3. 填写表单
      await dialog.locator('.el-form-item', { hasText: 'Agent ID' }).locator('input').fill(agentId);
      await dialog.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(agentName);
      
      // 4. 保存
      await dialog.locator('.el-dialog__footer button:has-text("保存")').click();
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      
      // 5. 验证 UI 显示新 Agent
      await expect(page.locator('.agent-name', { hasText: agentName })).toBeVisible({ timeout: 5000 });
      
      // 6. API 详细验证（完整链路验证 - Issue 2）
      const config = await getConfig();
      const agentsConfig = config.agents;
      console.log('[Agent] API agents 配置 keys:', Object.keys(agentsConfig || {}));
      
      // 核心断言：精确验证新 agentId 在 config.agents.list 中
      expect(agentsConfig).toBeTruthy();
      const agentList = agentsConfig.list || [];
      const foundAgent = agentList.find((a: any) => a.id === agentId);
      expect(foundAgent).toBeTruthy();
      
      // 验证 agent 名称字段
      expect(foundAgent.name).toBe(agentName);
      console.log('[Agent] API 验证通过：agentId 和 name 字段正确');
      
      // 文件层验证（ConfigVerifier 集成 - P1）
      const inFile = await ConfigVerifier.verifyAgentExists(agentId, { name: agentName });
      expect(inFile).toBeTruthy();
      console.log('[Agent] ConfigVerifier 文件验证通过：agent name 字段匹配');
    } finally {
      // 清理
      const config = await getConfig();
      const agentList = config.agents?.list || [];
      if (agentList.find((a: any) => a.id === agentId)) {
        await page.request.delete(`${API_BASE}/api/agents/${agentId}`);
        console.log('[Agent] 清理完成:', agentId);
      }
    }
  });

  test('删除 Agent - 验证 UI 删除后 API 和文件层数据已移除', async ({ page }) => {
    const agentId = `e2e-agent-del-${Date.now()}`;
    const agentName = `E2E Agent Del ${Date.now()}`;
    
    try {
      // 1. 切换到 Multi-Agent 模式
      await page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' }).click();
      await page.waitForTimeout(300);
      
      // 2. 添加 Agent
      await page.click('button:has-text("+ 添加 Agent")');
      const dialog = page.locator('.el-dialog');
      await expect(dialog).toBeVisible();
      
      await dialog.locator('.el-form-item', { hasText: 'Agent ID' }).locator('input').fill(agentId);
      await dialog.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(agentName);
      
      await dialog.locator('.el-dialog__footer button:has-text("保存")').click();
      await expect(dialog).not.toBeVisible({ timeout: 5000 });
      
      // 3. 验证 UI 显示新 Agent
      await expect(page.locator('.agent-name', { hasText: agentName })).toBeVisible({ timeout: 5000 });
      
      // 4. API 验证添加成功
      let config = await getConfig();
      const agentListAdd = config.agents?.list || [];
      const foundAgentAdd = agentListAdd.find((a: any) => a.id === agentId);
      expect(foundAgentAdd).toBeTruthy();
      expect(foundAgentAdd.name).toBe(agentName);
      
      // 5. 点击删除按钮
      const agentCard = page.locator('.agent-card, .agent-item')
        .filter({ hasText: agentName })
        .first();
      
      // 在点击删除之前注册 dialog 监听器（避免竞态）
      const dialogPromise = page.waitForSelector('.el-message-box', { timeout: 3000 });
      await agentCard.locator('button:has-text("删除")').click();
      
      // 等待确认对话框出现
      await dialogPromise;
      
      // 点击确定按钮
      await page.locator('.el-message-box__wrapper button, .el-message-box button')
        .filter({ hasText: '确定' })
        .click();
      
      await page.waitForTimeout(1000);
      
      // 6. UI 验证：Agent 名称不再显示
      await expect(page.locator('.agent-name', { hasText: agentName })).not.toBeVisible({ timeout: 5000 });
      
      // 7. API 验证：UI 删除后 API 返回数据已移除
      config = await getConfig();
      const agentListDel = config.agents?.list || [];
      const agentDeleted = !agentListDel.find((a: any) => a.id === agentId);
      expect(agentDeleted).toBeTruthy();
      console.log('[Agent Delete] API 验证通过：agent 已从后端移除');
      
      // 8. 文件层验证：Agent 已从 openclaw.json 移除
      const inFile = await ConfigVerifier.verifyAgentExists(agentId);
      expect(!inFile).toBeTruthy();
      console.log('[Agent Delete] ConfigVerifier 文件验证通过：agent 已从 openclaw.json 移除');
    } finally {
      // 确保清理
      const config = await getConfig();
      const agentListCleanup = config.agents?.list || [];
      if (agentListCleanup.find((a: any) => a.id === agentId)) {
        // 使用 fetch 而非 page.request
        await fetch(`${API_BASE}/api/agents/${agentId}`, { method: 'DELETE' }).catch(() => {});
      }
    }
  });

  test('保存 Agent 配置按钮存在', async ({ page }) => {
    await expect(page.locator('button:has-text("保存 Agent 配置")')).toBeVisible();
  });
});
