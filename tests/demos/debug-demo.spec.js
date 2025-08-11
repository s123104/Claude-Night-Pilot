// 調試測試 - 檢查前端 API 實際返回值

import { test, expect } from "@playwright/test";

test.describe("調試測試", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("http://localhost:8081", {
      waitUntil: "networkidle",
      timeout: 30000,
    });
  });

  test("檢查前端 API 實際返回值", async ({ page }) => {
    const debugInfo = await page.evaluate(async () => {
      const results = {};
      
      try {
        // 檢查 window 上的對象
        results.hasApiClient = !!window.apiClient;
        results.apiClientType = typeof window.apiClient;
        
        if (window.apiClient) {
          results.apiClientMethods = Object.getOwnPropertyNames(Object.getPrototypeOf(window.apiClient));
          
          // 測試 get_app_info
          try {
            const appInfo = await window.apiClient.invokeCommand("get_app_info");
            results.appInfo = appInfo;
            results.appInfoType = typeof appInfo;
          } catch (error) {
            results.appInfoError = error.message;
          }
          
          // 測試其他命令
          try {
            const prompts = await window.apiClient.invokeCommand("get_prompts");
            results.promptsCount = Array.isArray(prompts) ? prompts.length : 0;
            results.promptsType = typeof prompts;
          } catch (error) {
            results.promptsError = error.message;
          }
          
          // 測試 mock response
          try {
            const mockResult = window.apiClient.mockResponse("get_app_info", {});
            results.mockAppInfo = mockResult;
          } catch (error) {
            results.mockError = error.message;
          }
        }
        
        return results;
      } catch (error) {
        return { error: error.message, stack: error.stack };
      }
    });

    console.log("Debug Info:", JSON.stringify(debugInfo, null, 2));
    
    // 基本檢查
    expect(debugInfo.hasApiClient).toBe(true);
    
    // 根據實際返回值調整期望
    if (debugInfo.appInfo) {
      console.log("App Info:", debugInfo.appInfo);
    }
    if (debugInfo.mockAppInfo) {
      console.log("Mock App Info:", debugInfo.mockAppInfo);
    }
    
    // 這個測試主要是用來調試，所以不會失敗
    expect(true).toBe(true);
  });
});