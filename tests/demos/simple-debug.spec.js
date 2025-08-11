// 簡單調試測試

import { test, expect } from "@playwright/test";

test("簡單調試", async ({ page }) => {
  await page.goto("http://localhost:8081");
  // 等待應用就緒（任一條件）
  await Promise.race([
    page.waitForFunction(() => window.__APP_READY__ === true, {
      timeout: 30000,
    }),
    page.waitForSelector(".app-container", { timeout: 30000 }),
  ]);

  const result = await page.evaluate(async () => {
    // 直接調用 unifiedApiClient 的 mock 或 invoke
    const mockResult = window.unifiedApiClient?.mockResponse?.(
      "get_system_info",
      {}
    ) ?? { version: "dev", tauri_version: "dev" };

    // 調用 invokeCommand 方法
    const invokeResult = await window.apiClient.invokeCommand(
      "get_system_info"
    );

    return {
      mockResult: mockResult,
      invokeResult: invokeResult,
      hasTauriApi: !!window.__TAURI_API__,
      actualType: typeof invokeResult,
    };
  });

  console.log("結果:", JSON.stringify(result, null, 2));

  // 檢查 mock 是否返回正確數據
  if (result.mockResult) {
    expect(result.mockResult.version).toBe("0.2.0");
    expect(result.mockResult.tauri_version).toBe("2.0.0");
  }

  // 檢查 invoke 是否返回正確數據
  if (result.invokeResult && typeof result.invokeResult === "object") {
    console.log("Invoke result 結構:", Object.keys(result.invokeResult));
  }
});
