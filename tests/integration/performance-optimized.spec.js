// Optimized performance integration tests
// Tests database concurrency, batch operations, and system performance

import { test, expect } from "@playwright/test";
import { 
  PerformanceProfiler, 
  ConcurrentTester, 
  DatabasePerformanceTester,
  ResourceMonitor,
  TestDataGenerator,
  PerformanceAssertions
} from "../utils/performance-helpers.js";

test.describe("Performance Optimization Tests", () => {
  let page;

  test.beforeEach(async ({ page: testPage }) => {
    page = testPage;
    await page.goto("http://localhost:8080", {
      waitUntil: "networkidle",
      timeout: 60000,
    });
    
    // Wait for API client to be ready
    await page.waitForFunction(() => window.apiClient !== undefined, {
      timeout: 30000
    });
  });

  test.describe("Database Concurrency Tests", () => {
    test("should handle concurrent prompt creation efficiently", async () => {
      const profiler = new PerformanceProfiler("Concurrent Prompt Creation");
      const concurrentTester = new ConcurrentTester(page);
      
      profiler.start();
      
      // Define concurrent operations
      const createPromptOps = Array.from({ length: 20 }, (_, i) => 
        async (page) => {
          return await page.evaluate(async (index) => {
            const result = await window.apiClient.invokeCommand("create_prompt", {
              title: `Concurrent Prompt ${index}`,
              content: `Concurrent content ${index}`,
              tags: `concurrent,test,${index}`
            });
            return result;
          }, i);
        }
      );
      
      profiler.checkpoint("operations_prepared");
      
      // Execute concurrent operations
      const results = await concurrentTester.runConcurrent(createPromptOps, 10);
      
      profiler.checkpoint("concurrent_execution_complete");
      profiler.end();
      
      // Performance assertions
      const report = concurrentTester.getReport();
      const perfReport = profiler.getReport();
      
      // Assert performance requirements
      PerformanceAssertions.assertConcurrentOperations(results, 90, 2000); // 90% success, max 2s avg
      PerformanceAssertions.assertResponseTime(perfReport.totalDuration, 10000, "concurrent prompt creation");
      
      expect(report.successRate).toBeGreaterThanOrEqual(90);
      expect(report.totalOperations).toBe(20);
      
      console.log(`✅ Concurrent test completed: ${report.successRate.toFixed(1)}% success rate in ${perfReport.totalDuration.toFixed(2)}ms`);
    });

    test("should maintain performance under database load", async () => {
      const dbTester = new DatabasePerformanceTester(page);
      const resourceMonitor = new ResourceMonitor(page);
      
      await resourceMonitor.takeSnapshot("before_load_test");
      
      // Generate test data
      const prompts = TestDataGenerator.generatePrompts(50, {
        titlePrefix: "Load Test Prompt",
        contentPrefix: "Load testing database performance with bulk data",
        includeTags: true
      });
      
      // Test batch creation performance
      const batchResult = await dbTester.testBatchOperations("createPrompt", prompts, 10);
      
      await resourceMonitor.takeSnapshot("after_batch_creation");
      
      // Test query performance under load
      const queryResult = await dbTester.measureQueryPerformance("listPrompts", {}, 10);
      
      await resourceMonitor.takeSnapshot("after_query_load");
      
      // Performance assertions
      expect(batchResult.results.length).toBeGreaterThan(0);
      expect(queryResult.successfulIterations).toBeGreaterThanOrEqual(8); // 80% success rate
      
      PerformanceAssertions.assertResponseTime(queryResult.avgDuration, 500, "list prompts under load");
      
      // Memory growth check
      const memoryGrowth = resourceMonitor.getMemoryGrowth();
      if (memoryGrowth) {
        PerformanceAssertions.assertMemoryUsage(memoryGrowth, 50, "database load test");
      }
      
      console.log(`✅ Database load test completed: ${batchResult.results.length} operations, avg query time ${queryResult.avgDuration.toFixed(2)}ms`);
    });

    test("should optimize read operations with caching", async () => {
      const profiler = new PerformanceProfiler("Cache Optimization Test");
      
      // Create test data first
      const testPrompts = TestDataGenerator.generatePrompts(25);
      
      profiler.start();
      
      // Create prompts
      for (const prompt of testPrompts.slice(0, 5)) {
        await page.evaluate(async (promptData) => {
          await window.apiClient.invokeCommand("create_prompt", promptData);
        }, prompt);
      }
      
      profiler.checkpoint("test_data_created");
      
      // First read (cache miss)
      const firstReadStart = performance.now();
      const firstResult = await page.evaluate(async () => {
        return await window.apiClient.invokeCommand("list_prompts");
      });
      const firstReadTime = performance.now() - firstReadStart;
      
      profiler.checkpoint("first_read_complete");
      
      // Second read (potential cache hit)
      const secondReadStart = performance.now();
      const secondResult = await page.evaluate(async () => {
        return await window.apiClient.invokeCommand("list_prompts");
      });
      const secondReadTime = performance.now() - secondReadStart;
      
      profiler.checkpoint("second_read_complete");
      profiler.end();
      
      // Performance analysis
      expect(firstResult.length).toBeGreaterThanOrEqual(5);
      expect(secondResult.length).toBe(firstResult.length);
      
      // Cache should improve performance (allowing for some variance)
      const performanceImprovement = firstReadTime > secondReadTime;
      const improvementRatio = secondReadTime / firstReadTime;
      
      console.log(`📊 Cache performance: First read ${firstReadTime.toFixed(2)}ms, Second read ${secondReadTime.toFixed(2)}ms`);
      console.log(`📊 Performance ratio: ${improvementRatio.toFixed(2)} (lower is better)`);
      
      // Accept if second read is within reasonable bounds (not significantly slower)
      expect(improvementRatio).toBeLessThan(2.0); // Second read shouldn't be more than 2x slower
      
      profiler.getReport();
    });
  });

  test.describe("Batch Operation Performance", () => {
    test("should efficiently handle bulk operations", async () => {
      const profiler = new PerformanceProfiler("Bulk Operations Test");
      const dbTester = new DatabasePerformanceTester(page);
      
      profiler.start();
      
      // Generate large dataset
      const bulkPrompts = TestDataGenerator.generatePrompts(100, {
        titlePrefix: "Bulk Test",
        contentPrefix: "Bulk operation performance test content",
        includeTags: true
      });
      
      profiler.checkpoint("bulk_data_generated");
      
      // Test different batch sizes
      const batchSizes = [5, 10, 20, 50];
      const batchResults = {};
      
      for (const batchSize of batchSizes) {
        const batchStart = performance.now();
        const result = await dbTester.testBatchOperations(
          "createPrompt", 
          bulkPrompts.slice(0, batchSize), 
          Math.min(batchSize, 10)
        );
        const batchDuration = performance.now() - batchStart;
        
        batchResults[batchSize] = {
          duration: batchDuration,
          throughput: (batchSize / batchDuration) * 1000, // operations per second
          successRate: result.results.filter(r => r.success).length / result.results.length * 100
        };
        
        profiler.checkpoint(`batch_${batchSize}_complete`);
      }
      
      profiler.end();
      
      // Performance analysis
      const bestThroughput = Math.max(...Object.values(batchResults).map(r => r.throughput));
      const avgSuccessRate = Object.values(batchResults).reduce((sum, r) => sum + r.successRate, 0) / batchSizes.length;
      
      // Performance assertions
      expect(bestThroughput).toBeGreaterThan(5); // At least 5 operations per second
      expect(avgSuccessRate).toBeGreaterThan(90); // 90% success rate
      
      console.log("📊 Batch Performance Results:");
      for (const [batchSize, results] of Object.entries(batchResults)) {
        console.log(`   Batch ${batchSize}: ${results.throughput.toFixed(2)} ops/sec, ${results.successRate.toFixed(1)}% success`);
      }
      
      profiler.getReport();
    });

    test("should maintain data consistency during concurrent writes", async () => {
      const profiler = new PerformanceProfiler("Data Consistency Test");
      const concurrentTester = new ConcurrentTester(page);
      
      profiler.start();
      
      // Create operations that should result in unique data
      const writeOps = Array.from({ length: 30 }, (_, i) => 
        async (page, index) => {
          const uniqueId = `${Date.now()}-${index}`;
          const result = await page.evaluate(async (data) => {
            const response = await window.apiClient.invokeCommand("create_prompt", {
              title: `Consistency Test ${data.uniqueId}`,
              content: `Unique content for ${data.uniqueId}`,
              tags: `consistency,test,${data.uniqueId}`
            });
            return { id: response, uniqueId: data.uniqueId };
          }, { uniqueId });
          return result;
        }
      );
      
      profiler.checkpoint("consistency_ops_prepared");
      
      // Execute concurrent writes
      const results = await concurrentTester.runConcurrent(writeOps, 15);
      
      profiler.checkpoint("concurrent_writes_complete");
      
      // Verify all operations were successful and unique
      const successfulResults = results.filter(r => r.success);
      const uniqueIds = new Set(successfulResults.map(r => r.result.uniqueId));
      
      // Verify data integrity by reading back
      const allPrompts = await page.evaluate(async () => {
        return await window.apiClient.invokeCommand("list_prompts");
      });
      
      profiler.checkpoint("data_verification_complete");
      profiler.end();
      
      // Data consistency assertions
      expect(successfulResults.length).toBeGreaterThanOrEqual(25); // At least 83% success
      expect(uniqueIds.size).toBe(successfulResults.length); // All results should be unique
      expect(allPrompts.length).toBeGreaterThanOrEqual(successfulResults.length);
      
      // Performance assertions
      const report = concurrentTester.getReport();
      PerformanceAssertions.assertSuccessRate(report.successRate, 80, "concurrent data consistency");
      
      console.log(`✅ Consistency test: ${successfulResults.length} unique operations, ${uniqueIds.size} unique IDs`);
      
      profiler.getReport();
    });
  });

  test.describe("Memory and Resource Performance", () => {
    test("should maintain reasonable memory usage during extended operations", async () => {
      const resourceMonitor = new ResourceMonitor(page);
      const profiler = new PerformanceProfiler("Memory Usage Test");
      
      await resourceMonitor.takeSnapshot("initial_state");
      profiler.start();
      
      // Simulate extended application usage
      const operations = [
        // Create phase
        async () => {
          for (let i = 0; i < 50; i++) {
            await page.evaluate(async (index) => {
              await window.apiClient.invokeCommand("create_prompt", {
                title: `Memory Test ${index}`,
                content: `Content for memory test ${index}`,
                tags: `memory,test,${index}`
              });
            }, i);
          }
          await resourceMonitor.takeSnapshot("after_creation_phase");
        },
        
        // Read phase
        async () => {
          for (let i = 0; i < 20; i++) {
            await page.evaluate(async () => {
              await window.apiClient.invokeCommand("list_prompts");
            });
          }
          await resourceMonitor.takeSnapshot("after_read_phase");
        },
        
        // Mixed operations phase
        async () => {
          for (let i = 0; i < 25; i++) {
            if (i % 3 === 0) {
              await page.evaluate(async (index) => {
                await window.apiClient.invokeCommand("create_prompt", {
                  title: `Mixed Op ${index}`,
                  content: `Mixed operation ${index}`,
                  tags: `mixed,${index}`
                });
              }, i);
            } else {
              await page.evaluate(async () => {
                await window.apiClient.invokeCommand("list_prompts");
              });
            }
          }
          await resourceMonitor.takeSnapshot("after_mixed_phase");
        }
      ];
      
      // Execute operations sequentially
      for (let i = 0; i < operations.length; i++) {
        await operations[i]();
        profiler.checkpoint(`operation_phase_${i + 1}_complete`);
      }
      
      await resourceMonitor.takeSnapshot("final_state");
      profiler.end();
      
      // Analyze memory usage
      const memoryGrowth = resourceMonitor.getMemoryGrowth();
      const resourceReport = resourceMonitor.getReport();
      
      if (memoryGrowth) {
        const growthMB = memoryGrowth.usedJSHeapGrowth / 1024 / 1024;
        console.log(`📊 Memory growth: ${growthMB.toFixed(2)}MB over ${memoryGrowth.timeSpan}ms`);
        
        // Memory should not grow excessively (allow up to 100MB growth)
        PerformanceAssertions.assertMemoryUsage(memoryGrowth, 100, "extended operations");
        
        // Growth rate should be reasonable
        const growthRate = growthMB / (memoryGrowth.timeSpan / 1000); // MB per second
        expect(growthRate).toBeLessThan(5); // Less than 5MB per second growth
      }
      
      profiler.getReport();
      resourceReport;
    });

    test("should handle stress testing gracefully", async () => {
      const profiler = new PerformanceProfiler("Stress Test");
      const concurrentTester = new ConcurrentTester(page);
      const resourceMonitor = new ResourceMonitor(page);
      
      await resourceMonitor.takeSnapshot("before_stress");
      profiler.start();
      
      // Create a high-stress scenario with many concurrent operations
      const stressOps = Array.from({ length: 100 }, (_, i) => 
        async (page, index) => {
          const operationType = index % 4;
          
          switch (operationType) {
            case 0: // Create
              return await page.evaluate(async (idx) => {
                return await window.apiClient.invokeCommand("create_prompt", {
                  title: `Stress ${idx}`,
                  content: `Stress content ${idx}`,
                  tags: `stress,${idx}`
                });
              }, index);
            
            case 1: // List
              return await page.evaluate(async () => {
                return await window.apiClient.invokeCommand("list_prompts");
              });
            
            case 2: // Get stats
              return await page.evaluate(async () => {
                return await window.apiClient.invokeCommand("get_token_usage_stats");
              });
            
            case 3: // Get schedules
              return await page.evaluate(async () => {
                return await window.apiClient.invokeCommand("get_pending_schedules");
              });
            
            default:
              return null;
          }
        }
      );
      
      profiler.checkpoint("stress_ops_prepared");
      
      // Execute with higher concurrency
      const results = await concurrentTester.runConcurrent(stressOps, 25);
      
      profiler.checkpoint("stress_execution_complete");
      await resourceMonitor.takeSnapshot("after_stress");
      profiler.end();
      
      // Stress test analysis
      const report = concurrentTester.getReport();
      const perfReport = profiler.getReport();
      
      // Under stress, we expect some degradation but system should remain functional
      expect(report.successRate).toBeGreaterThanOrEqual(70); // 70% success under stress
      expect(report.totalOperations).toBe(100);
      
      // System should complete within reasonable time even under stress
      PerformanceAssertions.assertResponseTime(perfReport.totalDuration, 60000, "stress test completion");
      
      console.log(`🔥 Stress test completed: ${report.successRate.toFixed(1)}% success rate under high load`);
      console.log(`🔥 Average operation time under stress: ${report.avgTime.toFixed(2)}ms`);
      
      // Check memory stability under stress
      const memoryGrowth = resourceMonitor.getMemoryGrowth();
      if (memoryGrowth) {
        PerformanceAssertions.assertMemoryUsage(memoryGrowth, 150, "stress test");
      }
      
      profiler.getReport();
    });
  });

  test.describe("Query Optimization Tests", () => {
    test("should optimize complex queries efficiently", async () => {
      const dbTester = new DatabasePerformanceTester(page);
      const profiler = new PerformanceProfiler("Query Optimization");
      
      profiler.start();
      
      // Create diverse test data for complex queries
      const complexPrompts = [];
      for (let i = 0; i < 200; i++) {
        complexPrompts.push({
          title: `Complex Query Test ${i}`,
          content: `Complex content with keywords: ${i % 10 === 0 ? 'special' : 'normal'} test data ${i}`,
          tags: `complex,test,${i % 5},${i % 3 === 0 ? 'priority' : 'standard'}`
        });
      }
      
      // Create test data in batches
      const batchResult = await dbTester.testBatchOperations("createPrompt", complexPrompts, 25);
      profiler.checkpoint("complex_data_created");
      
      // Test various query patterns
      const queryTests = [
        { name: "listPrompts", iterations: 10 },
        { name: "getPendingSchedules", iterations: 5 },
        { name: "getTokenUsageStats", iterations: 15 }
      ];
      
      const queryResults = {};
      
      for (const queryTest of queryTests) {
        const result = await dbTester.measureQueryPerformance(
          queryTest.name, 
          {}, 
          queryTest.iterations
        );
        queryResults[queryTest.name] = result;
        profiler.checkpoint(`${queryTest.name}_queries_complete`);
      }
      
      profiler.end();
      
      // Query performance assertions
      for (const [queryName, result] of Object.entries(queryResults)) {
        expect(result.successfulIterations).toBeGreaterThanOrEqual(result.iterations * 0.8); // 80% success
        
        // Different query types have different performance expectations
        const maxTime = queryName === "listPrompts" ? 1000 : 500;
        PerformanceAssertions.assertResponseTime(result.avgDuration, maxTime, `${queryName} query`);
      }
      
      console.log("📊 Query Performance Summary:");
      for (const [name, result] of Object.entries(queryResults)) {
        console.log(`   ${name}: ${result.avgDuration.toFixed(2)}ms avg, ${result.successfulIterations}/${result.iterations} successful`);
      }
      
      profiler.getReport();
    });
  });
});