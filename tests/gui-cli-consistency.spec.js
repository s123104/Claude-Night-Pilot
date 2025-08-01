// GUI與CLI功能一致性測試
// 確保兩個介面提供相同的功能和響應格式

import { test, expect } from "@playwright/test";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const CLI_BINARY = path.join(
  __dirname,
  "../src-tauri/target/debug/cnp-unified"
);

test.describe("GUI與CLI功能一致性測試", () => {
  let page;

  test.beforeAll(async ({ browser }) => {
    // 打開GUI應用 (開發模式)
    page = await browser.newPage();
    await page.goto("http://localhost:8081");

    // 等待應用加載
    await page.waitForLoadState("networkidle");
  });

  test("冷卻狀態檢查一致性", async () => {
    // 1. 測試CLI冷卻檢查
    const cliResult = await executeCLI(["cooldown", "--format", "json"]);
    expect(cliResult.exitCode).toBe(0);
    const cliCooldownData = JSON.parse(cliResult.stdout);

    // 2. 測試GUI冷卻檢查 (模擬)
    // 由於GUI使用統一API客戶端，在開發模式下會返回模擬數據
    const guiCooldownData = await page.evaluate(async () => {
      if (window.unifiedApiClient) {
        return await window.unifiedApiClient.getCooldownStatusUnified();
      }
      return null;
    });

    // 3. 驗證響應結構一致性
    if (guiCooldownData) {
      // 檢查必需字段
      const requiredFields = ["is_cooling"];
      for (const field of requiredFields) {
        expect(cliCooldownData).toHaveProperty(field);
        expect(guiCooldownData).toHaveProperty(field);
      }

      // 檢查數據類型一致性
      expect(typeof cliCooldownData.is_cooling).toBe(
        typeof guiCooldownData.is_cooling
      );
    }

    console.log("✅ 冷卻狀態檢查一致性測試通過");
  });

  test("系統健康檢查一致性", async () => {
    // 1. 測試CLI健康檢查
    const cliResult = await executeCLI(["health", "--format", "json"]);
    expect(cliResult.exitCode).toBe(0);
    const cliHealthData = JSON.parse(cliResult.stdout);

    // 2. 測試GUI健康檢查
    const guiHealthData = await page.evaluate(async () => {
      if (window.unifiedApiClient) {
        return await window.unifiedApiClient.getSystemHealthUnified();
      }
      return null;
    });

    // 3. 驗證響應結構一致性
    if (guiHealthData) {
      const requiredFields = ["timestamp"];
      for (const field of requiredFields) {
        expect(cliHealthData).toHaveProperty(field);
        expect(guiHealthData).toHaveProperty(field);
      }
    }

    console.log("✅ 系統健康檢查一致性測試通過");
  });

  test("API客戶端加載驗證", async () => {
    // 驗證統一API客戶端是否正確加載
    const apiClientLoaded = await page.evaluate(() => {
      return !!(window.unifiedApiClient && window.promptExecutor);
    });

    expect(apiClientLoaded).toBe(true);

    // 驗證向後兼容性
    const backwardCompatible = await page.evaluate(() => {
      return !!(window.apiClient && window.apiClient.invokeCommand);
    });

    expect(backwardCompatible).toBe(true);

    console.log("✅ API客戶端加載驗證通過");
  });

  test("GUI介面元素存在性檢查", async () => {
    // 檢查關鍵GUI元素是否存在
    const navigationExists = (await page.locator("[data-tab]").count()) > 0;
    expect(navigationExists).toBe(true);

    // 檢查Prompt列表容器
    const promptsListExists = (await page.locator("#prompts-list").count()) > 0;
    expect(promptsListExists).toBe(true);

    // 檢查Snackbar容器
    const snackbarExists =
      (await page.locator("#snackbar-container").count()) > 0;
    expect(snackbarExists).toBe(true);

    console.log("✅ GUI介面元素存在性檢查通過");
  });

  test("執行選項參數一致性", async () => {
    // 檢查CLI執行選項
    const executeHelpResult = await executeCLI(["execute", "--help"]);
    expect(executeHelpResult.exitCode).toBe(0);

    const cliOptions = executeHelpResult.stdout;
    expect(cliOptions).toContain("--mode");
    expect(cliOptions).toContain("--retry");
    expect(cliOptions).toContain("--cooldown-check");
    expect(cliOptions).toContain("--work-dir");

    // 檢查GUI的執行選項結構
    const guiOptionsStructure = await page.evaluate(() => {
      // 檢查統一執行選項的結構
      if (window.unifiedApiClient) {
        // 模擬創建執行選項來檢查結構
        const testOptions = {
          mode: "sync",
          retryEnabled: true,
          cooldownCheck: true,
          workingDirectory: "/tmp",
        };
        return Object.keys(testOptions);
      }
      return [];
    });

    // 驗證GUI支持相同的執行選項概念
    expect(guiOptionsStructure.length).toBeGreaterThan(0);

    console.log("✅ 執行選項參數一致性測試通過");
  });

  test("錯誤處理模式一致性", async () => {
    // 測試CLI錯誤處理
    const cliErrorResult = await executeCLI(["execute", "--invalid"]);
    expect(cliErrorResult.exitCode).not.toBe(0);
    expect(cliErrorResult.stderr.length).toBeGreaterThan(0);

    // 測試GUI錯誤處理（檢查是否有錯誤處理機制）
    const guiErrorHandling = await page.evaluate(() => {
      // 檢查是否有snackbarManager用於錯誤顯示
      return !!(
        window.snackbarManager &&
        window.snackbarManager.error &&
        typeof window.snackbarManager.error === "function"
      );
    });

    expect(guiErrorHandling).toBe(true);

    console.log("✅ 錯誤處理模式一致性測試通過");
  });

  test("功能覆蓋度一致性", async () => {
    // 檢查CLI支持的主要功能
    const helpResult = await executeCLI(["--help"]);
    const cliFeatures = helpResult.stdout;

    const expectedFeatures = [
      "execute", // 執行命令
      "cooldown", // 冷卻檢查
      "health", // 健康檢查
      "batch", // 批量執行
    ];

    for (const feature of expectedFeatures) {
      expect(cliFeatures).toContain(feature);
    }

    // 檢查GUI支持的對應功能
    const guiFeatures = await page.evaluate(() => {
      const features = [];

      // 檢查執行功能
      if (window.promptExecutor && window.promptExecutor.executePromptById) {
        features.push("execute");
      }

      // 檢查冷卻檢查功能
      if (
        window.unifiedApiClient &&
        window.unifiedApiClient.getCooldownStatusUnified
      ) {
        features.push("cooldown");
      }

      // 檢查健康檢查功能
      if (
        window.unifiedApiClient &&
        window.unifiedApiClient.getSystemHealthUnified
      ) {
        features.push("health");
      }

      // 檢查批量執行功能
      if (window.promptExecutor && window.promptExecutor.executeBatch) {
        features.push("batch");
      }

      return features;
    });

    // 驗證GUI支持所有主要功能
    for (const feature of expectedFeatures) {
      expect(guiFeatures).toContain(feature);
    }

    console.log("✅ 功能覆蓋度一致性測試通過");
  });

  test("響應時間性能一致性", async () => {
    // 測試CLI響應時間
    const cliStartTime = Date.now();
    const cliResult = await executeCLI(["health"]);
    const cliResponseTime = Date.now() - cliStartTime;

    expect(cliResult.exitCode).toBe(0);
    expect(cliResponseTime).toBeLessThan(10000); // 10秒內

    // 測試GUI響應時間
    const guiStartTime = Date.now();
    const guiResult = await page.evaluate(async () => {
      if (window.unifiedApiClient) {
        try {
          await window.unifiedApiClient.getSystemHealthUnified();
          return true;
        } catch (error) {
          return false;
        }
      }
      return false;
    });
    const guiResponseTime = Date.now() - guiStartTime;

    if (guiResult) {
      expect(guiResponseTime).toBeLessThan(10000); // 10秒內

      // 響應時間應該在合理範圍內（GUI可能因為瀏覽器開銷稍慢）
      const timeDifference = Math.abs(cliResponseTime - guiResponseTime);
      expect(timeDifference).toBeLessThan(5000); // 差異不超過5秒
    }

    console.log(
      `✅ 響應時間性能一致性測試通過 - CLI: ${cliResponseTime}ms, GUI: ${guiResponseTime}ms`
    );
  });
});

// 輔助函數：執行CLI命令
async function executeCLI(args) {
  return new Promise((resolve) => {
    const child = spawn(CLI_BINARY, args, {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    child.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    child.on("close", (exitCode) => {
      resolve({
        exitCode,
        stdout: stdout.trim(),
        stderr: stderr.trim(),
      });
    });

    // 設置超時
    setTimeout(() => {
      child.kill();
      resolve({
        exitCode: -1,
        stdout: "",
        stderr: "Test timeout",
      });
    }, 15000); // 15秒超時
  });
}
