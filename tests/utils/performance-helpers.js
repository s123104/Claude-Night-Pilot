// Performance testing utilities for Claude Night Pilot
// Optimized helpers for concurrent testing and performance measurement

import { expect } from "@playwright/test";

/**
 * Performance measurement utilities
 */
export class PerformanceProfiler {
  constructor(testName) {
    this.testName = testName;
    this.startTime = null;
    this.endTime = null;
    this.checkpoints = new Map();
  }

  start() {
    this.startTime = performance.now();
    return this;
  }

  checkpoint(name) {
    if (!this.startTime) {
      throw new Error('Must call start() before checkpoint()');
    }
    this.checkpoints.set(name, performance.now() - this.startTime);
    return this;
  }

  end() {
    if (!this.startTime) {
      throw new Error('Must call start() before end()');
    }
    this.endTime = performance.now();
    return this;
  }

  getDuration() {
    if (!this.startTime || !this.endTime) {
      throw new Error('Must call start() and end() before getDuration()');
    }
    return this.endTime - this.startTime;
  }

  getReport() {
    const totalDuration = this.getDuration();
    const report = {
      testName: this.testName,
      totalDuration,
      checkpoints: Object.fromEntries(this.checkpoints),
    };
    
    console.log(`⏱️  Performance Report for ${this.testName}:`);
    console.log(`   Total Duration: ${totalDuration.toFixed(2)}ms`);
    
    for (const [name, time] of this.checkpoints) {
      console.log(`   ${name}: ${time.toFixed(2)}ms`);
    }
    
    return report;
  }

  assertPerformance(maxDuration, checkpointLimits = {}) {
    const totalDuration = this.getDuration();
    
    expect(totalDuration).toBeLessThan(maxDuration);
    
    for (const [name, limit] of Object.entries(checkpointLimits)) {
      const checkpointTime = this.checkpoints.get(name);
      if (checkpointTime !== undefined) {
        expect(checkpointTime).toBeLessThan(limit);
      }
    }
    
    return this;
  }
}

/**
 * Concurrent operation utilities
 */
export class ConcurrentTester {
  constructor(page) {
    this.page = page;
    this.results = [];
  }

  async runConcurrent(operations, concurrencyLevel = 5) {
    const batches = [];
    for (let i = 0; i < operations.length; i += concurrencyLevel) {
      batches.push(operations.slice(i, i + concurrencyLevel));
    }

    for (const batch of batches) {
      const promises = batch.map(async (operation, index) => {
        const start = performance.now();
        try {
          const result = await operation(this.page, index);
          const duration = performance.now() - start;
          return { success: true, result, duration, operation: operation.name || `operation_${index}` };
        } catch (error) {
          const duration = performance.now() - start;
          return { success: false, error: error.message, duration, operation: operation.name || `operation_${index}` };
        }
      });

      const batchResults = await Promise.all(promises);
      this.results.push(...batchResults);
    }

    return this.results;
  }

  getSuccessRate() {
    const successful = this.results.filter(r => r.success).length;
    return (successful / this.results.length) * 100;
  }

  getAverageOperationTime() {
    const total = this.results.reduce((sum, r) => sum + r.duration, 0);
    return total / this.results.length;
  }

  getReport() {
    const successRate = this.getSuccessRate();
    const avgTime = this.getAverageOperationTime();
    const totalOperations = this.results.length;
    const failedOperations = this.results.filter(r => !r.success);

    const report = {
      totalOperations,
      successRate,
      avgTime,
      failedOperations: failedOperations.length,
      details: this.results,
    };

    console.log(`🔄 Concurrent Operations Report:`);
    console.log(`   Total Operations: ${totalOperations}`);
    console.log(`   Success Rate: ${successRate.toFixed(1)}%`);
    console.log(`   Average Time: ${avgTime.toFixed(2)}ms`);
    console.log(`   Failed Operations: ${failedOperations.length}`);

    if (failedOperations.length > 0) {
      console.log(`   Failures:`);
      failedOperations.forEach(op => {
        console.log(`     - ${op.operation}: ${op.error}`);
      });
    }

    return report;
  }
}

/**
 * Database performance testing utilities
 */
export class DatabasePerformanceTester {
  constructor(page) {
    this.page = page;
  }

