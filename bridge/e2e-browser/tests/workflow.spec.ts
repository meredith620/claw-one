/**
 * 用户工作流 Browser 测试
 * 测试矩阵功能模块 #5 (高优先级)
 * 
 * 测试完整的用户配置流程
 */

import { test, expect, testData } from '../fixtures';

test.describe('User Workflows', () => {
  test('完整配置流程：Provider → Channel → Agent', async ({ page, providerPage, channelPage, agentPage }) => {
    // Step 1: 配置 Provider
    await providerPage.goto();
    await providerPage.addProvider(testData.provider.openai);
    await providerPage.waitForToast('保存成功');
    await providerPage.verifyProviderExists(testData.provider.openai.name);
    
    // Step 2: 配置 Channel
    await channelPage.goto();
    await channelPage.addChannel(testData.channel.mattermost);
    await channelPage.waitForToast('保存成功');
    await channelPage.verifyChannelExists(testData.channel.mattermost.name);
    
    // Step 3: 配置 Agent
    await agentPage.goto();
    await agentPage.addAgent(testData.agent.testAgent);
    await agentPage.waitForToast('保存成功');
    await agentPage.verifyAgentExists(testData.agent.testAgent.name);
    
    // Step 4: 验证导航和状态页面
    await page.click('a[href="/status"]');
    await page.waitForLoadState('networkidle');
    
    // 状态页面应该显示配置概览
    await expect(page.locator('h1, h2, h3', { hasText: /状态|Status|概览|Overview/i })).toBeVisible();
  });

  test('配置页面导航', async ({ page }) => {
    // 访问配置页面
    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    
    // 验证所有 Tab 可以正常切换
    const tabs = ['Provider', 'Channel', 'Agent', 'Memory'];
    
    for (const tabName of tabs) {
      await page.click(`.el-tabs__item:has-text("${tabName}")`);
      await page.waitForTimeout(300);
      
      // 验证 Tab 被激活
      const activeTab = page.locator('.el-tabs__item.is-active', { hasText: tabName });
      await expect(activeTab).toBeVisible();
    }
  });

  test('首页到配置的跳转', async ({ page }) => {
    // 访问首页
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // 点击配置链接/按钮
    const configLink = page.locator('a[href="/config"], button:has-text("配置")').first();
    if (await configLink.isVisible().catch(() => false)) {
      await configLink.click();
      await page.waitForURL('**/config');
      
      // 验证到达配置页面
      await expect(page.locator('.el-tabs')).toBeVisible();
    }
  });

  test('配置变更实时预览', async ({ page, channelPage }) => {
    await channelPage.goto();
    
    // 添加一个 Channel
    const data = testData.channel.mattermost;
    await channelPage.addChannel(data);
    await channelPage.waitForToast('保存成功');
    
    // 验证实时显示在列表中
    const row = page.locator('.el-table__row', { hasText: data.name });
    await expect(row).toBeVisible();
    
    // 验证 ID 和名称都正确显示
    await expect(row.locator('text=' + data.id)).toBeVisible();
    await expect(row.locator('text=' + data.name)).toBeVisible();
  });

  test('配置导出功能检查', async ({ page }) => {
    await page.goto('/config');
    await page.waitForLoadState('networkidle');
    
    // 查找导出按钮
    const exportButton = page.locator('button:has-text("导出"), button:has-text("Export")').first();
    
    if (await exportButton.isVisible().catch(() => false)) {
      // 设置下载监听
      const [download] = await Promise.all([
        page.waitForEvent('download'),
        exportButton.click(),
      ]);
      
      // 验证下载了文件
      expect(download.suggestedFilename()).toContain('.json');
    }
  });
});
