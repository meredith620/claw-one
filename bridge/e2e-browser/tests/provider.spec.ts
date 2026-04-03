/**
 * Provider CRUD Browser 测试
 * 测试矩阵功能模块 #1
 */

import { test, expect, testData } from '../fixtures';

test.describe('Provider CRUD', () => {
  test.beforeEach(async ({ providerPage }) => {
    await providerPage.goto();
  });

  test('添加 OpenAI Provider', async ({ page, providerPage }) => {
    const data = testData.provider.openai;
    
    await providerPage.addProvider(data);
    
    // 验证保存成功提示
    await providerPage.waitForToast('保存成功');
    
    // 验证 Provider 出现在列表中
    await providerPage.verifyProviderExists(data.name);
  });

  test('添加 Anthropic Provider', async ({ page, providerPage }) => {
    const data = testData.provider.anthropic;
    
    await providerPage.addProvider(data);
    
    await providerPage.waitForToast('保存成功');
    await providerPage.verifyProviderExists(data.name);
  });

  test('删除 Provider', async ({ page, providerPage }) => {
    // 先添加一个 Provider
    const data = testData.provider.openai;
    await providerPage.addProvider(data);
    await providerPage.waitForToast('保存成功');
    
    // 删除它
    await providerPage.deleteProvider(data.name);
    await providerPage.waitForToast('删除成功');
    
    // 验证已删除
    await expect(page.locator('text=' + data.name)).not.toBeVisible();
  });

  test('表单验证 - 必填项', async ({ page }) => {
    await page.click('button:has-text("添加 Provider")');
    
    // 直接点击保存，不填写任何内容
    await page.click('.el-dialog__footer button:has-text("保存")');
    
    // 验证表单验证提示
    await expect(page.locator('.el-form-item__error')).toBeVisible();
    
    // 对话框不应关闭（因为有验证错误）
    await expect(page.locator('.el-dialog')).toBeVisible();
  });
});
