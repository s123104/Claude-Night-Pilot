#!/usr/bin/env node

/**
 * Claude Night Pilot - 立即執行專案分析
 */

import fs from "fs";
import path from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function runImmediateAnalysis() {
  console.log(`
╔══════════════════════════════════════════════════════════════╗
║                Claude Night Pilot                            ║
║              立即專案分析執行中...                            ║
╚══════════════════════════════════════════════════════════════╝
`);

  const reportDir = path.join(process.cwd(), "analysis", "reports");
  fs.mkdirSync(reportDir, { recursive: true });

  const startTime = Date.now();
  const results = {
    timestamp: new Date().toISOString(),
    cli_analysis: {},
    file_analysis: {},
    architecture_analysis: {},
    technical_debt: {},
    summary: {},
  };

  try {
    // 1. CLI 功能分析和測試
    console.log("🔍 1. CLI 功能分析...");
    results.cli_analysis = await analyzeCLI();

    // 2. 檔案結構分析
    console.log("📁 2. 檔案結構分析...");
    results.file_analysis = await analyzeFiles();

    // 3. 架構分析
    console.log("🏗️  3. 架構分析...");
    results.architecture_analysis = await analyzeArchitecture();

    // 4. 技術債務分析
    console.log("🔧 4. 技術債務分析...");
    results.technical_debt = await analyzeTechnicalDebt();

    // 5. 生成摘要和建議
    console.log("📊 5. 生成分析摘要...");
    results.summary = generateSummary(results);

    // 保存完整結果
    const reportFile = path.join(reportDir, "immediate-analysis-report.json");
    fs.writeFileSync(reportFile, JSON.stringify(results, null, 2));

    // 生成人類可讀報告
    await generateReadableReport(results, reportDir);

    const duration = Math.round((Date.now() - startTime) / 1000);
    console.log(`
🎉 專案分析完成！

⏱️  執行時間: ${duration} 秒
📊 分析結果: analysis/reports/immediate-analysis-report.json
📄 摘要報告: analysis/reports/IMMEDIATE_ANALYSIS_SUMMARY.md

🚀 下一步:
1. 查看分析報告了解詳細結果
2. 根據建議優先處理高優先級問題
3. 實施架構改進和代碼清理
`);
  } catch (error) {
    console.error("❌ 分析執行失敗:", error.message);

    // 保存錯誤報告
    const errorReport = {
      timestamp: new Date().toISOString(),
      error: error.message,
      stack: error.stack,
      partialResults: results,
    };

    const errorFile = path.join(reportDir, "analysis-error.json");
    fs.writeFileSync(errorFile, JSON.stringify(errorReport, null, 2));

    throw error;
  }
}

async function analyzeCLI() {
  const analysis = {
    timestamp: new Date().toISOString(),
    binaries: {},
    tests: [],
    recommendations: [],
  };

  try {
    // 測試 cnp-optimized 指令
    console.log("  🧪 測試 cnp-optimized 指令...");

    const optimizedTests = [
      { cmd: "status", expected: "json" },
      { cmd: "health --format json", expected: "json" },
      { cmd: "cooldown", expected: "text" },
    ];

    analysis.binaries["cnp-optimized"] = {
      available: true,
      tests: [],
    };

    for (const test of optimizedTests) {
      try {
        const { stdout, stderr } = await execAsync(
          `npm run cli:optimized -- ${test.cmd}`,
          { timeout: 30000 }
        );

        const result = {
          command: test.cmd,
          status: "passed",
          output: stdout.substring(0, 200),
          isJson: test.expected === "json" ? isValidJSON(stdout) : null,
        };

        analysis.binaries["cnp-optimized"].tests.push(result);
        analysis.tests.push(result);
      } catch (error) {
        const result = {
          command: test.cmd,
          status: "failed",
          error: error.message.substring(0, 200),
        };

        analysis.binaries["cnp-optimized"].tests.push(result);
        analysis.tests.push(result);
      }
    }

    // 測試 cnp-unified 指令
    console.log("  🧪 測試 cnp-unified 指令...");

    const unifiedTests = [
      { cmd: "status", expected: "text" },
      { cmd: "prompt list", expected: "text" },
    ];

    analysis.binaries["cnp-unified"] = {
      available: true,
      tests: [],
    };

    for (const test of unifiedTests) {
      try {
        const { stdout, stderr } = await execAsync(
          `npm run cli -- ${test.cmd}`,
          { timeout: 30000 }
        );

        const result = {
          command: test.cmd,
          status: "passed",
          output: stdout.substring(0, 200),
        };

        analysis.binaries["cnp-unified"].tests.push(result);
        analysis.tests.push(result);
      } catch (error) {
        const result = {
          command: test.cmd,
          status: "failed",
          error: error.message.substring(0, 200),
        };

        analysis.binaries["cnp-unified"].tests.push(result);
        analysis.tests.push(result);
      }
    }

    // 生成 CLI 建議
    const passedTests = analysis.tests.filter(
      (t) => t.status === "passed"
    ).length;
    const totalTests = analysis.tests.length;

    if (passedTests < totalTests) {
      analysis.recommendations.push({
        type: "cli_reliability",
        priority: "high",
        title: "Fix failing CLI commands",
        description: `${
          totalTests - passedTests
        } out of ${totalTests} CLI tests failed`,
        action: "Debug and fix failing CLI commands",
      });
    }

    analysis.recommendations.push({
      type: "cli_testing",
      priority: "medium",
      title: "Implement comprehensive CLI testing",
      description: "Add BDD tests for all CLI commands",
      action: "Create Playwright tests for CLI functionality",
    });
  } catch (error) {
    analysis.error = error.message;
  }

  return analysis;
}