  async testBatchOperations(operationType, data, batchSize = 10) {
    const profiler = new PerformanceProfiler(`Batch ${operationType}`);
    profiler.start();

    const results = [];
    const batches = [];
    for (let i = 0; i < data.length; i += batchSize) {
      batches.push(data.slice(i, i + batchSize));
    }

    profiler.checkpoint('batch_preparation');

    for (let i = 0; i < batches.length; i++) {
      const batch = batches[i];
      const batchStart = performance.now();
      
      try {
        const batchResults = await this.executeBatch(operationType, batch);
        results.push(...batchResults);
        
        profiler.checkpoint(`batch_${i + 1}_completed`);
      } catch (error) {
        console.error(`Batch ${i + 1} failed:`, error);
        profiler.checkpoint(`batch_${i + 1}_failed`);
      }
    }

    profiler.end();
    const report = profiler.getReport();
    
    return {
      results,
      performance: report,
      batchCount: batches.length,
      totalItems: data.length,
    };
  }

  async executeBatch(operationType, batch) {
    return await this.page.evaluate(async (args) => {
      const { operationType, batch } = args;
      const results = [];

      for (const item of batch) {
        try {
          let result;
          switch (operationType) {
            case 'createPrompt':
              result = await window.apiClient.invokeCommand("create_prompt", item);
              break;
            case 'createSchedule':
              result = await window.apiClient.invokeCommand("create_schedule", item);
              break;
            case 'getPrompt':
              result = await window.apiClient.invokeCommand("get_prompt", { id: item.id });
              break;
            default:
              throw new Error(`Unknown operation type: ${operationType}`);
          }
          results.push({ success: true, result });
        } catch (error) {
          results.push({ success: false, error: error.message });
        }
      }

      return results;
    }, { operationType, batch });
  }

  async measureQueryPerformance(queryName, queryParams = {}, iterations = 5) {
    const profiler = new PerformanceProfiler(`Query: ${queryName}`);
    const results = [];

    profiler.start();

    for (let i = 0; i < iterations; i++) {
      const iterationStart = performance.now();
      
      try {
        const result = await this.page.evaluate(async (args) => {
          const { queryName, queryParams } = args;
          const start = performance.now();
          
          let queryResult;
          switch (queryName) {
            case 'listPrompts':
              queryResult = await window.apiClient.invokeCommand("list_prompts");
              break;
            case 'getPendingSchedules':
              queryResult = await window.apiClient.invokeCommand("get_pending_schedules");
              break;
            case 'getTokenUsageStats':
              queryResult = await window.apiClient.invokeCommand("get_token_usage_stats");
              break;
            default:
              throw new Error(`Unknown query: ${queryName}`);
          }
          
          const duration = performance.now() - start;
          return { queryResult, duration };
        }, { queryName, queryParams });

        results.push({
          iteration: i + 1,
          duration: result.duration,
          resultSize: Array.isArray(result.queryResult) ? result.queryResult.length : 1,
          success: true,
        });

        profiler.checkpoint(`iteration_${i + 1}`);
      } catch (error) {
        results.push({
          iteration: i + 1,
          error: error.message,
          success: false,
        });
        profiler.checkpoint(`iteration_${i + 1}_failed`);
      }
    }

    profiler.end();

    const successfulResults = results.filter(r => r.success);
    const avgDuration = successfulResults.length > 0 
      ? successfulResults.reduce((sum, r) => sum + r.duration, 0) / successfulResults.length
      : 0;

    const report = {
      queryName,
      iterations,
      successfulIterations: successfulResults.length,
      avgDuration,
      minDuration: Math.min(...successfulResults.map(r => r.duration)),
      maxDuration: Math.max(...successfulResults.map(r => r.duration)),
      results,
      performance: profiler.getReport(),
    };

    console.log(`📊 Query Performance Report for ${queryName}:`);
    console.log(`   Successful iterations: ${successfulResults.length}/${iterations}`);
    console.log(`   Average duration: ${avgDuration.toFixed(2)}ms`);
    console.log(`   Min duration: ${report.minDuration.toFixed(2)}ms`);
    console.log(`   Max duration: ${report.maxDuration.toFixed(2)}ms`);

    return report;
  }
}

/**
 * Memory and resource monitoring utilities
 */
export class ResourceMonitor {
  constructor(page) {
    this.page = page;
    this.snapshots = [];
  }

