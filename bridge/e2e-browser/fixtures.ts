/**
 * 测试夹具和工具函数
 * 用于 Playwright 前端 E2E 测试
 */

import { test as base, expect, Page } from '@playwright/test';

// 测试数据生成器
export const testData = {
  provider: {
    openai: {
      id: `test-openai-${Date.now()}`,
      name: 'Test OpenAI Provider',
      apiKey: 'sk-test-key-12345',
      baseUrl: 'https://api.openai.com/v1',
    },
    anthropic: {
      id: `test-claude-${Date.now()}`,
      name: 'Test Claude Provider',
      apiKey: 'sk-ant-test-key',
      baseUrl: 'https://api.anthropic.com',
    },
  },
  channel: {
    mattermost: {
      id: `test-mm-${Date.now()}`,
      name: 'Test Mattermost Bot',
      token: 'test-token-12345',
      url: 'https://mm.example.com',
    },
    telegram: {
      id: `test-tg-${Date.now()}`,
      name: 'Test Telegram Bot',
      token: '123456789:test-token',
      url: '',
    },
  },
  agent: {
    testAgent: {
      id: `test-agent-${Date.now()}`,
      name: 'Test Agent',
      provider: 'openai',
      model: 'gpt-4',
      systemPrompt: 'You are a helpful assistant.',
    },
  },
  memory: {
    testMemory: {
      id: `test-memory-${Date.now()}`,
      name: 'Test Memory Store',
      type: 'file',
      path: '/tmp/test-memory',
    },
  },
};

// 页面操作封装
export class ConfigPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto('/config');
    await this.page.waitForLoadState('networkidle');
  }

  async waitForToast(message: string) {
    await expect(this.page.locator('.el-message')).toContainText(message);
  }

  async clickTab(tabName: string) {
    await this.page.locator('.el-tabs__item', { hasText: tabName }).click();
    await this.page.waitForTimeout(300); // 等待 Tab 切换动画
  }
}

// Provider 配置页
export class ProviderPage extends ConfigPage {
  async goto() {
    await super.goto();
    await this.clickTab('Provider');
  }

  async addProvider(data: typeof testData.provider.openai) {
    await this.page.click('button:has-text("添加 Provider")');
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    await this.page.fill('input[placeholder*="ID"]', data.id);
    await this.page.fill('input[placeholder*="名称"]', data.name);
    await this.page.fill('input[type="password"]', data.apiKey);
    await this.page.fill('input[placeholder*="URL"]', data.baseUrl);
    
    await this.page.click('.el-dialog__footer button:has-text("保存")');
  }

  async verifyProviderExists(name: string) {
    await expect(this.page.locator('text=' + name).first()).toBeVisible();
  }

  async deleteProvider(name: string) {
    const row = this.page.locator('.el-table__row', { hasText: name });
    await row.locator('button:has-text("删除")').click();
    await this.page.click('.el-message-box__btns button:has-text("确定")');
  }
}

// Channel 配置页
export class ChannelPage extends ConfigPage {
  async goto() {
    await super.goto();
    await this.clickTab('Channel');
  }

  async addChannel(data: typeof testData.channel.mattermost) {
    // 点击添加账号按钮
    await this.page.click('button:has-text("+ 添加账号")');
    
    // 填写表单
    await this.page.fill('input[placeholder*="default"]', data.id);
    await this.page.fill('input[placeholder*="Bot"]', data.name);
    await this.page.fill('input[type="password"]', data.token);
    await this.page.fill('input[placeholder*="https"]', data.url);
    
    // 保存 - 关键验证点：页面不卡死，3秒内返回
    const saveButton = this.page.locator('button:has-text("保存")');
    await saveButton.click();
    
    // 等待对话框关闭（证明没有卡死）
    await expect(this.page.locator('.el-dialog')).not.toBeVisible({ timeout: 5000 });
  }

  async verifyChannelExists(name: string) {
    await expect(this.page.locator('text=' + name).first()).toBeVisible();
  }

  async deleteChannel(name: string) {
    const row = this.page.locator('.el-table__row', { hasText: name });
    await row.locator('button:has-text("删除")').click();
    await this.page.click('.el-message-box__btns button:has-text("确定")');
  }
}

// Agent 配置页
export class AgentPage extends ConfigPage {
  async goto() {
    await super.goto();
    await this.clickTab('Agent');
  }

  async addAgent(data: typeof testData.agent.testAgent) {
    await this.page.click('button:has-text("添加 Agent")');
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    await this.page.fill('input[placeholder*="ID"]', data.id);
    await this.page.fill('input[placeholder*="名称"]', data.name);
    
    // 选择 Provider
    await this.page.click('.el-select');
    await this.page.click(`.el-select-dropdown__item:has-text("${data.provider}")`);
    
    // 填写模型
    await this.page.fill('input[placeholder*="模型"]', data.model);
    
    // 填写系统提示词
    await this.page.fill('textarea', data.systemPrompt);
    
    await this.page.click('.el-dialog__footer button:has-text("保存")');
  }

  async verifyAgentExists(name: string) {
    await expect(this.page.locator('text=' + name).first()).toBeVisible();
  }
}

// Memory 配置页
export class MemoryPage extends ConfigPage {
  async goto() {
    await super.goto();
    await this.clickTab('Memory');
  }

  async addMemory(data: typeof testData.memory.testMemory) {
    await this.page.click('button:has-text("添加 Memory")');
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    await this.page.fill('input[placeholder*="ID"]', data.id);
    await this.page.fill('input[placeholder*="名称"]', data.name);
    
    // 选择类型
    await this.page.click('.el-select');
    await this.page.click(`.el-select-dropdown__item:has-text("${data.type}")`);
    
    await this.page.fill('input[placeholder*="路径"]', data.path);
    
    await this.page.click('.el-dialog__footer button:has-text("保存")');
  }

  async verifyMemoryExists(name: string) {
    await expect(this.page.locator('text=' + name).first()).toBeVisible();
  }
}

// 扩展基础测试
export const test = base.extend<{
  providerPage: ProviderPage;
  channelPage: ChannelPage;
  agentPage: AgentPage;
  memoryPage: MemoryPage;
}>({
  providerPage: async ({ page }, use) => {
    await use(new ProviderPage(page));
  },
  channelPage: async ({ page }, use) => {
    await use(new ChannelPage(page));
  },
  agentPage: async ({ page }, use) => {
    await use(new AgentPage(page));
  },
  memoryPage: async ({ page }, use) => {
    await use(new MemoryPage(page));
  },
});

export { expect };