async function analyzeFiles() {
  const analysis = {
    timestamp: new Date().toISOString(),
    structure: {},
    issues: [],
    recommendations: [],
  };

  try {
    // 分析專案結構
    console.log("  📋 分析專案結構...");

    const { stdout: allFiles } = await execAsync(
      'find . -type f -name "*.rs" -o -name "*.js" -o -name "*.md" -o -name "*.json" | grep -v node_modules | grep -v target | head -50'
    );

    const files = allFiles
      .trim()
      .split("\n")
      .filter((f) => f);
    analysis.structure.totalFiles = files.length;

    // 按類型分類
    analysis.structure.byType = {
      rust: files.filter((f) => f.endsWith(".rs")).length,
      javascript: files.filter((f) => f.endsWith(".js")).length,
      markdown: files.filter((f) => f.endsWith(".md")).length,
      json: files.filter((f) => f.endsWith(".json")).length,
    };

    // 查找可能的問題檔案
    console.log("  🔍 查找問題檔案...");

    try {
      const { stdout: tempFiles } = await execAsync(
        'find . -name "*temp*" -o -name "*backup*" -o -name "*old*" | grep -v node_modules | grep -v target'
      );

      if (tempFiles.trim()) {
        analysis.issues.push({
          type: "temporary_files",
          files: tempFiles.trim().split("\n"),
          description: "Found temporary or backup files that might be obsolete",
        });
      }
    } catch (error) {
      // 沒有找到臨時檔案，這是好事
    }

    // 查找空目錄
    try {
      const { stdout: emptyDirs } = await execAsync(
        "find . -type d -empty | grep -v node_modules | grep -v target | head -10"
      );

      if (emptyDirs.trim()) {
        analysis.issues.push({
          type: "empty_directories",
          directories: emptyDirs.trim().split("\n"),
          description: "Found empty directories that could be removed",
        });
      }
    } catch (error) {
      // 沒有空目錄
    }

    // 生成檔案建議
    if (analysis.issues.length > 0) {
      analysis.recommendations.push({
        type: "file_cleanup",
        priority: "low",
        title: "Clean up obsolete files and directories",
        description: `Found ${analysis.issues.length} types of cleanup opportunities`,
        action: "Review and remove unnecessary files and empty directories",
      });
    }

    analysis.recommendations.push({
      type: "file_organization",
      priority: "medium",
      title: "Improve file organization",
      description:
        "Consider organizing files into more logical directory structures",
      action: "Review and reorganize file structure based on functionality",
    });
  } catch (error) {
    analysis.error = error.message;
  }

  return analysis;
}
async function analyzeArchitecture() {
  const analysis = {
    timestamp: new Date().toISOString(),
    current_structure: {},
    vibe_kanban_comparison: {},
    recommendations: [],
  };

  try {
    // 分析當前架構
    console.log("  🏗️  分析當前架構...");

    const { stdout: rustModules } = await execAsync(
      'find src-tauri/src -name "*.rs" | head -20'
    );

    const { stdout: jsModules } = await execAsync(
      'find src -name "*.js" -o -name "*.html" -o -name "*.css" | head -20'
    );

    analysis.current_structure = {
      backend: {
        modules: rustModules
          .trim()
          .split("\n")
          .filter((f) => f),
        hasCore: rustModules.includes("core"),
        hasAPI: rustModules.includes("api"),
        hasDB: rustModules.includes("db"),
      },
      frontend: {
        modules: jsModules
          .trim()
          .split("\n")
          .filter((f) => f),
        hasComponents: jsModules.includes("components"),
        hasServices: jsModules.includes("services"),
      },
    };

    // 與 Vibe-Kanban 比較
    console.log("  🔄 與 Vibe-Kanban 架構比較...");

    try {
      const { stdout: vibeFiles } = await execAsync(
        'find research-projects/vibe-kanban -name "*.rs" | head -20'
      );

      analysis.vibe_kanban_comparison = {
        hasExecutors: vibeFiles.includes("executors"),
        hasAPI: vibeFiles.includes("api"),
        hasDB: vibeFiles.includes("db"),
        similarities: [
          "Both use Rust backend",
          "Both use SQLite database",
          "Both have CLI interfaces",
        ],
        differences: [
          "Vibe-Kanban uses Axum web framework",
          "Vibe-Kanban has executor pattern",
          "Vibe-Kanban has React frontend",
        ],
      };

      // 基於比較生成建議
      if (vibeFiles.includes("executors")) {
        analysis.recommendations.push({
          type: "architecture_pattern",
          priority: "high",
          title: "Implement Executor Pattern",
          description:
            "Adopt the executor pattern from Vibe-Kanban for better modularity",
          action:
            "Create src-tauri/src/executors/ directory and implement executor traits",
        });
      }

      if (vibeFiles.includes("api")) {
        analysis.recommendations.push({
          type: "api_structure",
          priority: "medium",
          title: "Implement structured API layer",
          description: "Create dedicated API module like Vibe-Kanban",
          action: "Create src-tauri/src/api/ directory for API endpoints",
        });
      }
    } catch (error) {
      analysis.vibe_kanban_comparison.error = error.message;
    }

    // 通用架構建議
    analysis.recommendations.push({
      type: "modularization",
      priority: "high",
      title: "Improve code modularization",
      description: "Break down large modules into smaller, focused components",
      action:
        "Refactor large files into smaller, single-responsibility modules",
    });

    analysis.recommendations.push({
      type: "service_layer",
      priority: "medium",
      title: "Implement service layer",
      description:
        "Add service layer to separate business logic from API handlers",
      action: "Create src-tauri/src/services/ directory for business logic",
    });
  } catch (error) {
    analysis.error = error.message;
  }

  return analysis;
}

