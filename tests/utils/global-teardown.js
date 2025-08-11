/**
 * 全域測試清理
 * 
 * 在所有測試完成後執行的清理作業
 */

import { cleanupTestDatabase } from './db-setup.js';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function globalTeardown(config) {
  console.log('🧹 開始全域測試清理...');
  
  try {
    // 1. 清理測試資料庫
    console.log('📦 清理測試資料庫...');
    const dbCleaned = await cleanupTestDatabase();
    if (!dbCleaned) {
      console.warn('⚠️ 測試資料庫清理失敗');
    }
    
    // 2. 生成測試摘要報告
    console.log('📊 生成測試摘要報告...');
    await generateTestSummary();
    
    // 3. 清理臨時檔案
    console.log('🗑️ 清理臨時檔案...');
    await cleanupTemporaryFiles();
    
    // 4. 保存測試指標
    console.log('📈 保存測試指標...');
    await saveTestMetrics();
    
    console.log('✅ 全域測試清理完成');
    
  } catch (error) {
    console.error('❌ 全域測試清理失敗:', error);
    // 不要因為清理失敗而導致測試失敗
  }
}

/**
 * 生成測試摘要報告
 */
async function generateTestSummary() {
  try {
    const fs = await import('fs/promises');
    
    // 讀取測試結果 JSON（如果存在）
    const resultsPath = path.join(__dirname, '../../coverage/test-results.json');
    
    let testResults = null;
    try {
      const resultsContent = await fs.readFile(resultsPath, 'utf8');
      testResults = JSON.parse(resultsContent);
    } catch (error) {
      console.log('  ℹ️ 未找到測試結果檔案，跳過摘要生成');
      return;
    }
    
    // 分析測試結果
    const summary = analyzeTestResults(testResults);
    
    // 生成摘要報告
    const summaryReport = generateSummaryReport(summary);
    
    // 保存摘要報告
    const summaryPath = path.join(__dirname, '../../coverage/test-summary.md');
    await fs.writeFile(summaryPath, summaryReport, 'utf8');
    
    console.log('  📄 測試摘要報告已生成:', summaryPath);
    
    // 輸出關鍵指標到控制台
    console.log(`  📊 測試統計: 通過 ${summary.passed}, 失敗 ${summary.failed}, 跳過 ${summary.skipped}`);
    console.log(`  ⏱️ 總執行時間: ${summary.duration}ms`);
    console.log(`  📈 成功率: ${summary.passRate.toFixed(1)}%`);
    
  } catch (error) {
    console.warn('  ⚠️ 生成測試摘要失敗:', error.message);
  }
}

/**
 * 分析測試結果
 */
function analyzeTestResults(results) {
  const summary = {
    total: 0,
    passed: 0,
    failed: 0,
    skipped: 0,
    duration: 0,
    passRate: 0,
    categories: {
      'e2e-gui': { passed: 0, failed: 0, duration: 0 },
      'e2e-cli': { passed: 0, failed: 0, duration: 0 },
      'integration': { passed: 0, failed: 0, duration: 0 },
      'cross-platform': { passed: 0, failed: 0, duration: 0 }
    }
  };
  
  if (!results.suites) return summary;
  
  // 遞歸分析測試套件
  function analyzeSuite(suite) {
    suite.tests?.forEach(test => {
      summary.total++;
      summary.duration += test.duration || 0;
      
      switch (test.outcome) {
        case 'passed':
          summary.passed++;
          break;
        case 'failed':
          summary.failed++;
          break;
        case 'skipped':
          summary.skipped++;
          break;
      }
      
      // 分類統計
      const category = getCategoryFromTitle(suite.title);
      if (summary.categories[category]) {
        if (test.outcome === 'passed') {
          summary.categories[category].passed++;
        } else if (test.outcome === 'failed') {
          summary.categories[category].failed++;
        }
        summary.categories[category].duration += test.duration || 0;
      }
    });
    
    suite.suites?.forEach(analyzeSuite);
  }
  
  results.suites.forEach(analyzeSuite);
  
  summary.passRate = summary.total > 0 ? (summary.passed / summary.total) * 100 : 0;
  
  return summary;
}

/**
 * 從測試標題判斷分類
 */
