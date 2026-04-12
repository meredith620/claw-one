/**
 * 测试夹具和工具函数
 * 用于 Playwright 前端 E2E 测试
 */

import { test as base, expect, Page } from '@playwright/test';

// 测试数据生成器
export const testData = {
  provider: {
    openai: {
      name: 'test-openai',
      apiKey: 'sk-test-key-12345',
    },
    anthropic: {
      name: 'test-claude',
      apiKey: 'sk-ant-test-key',
    },
  },
  channel: {
    mattermost: {
      id: `test-mm-${Date.now()}`,
      name: 'Test Mattermost Bot',
      token: 'test-token-12345',
      url: 'https://mm.example.com',
    },
  },
  agent: {
    testAgent: {
      id: `test-agent-${Date.now()}`,
      name: 'Test Agent',
      workspace: '',
      agentDir: '',
    },
  },
};

// 页面操作封装
export class ConfigPage {
  constructor(protected page: Page) {}

  async goto() {
    await this.page.goto('/config');
    await this.page.waitForLoadState('networkidle');
  }

  async waitForToast(message: string, timeout = 5000) {
    await expect(this.page.locator('.el-message').last()).toContainText(message, { timeout });
  }

  async navigateTo(module: string) {
    // 使用侧边栏导航
    await this.page.locator('.menu-item', { hasText: module }).click();
    await this.page.waitForTimeout(500);
  }
}

// Provider 配置页
export class ProviderPage extends ConfigPage {
  async goto() {
    await this.page.goto('/config/provider');
    await this.page.waitForLoadState('networkidle');
  }

  async addProvider(data: typeof testData.provider.openai, type = 'openai') {
    // 点击对应 Provider 类型的添加实例按钮
    const typeSection = this.page.locator('.provider-section, .section-card', { hasText: type });
    await typeSection.locator('button:has-text("+ 添加实例")').click();
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 等待表单渲染
    await this.page.waitForTimeout(500);
    
    // 填写实例名称 - 更精确的选择器
    const nameInput = dialog.locator('.el-form-item').filter({ hasText: '实例名称' }).locator('input').first();
    await nameInput.fill(data.name);
    
    // 填写 API Key (password 类型)
    const apiKeyInput = dialog.locator('input[type="password"]').first();
    await apiKeyInput.fill(data.apiKey);
    
    // 填写默认模型 (必填字段) - 点击选择器本身而不是输入框
    const modelSelect = dialog.locator('.el-form-item').filter({ hasText: '默认模型' }).locator('.el-select, [class*="select"]').first();
    await modelSelect.click();
    await this.page.waitForTimeout(300);
    
    // 选择第一个选项
    const firstOption = this.page.locator('.el-select-dropdown__item:visible').first();
    if (await firstOption.isVisible({ timeout: 2000 }).catch(() => false)) {
      await firstOption.click();
    }
    await this.page.waitForTimeout(200);
    
    // 点击保存
    const saveButton = dialog.locator('.el-dialog__footer button:has-text("保存")');
    await saveButton.click();
    
    // 等待对话框关闭 - 增加超时时间
    await expect(dialog).not.toBeVisible({ timeout: 10000 });
  }

  async verifyProviderExists(name: string) {
    await expect(this.page.locator('.instance-card', { hasText: name }).first()).toBeVisible();
  }

  async deleteProvider(name: string) {
    const card = this.page.locator('.instance-card', { hasText: name });
    await card.locator('button:has-text("删除")').click();
    
    // 确认删除对话框
    await this.page.click('.el-message-box__btns button:has-text("确定")');
  }
}

// Channel 配置页
export class ChannelPage extends ConfigPage {
  async goto() {
    await this.page.goto('/config/channel');
    await this.page.waitForLoadState('networkidle');
  }

  async enableMattermost() {
    // 先启用 Mattermost 开关
    const switch_ = this.page.locator('.channel-section', { hasText: 'Mattermost' }).locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked();
    if (!isChecked) {
      await switch_.click();
      await this.page.waitForTimeout(300);
    }
  }