async function analyzeTechnicalDebt() {
  const analysis = {
    timestamp: new Date().toISOString(),
    code_quality: {},
    dependencies: {},
    performance: {},
    recommendations: [],
  };

  try {
    // 代碼品質分析
    console.log("  🔍 代碼品質檢查...");

    try {
      const { stderr: rustWarnings } = await execAsync(
        "cd src-tauri && cargo clippy 2>&1 || true",
        { timeout: 60000 }
      );

      analysis.code_quality.rust = {
        warnings: (rustWarnings.match(/warning/g) || []).length,
        errors: (rustWarnings.match(/error/g) || []).length,
        hasIssues:
          rustWarnings.includes("warning") || rustWarnings.includes("error"),
      };
    } catch (error) {
      analysis.code_quality.rust = { error: error.message };
    }

    try {
      const { stdout: jsLint } = await execAsync(
        "npm run lint:check 2>&1 || true",
        { timeout: 30000 }
      );

      analysis.code_quality.javascript = {
        warnings: (jsLint.match(/warning/g) || []).length,
        errors: (jsLint.match(/error/g) || []).length,
        hasIssues: jsLint.includes("warning") || jsLint.includes("error"),
      };
    } catch (error) {
      analysis.code_quality.javascript = { error: error.message };
    }

    // 依賴安全檢查
    console.log("  🔒 依賴安全檢查...");

    try {
      const { stdout: npmAudit } = await execAsync(
        "npm audit --json 2>&1 || true",
        { timeout: 30000 }
      );

      try {
        const auditData = JSON.parse(npmAudit);
        analysis.dependencies.npm = {
          vulnerabilities: auditData.metadata?.vulnerabilities?.total || 0,
          hasVulnerabilities:
            (auditData.metadata?.vulnerabilities?.total || 0) > 0,
        };
      } catch {
        analysis.dependencies.npm = { error: "Could not parse audit results" };
      }
    } catch (error) {
      analysis.dependencies.npm = { error: error.message };
    }

    try {
      const { stdout: cargoAudit } = await execAsync(
        "cd src-tauri && cargo audit 2>&1 || true",
        { timeout: 30000 }
      );

      analysis.dependencies.cargo = {
        hasVulnerabilities: cargoAudit.includes("Vulnerabilities found"),
        output: cargoAudit.substring(0, 200),
      };
    } catch (error) {
      analysis.dependencies.cargo = { error: error.message };
    }

    // 性能分析
    console.log("  ⚡ 性能分析...");

    try {
      const start = Date.now();
      await execAsync("npm run cli:optimized -- --help", { timeout: 15000 });
      analysis.performance.cli_startup = Date.now() - start;
    } catch (error) {
      analysis.performance.cli_startup = { error: error.message };
    }

    // 生成技術債務建議
    if (analysis.code_quality.rust?.warnings > 0) {
      analysis.recommendations.push({
        type: "code_quality",
        priority: "medium",
        title: "Fix Rust compiler warnings",
        description: `Found ${analysis.code_quality.rust.warnings} Rust warnings`,
        action: "Run cargo clippy --fix to automatically fix warnings",
      });
    }

    if (analysis.code_quality.javascript?.warnings > 0) {
      analysis.recommendations.push({
        type: "code_quality",
        priority: "medium",
        title: "Fix JavaScript linting issues",
        description: `Found ${analysis.code_quality.javascript.warnings} JS warnings`,
        action: "Run npm run lint --fix to automatically fix issues",
      });
    }

    if (analysis.dependencies.npm?.hasVulnerabilities) {
      analysis.recommendations.push({
        type: "security",
        priority: "high",
        title: "Fix npm security vulnerabilities",
        description: `Found ${analysis.dependencies.npm.vulnerabilities} npm vulnerabilities`,
        action: "Run npm audit fix to update vulnerable dependencies",
      });
    }

    if (analysis.dependencies.cargo?.hasVulnerabilities) {
      analysis.recommendations.push({
        type: "security",
        priority: "high",
        title: "Fix Cargo security vulnerabilities",
        description: "Found security vulnerabilities in Rust dependencies",
        action: "Update vulnerable Cargo dependencies",
      });
    }

    if (analysis.performance.cli_startup > 5000) {
      analysis.recommendations.push({
        type: "performance",
        priority: "low",
        title: "Optimize CLI startup time",
        description: `CLI startup takes ${analysis.performance.cli_startup}ms`,
        action: "Profile and optimize CLI startup performance",
      });
    }
  } catch (error) {
    analysis.error = error.message;
  }

  return analysis;
}

