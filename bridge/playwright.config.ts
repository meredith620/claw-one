// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright 前端 E2E 测试配置
 * 覆盖测试矩阵中的5个功能模块：
 * 1. Provider CRUD
 * 2. Agent CRUD
 * 3. Channel CRUD
 * 4. Memory 配置
 * 5. 用户工作流
 */
export default defineConfig({
  testDir: './e2e-browser',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [['list'], ['html', { outputFolder: 'playwright-report' }]],
  use: {
    baseURL: process.env.CLAW_ONE_URL || 'http://localhost:8080',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    // 容器环境中使用 headless 模式
    headless: true,
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    // Firefox 在容器中有网络问题，暂禁用
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
  ],
});
