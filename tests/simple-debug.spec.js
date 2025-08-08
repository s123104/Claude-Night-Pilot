// 簡單調試測試

import { test, expect } from "@playwright/test";

test("簡單調試", async ({ page }) => {
  await page.goto("http://localhost:8081");
  await page.waitForLoadState("networkidle");

  const result = await page.evaluate(async () => {
    // 直接調用 mockResponse 方法
    const mockResult = window.apiClient.mockResponse("get_app_info", {});
    
    // 調用 invokeCommand 方法
    const invokeResult = await window.apiClient.invokeCommand("get_app_info");
    
    return {
      mockResult: mockResult,
      invokeResult: invokeResult,
      hasTauriApi: !!window.__TAURI_API__,
      actualType: typeof invokeResult
    };
  });

  console.log("結果:", JSON.stringify(result, null, 2));
  
  // 檢查 mock 是否返回正確數據
  if (result.mockResult) {
    expect(result.mockResult.version).toBe("0.2.0");
    expect(result.mockResult.tauri_version).toBe("2.0.0");
  }
  
  // 檢查 invoke 是否返回正確數據
  if (result.invokeResult && typeof result.invokeResult === 'object') {
    console.log("Invoke result 結構:", Object.keys(result.invokeResult));
  }
});