function generateSummary(results) {
  const summary = {
    timestamp: new Date().toISOString(),
    total_recommendations: 0,
    high_priority_issues: 0,
    medium_priority_issues: 0,
    low_priority_issues: 0,
    categories: {
      cli: 0,
      architecture: 0,
      files: 0,
      technical_debt: 0,
    },
    next_steps: [],
  };

  // 統計所有建議
  const allRecommendations = [
    ...(results.cli_analysis.recommendations || []),
    ...(results.file_analysis.recommendations || []),
    ...(results.architecture_analysis.recommendations || []),
    ...(results.technical_debt.recommendations || []),
  ];

  summary.total_recommendations = allRecommendations.length;

  // 按優先級統計
  for (const rec of allRecommendations) {
    switch (rec.priority) {
      case "high":
        summary.high_priority_issues++;
        break;
      case "medium":
        summary.medium_priority_issues++;
        break;
      case "low":
        summary.low_priority_issues++;
        break;
    }
  }

  // 按類別統計
  summary.categories.cli = results.cli_analysis.recommendations?.length || 0;
  summary.categories.architecture =
    results.architecture_analysis.recommendations?.length || 0;
  summary.categories.files = results.file_analysis.recommendations?.length || 0;
  summary.categories.technical_debt =
    results.technical_debt.recommendations?.length || 0;

  // 生成下一步建議
  if (summary.high_priority_issues > 0) {
    summary.next_steps.push(
      "Address high priority issues first, especially security vulnerabilities"
    );
  }

  if (
    results.architecture_analysis.recommendations?.some(
      (r) => r.type === "architecture_pattern"
    )
  ) {
    summary.next_steps.push(
      "Consider implementing the executor pattern from Vibe-Kanban"
    );
  }

  if (results.cli_analysis.tests?.some((t) => t.status === "failed")) {
    summary.next_steps.push("Fix failing CLI commands to ensure reliability");
  }

  summary.next_steps.push(
    "Implement comprehensive testing for all CLI commands"
  );
  summary.next_steps.push(
    "Set up continuous integration to prevent regression"
  );

  return summary;
}

