// CLI 壓力測試和 Tauri 命令並發測試
// 測試 CLI 工具和 Tauri 命令在高負載和並發情況下的表現

import { test, expect } from "@playwright/test";
import { exec, spawn } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

test.describe("CLI 壓力測試和 Tauri 命令並發測試", () => {
  const CLI_BINARY = "cd src-tauri && cargo run --bin cnp-unified --";

  test.beforeEach(async ({ page }) => {
    await page.goto("http://localhost:8081", {
      waitUntil: "networkidle",
      timeout: 30000,
    });
  });

  test.describe("CLI 壓力測試", () => {
    test("大量並發 CLI 命令執行", async () => {
      const concurrencyLevel = 10;
      const startTime = Date.now();
      
      const promises = [];
      for (let i = 0; i < concurrencyLevel; i++) {
        promises.push(
          execAsync(`${CLI_BINARY} status`)
            .then(result => ({
              success: true,
              index: i,
              stdout: result.stdout.trim(),
              stderr: result.stderr.trim(),
              duration: Date.now() - startTime
            }))
            .catch(error => ({
              success: false,
              index: i,
              error: error.message,
              duration: Date.now() - startTime
            }))
        );
      }

      const results = await Promise.all(promises);
      const endTime = Date.now();
      const totalDuration = endTime - startTime;

      const successful = results.filter(r => r.success).length;
      const failed = results.filter(r => !r.success).length;
      const successRate = successful / results.length;

      expect(successful).toBeGreaterThan(concurrencyLevel * 0.7); // 至少70%成功
      expect(totalDuration).toBeLessThan(30000); // 30秒內完成

      console.log(`✅ CLI 並發測試: ${successful}/${concurrencyLevel} 成功 (${(successRate * 100).toFixed(1)}%), 耗時: ${totalDuration}ms`);
      
      // 檢查每個成功的結果都包含期望內容
      const validResults = results.filter(r => r.success && r.stdout.length > 0).length;
      expect(validResults).toBeGreaterThan(0);
    });

    test("快速連續 CLI 命令執行", async () => {
      const commandSequence = [
        "status",
        "cooldown", 
        "health",
        "status"
      ];

      const startTime = Date.now();
      const results = [];

      for (const command of commandSequence) {
        try {
          const result = await execAsync(`${CLI_BINARY} ${command}`);
          results.push({
            success: true,
            command: command,
            stdout: result.stdout.trim(),
            duration: Date.now() - startTime
          });
        } catch (error) {
          results.push({
            success: false,
            command: command,
            error: error.message,
            duration: Date.now() - startTime
          });
        }
      }

      const endTime = Date.now();
      const totalDuration = endTime - startTime;

      const successful = results.filter(r => r.success).length;
      expect(successful).toBe(commandSequence.length); // 所有命令都應該成功
      expect(totalDuration).toBeLessThan(15000); // 15秒內完成

      console.log(`✅ CLI 連續執行測試: ${successful}/${commandSequence.length} 成功, 耗時: ${totalDuration}ms`);
    });

    test("CLI 長時間運行壓力測試", async () => {
      const testDuration = 10000; // 10秒測試
      const interval = 500; // 每500ms一個命令
      
      const startTime = Date.now();
      const results = [];
      let operationCount = 0;

      while (Date.now() - startTime < testDuration) {
        try {
          const commandStartTime = Date.now();
          const result = await execAsync(`${CLI_BINARY} status`);
          const commandDuration = Date.now() - commandStartTime;
          
          results.push({
            success: true,
            index: operationCount,
            duration: commandDuration,
            outputLength: result.stdout.length
          });
          operationCount++;
          
          // 等待間隔
          await new Promise(resolve => setTimeout(resolve, interval));
        } catch (error) {
          results.push({
            success: false,
            index: operationCount,
            error: error.message
          });
          operationCount++;
        }
      }

      const endTime = Date.now();
      const totalDuration = endTime - startTime;
      const successful = results.filter(r => r.success).length;
      const successRate = successful / results.length;
      const avgDuration = results
        .filter(r => r.success && r.duration)
        .reduce((sum, r) => sum + r.duration, 0) / successful;

      expect(successRate).toBeGreaterThan(0.8); // 80% 成功率
      expect(avgDuration).toBeLessThan(5000); // 平均每個命令5秒內完成

      console.log(`✅ CLI 長時間壓力測試: ${successful}/${operationCount} 成功 (${(successRate * 100).toFixed(1)}%)`);
      console.log(`   總耗時: ${totalDuration}ms, 平均命令耗時: ${avgDuration.toFixed(2)}ms`);
    });
  });

  test.describe("Tauri 命令並發測試", () => {
    test("大量並發 Tauri 命令", async ({ page }) => {
      const concurrencyLevel = 25;

      const result = await page.evaluate(async (level) => {
        const startTime = performance.now();
        
        try {
          const promises = [];
          
          // 混合不同類型的命令
          const commands = [
            () => window.apiClient.invokeCommand("health_check"),
            () => window.apiClient.invokeCommand("check_cooldown"),
            () => window.apiClient.invokeCommand("get_token_usage_stats"),
            () => window.apiClient.invokeCommand("list_prompts"),
            () => window.apiClient.invokeCommand("get_pending_schedules")
          ];

          for (let i = 0; i < level; i++) {
            const command = commands[i % commands.length];
            promises.push(
              command()
                .then(result => ({
                  success: true,
                  index: i,
                  result: result,
                  commandType: command.name
                }))
                .catch(error => ({
                  success: false,
                  index: i,
                  error: error.message,
                  commandType: command.name
                }))
            );
          }

          const results = await Promise.all(promises);
          const endTime = performance.now();

          const successful = results.filter(r => r.success).length;
          const failed = results.filter(r => !r.success).length;

          return {
            success: true,
            totalRequests: level,
            successful: successful,
            failed: failed,
            duration: endTime - startTime,
            results: results
          };
        } catch (error) {
          const endTime = performance.now();
          return {
            success: false,
            error: error.message,
            duration: endTime - startTime
          };
        }
      }, concurrencyLevel);

      expect(result.success).toBe(true);
      expect(result.successful).toBeGreaterThan(concurrencyLevel * 0.8); // 80% 成功率
      expect(result.duration).toBeLessThan(10000); // 10秒內完成

      const successRate = result.successful / result.totalRequests;
      console.log(`✅ Tauri 並發測試: ${result.successful}/${result.totalRequests} 成功 (${(successRate * 100).toFixed(1)}%), 耗時: ${result.duration.toFixed(2)}ms`);
    });

    test("混合讀寫操作並發測試", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const startTime = performance.now();
        
        try {
          const promises = [];
          
          // 創建一些讀操作
          for (let i = 0; i < 10; i++) {
            promises.push(
              window.apiClient.invokeCommand("health_check")
                .then(result => ({ type: 'read', success: true, index: i, result: result }))
                .catch(error => ({ type: 'read', success: false, index: i, error: error.message }))
            );
          }

          // 創建一些寫操作
          for (let i = 0; i < 10; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `並發寫測試 ${i}`,
                content: `並發寫入內容 ${i}`,
                tags: `concurrent,write,test,${i}`
              })
                .then(result => ({ type: 'write', success: true, index: i, result: result }))
                .catch(error => ({ type: 'write', success: false, index: i, error: error.message }))
            );
          }

          // 創建一些混合操作
          for (let i = 0; i < 5; i++) {
            promises.push(
              window.apiClient.invokeCommand("get_token_usage_stats")
                .then(result => ({ type: 'mixed', success: true, index: i, result: result }))
                .catch(error => ({ type: 'mixed', success: false, index: i, error: error.message }))
            );
          }

          const results = await Promise.all(promises);
          const endTime = performance.now();

          const byType = results.reduce((acc, r) => {
            if (!acc[r.type]) acc[r.type] = { successful: 0, failed: 0 };
            if (r.success) acc[r.type].successful++;
            else acc[r.type].failed++;
            return acc;
          }, {});

          const totalSuccessful = results.filter(r => r.success).length;
          const totalFailed = results.filter(r => !r.success).length;

          return {
            success: true,
            total: results.length,
            successful: totalSuccessful,
            failed: totalFailed,
            duration: endTime - startTime,
            byType: byType
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

      expect(result.success).toBe(true);
      expect(result.successful).toBeGreaterThan(result.total * 0.8); // 80% 成功率
      expect(result.duration).toBeLessThan(15000); // 15秒內完成

      console.log(`✅ 混合讀寫並發測試: ${result.successful}/${result.total} 成功, 耗時: ${result.duration.toFixed(2)}ms`);
      console.log(`   讀操作: ${result.byType.read?.successful || 0}/${(result.byType.read?.successful || 0) + (result.byType.read?.failed || 0)}`);
      console.log(`   寫操作: ${result.byType.write?.successful || 0}/${(result.byType.write?.successful || 0) + (result.byType.write?.failed || 0)}`);
      console.log(`   混合操作: ${result.byType.mixed?.successful || 0}/${(result.byType.mixed?.successful || 0) + (result.byType.mixed?.failed || 0)}`);
    });

    test("資料庫事務並發安全測試", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const startTime = performance.now();
        
        try {
          // 創建大量併發的 CRUD 操作來測試事務安全
          const promises = [];
          const testData = [];
          
          // 第一批：創建操作
          for (let i = 0; i < 15; i++) {
            const promptData = {
              title: `事務測試 ${i}`,
              content: `事務安全測試內容 ${i}`,
              tags: `transaction,safety,test,${i}`
            };
            testData.push(promptData);
            
            promises.push(
              window.apiClient.invokeCommand("create_prompt", promptData)
                .then(id => ({ operation: 'create', success: true, index: i, id: id }))
                .catch(error => ({ operation: 'create', success: false, index: i, error: error.message }))
            );
          }

          // 等待創建操作完成
          const createResults = await Promise.all(promises);
          const successfulCreates = createResults.filter(r => r.success);
          
          // 第二批：基於成功創建的項目進行讀取操作
          const readPromises = [];
          for (const createResult of successfulCreates.slice(0, 10)) {
            readPromises.push(
              window.apiClient.invokeCommand("get_prompt", { id: createResult.id })
                .then(prompt => ({ operation: 'read', success: true, id: createResult.id, prompt: prompt }))
                .catch(error => ({ operation: 'read', success: false, id: createResult.id, error: error.message }))
            );
          }

          const readResults = await Promise.all(readPromises);

          // 第三批：排程創建（如果有成功的 Prompt）
          const schedulePromises = [];
          for (const createResult of successfulCreates.slice(0, 5)) {
            schedulePromises.push(
              window.apiClient.invokeCommand("create_schedule", {
                promptId: createResult.id,
                scheduleTime: new Date(Date.now() + (3600000 * Math.random())).toISOString(),
                cronExpr: "0 * * * *"
              })
                .then(scheduleId => ({ operation: 'schedule', success: true, promptId: createResult.id, scheduleId: scheduleId }))
                .catch(error => ({ operation: 'schedule', success: false, promptId: createResult.id, error: error.message }))
            );
          }

          const scheduleResults = await Promise.all(schedulePromises);

          const endTime = performance.now();

          const allResults = [...createResults, ...readResults, ...scheduleResults];
          const successful = allResults.filter(r => r.success).length;
          const failed = allResults.filter(r => !r.success).length;

          return {
            success: true,
            total: allResults.length,
            successful: successful,
            failed: failed,
            duration: endTime - startTime,
            breakdown: {
              creates: { total: createResults.length, successful: createResults.filter(r => r.success).length },
              reads: { total: readResults.length, successful: readResults.filter(r => r.success).length },
              schedules: { total: scheduleResults.length, successful: scheduleResults.filter(r => r.success).length }
            }
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

      expect(result.success).toBe(true);
      expect(result.successful).toBeGreaterThan(result.total * 0.7); // 70% 成功率
      expect(result.duration).toBeLessThan(20000); // 20秒內完成

      console.log(`✅ 資料庫事務並發安全測試: ${result.successful}/${result.total} 成功, 耗時: ${result.duration.toFixed(2)}ms`);
      console.log(`   創建: ${result.breakdown.creates.successful}/${result.breakdown.creates.total}`);
      console.log(`   讀取: ${result.breakdown.reads.successful}/${result.breakdown.reads.total}`);
      console.log(`   排程: ${result.breakdown.schedules.successful}/${result.breakdown.schedules.total}`);
    });
  });

  test.describe("效能基準測試", () => {
    test("單個 Tauri 命令響應時間基準", async ({ page }) => {
      const commands = [
        { name: "health_check", params: {} },
        { name: "check_cooldown", params: {} },
        { name: "get_token_usage_stats", params: {} },
        { name: "list_prompts", params: {} }
      ];

      for (const command of commands) {
        const result = await page.evaluate(async (cmd) => {
          const measurements = [];
          const iterations = 10;

          for (let i = 0; i < iterations; i++) {
            const startTime = performance.now();
            try {
              await window.apiClient.invokeCommand(cmd.name, cmd.params);
              const endTime = performance.now();
              measurements.push(endTime - startTime);
            } catch (error) {
              measurements.push(-1); // 標記失敗
            }
          }

          const successful = measurements.filter(m => m > 0);
          const avgTime = successful.reduce((sum, time) => sum + time, 0) / successful.length;
          const minTime = Math.min(...successful);
          const maxTime = Math.max(...successful);

          return {
            command: cmd.name,
            iterations: iterations,
            successful: successful.length,
            avgTime: avgTime,
            minTime: minTime,
            maxTime: maxTime
          };
        }, command);

        expect(result.successful).toBeGreaterThan(result.iterations * 0.9); // 90% 成功率
        expect(result.avgTime).toBeLessThan(1000); // 平均1秒內響應
        expect(result.maxTime).toBeLessThan(3000); // 最大3秒內響應

        console.log(`✅ ${result.command}: 平均 ${result.avgTime.toFixed(2)}ms (${result.minTime.toFixed(2)}-${result.maxTime.toFixed(2)}ms), 成功率 ${result.successful}/${result.iterations}`);
      }
    });

    test("CLI 命令響應時間基準", async () => {
      const commands = ["status", "cooldown", "health"];

      for (const command of commands) {
        const measurements = [];
        const iterations = 5;

        for (let i = 0; i < iterations; i++) {
          const startTime = Date.now();
          try {
            await execAsync(`${CLI_BINARY} ${command}`);
            const endTime = Date.now();
            measurements.push(endTime - startTime);
          } catch (error) {
            measurements.push(-1); // 標記失敗
          }
        }

        const successful = measurements.filter(m => m > 0);
        const avgTime = successful.reduce((sum, time) => sum + time, 0) / successful.length;
        const minTime = Math.min(...successful);
        const maxTime = Math.max(...successful);

        expect(successful.length).toBeGreaterThan(iterations * 0.8); // 80% 成功率
        expect(avgTime).toBeLessThan(10000); // 平均10秒內響應（包含編譯時間）
        expect(maxTime).toBeLessThan(20000); // 最大20秒內響應

        console.log(`✅ CLI ${command}: 平均 ${avgTime.toFixed(2)}ms (${minTime.toFixed(2)}-${maxTime.toFixed(2)}ms), 成功率 ${successful.length}/${iterations}`);
      }
    });

    test("大數據量處理效能測試", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const startTime = performance.now();
        
        try {
          // 創建大批量資料
          const batchSize = 50;
          const largeContent = "大量資料測試內容 ".repeat(100); // ~2KB per prompt
          
          const promises = [];
          for (let i = 0; i < batchSize; i++) {
            promises.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `大數據測試 ${i}`,
                content: largeContent,
                tags: `big,data,performance,test,${i}`
              })
            );
          }

          const createResults = await Promise.all(promises);
          const createEndTime = performance.now();
          const createDuration = createEndTime - startTime;

          // 批量讀取
          const readStartTime = performance.now();
          const readPromises = createResults.map(id =>
            window.apiClient.invokeCommand("get_prompt", { id: id })
          );

          const readResults = await Promise.all(readPromises);
          const readEndTime = performance.now();
          const readDuration = readEndTime - readStartTime;

          return {
            success: true,
            batchSize: batchSize,
            contentSizeKB: (largeContent.length * batchSize) / 1024,
            createDuration: createDuration,
            readDuration: readDuration,
            totalDuration: readEndTime - startTime,
            avgCreateTime: createDuration / batchSize,
            avgReadTime: readDuration / batchSize,
            throughputCreatesPerSec: batchSize / (createDuration / 1000),
            throughputReadsPerSec: batchSize / (readDuration / 1000)
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

      expect(result.success).toBe(true);
      expect(result.avgCreateTime).toBeLessThan(500); // 每個創建操作 < 500ms
      expect(result.avgReadTime).toBeLessThan(100); // 每個讀取操作 < 100ms
      expect(result.throughputCreatesPerSec).toBeGreaterThan(2); // 至少每秒2個創建操作
      expect(result.throughputReadsPerSec).toBeGreaterThan(10); // 至少每秒10個讀取操作

      console.log(`✅ 大數據量效能測試: ${result.batchSize} 項目, ${result.contentSizeKB.toFixed(2)}KB`);
      console.log(`   創建: ${result.createDuration.toFixed(2)}ms (平均 ${result.avgCreateTime.toFixed(2)}ms/個, ${result.throughputCreatesPerSec.toFixed(1)} 個/秒)`);
      console.log(`   讀取: ${result.readDuration.toFixed(2)}ms (平均 ${result.avgReadTime.toFixed(2)}ms/個, ${result.throughputReadsPerSec.toFixed(1)} 個/秒)`);
    });
  });

  test.describe("資源使用監控", () => {
    test("記憶體使用量監控", async ({ page }) => {
      const result = await page.evaluate(async () => {
        if (!performance.memory) {
          return { supported: false, message: "Performance.memory API 不支援" };
        }

        const initialMemory = {
          used: performance.memory.usedJSHeapSize,
          total: performance.memory.totalJSHeapSize,
          limit: performance.memory.jsHeapSizeLimit
        };

        try {
          // 執行一些記憶體密集操作
          const operations = [];
          for (let i = 0; i < 20; i++) {
            operations.push(
              window.apiClient.invokeCommand("create_prompt", {
                title: `記憶體測試 ${i}`,
                content: "記憶體使用量測試內容 ".repeat(500),
                tags: `memory,monitoring,test,${i}`
              })
            );
          }

          await Promise.all(operations);

          const finalMemory = {
            used: performance.memory.usedJSHeapSize,
            total: performance.memory.totalJSHeapSize,
            limit: performance.memory.jsHeapSizeLimit
          };

          const memoryIncrease = finalMemory.used - initialMemory.used;
          const memoryUsagePercent = (finalMemory.used / finalMemory.limit) * 100;

          return {
            supported: true,
            initialMemory: initialMemory,
            finalMemory: finalMemory,
            memoryIncrease: memoryIncrease,
            memoryIncreaseKB: memoryIncrease / 1024,
            memoryUsagePercent: memoryUsagePercent,
            avgMemoryPerOperation: memoryIncrease / 20
          };
        } catch (error) {
          return {
            supported: true,
            error: error.message,
            initialMemory: initialMemory
          };
        }
      });

      if (result.supported && !result.error) {
        expect(result.memoryUsagePercent).toBeLessThan(80); // 記憶體使用不超過80%
        expect(result.avgMemoryPerOperation).toBeLessThan(1024 * 1024); // 每個操作平均記憶體增長 < 1MB

        console.log(`✅ 記憶體監控: 增長 ${result.memoryIncreaseKB.toFixed(2)}KB, 使用率 ${result.memoryUsagePercent.toFixed(2)}%`);
        console.log(`   平均每操作: ${(result.avgMemoryPerOperation / 1024).toFixed(2)}KB`);
      } else {
        console.log(`ℹ️ 記憶體監控: ${result.message || result.error}`);
      }
    });

    test("長時間運行穩定性監控", async ({ page }) => {
      const result = await page.evaluate(async () => {
        const startTime = performance.now();
        const testDuration = 8000; // 8秒測試
        const operationInterval = 400; // 每400ms一個操作
        
        const operations = [];
        const memorySnapshots = [];
        let operationCount = 0;

        try {
          while (performance.now() - startTime < testDuration) {
            const opStartTime = performance.now();
            
            try {
              const id = await window.apiClient.invokeCommand("create_prompt", {
                title: `穩定性監控 ${operationCount}`,
                content: `長時間運行測試 ${operationCount}`,
                tags: `stability,monitoring,test,${operationCount}`
              });

              const opEndTime = performance.now();
              operations.push({
                success: true,
                index: operationCount,
                duration: opEndTime - opStartTime,
                id: id
              });

              // 記錄記憶體快照（如果支援）
              if (performance.memory) {
                memorySnapshots.push({
                  timestamp: opEndTime - startTime,
                  used: performance.memory.usedJSHeapSize,
                  total: performance.memory.totalJSHeapSize
                });
              }

              operationCount++;
              
              // 等待間隔
              await new Promise(resolve => setTimeout(resolve, operationInterval));
            } catch (error) {
              const opEndTime = performance.now();
              operations.push({
                success: false,
                index: operationCount,
                duration: opEndTime - opStartTime,
                error: error.message
              });
              operationCount++;
            }
          }

          const endTime = performance.now();
          const totalDuration = endTime - startTime;

          const successful = operations.filter(op => op.success).length;
          const failed = operations.filter(op => !op.success).length;
          const successRate = successful / operations.length;
          
          const avgDuration = operations
            .filter(op => op.success && op.duration)
            .reduce((sum, op) => sum + op.duration, 0) / successful;

          // 分析記憶體趨勢
          let memoryTrend = null;
          if (memorySnapshots.length > 2) {
            const firstSnapshot = memorySnapshots[0];
            const lastSnapshot = memorySnapshots[memorySnapshots.length - 1];
            const memoryGrowth = lastSnapshot.used - firstSnapshot.used;
            const timeSpan = lastSnapshot.timestamp - firstSnapshot.timestamp;
            memoryTrend = {
              growthKB: memoryGrowth / 1024,
              growthRateKBPerSec: (memoryGrowth / 1024) / (timeSpan / 1000)
            };
          }

          return {
            success: true,
            totalDuration: totalDuration,
            operationCount: operationCount,
            successful: successful,
            failed: failed,
            successRate: successRate,
            avgOperationDuration: avgDuration,
            operationsPerSecond: operationCount / (totalDuration / 1000),
            memoryTrend: memoryTrend
          };
        } catch (error) {
          const endTime = performance.now();
          return {
            success: false,
            error: error.message,
            duration: endTime - startTime,
            operationCount: operationCount
          };
        }
      });

      expect(result.success).toBe(true);
      expect(result.successRate).toBeGreaterThan(0.9); // 90% 成功率
      expect(result.avgOperationDuration).toBeLessThan(2000); // 平均操作時間 < 2秒
      expect(result.operationsPerSecond).toBeGreaterThan(1); // 至少每秒1個操作

      console.log(`✅ 長時間穩定性監控: ${result.operationCount} 操作, 成功率 ${(result.successRate * 100).toFixed(1)}%`);
      console.log(`   平均操作時間: ${result.avgOperationDuration.toFixed(2)}ms, 吞吐量: ${result.operationsPerSecond.toFixed(2)} 操作/秒`);
      
      if (result.memoryTrend) {
        console.log(`   記憶體增長: ${result.memoryTrend.growthKB.toFixed(2)}KB (${result.memoryTrend.growthRateKBPerSec.toFixed(2)}KB/秒)`);
      }
    });
  });
});