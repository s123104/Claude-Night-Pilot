import { defineConfig, devices } from "@playwright/test";

/**
 * Claude Night Pilot E2E 測試配置
 */
export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",
  use: {
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  // 開發伺服器設定 - 使用手動啟動的 HTTP 伺服器
  webServer: {
    command: "echo 'Using existing server on port 8081'",
    port: 8081,
    reuseExistingServer: true,
    timeout: 10000, // 10 秒等待
  },
});
