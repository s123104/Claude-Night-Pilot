/**
 * 測試專用的 Playwright 配置
 * 
 * 針對重構後的測試結構進行優化
 */

import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "../..", // 指向 tests 根目錄
  
  // 測試並行設定
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  
  // 報告設定
  reporter: [
    ['html', { outputFolder: './coverage/playwright-report' }],
    ['json', { outputFile: './coverage/test-results.json' }],
    ['junit', { outputFile: './coverage/junit-results.xml' }]
  ],
  
  // 全域設定
  use: {
    baseURL: "http://localhost:8081",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    
    // 等待設定
    actionTimeout: 10000,
    navigationTimeout: 30000,
    
    // 瀏覽器設定
    ignoreHTTPSErrors: true,
    
    // 自定義等待條件
    waitForFunction: {
      timeout: 30000
    }
  },
  
  // 測試專案設定
  projects: [
    {
      name: "gui-tests",
      testDir: "./e2e/gui",
      use: { 
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 }
      },
    },
    {
      name: "cli-tests", 
      testDir: "./e2e/cli",
      use: { 
        ...devices["Desktop Chrome"]
      },
    },
    {
      name: "integration-tests",
      testDir: "./integration",
      use: { 
        ...devices["Desktop Chrome"]
      },
    },
    {
      name: "cross-platform-tests",
      testDir: "./e2e/cross-platform",
      use: { 
        ...devices["Desktop Chrome"]
      },
    },
    // 行動裝置測試
    {
      name: "mobile-chrome",
      testDir: "./e2e/gui",
      use: { 
        ...devices["Pixel 5"]
      },
    },
    {
      name: "mobile-safari",
      testDir: "./e2e/gui",
      use: { 
        ...devices["iPhone 12"]
      },
    }
  ],
  
  // 開發伺服器設定
  webServer: {
    command: "npm run dev:frontend",
    port: 8081,
    reuseExistingServer: !process.env.CI,
    timeout: 30000,
    env: {
      NODE_ENV: 'test'
    }
  },
  
  // 全域設定和清理
  globalSetup: require.resolve('../utils/global-setup.js'),
  globalTeardown: require.resolve('../utils/global-teardown.js'),
  
  // 測試配置
  timeout: 60000, // 單個測試超時
  expect: {
    timeout: 10000 // 斷言超時
  },
  
  // 測試文件匹配模式
  testMatch: [
    '**/*.spec.js',
    '**/*.test.js'
  ],
  
  // 忽略的測試文件
  testIgnore: [
    '**/node_modules/**',
    '**/coverage/**',
    '**/demos/**'
  ]
});