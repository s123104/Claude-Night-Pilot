import { defineConfig } from "@playwright/test";
import os from "os";
import process from "process";

/**
 * Claude Night Pilot - CLI 專用測試配置
 * 不依賴前端開發伺服器，專注於 CLI 工具測試
 */
export default defineConfig({
  testDir: "./tests/e2e/cli",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Force serial execution to avoid Cargo lock contention

  reporter: [
    ["html", { outputFolder: "./coverage/cli-test-report" }],
    ["json", { outputFile: "./coverage/cli-test-results.json" }],
    ["junit", { outputFile: "./coverage/cli-junit-results.xml" }],
    ...(process.env.CI ? [["github"]] : []),
  ],

  use: {
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",

    // CLI 操作通常較慢，需要更長的超時時間
    actionTimeout: 30000,
    navigationTimeout: 60000,

    // CLI 測試不需要瀏覽器優化
    launchOptions: {
      args: [
        "--disable-web-security",
        "--disable-features=TranslateUI",
        "--no-sandbox",
        "--disable-dev-shm-usage",
      ],
    },
  },

  // CLI 測試可能需要更長時間
  timeout: 180000, // 3 分鐘
  expect: {
    timeout: 30000, // 30 秒
  },

  globalTimeout: 1800000, // 30 分鐘總超時

  projects: [
    {
      name: "cli-basic",
      testDir: "./tests/e2e/cli",
      testMatch: "**/basic-commands.spec.js",
      use: {
        // 基礎 CLI 測試的特定配置
        actionTimeout: 45000,
      },
    },
    {
      name: "cli-stress",
      testDir: "./tests/e2e/cli",
      testMatch: "**/stress-testing.spec.js",
      use: {
        // 壓力測試需要更長超時
        actionTimeout: 60000,
      },
      // 壓力測試串行執行避免資源競爭
      fullyParallel: false,
    },
  ],

  // CLI 測試不需要 web server
  // webServer: undefined,
});