async function generateReadableReport(results, reportDir) {
  const report = `# Claude Night Pilot - 專案分析報告

**分析時間**: ${new Date().toLocaleString("zh-TW")}
**狀態**: ✅ 分析完成

## 📊 分析摘要

- **總建議數**: ${results.summary.total_recommendations}
- **高優先級問題**: ${results.summary.high_priority_issues}
- **中優先級問題**: ${results.summary.medium_priority_issues}
- **低優先級問題**: ${results.summary.low_priority_issues}

## 🔍 CLI 功能分析

### 測試結果
${
  results.cli_analysis.tests
    ?.map(
      (test) =>
        `- **${test.command}**: ${test.status === "passed" ? "✅" : "❌"} ${
          test.status
        }`
    )
    .join("\n") || "無測試結果"
}

### CLI 建議
${
  results.cli_analysis.recommendations
    ?.map((rec) => `- **${rec.title}** (${rec.priority}): ${rec.description}`)
    .join("\n") || "無建議"
}

## 📁 檔案結構分析

### 專案結構
- **總檔案數**: ${results.file_analysis.structure?.totalFiles || 0}
- **Rust 檔案**: ${results.file_analysis.structure?.byType?.rust || 0}
- **JavaScript 檔案**: ${
    results.file_analysis.structure?.byType?.javascript || 0
  }
- **Markdown 檔案**: ${results.file_analysis.structure?.byType?.markdown || 0}

### 發現的問題
${
  results.file_analysis.issues
    ?.map((issue) => `- **${issue.type}**: ${issue.description}`)
    .join("\n") || "無問題發現"
}

### 檔案建議
${
  results.file_analysis.recommendations
    ?.map((rec) => `- **${rec.title}** (${rec.priority}): ${rec.description}`)
    .join("\n") || "無建議"
}

## 🏗️ 架構分析

### 與 Vibe-Kanban 比較
${
  results.architecture_analysis.vibe_kanban_comparison?.similarities
    ?.map((sim) => `- ✅ ${sim}`)
    .join("\n") || ""
}

${
  results.architecture_analysis.vibe_kanban_comparison?.differences
    ?.map((diff) => `- 🔄 ${diff}`)
    .join("\n") || ""
}

### 架構建議
${
  results.architecture_analysis.recommendations
    ?.map((rec) => `- **${rec.title}** (${rec.priority}): ${rec.description}`)
    .join("\n") || "無建議"
}

## 🔧 技術債務分析

### 代碼品質
- **Rust 警告**: ${results.technical_debt.code_quality?.rust?.warnings || 0}
- **JavaScript 警告**: ${
    results.technical_debt.code_quality?.javascript?.warnings || 0
  }

### 安全性
- **npm 漏洞**: ${
    results.technical_debt.dependencies?.npm?.vulnerabilities || 0
  }
- **Cargo 漏洞**: ${
    results.technical_debt.dependencies?.cargo?.hasVulnerabilities
      ? "發現"
      : "無"
  }

### 性能
- **CLI 啟動時間**: ${
    results.technical_debt.performance?.cli_startup || "N/A"
  }ms

### 技術債務建議
${
  results.technical_debt.recommendations
    ?.map((rec) => `- **${rec.title}** (${rec.priority}): ${rec.description}`)
    .join("\n") || "無建議"
}

## 🚀 建議的實施順序

### 1. 立即處理 (高優先級)
${
  results.summary.high_priority_issues > 0
    ? "- 修復安全漏洞\n- 修復失敗的 CLI 指令\n- 實施關鍵架構改進"
    : "✅ 無高優先級問題"
}

### 2. 計劃實施 (中優先級)
${
  results.summary.medium_priority_issues > 0
    ? "- 改進代碼品質\n- 實施架構模式\n- 優化檔案結構"
    : "✅ 無中優先級問題"
}

### 3. 持續改進 (低優先級)
${
  results.summary.low_priority_issues > 0
    ? "- 性能優化\n- 檔案清理\n- 文檔改進"
    : "✅ 無低優先級問題"
}

## 📋 下一步行動

${results.summary.next_steps.map((step) => `- ${step}`).join("\n")}

## 🎯 成功指標

- [ ] 所有 CLI 指令測試通過
- [ ] 零安全漏洞
- [ ] 實施 Vibe-Kanban 架構模式
- [ ] 代碼品質達到企業級標準
- [ ] 完整的測試覆蓋

---

**分析工具**: Claude Night Pilot Immediate Analysis
**版本**: 1.0.0
**生成時間**: ${new Date().toLocaleString("zh-TW")}
`;

  const summaryFile = path.join(reportDir, "IMMEDIATE_ANALYSIS_SUMMARY.md");
  fs.writeFileSync(summaryFile, report);
}

function isValidJSON(str) {
  try {
    JSON.parse(str);
    return true;
  } catch {
    return false;
  }
}

// 執行分析
runImmediateAnalysis().catch(console.error);
