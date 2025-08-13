#!/usr/bin/env node

/**
 * Claude Night Pilot - 專案分析啟動器
 * 一鍵啟動完整的專案分析流程
 */

import ProjectAnalysisOrchestrator from "./analysis/run-analysis.js";

console.log(`
╔══════════════════════════════════════════════════════════════╗
║                Claude Night Pilot                            ║
║              專案分析與重構系統                                ║
║                                                              ║
║  🎯 目標: 建立乾淨整潔專業的開源專案架構                        ║
║  🔧 方法: 多個 Claude Code sessions 並行分析                   ║
║  📊 輸出: 完整的分析報告和實施計劃                              ║
╚══════════════════════════════════════════════════════════════╝

🚀 啟動專案分析...
`);

async function main() {
  try {
    const orchestrator = new ProjectAnalysisOrchestrator();
    await orchestrator.run();

    console.log(`
🎉 專案分析完成！

📋 查看結果:
- analysis/ANALYSIS_COMPLETE.md - 完成摘要
- analysis/reports/ - 詳細分析報告
- analysis/logs/ - 執行日誌

🚀 下一步:
1. 查看 analysis/reports/implementation-plan.json
2. 按階段執行改進措施
3. 驗證改進效果
`);
  } catch (error) {
    console.error(`
❌ 分析失敗: ${error.message}

🔍 檢查日誌: analysis/logs/orchestrator.log
💡 建議: 確保所有依賴已安裝並且專案結構完整
`);
    process.exit(1);
  }
}

main();