  async takeSnapshot(label = '') {
    const snapshot = await this.page.evaluate(() => {
      return {
        timestamp: Date.now(),
        memory: performance.memory ? {
          usedJSHeapSize: performance.memory.usedJSHeapSize,
          totalJSHeapSize: performance.memory.totalJSHeapSize,
          jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
        } : null,
        timing: performance.timing,
        navigation: performance.navigation,
      };
    });

    snapshot.label = label;
    this.snapshots.push(snapshot);
    return snapshot;
  }

  getMemoryGrowth() {
    if (this.snapshots.length < 2) {
      return null;
    }

    const first = this.snapshots[0];
    const last = this.snapshots[this.snapshots.length - 1];

    if (!first.memory || !last.memory) {
      return null;
    }

    return {
      usedJSHeapGrowth: last.memory.usedJSHeapSize - first.memory.usedJSHeapSize,
      totalJSHeapGrowth: last.memory.totalJSHeapSize - first.memory.totalJSHeapSize,
      timeSpan: last.timestamp - first.timestamp,
    };
  }

  getReport() {
    const memoryGrowth = this.getMemoryGrowth();
    
    const report = {
      snapshotCount: this.snapshots.length,
      memoryGrowth,
      snapshots: this.snapshots,
    };

    console.log(`🔍 Resource Monitor Report:`);
    console.log(`   Snapshots taken: ${this.snapshots.length}`);
    
    if (memoryGrowth) {
      console.log(`   Memory growth: ${(memoryGrowth.usedJSHeapGrowth / 1024 / 1024).toFixed(2)} MB over ${memoryGrowth.timeSpan}ms`);
    }

    return report;
  }
}

/**
 * Test data generators for performance testing
 */
export class TestDataGenerator {
  static generatePrompts(count, options = {}) {
    const prompts = [];
    const { 
      titlePrefix = 'Performance Test Prompt',
      contentPrefix = 'This is test content for performance testing',
      includeTags = true,
      tagPrefix = 'perf,test'
    } = options;

    for (let i = 1; i <= count; i++) {
      prompts.push({
        title: `${titlePrefix} ${i}`,
        content: `${contentPrefix} ${i}`,
        tags: includeTags ? `${tagPrefix},${i}` : null,
      });
    }

    return prompts;
  }

  static generateSchedules(promptIds, options = {}) {
    const schedules = [];
    const { 
      scheduleTimeBase = new Date(Date.now() + 3600000), // 1 hour from now
      cronExpr = '0 */1 * * *',
    } = options;

    promptIds.forEach((promptId, index) => {
      const scheduleTime = new Date(scheduleTimeBase.getTime() + (index * 60000)); // 1 minute apart
      schedules.push({
        promptId: promptId,
        scheduleTime: scheduleTime.toISOString(),
        cronExpr: cronExpr,
      });
    });

    return schedules;
  }
}

/**
 * Assertion helpers for performance testing
 */
export class PerformanceAssertions {
  static assertResponseTime(duration, maxTime, operationName = 'operation') {
    expect(duration).toBeLessThan(maxTime);
    console.log(`✅ ${operationName} completed in ${duration.toFixed(2)}ms (limit: ${maxTime}ms)`);
  }

  static assertSuccessRate(successRate, minRate, operationName = 'operation') {
    expect(successRate).toBeGreaterThanOrEqual(minRate);
    console.log(`✅ ${operationName} success rate: ${successRate.toFixed(1)}% (minimum: ${minRate}%)`);
  }

  static assertMemoryUsage(memoryGrowth, maxGrowthMB, operationName = 'operation') {
    if (!memoryGrowth) {
      console.log(`⚠️  Memory growth data not available for ${operationName}`);
      return;
    }

    const growthMB = memoryGrowth.usedJSHeapGrowth / 1024 / 1024;
    expect(growthMB).toBeLessThan(maxGrowthMB);
    console.log(`✅ ${operationName} memory growth: ${growthMB.toFixed(2)}MB (limit: ${maxGrowthMB}MB)`);
  }

  static assertConcurrentOperations(results, minSuccessRate = 95, maxAvgTime = 1000) {
    const tester = { results };
    const successRate = (results.filter(r => r.success).length / results.length) * 100;
    const avgTime = results.reduce((sum, r) => sum + r.duration, 0) / results.length;

    this.assertSuccessRate(successRate, minSuccessRate, 'concurrent operations');
    this.assertResponseTime(avgTime, maxAvgTime, 'average concurrent operation');
  }
}