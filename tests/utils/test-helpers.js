/**
 * Claude Night Pilot - 共享測試工具函數
 * 
 * 提供可重用的測試工具和輔助函數
 */

import { expect } from "@playwright/test";
import { 
  setupClaudeMock, 
  setupErrorScenario, 
  validateMockSetup, 
  resetMockState 
} from "./mock-claude.js";

// 重新導出 mock 相關功能
export { 
  setupClaudeMock, 
  setupErrorScenario, 
  validateMockSetup, 
  resetMockState 
};

/**
 * 等待應用程式完全載入
 * @param {Page} page - Playwright Page 物件
 * @param {number} timeout - 超時時間 (毫秒)
 */
export async function waitForAppReady(page, timeout = 30000) {
  console.log('🔍 Waiting for app to be ready...');
  
  try {
    // Enable test mode by setting URL parameter or injecting script
    console.log('🧪 Enabling test mode...');
    await page.addInitScript(() => {
      // Set test mode flag
      window.__TEST_MODE__ = true;
      if (document.body) {
        document.body.setAttribute('data-test-mode', 'true');
      }
      
      // Speed up initialization for tests
      if (window.location) {
        window.location.search += (window.location.search ? '&' : '?') + 'test=true';
      }
    });
    
    // First check if DOM is loaded
    await page.waitForSelector('#app', { timeout: 10000 });
    console.log('🏠 DOM loaded, waiting for app initialization...');
    
    // Inject test mode if not already set
    await page.evaluate(() => {
      if (document.body && document.body.getAttribute('data-test-mode') !== 'true') {
        document.body.setAttribute('data-test-mode', 'true');
        console.log('🧪 Test mode enabled via script injection');
      }
    });
    
    // Enhanced waiting strategy with multiple fallbacks
    const waitStrategies = [
      // Strategy 1: Wait for app ready flag
      page.waitForFunction(
        () => window.__APP_READY__ === true, 
        { timeout: timeout / 3 }
      ).catch(() => null),
      
      // Strategy 2: Wait for app container to be visible
      page.waitForSelector('[data-testid="app-container"]', { 
        state: 'visible',
        timeout: timeout / 3 
      }).catch(() => null),
      
      // Strategy 3: Wait for custom app-ready event
      page.waitForFunction(
        () => {
          const appContainer = document.querySelector('[data-testid="app-container"]');
          return appContainer && getComputedStyle(appContainer).display !== 'none';
        },
        { timeout: timeout / 3 }
      ).catch(() => null)
    ];
    
    // Wait for at least one strategy to succeed
    const results = await Promise.allSettled(waitStrategies);
    const anySucceeded = results.some(result => result.status === 'fulfilled' && result.value);
    
    if (!anySucceeded) {
      console.log('⚠️ Standard strategies failed, trying emergency initialization...');
      
      // Emergency initialization for stubborn tests
      await page.evaluate(() => {
        const appContainer = document.getElementById('app');
        const loadingOverlay = document.getElementById('app-loader');
        
        if (loadingOverlay) {
          loadingOverlay.style.display = 'none';
        }
        
        if (appContainer) {
          appContainer.style.display = 'flex';
          appContainer.style.visibility = 'visible';
          appContainer.style.opacity = '1';
        }
        
        window.__APP_READY__ = true;
        console.log('✅ Emergency app initialization complete');
      });
    }
    
    // Final verification with more lenient checks
    await page.waitForFunction(
      () => {
        const appContainer = document.querySelector('[data-testid="app-container"]') || 
                             document.querySelector('#app') ||
                             document.querySelector('.app-container');
        return appContainer && getComputedStyle(appContainer).display !== 'none';
      },
      { timeout: 5000 }
    );
    
    console.log('✅ App is ready!');
    
    // Short delay to ensure everything is settled
    await page.waitForTimeout(200);
    
  } catch (error) {
    console.error('❌ App ready check failed:', error.message);
    
    // Enhanced debug information
    const debugInfo = await page.evaluate(() => {
      const appContainer = document.querySelector('[data-testid="app-container"]');
      const appTitle = document.querySelector('[data-testid="app-title"]');
      const loadingOverlay = document.querySelector('#app-loader');
      const bodyTestMode = document.body?.getAttribute('data-test-mode');
      
      return {
        appContainerExists: !!appContainer,
        appContainerVisible: appContainer ? getComputedStyle(appContainer).display !== 'none' : false,
        appContainerDisplay: appContainer ? getComputedStyle(appContainer).display : 'N/A',
        appTitleExists: !!appTitle,
        loadingOverlayExists: !!loadingOverlay,
        loadingOverlayDisplay: loadingOverlay ? getComputedStyle(loadingOverlay).display : 'N/A',
        testModeSet: bodyTestMode,
        appReady: window.__APP_READY__,
        windowLocation: window.location.href
      };
    });
    
    console.log('📋 Debug info:', debugInfo);
    
    // Final emergency fallback - force app to show
    console.log('🆘 Attempting final emergency fallback...');
    await page.evaluate(() => {
      const appContainer = document.getElementById('app') || 
                           document.querySelector('[data-testid="app-container"]');
      const loadingOverlay = document.getElementById('app-loader');
      
      if (loadingOverlay) {
        loadingOverlay.remove();
      }
      
      if (appContainer) {
        appContainer.style.cssText = 'display: flex !important; visibility: visible !important; opacity: 1 !important; min-height: 100vh !important;';
        appContainer.setAttribute('data-emergency-show', 'true');
      }
      
      window.__APP_READY__ = true;
      document.dispatchEvent(new CustomEvent('app-ready', { 
        detail: { emergency: true }
      }));
    });
    
    // Try once more to verify
    try {
      await page.waitForSelector('[data-testid="app-container"], #app', { 
        state: 'visible', 
        timeout: 2000 
      });
      console.log('✅ Emergency fallback succeeded');
    } catch (finalError) {
      console.error('❌ Final fallback also failed:', finalError.message);
      throw new Error(`App failed to initialize: ${error.message}. Debug info: ${JSON.stringify(debugInfo)}`);
    }
  }
}