  async addChannel(data: typeof testData.channel.mattermost) {
    // 确保 Mattermost 已启用
    await this.enableMattermost();
    
    // 点击添加账号按钮
    await this.page.click('button:has-text("+ 添加账号")');
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 填写表单 - 使用 label 定位
    await this.page.locator('.el-form-item', { hasText: '账号 ID' }).locator('input').fill(data.id);
    await this.page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(data.name);
    await this.page.locator('.el-form-item', { hasText: 'Bot Token' }).locator('input').fill(data.token);
    await this.page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(data.url);
    
    // 保存 - 关键验证点：页面不卡死，3秒内返回
    await this.page.click('.el-dialog__footer button:has-text("保存")');
    
    // 等待对话框关闭（证明没有卡死）
    await expect(dialog).not.toBeVisible({ timeout: 5000 });
  }

  async verifyChannelExists(name: string) {
    await expect(this.page.locator('.account-name', { hasText: name }).first()).toBeVisible();
  }

  async deleteChannel(name: string) {
    const card = this.page.locator('.account-card', { hasText: name });
    await card.locator('button:has-text("删除")').click();
    
    // 确认删除对话框
    await this.page.click('.el-message-box__btns button:has-text("确定")');
  }
}

// Agent 配置页 (Multi-Agent 模式)
export class AgentPage extends ConfigPage {
  async goto() {
    await this.page.goto('/config/agent');
    await this.page.waitForLoadState('networkidle');
  }

  async switchToMultiAgentMode() {
    // 切换到 Multi-Agent 模式
    await this.page.locator('.el-radio-button', { hasText: 'Multi-Agent 模式' }).click();
    await this.page.waitForTimeout(300);
  }

  async addAgent(data: typeof testData.agent.testAgent) {
    // 确保在 Multi-Agent 模式
    await this.switchToMultiAgentMode();
    
    await this.page.click('button:has-text("+ 添加 Agent")');
    
    const dialog = this.page.locator('.el-dialog');
    await expect(dialog).toBeVisible();
    
    // 填写 Agent ID
    await this.page.locator('.el-form-item', { hasText: 'Agent ID' }).locator('input').fill(data.id);
    
    // 填写显示名称
    await this.page.locator('.el-form-item', { hasText: '显示名称' }).locator('input').fill(data.name);
    
    await this.page.click('.el-dialog__footer button:has-text("保存")');
    await expect(dialog).not.toBeVisible({ timeout: 5000 });
  }

  async verifyAgentExists(name: string) {
    await expect(this.page.locator('.agent-name', { hasText: name }).first()).toBeVisible();
  }

  async deleteAgent(name: string) {
    const card = this.page.locator('.agent-card', { hasText: name });
    await card.locator('button:has-text("删除")').click();
    await this.page.click('.el-message-box__btns button:has-text("确定")');
  }

  async saveAgentConfig() {
    await this.page.click('button:has-text("保存 Agent 配置")');
    await this.waitForToast('保存成功');
  }
}

// Memory 配置页
export class MemoryPage extends ConfigPage {
  async goto() {
    await this.page.goto('/config/memory');
    await this.page.waitForLoadState('networkidle');
  }

  async enableMemory() {
    const switch_ = this.page.locator('.el-form-item', { hasText: '启用 Memory' }).locator('.el-switch');
    const isChecked = await switch_.locator('input').isChecked();
    if (!isChecked) {
      await switch_.click();
      await this.page.waitForTimeout(300);
    }
  }

  async configureOllama(baseUrl = 'http://localhost:11434') {
    // 选择 Ollama Provider
    await this.page.locator('.el-radio-button', { hasText: 'Ollama' }).click();
    await this.page.waitForTimeout(300);
    
    // 填写 Base URL
    await this.page.locator('.el-form-item', { hasText: 'Base URL' }).locator('input').fill(baseUrl);
  }

  async saveMemoryConfig() {
    await this.page.click('button:has-text("保存 Memory 配置")');
    await this.waitForToast('保存成功');
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
