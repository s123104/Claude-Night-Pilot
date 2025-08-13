import { defineConfig, devices } from "@playwright/test";
import os from "os";
import process from "process";

/**
 * Claude Night Pilot - GUI 專用測試配置
 * 依賴前端開發伺服器，專注於用戶界面測試
 */
export default defineConfig({
  testDir: "./tests/e2e/gui",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI
    ? 2
    : Math.max(1, Math.floor(os.cpus().length * 0.75)),

  reporter: [
    ["html", { outputFolder: "./coverage/gui-test-report" }],
    ["json", { outputFile: "./coverage/gui-test-results.json" }],
    ["junit", { outputFile: "./coverage/gui-junit-results.xml" }],
    ...(process.env.CI ? [["github"]] : []),
  ],

  use: {
    baseURL: "http://localhost:8081",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",

    // GUI 操作相對較快 (優化超時設置)
    actionTimeout: 10000, // 減少到 10 秒
    navigationTimeout: 30000, // 減少到 30 秒

    // 瀏覽器優化配置
    launchOptions: {
      args: [
        "--disable-web-security",
        "--disable-features=TranslateUI",
        "--disable-ipc-flooding-protection",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--disable-background-timer-throttling",
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--memory-pressure-off",
        "--allow-running-insecure-content",
        "--ignore-certificate-errors",
        "--ignore-ssl-errors",
      ],
      ignoreDefaultArgs: ["--disable-extensions"],
    },
  },

  timeout: 60000, // 1 分鐘 (減少單個測試超時)
  expect: {
    timeout: 10000, // 10 秒 (減少期望超時)
  },

  globalTimeout: 300000, // 5 分鐘總超時 (減少全局超時)

  projects: [
    {
      name: "gui-features",
      testDir: "./tests/e2e/gui",
      testMatch: "**/frontend-features.spec.js",
      use: {
        viewport: { width: 1280, height: 720 },
      },
    },
    {
      name: "gui-prompts",
      testDir: "./tests/e2e/gui",
      testMatch: "**/prompt-management.spec.js",
      use: {
        viewport: { width: 1280, height: 720 },
      },
    },
    {
      name: "gui-material",
      testDir: "./tests/e2e/gui",
      testMatch: "**/material-design-ui.spec.js",
      use: {
        viewport: { width: 1280, height: 720 },
      },
    },
    {
      name: "gui-mobile",
      testDir: "./tests/e2e/gui",
      testMatch: "**/frontend-features.spec.js",
      use: {
        ...devices["iPhone 12"],
        // 行動裝置測試需要更長超時 (但仍然優化)
        actionTimeout: 15000, // 減少到 15 秒
        navigationTimeout: 45000, // 減少到 45 秒
      },
      // 只在明確要求時執行行動測試
      grep: process.env.MOBILE_TESTS ? undefined : /^(?!.*mobile).*$/,
    },
  ],

  // 前端開發伺服器配置 (使用簡單伺服器)
  webServer: {
    command: "node scripts/simple-server.js",
    port: 8081,
    reuseExistingServer: !process.env.CI, // CI 環境中總是啟動新伺服器
    timeout: 15000, // 減少伺服器啟動超時到 15 秒
    env: {
      NODE_ENV: "test",
      PORT: "8081",
    },
    stdout: "pipe",
    stderr: "pipe",
  },

  // 測試匹配模式
  testMatch: ["**/*.spec.js", "**/*.test.js"],

  // 忽略某些測試文件
  testIgnore: ["**/node_modules/**", "**/coverage/**", "**/test-results/**"],
});