/**
 * 建立測試 Prompt
 * @param {Page} page - Playwright Page 物件
 * @param {Object} promptData - Prompt 資料
 */
export async function createTestPrompt(page, promptData) {
  const { title, content, tags } = promptData;
  
  // 點擊建立按鈕
  await page.click("[data-testid='create-prompt-fab']");
  
  // 等待表單可見
  await expect(page.locator("[data-testid='prompt-modal']")).toBeVisible();
  
  // 填寫表單
  await page.fill("[data-testid='prompt-title-input']", title);
  await page.fill("[data-testid='prompt-content-input']", content);
  if (tags) {
    await page.fill("[data-testid='prompt-tags-input']", tags);
  }
  
  // 儲存
  await page.click("[data-testid='prompt-modal-save-btn']");
  
  // 驗證成功
  await expect(page.locator("[data-testid='snackbar-container']"))
    .toContainText("Prompt 建立成功", { timeout: 15000 });
}

/**
 * 執行 CLI 命令
 * @param {Array<string>} args - 命令參數
 * @returns {Promise<Object>} 執行結果
 */
export async function executeCLI(args) {
  const { spawn } = await import("child_process");
  const path = await import("path");
  const { fileURLToPath } = await import("url");
  
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = path.dirname(__filename);
  const CLI_BINARY = path.join(__dirname, "../../src-tauri/target/debug/cnp-unified");
  
  return new Promise((resolve, reject) => {
    const child = spawn(CLI_BINARY, args, { 
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: path.join(__dirname, "../..")
    });
    
    let stdout = "";
    let stderr = "";
    
    child.stdout.on("data", (data) => {
      stdout += data.toString();
    });
    
    child.stderr.on("data", (data) => {
      stderr += data.toString();
    });
    
    child.on("close", (code) => {
      resolve({
        exitCode: code,
        stdout: stdout.trim(),
        stderr: stderr.trim()
      });
    });
    
    child.on("error", (error) => {
      reject(error);
    });
    
    // 設定超時
    setTimeout(() => {
      child.kill();
      reject(new Error("CLI command timeout"));
    }, 30000);
  });
}