function getCategoryFromTitle(title) {
  if (title.includes('GUI') || title.includes('Material Design')) {
    return 'e2e-gui';
  } else if (title.includes('CLI')) {
    return 'e2e-cli';
  } else if (title.includes('整合') || title.includes('Integration')) {
    return 'integration';
  } else if (title.includes('跨平台') || title.includes('Cross Platform')) {
    return 'cross-platform';
  }
  return 'other';
}

/**
 * 生成摘要報告 Markdown
 */
function generateSummaryReport(summary) {
  const timestamp = new Date().toISOString();
  
  return `# Claude Night Pilot 測試摘要報告

生成時間: ${timestamp}

## 📊 整體統計

- **總測試數**: ${summary.total}
- **通過**: ${summary.passed} ✅
- **失敗**: ${summary.failed} ❌
- **跳過**: ${summary.skipped} ⏭️
- **成功率**: ${summary.passRate.toFixed(1)}%
- **總執行時間**: ${(summary.duration / 1000).toFixed(2)}s

## 🎯 分類統計

| 分類 | 通過 | 失敗 | 執行時間 | 成功率 |
|------|------|------|----------|--------|
${Object.entries(summary.categories).map(([category, stats]) => {
  const total = stats.passed + stats.failed;
  const rate = total > 0 ? ((stats.passed / total) * 100).toFixed(1) : 0;
  return `| ${category} | ${stats.passed} | ${stats.failed} | ${(stats.duration / 1000).toFixed(2)}s | ${rate}% |`;
}).join('\n')}

## 🏗️ 測試架構

測試檔案已重構至以下結構：

\`\`\`
tests/
├── e2e/
│   ├── gui/          # GUI 功能測試
│   ├── cli/          # CLI 功能測試
│   └── cross-platform/ # 跨平台整合測試
├── integration/      # 整合測試
├── fixtures/         # 測試夾具和資料
├── utils/           # 共享測試工具
└── demos/           # 演示和除錯測試
\`\`\`

## 💡 建議

${summary.passRate >= 95 ? 
  '🎉 測試覆蓋率優秀！繼續保持高品質的測試實踐。' : 
  summary.passRate >= 80 ? 
    '👍 測試覆蓋率良好，可以考慮增加更多邊界情況的測試。' : 
    '⚠️ 測試覆蓋率需要改進，建議增加更多測試案例。'
}

${summary.failed > 0 ? 
  `\n⚠️ 有 ${summary.failed} 個測試失敗，請檢查詳細的測試報告。` : 
  '\n✅ 所有測試都通過了！'
}

---
*由 Claude Night Pilot 測試架構自動生成*
`;
}

/**
 * 清理臨時檔案
 */
async function cleanupTemporaryFiles() {
  try {
    const fs = await import('fs/promises');
    
    const tempPatterns = [
      '**/*.tmp',
      '**/*.temp',
      '**/test-*.db',
      '**/debug-*.log'
    ];
    
    const glob = await import('glob');
    
    for (const pattern of tempPatterns) {
      try {
        const files = await glob.glob(pattern, { 
          cwd: path.join(__dirname, '../..'),
          absolute: true 
        });
        
        for (const file of files) {
          await fs.unlink(file);
          console.log(`  🗑️ 已刪除臨時檔案: ${path.basename(file)}`);
        }
      } catch (error) {
        // 忽略不存在的檔案
      }
    }
    
  } catch (error) {
    console.warn('  ⚠️ 清理臨時檔案失敗:', error.message);
  }
}

/**
 * 保存測試指標
 */
async function saveTestMetrics() {
  try {
    const fs = await import('fs/promises');
    
    const metrics = {
      timestamp: new Date().toISOString(),
      platform: process.platform,
      nodeVersion: process.version,
      testDuration: process.hrtime.bigint(),
      memoryUsage: process.memoryUsage(),
      environmentVariables: {
        TEST_MODE: process.env.TEST_MODE,
        USE_MOCK_CLI: process.env.USE_MOCK_CLI,
        CI: process.env.CI
      }
    };
    
    const metricsPath = path.join(__dirname, '../../coverage/test-metrics.json');
    await fs.writeFile(metricsPath, JSON.stringify(metrics, null, 2), 'utf8');
    
    console.log('  📈 測試指標已保存');
    
  } catch (error) {
    console.warn('  ⚠️ 保存測試指標失敗:', error.message);
  }
}

export default globalTeardown;