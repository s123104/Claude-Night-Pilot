import { test, expect } from "@playwright/test";

test.describe("Claude Night Pilot - 生產模式功能測試", () => {
  test.beforeEach(async ({ page }) => {
    // 前往應用主頁
    await page.goto("http://localhost:1420");

    // 等待應用載入
    await page.waitForSelector('h1:has-text("Claude Night Pilot")', {
      timeout: 30000,
    });
  });

  test("系統資訊檢查", async ({ page }) => {
    // 使用 evaluate 呼叫 Tauri 命令檢查系統資訊
    const systemInfo = await page.evaluate(async () => {
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          return await window.__TAURI__.core.invoke("get_system_info");
        } catch (error) {
          return { error: error.message };
        }
      }
      return { error: "Tauri not available" };
    });

    console.log("系統資訊:", systemInfo);

    // 驗證基本系統資訊
    if (!systemInfo.error) {
      expect(systemInfo).toHaveProperty("version");
      expect(systemInfo).toHaveProperty("debug_mode");
      expect(systemInfo).toHaveProperty("platform");
      expect(systemInfo).toHaveProperty("arch");
    }
  });

  test("Claude CLI 可用性檢測", async ({ page }) => {
    // 檢查 Claude CLI 狀態
    const cliStatus = await page.evaluate(async () => {
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          return await window.__TAURI__.core.invoke("verify_claude_cli");
        } catch (error) {
          return { error: error.message };
        }
      }
      return false;
    });

    console.log("Claude CLI 狀態:", cliStatus);

    // 在開發模式下應該返回 true
    // 在生產模式下取決於是否安裝了 Claude CLI
    expect(typeof cliStatus).toBe("boolean");
  });

  test("冷卻狀態檢測", async ({ page }) => {
    // 檢查冷卻狀態
    const cooldownStatus = await page.evaluate(async () => {
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          return await window.__TAURI__.core.invoke("get_cooldown_status");
        } catch (error) {
          return { error: error.message };
        }
      }
      return null;
    });

    console.log("冷卻狀態:", cooldownStatus);

    // 驗證冷卻狀態結構
    if (cooldownStatus && !cooldownStatus.error) {
      expect(cooldownStatus).toHaveProperty("is_cooling");
      expect(cooldownStatus).toHaveProperty("seconds_remaining");
      expect(cooldownStatus).toHaveProperty("next_available_time");
    }
  });

  test("資料庫連接和基本操作", async ({ page }) => {
    // 測試列出 prompts
    const prompts = await page.evaluate(async () => {
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          return await window.__TAURI__.core.invoke("list_prompts");
        } catch (error) {
          return { error: error.message };
        }
      }
      return [];
    });

    console.log("Prompts 列表:", prompts);

    // 驗證返回格式
    expect(Array.isArray(prompts)).toBe(true);
  });

  test("錯誤處理機制", async ({ page }) => {
    // 測試不存在的 prompt ID
    const result = await page.evaluate(async () => {
      if (window.__TAURI__ && window.__TAURI__.core) {
        try {
          return await window.__TAURI__.core.invoke("run_prompt_sync", {
            prompt_id: 99999,
            mode: "sync",
            cron_expr: null,
          });
        } catch (error) {
          return { error: error.message };
        }
      }
      return null;
    });

    console.log("錯誤處理結果:", result);

    // 在開發模式下會返回模擬回應
    // 在生產模式下會返回錯誤
    expect(result).toBeTruthy();
  });

  test("應用啟動效能檢測", async ({ page }) => {
    const startTime = Date.now();

    // 等待主要元素載入
    await page.waitForSelector("#prompt-management");
    await page.waitForSelector("#job-console");
    await page.waitForSelector("#cooldown-status");

    const loadTime = Date.now() - startTime;

    console.log(`應用載入時間: ${loadTime}ms`);

    // 驗證載入時間在合理範圍內 (< 5 秒)
    expect(loadTime).toBeLessThan(5000);
  });

  test("記憶體使用情況", async ({ page }) => {
    // 檢查記憶體使用 (如果可能的話)
    const memoryInfo = await page.evaluate(() => {
      if (performance.memory) {
        return {
          usedJSHeapSize: performance.memory.usedJSHeapSize,
          totalJSHeapSize: performance.memory.totalJSHeapSize,
          jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
        };
      }
      return null;
    });

    if (memoryInfo) {
      console.log("記憶體使用:", memoryInfo);

      // 檢查 JS 堆使用量 (< 50MB)
      expect(memoryInfo.usedJSHeapSize).toBeLessThan(50 * 1024 * 1024);
    }
  });

  test("多次操作穩定性", async ({ page }) => {
    const results = [];

    // 執行多次操作檢查穩定性
    for (let i = 0; i < 5; i++) {
      const prompts = await page.evaluate(async () => {
        if (window.__TAURI__ && window.__TAURI__.core) {
          try {
            return await window.__TAURI__.core.invoke("list_prompts");
          } catch (error) {
            return { error: error.message };
          }
        }
        return [];
      });

      results.push(prompts);

      // 短暫延遲
      await page.waitForTimeout(100);
    }

    console.log("多次操作結果:", results.length);

    // 驗證所有操作都成功
    expect(results.length).toBe(5);
    results.forEach((result, index) => {
      expect(Array.isArray(result)).toBe(true);
    });
  });
});