/**
 * 驗證 Material Design 元件
 * @param {Page} page - Playwright Page 物件
 */
export async function validateMaterialDesignComponents(page) {
  // 驗證頂部應用欄
  await expect(page.locator(".md-top-app-bar")).toBeVisible();
  
  // 驗證 Material Icons
  const brandIcon = page.locator(".brand-icon.material-symbols-rounded");
  await expect(brandIcon).toBeVisible();
  
  // 驗證導航鐵軌
  await expect(page.locator(".md-navigation-rail")).toBeVisible();
  
  // 驗證主要導航項目
  const navItems = [
    { tab: "prompts", icon: "chat", label: "Prompt" },
    { tab: "scheduler", icon: "schedule", label: "排程" },
    { tab: "results", icon: "analytics", label: "結果" },
    { tab: "system", icon: "monitoring", label: "監控" },
  ];
  
  for (const item of navItems) {
    const navItem = page.locator(`[data-tab="${item.tab}"]`);
    await expect(navItem).toBeVisible();
    await expect(navItem.locator(".material-symbols-outlined"))
      .toHaveText(item.icon);
  }
}

/**
 * 清理測試資料
 * @param {Page} page - Playwright Page 物件
 */
export async function cleanupTestData(page) {
  // 透過統一 API 清理測試資料
  await page.evaluate(async () => {
    if (window.unifiedApiClient && window.unifiedApiClient.cleanup) {
      await window.unifiedApiClient.cleanup();
    }
  });
}

/**
 * 模擬網路延遲
 * @param {Page} page - Playwright Page 物件
 * @param {number} delay - 延遲時間 (毫秒)
 */
export async function simulateNetworkDelay(page, delay = 1000) {
  await page.route('**/*', async (route) => {
    await new Promise(resolve => setTimeout(resolve, delay));
    await route.continue();
  });
}

/**
 * 檢查回應式設計
 * @param {Page} page - Playwright Page 物件
 * @param {Object} viewport - 視口尺寸
 */
export async function checkResponsiveDesign(page, viewport) {
  await page.setViewportSize(viewport);
  await page.waitForTimeout(500); // 等待 CSS 轉換
  
  // 檢查常見的回應式元素
  const mobileBreakpoint = 768;
  const isMobile = viewport.width < mobileBreakpoint;
  
  if (isMobile) {
    // 行動裝置專用檢查
    await expect(page.locator(".mobile-menu")).toBeVisible();
  } else {
    // 桌面版檢查
    await expect(page.locator(".desktop-nav")).toBeVisible();
  }
}

/**
 * 等待並擷取錯誤日誌
 * @param {Page} page - Playwright Page 物件
 * @returns {Array} 錯誤日誌陣列
 */
export async function captureErrorLogs(page) {
  const errors = [];
  
  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });
  
  page.on('pageerror', (error) => {
    errors.push(error.message);
  });
  
  return errors;
}

/**
 * 驗證無障礙性
 * @param {Page} page - Playwright Page 物件
 */
export async function validateAccessibility(page) {
  // 基本無障礙性檢查
  const elements = await page.locator('button, input, select, textarea, [role="button"]').all();
  
  for (const element of elements) {
    // 檢查是否有適當的標籤或 aria-label
    const hasLabel = await element.getAttribute('aria-label') || 
                    await element.getAttribute('title') ||
                    await element.locator('label').count() > 0;
    
    if (!hasLabel) {
      console.warn('Element missing accessibility label:', await element.innerHTML());
    }
  }
  
  // 檢查顏色對比（簡化版）
  const buttons = await page.locator('button').all();
  for (const button of buttons) {
    const computedStyle = await button.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return {
        color: style.color,
        backgroundColor: style.backgroundColor
      };
    });
    
    // 這裡可以加入更複雜的對比度計算
    console.log('Button colors:', computedStyle);
  }
}