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

  // Tauri 應用測試設定
  webServer: {
    command: "npm run tauri dev",
    port: 1420,
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2 分鐘等待應用啟動
  },
});
