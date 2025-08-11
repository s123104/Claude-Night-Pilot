import { defineConfig, devices } from "@playwright/test";
import os from 'os';

/**
 * Claude Night Pilot E2E 測試配置 - 重構版
 * 
 * 支援新的測試架構：e2e, integration, cross-platform
 */
export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : Math.max(1, Math.floor(os.cpus().length * 0.75)),  // Optimize worker count
  
  // 增強報告配置
  reporter: [
    ["html", { outputFolder: "./coverage/playwright-report" }],
    ["json", { outputFile: "./coverage/test-results.json" }],
    ["junit", { outputFile: "./coverage/junit-results.xml" }],
    ...(process.env.CI ? [["github"]] : [])
  ],
  
  use: {
    baseURL: "http://localhost:8081",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    
    // Optimized timeout settings for performance
    actionTimeout: 15000,  // Increased for slower operations
    navigationTimeout: 45000,  // Increased for app startup
    
    // Performance optimizations
    launchOptions: {
      args: [
        '--disable-web-security',
        '--disable-features=TranslateUI',
        '--disable-ipc-flooding-protection',
        '--disable-backgrounding-occluded-windows',
        '--disable-renderer-backgrounding',
        '--disable-background-timer-throttling',
        '--no-sandbox',  // Faster startup in CI
        '--disable-dev-shm-usage',
        '--memory-pressure-off',
      ],
    },
  },

  // 分類測試專案
  projects: [
    {
      name: "gui-tests",
      testDir: "./tests/e2e/gui",
      use: { 
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
        // Parallel execution within project
        fullyParallel: true,
      },
    },
    {
      name: "cli-tests",
      testDir: "./tests/e2e/cli", 
      use: { 
        ...devices["Desktop Chrome"],
        fullyParallel: true,
      },
    },
    {
      name: "integration-tests",
      testDir: "./tests/integration",
      use: { 
        ...devices["Desktop Chrome"],
        fullyParallel: true,
      },
      // Higher retry count for integration tests
      retries: process.env.CI ? 3 : 1,
    },
    {
      name: "cross-platform-tests",
      testDir: "./tests/e2e/cross-platform",
      use: { 
        ...devices["Desktop Chrome"],
        fullyParallel: true,
      },
    },
    // Mobile tests - run separately to avoid conflicts
    {
      name: "mobile-chrome",
      testDir: "./tests/e2e/gui",
      use: { 
        ...devices["Pixel 5"],
        // Slightly longer timeouts for mobile
        actionTimeout: 20000,
        navigationTimeout: 60000,
      },
      // Run mobile tests only when specifically requested
      grep: process.env.MOBILE_TESTS ? undefined : /^(?!.*mobile).*$/,
    }
  ],

  // 全域設定（注意：暫時禁用以避免 ES 模組導入問題）
  // globalSetup: './tests/utils/global-setup.js',
  // globalTeardown: './tests/utils/global-teardown.js',

  // Optimized development server configuration
  webServer: {
    command: "npm run dev:frontend",
    port: 8081,
    reuseExistingServer: !process.env.CI,
    timeout: 60000,  // Increased timeout for slower systems
    env: {
      NODE_ENV: 'test'
    },
    // Optimize startup detection
    stdout: 'pipe',
    stderr: 'pipe',
  },
  
  // Optimized test timeout settings
  timeout: 120000,  // Increased for complex tests
  expect: {
    timeout: 15000  // Increased for async operations
  },
  
  // Global test configuration for better performance
  globalTimeout: 600000,  // 10 minutes for entire test suite
  
  // Optimize test execution
  maxFailures: process.env.CI ? 5 : undefined,  // Stop after too many failures
  
  // 測試匹配模式
  testMatch: [
    "**/*.spec.js",
    "**/*.test.js"
  ],
  
  // 忽略 demos 目錄（除非明確指定）
  testIgnore: process.env.INCLUDE_DEMOS ? [] : [
    "**/demos/**"
  ]
});
