// DatabaseManager 異步操作測試
// 測試新實施的 DatabaseManager 模式和異步操作的穩定性

import { test, expect } from "@playwright/test";

test.describe("DatabaseManager 異步操作測試", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("http://localhost:8080", {
      waitUntil: "networkidle",
      timeout: 30000,
    });
  });

  test.describe("基本數據庫連接測試", () => {
    test("資料庫管理器應正常初始化", async ({ page }) => {
      // 測試資料庫連接狀態
      const dbStatus = await page.evaluate(async () => {
        try {
          const response = await window.apiClient.invokeCommand("get_app_info");
          return response;
        } catch (error) {
          return { error: error.message };
        }
      });

      expect(dbStatus.version).toBe("0.2.0");
      expect(dbStatus.tauri_version).toBe("2.0.0");
    });

    test("資料庫配置應正確設置", async ({ page }) => {
      // 檢查資料庫配置是否正確
      const configCheck = await page.evaluate(async () => {
        try {
          // 呼叫 Tauri 命令檢查資料庫健康狀態
          const health = await window.apiClient.invokeCommand("health_check");
          return {
            success: true,
            health: health
          };
        } catch (error) {
          return {
            success: false,
            error: error.message
          };
        }
      });

      expect(configCheck.success).toBe(true);
      expect(configCheck.health.database).toBe("connected");
    });
  });

  test.describe("Prompt 異步 CRUD 操作", () => {
    test("應能異步創建 Prompt", async ({ page }) => {
      const testPrompt = {
        title: "異步測試 Prompt " + Date.now(),
        content: "這是異步創建的測試內容 @test.md",
        tags: "async,test,database"
      };

      const result = await page.evaluate(async (prompt) => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: prompt.title,
            content: prompt.content,
            tags: prompt.tags
          });
          return { success: true, id: promptId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, testPrompt);

      expect(result.success).toBe(true);
      expect(typeof result.id).toBe("number");
      expect(result.id).toBeGreaterThan(0);
    });

    test("應能異步讀取 Prompt", async ({ page }) => {
      // 先創建一個 Prompt
      const createResult = await page.evaluate(async () => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: "讀取測試 Prompt",
            content: "測試內容用於讀取操作",
            tags: "read,test"
          });
          return { success: true, id: promptId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(createResult.success).toBe(true);

      // 然後讀取這個 Prompt
      const readResult = await page.evaluate(async (promptId) => {
        try {
          const prompt = await window.apiClient.invokeCommand("get_prompt", { id: promptId });
          return { success: true, prompt: prompt };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, createResult.id);

      expect(readResult.success).toBe(true);
      expect(readResult.prompt).not.toBeNull();
      expect(readResult.prompt.title).toBe("讀取測試 Prompt");
      expect(readResult.prompt.content).toBe("測試內容用於讀取操作");
    });

    test("應能異步刪除 Prompt", async ({ page }) => {
      // 先創建一個 Prompt
      const createResult = await page.evaluate(async () => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: "刪除測試 Prompt",
            content: "這個 Prompt 將被刪除",
            tags: "delete,test"
          });
          return { success: true, id: promptId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(createResult.success).toBe(true);

      // 然後刪除這個 Prompt
      const deleteResult = await page.evaluate(async (promptId) => {
        try {
          const deleted = await window.apiClient.invokeCommand("delete_prompt", { id: promptId });
          return { success: true, deleted: deleted };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, createResult.id);

      expect(deleteResult.success).toBe(true);
      expect(deleteResult.deleted).toBe(true);
    });
  });

  test.describe("Schedule 異步操作", () => {
    test("應能異步創建排程", async ({ page }) => {
      // 先創建一個 Prompt
      const promptResult = await page.evaluate(async () => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: "排程測試 Prompt",
            content: "用於排程測試的內容",
            tags: "schedule,test"
          });
          return { success: true, id: promptId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(promptResult.success).toBe(true);

      // 創建排程
      const scheduleResult = await page.evaluate(async (promptId) => {
        try {
          const scheduleId = await window.apiClient.invokeCommand("create_schedule", {
            promptId: promptId,
            scheduleTime: new Date(Date.now() + 3600000).toISOString(), // 1小時後
            cronExpr: "0 */1 * * *" // 每小時執行
          });
          return { success: true, id: scheduleId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, promptResult.id);

      expect(scheduleResult.success).toBe(true);
      expect(typeof scheduleResult.id).toBe("number");
      expect(scheduleResult.id).toBeGreaterThan(0);
    });

    test("應能獲取待執行排程", async ({ page }) => {
      const pendingSchedules = await page.evaluate(async () => {
        try {
          const schedules = await window.apiClient.invokeCommand("get_pending_schedules");
          return { success: true, schedules: schedules };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(pendingSchedules.success).toBe(true);
      expect(Array.isArray(pendingSchedules.schedules)).toBe(true);
    });

    test("應能更新排程狀態", async ({ page }) => {
      // 先創建 Prompt 和排程
      const setupResult = await page.evaluate(async () => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: "更新測試 Prompt",
            content: "用於更新排程測試",
            tags: "update,schedule,test"
          });

          const scheduleId = await window.apiClient.invokeCommand("create_schedule", {
            promptId: promptId,
            scheduleTime: new Date(Date.now() + 7200000).toISOString(), // 2小時後
            cronExpr: "0 */2 * * *"
          });

          return { success: true, promptId: promptId, scheduleId: scheduleId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(setupResult.success).toBe(true);

      // 更新排程
      const updateResult = await page.evaluate(async (scheduleId) => {
        try {
          await window.apiClient.invokeCommand("update_schedule", {
            id: scheduleId,
            scheduleTime: null,
            status: "paused",
            cronExpr: "0 */4 * * *" // 改為每4小時執行
          });
          return { success: true };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, setupResult.scheduleId);

      expect(updateResult.success).toBe(true);
    });
  });

  test.describe("Token Usage 統計異步操作", () => {
    test("應能更新 Token 使用統計", async ({ page }) => {
      const updateResult = await page.evaluate(async () => {
        try {
          await window.apiClient.invokeCommand("update_token_usage", {
            inputTokens: 150,
            outputTokens: 75,
            costUsd: 0.002125
          });
          return { success: true };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(updateResult.success).toBe(true);
    });

    test("應能獲取 Token 使用統計", async ({ page }) => {
      const statsResult = await page.evaluate(async () => {
        try {
          const stats = await window.apiClient.invokeCommand("get_token_usage_stats");
          return { success: true, stats: stats };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(statsResult.success).toBe(true);
      
      if (statsResult.stats) {
        expect(typeof statsResult.stats.total_input_tokens).toBe("number");
        expect(typeof statsResult.stats.total_output_tokens).toBe("number");
        expect(typeof statsResult.stats.total_cost_usd).toBe("number");
        expect(statsResult.stats.total_input_tokens).toBeGreaterThanOrEqual(0);
        expect(statsResult.stats.total_output_tokens).toBeGreaterThanOrEqual(0);
        expect(statsResult.stats.total_cost_usd).toBeGreaterThanOrEqual(0);
      }
    });
  });

  test.describe("並發操作測試", () => {
    test("應能處理並發的 Prompt 創建操作", async ({ page }) => {
      const concurrentCreations = await page.evaluate(async () => {
        try {
          // 同時創建多個 Prompt
          const promises = [];
          for (let i = 0; i < 5; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `並發測試 Prompt ${i + 1}`,
                content: `並發創建的內容 ${i + 1}`,
                tags: `concurrent,test,${i + 1}`
              })
            );
          }

          const results = await Promise.all(promises);
          return { success: true, ids: results };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(concurrentCreations.success).toBe(true);
      expect(concurrentCreations.ids).toHaveLength(5);
      
      // 檢查所有 ID 都是唯一的正數
      const uniqueIds = new Set(concurrentCreations.ids);
      expect(uniqueIds.size).toBe(5);
      concurrentCreations.ids.forEach(id => {
        expect(typeof id).toBe("number");
        expect(id).toBeGreaterThan(0);
      });
    });

    test("應能處理並發的讀取操作", async ({ page }) => {
      // 先創建一些測試資料
      const setupResult = await page.evaluate(async () => {
        try {
          const promises = [];
          for (let i = 0; i < 3; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `讀取測試 Prompt ${i + 1}`,
                content: `用於並發讀取測試的內容 ${i + 1}`,
                tags: `read,concurrent,test,${i + 1}`
              })
            );
          }
          const ids = await Promise.all(promises);
          return { success: true, ids: ids };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(setupResult.success).toBe(true);

      // 並發讀取
      const concurrentReads = await page.evaluate(async (ids) => {
        try {
          const promises = ids.map(id =>
            window.apiClient.invokeCommand("get_prompt", { id: id })
          );

          const results = await Promise.all(promises);
          return { success: true, prompts: results };
        } catch (error) {
          return { success: false, error: error.message };
        }
      }, setupResult.ids);

      expect(concurrentReads.success).toBe(true);
      expect(concurrentReads.prompts).toHaveLength(3);
      
      concurrentReads.prompts.forEach((prompt, index) => {
        expect(prompt).not.toBeNull();
        expect(prompt.title).toBe(`讀取測試 Prompt ${index + 1}`);
      });
    });
  });

  test.describe("錯誤處理測試", () => {
    test("應正確處理無效的 Prompt ID", async ({ page }) => {
      const invalidIdResult = await page.evaluate(async () => {
        try {
          const prompt = await window.apiClient.invokeCommand("get_prompt", { id: 999999 });
          return { success: true, prompt: prompt };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      // 無效 ID 應該返回 null 而不是錯誤
      expect(invalidIdResult.success).toBe(true);
      expect(invalidIdResult.prompt).toBeNull();
    });

    test("應正確處理空的創建參數", async ({ page }) => {
      const emptyParamsResult = await page.evaluate(async () => {
        try {
          const promptId = await window.apiClient.invokeCommand("create_prompt", {
            title: "",
            content: "",
            tags: null
          });
          return { success: true, id: promptId };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      // 空參數應該被允許（或有適當的驗證）
      if (emptyParamsResult.success) {
        expect(typeof emptyParamsResult.id).toBe("number");
        expect(emptyParamsResult.id).toBeGreaterThan(0);
      } else {
        // 如果不允許空參數，應該有明確的錯誤訊息
        expect(emptyParamsResult.error).toBeDefined();
        expect(emptyParamsResult.error.length).toBeGreaterThan(0);
      }
    });

    test("應正確處理資料庫鎖定情況", async ({ page }) => {
      // 測試在高並發情況下的資料庫鎖定處理
      const lockTestResult = await page.evaluate(async () => {
        try {
          // 快速連續執行多個寫入操作
          const promises = [];
          for (let i = 0; i < 10; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `鎖定測試 ${i}`,
                content: `測試資料庫鎖定處理 ${i}`,
                tags: `lock,test,${i}`
              }).then(id => ({ success: true, id: id }))
              .catch(error => ({ success: false, error: error.message }))
            );
          }

          const results = await Promise.all(promises);
          return { success: true, results: results };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(lockTestResult.success).toBe(true);
      
      // 檢查結果 - 大部分操作應該成功
      const successCount = lockTestResult.results.filter(r => r.success).length;
      expect(successCount).toBeGreaterThan(5); // 至少一半操作應該成功
    });
  });

  test.describe("效能測試", () => {
    test("大批量創建操作應在合理時間內完成", async ({ page }) => {
      const performanceTest = await page.evaluate(async () => {
        const startTime = performance.now();
        
        try {
          const batchSize = 20;
          const promises = [];
          
          for (let i = 0; i < batchSize; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `效能測試 Prompt ${i + 1}`,
                content: `大批量創建測試內容 ${i + 1}`,
                tags: `performance,batch,test,${i + 1}`
              })
            );
          }

          const results = await Promise.all(promises);
          const endTime = performance.now();
          const duration = endTime - startTime;

          return {
            success: true,
            duration: duration,
            count: results.length,
            avgTime: duration / results.length
          };
        } catch (error) {
          const endTime = performance.now();
          return {
            success: false,
            error: error.message,
            duration: endTime - startTime
          };
        }
      });

      expect(performanceTest.success).toBe(true);
      expect(performanceTest.count).toBe(20);
      
      // 每個操作平均應該在 500ms 內完成
      expect(performanceTest.avgTime).toBeLessThan(500);
      
      // 總時間應該在 10 秒內完成
      expect(performanceTest.duration).toBeLessThan(10000);
      
      console.log(`✅ 批量創建效能測試: ${performanceTest.count} 操作在 ${performanceTest.duration.toFixed(2)}ms 內完成 (平均 ${performanceTest.avgTime.toFixed(2)}ms/操作)`);
    });

    test("資料庫查詢操作應有良好的響應時間", async ({ page }) => {
      // 先創建一些測試資料
      const setupResult = await page.evaluate(async () => {
        try {
          const promises = [];
          for (let i = 0; i < 10; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `查詢效能測試 ${i + 1}`,
                content: `用於查詢效能測試 ${i + 1}`,
                tags: `query,performance,test`
              })
            );
          }
          const ids = await Promise.all(promises);
          return { success: true, ids: ids };
        } catch (error) {
          return { success: false, error: error.message };
        }
      });

      expect(setupResult.success).toBe(true);

      // 測試查詢效能
      const queryPerformance = await page.evaluate(async (ids) => {
        const startTime = performance.now();
        
        try {
          const promises = ids.map(id =>
            window.apiClient.invokeCommand("get_prompt", { id: id })
          );

          const results = await Promise.all(promises);
          const endTime = performance.now();
          const duration = endTime - startTime;

          return {
            success: true,
            duration: duration,
            count: results.length,
            avgTime: duration / results.length
          };
        } catch (error) {
          const endTime = performance.now();
          return {
            success: false,
            error: error.message,
            duration: endTime - startTime
          };
        }
      }, setupResult.ids);

      expect(queryPerformance.success).toBe(true);
      expect(queryPerformance.count).toBe(10);
      
      // 每個查詢操作應該在 100ms 內完成
      expect(queryPerformance.avgTime).toBeLessThan(100);
      
      // 總查詢時間應該在 2 秒內完成
      expect(queryPerformance.duration).toBeLessThan(2000);
      
      console.log(`✅ 查詢效能測試: ${queryPerformance.count} 查詢在 ${queryPerformance.duration.toFixed(2)}ms 內完成 (平均 ${queryPerformance.avgTime.toFixed(2)}ms/查詢)`);
    });
  });
});