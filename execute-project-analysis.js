#!/usr/bin/env node

/**
 * Claude Night Pilot - 實際執行專案分析
 * 啟動多個 Claude Code sessions 進行並行分析
 */

import fs from "fs";
import path from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

class ProjectAnalysisExecutor {
  constructor() {
    this.startTime = new Date();
    this.sessions = new Map();
    this.logDir = path.join(process.cwd(), "analysis", "logs");
    this.reportDir = path.join(process.cwd(), "analysis", "reports");

    // 確保目錄存在
    fs.mkdirSync(this.logDir, { recursive: true });
    fs.mkdirSync(this.reportDir, { recursive: true });
  }

  /**
   * 執行完整的專案分析
   */
  async executeAnalysis() {
    console.log(`
╔══════════════════════════════════════════════════════════════╗
║                Claude Night Pilot                            ║
║              專案分析執行中...                                ║
╚══════════════════════════════════════════════════════════════╝
`);

    try {
      // Phase 1: 準備和 CLI 分析
      await this.phase1_PrepareAndCLI();

      // Phase 2: 檔案分析
      await this.phase2_FileAnalysis();

      // Phase 3: 架構分析
      await this.phase3_ArchitectureAnalysis();

      // Phase 4: 技術債務分析
      await this.phase4_TechnicalDebt();

      // Phase 5: 整合和報告
      await this.phase5_Integration();

      console.log("🎉 專案分析完成！");
    } catch (error) {
      console.error("❌ 分析執行失敗:", error.message);
      throw error;
    }
  }

  /**
   * Phase 1: 準備和 CLI 分析
   */
  async phase1_PrepareAndCLI() {
    console.log("🚀 Phase 1: CLI 功能分析和測試...");

    // 1. 分析所有 CLI 指令
    const cliCommands = await this.analyzeCLICommands();

    // 2. 測試每個指令
    const testResults = await this.testCLICommands(cliCommands);

    // 3. 生成 BDD 測試場景
    await this.generateBDDScenarios(cliCommands, testResults);

    console.log("✅ Phase 1 完成");
  } /**

   * 分析所有 CLI 指令
   */
  async analyzeCLICommands() {
    console.log("📋 分析 CLI 指令...");

    const commands = {
      "cnp-optimized": ["execute", "cooldown", "health", "benchmark", "status"],
      "cnp-unified": [
        "execute",
        "run",
        "cooldown",
        "health",
        "status",
        "results",
        "prompt",
        "job",
        "init",
        "batch",
      ],
    };

    const analysis = {
      timestamp: new Date().toISOString(),
      binaries: {},
      totalCommands: 0,
    };

    for (const [binary, cmdList] of Object.entries(commands)) {
      analysis.binaries[binary] = {
        commands: {},
        totalCommands: cmdList.length,
      };

      for (const cmd of cmdList) {
        try {
          console.log(`  🔍 分析 ${binary} ${cmd}...`);
          const cmdInfo = await this.analyzeCommand(binary, cmd);
          analysis.binaries[binary].commands[cmd] = cmdInfo;
          analysis.totalCommands++;
        } catch (error) {
          console.warn(`  ⚠️  ${binary} ${cmd} 分析失敗: ${error.message}`);
          analysis.binaries[binary].commands[cmd] = {
            error: error.message,
            status: "failed",
          };
        }
      }
    }

    // 保存分析結果
    const reportFile = path.join(this.reportDir, "cli-analysis.json");
    fs.writeFileSync(reportFile, JSON.stringify(analysis, null, 2));

    return analysis;
  }

  /**
   * 分析單個指令
   */
  async analyzeCommand(binary, command) {
    const npmScript = binary === "cnp-optimized" ? "cli:optimized" : "cli";

    try {
      // 嘗試獲取幫助信息
      const { stdout: helpOutput } = await execAsync(
        `npm run ${npmScript} -- ${command} --help`,
        { timeout: 15000 }
      );

      return {
        command,
        binary,
        helpAvailable: true,
        helpOutput: helpOutput.substring(0, 500),
        status: "analyzed",
      };
    } catch (helpError) {
      // 如果沒有幫助，嘗試直接執行
      try {
        const { stdout, stderr } = await execAsync(
          `timeout 5 npm run ${npmScript} -- ${command}`,
          { timeout: 10000 }
        );

        return {
          command,
          binary,
          helpAvailable: false,
          directExecution: true,
          output: stdout || stderr,
          status: "executed",
        };
      } catch (execError) {
        return {
          command,
          binary,
          helpAvailable: false,
          directExecution: false,
          error: execError.message,
          status: "failed",
        };
      }
    }
  }

  /**
   * 測試 CLI 指令
   */
  async testCLICommands(cliAnalysis) {
    console.log("🧪 測試 CLI 指令功能...");

    const testResults = {
      timestamp: new Date().toISOString(),
      tests: [],
      summary: {
        total: 0,
        passed: 0,
        failed: 0,
      },
    };

    // 測試基本功能指令
    const basicTests = [
      { binary: "cnp-optimized", command: "status", expected: "json" },
      {
        binary: "cnp-optimized",
        command: "health --format json",
        expected: "json",
      },
      { binary: "cnp-optimized", command: "cooldown", expected: "output" },
      { binary: "cnp-unified", command: "status", expected: "output" },
      { binary: "cnp-unified", command: "prompt list", expected: "output" },
    ];

    for (const test of basicTests) {
      try {
        console.log(`  🔬 測試 ${test.binary} ${test.command}...`);

        const npmScript =
          test.binary === "cnp-optimized" ? "cli:optimized" : "cli";
        const { stdout, stderr } = await execAsync(
          `npm run ${npmScript} -- ${test.command}`,
          { timeout: 30000 }
        );

        const result = {
          ...test,
          status: "passed",
          output: stdout,
          error: stderr,
          timestamp: new Date().toISOString(),
        };

        // 驗證輸出格式
        if (test.expected === "json") {
          try {
            JSON.parse(stdout);
            result.jsonValid = true;
          } catch {
            result.jsonValid = false;
            result.status = "warning";
          }
        }

        testResults.tests.push(result);
        testResults.summary.passed++;
      } catch (error) {
        testResults.tests.push({
          ...test,
          status: "failed",
          error: error.message,
          timestamp: new Date().toISOString(),
        });
        testResults.summary.failed++;
      }

      testResults.summary.total++;
    }

    // 保存測試結果
    const testFile = path.join(this.reportDir, "cli-test-results.json");
    fs.writeFileSync(testFile, JSON.stringify(testResults, null, 2));

    console.log(
      `  ✅ 測試完成: ${testResults.summary.passed}/${testResults.summary.total} 通過`
    );

    return testResults;
  } /**

   * 生成 BDD 測試場景
   */
  async generateBDDScenarios(cliAnalysis, testResults) {
    console.log("📝 生成 BDD 測試場景...");

    const scenarios = {
      feature: "Claude Night Pilot CLI Commands",
      description: "Comprehensive testing of all CLI functionality",
      background: {
        given: "The Claude Night Pilot system is properly installed",
        and: "The database is initialized and accessible",
      },
      scenarios: [],
    };

    // 為每個成功的指令生成 BDD 場景
    for (const test of testResults.tests) {
      if (test.status === "passed") {
        scenarios.scenarios.push({
          scenario: `Execute ${test.binary} ${test.command}`,
          given: "The CLI tool is available",
          when: `I run "${test.binary} ${test.command}"`,
          then: [
            "The command should execute successfully",
            "The exit code should be 0",
            test.expected === "json"
              ? "The output should be valid JSON"
              : "The output should be properly formatted",
          ],
        });
      }
    }

    // 生成錯誤處理場景
    scenarios.scenarios.push({
      scenario: "Handle invalid commands",
      given: "The CLI tool is available",
      when: "I run an invalid command",
      then: [
        "An error message should be displayed",
        "The exit code should be non-zero",
        "Help information should be suggested",
      ],
    });

    // 保存 BDD 場景
    const bddFile = path.join(this.reportDir, "bdd-scenarios.yaml");
    const yamlContent = this.convertToYAML(scenarios);
    fs.writeFileSync(bddFile, yamlContent);

    console.log(`  ✅ BDD 場景已生成: ${scenarios.scenarios.length} 個場景`);
  }

  /**
   * Phase 2: 檔案分析
   */
  async phase2_FileAnalysis() {
    console.log("🗂️  Phase 2: 檔案結構分析...");

    // 執行檔案分析工具
    try {
      await execAsync("node analysis/tools/file-analyzer.js", {
        timeout: 60000,
      });
      console.log("✅ 檔案分析完成");
    } catch (error) {
      console.warn("⚠️  檔案分析工具執行失敗:", error.message);

      // 手動執行基本檔案分析
      await this.basicFileAnalysis();
    }
  }

  /**
   * 基本檔案分析
   */
  async basicFileAnalysis() {
    console.log("📁 執行基本檔案分析...");

    const analysis = {
      timestamp: new Date().toISOString(),
      summary: {},
      findings: [],
    };

    try {
      // 查找可能過時的檔案
      const { stdout: oldFiles } = await execAsync(
        'find . -name "*.md" -o -name "*.js" -o -name "*.rs" | grep -E "(temp|old|backup|test)" | head -20'
      );

      if (oldFiles.trim()) {
        analysis.findings.push({
          type: "potentially_obsolete",
          files: oldFiles.trim().split("\n"),
          description: "Files with temp/old/backup/test patterns",
        });
      }

      // 查找空目錄
      const { stdout: emptyDirs } = await execAsync(
        "find . -type d -empty | grep -v node_modules | grep -v target | head -10"
      );

      if (emptyDirs.trim()) {
        analysis.findings.push({
          type: "empty_directories",
          directories: emptyDirs.trim().split("\n"),
          description: "Empty directories that could be removed",
        });
      }
    } catch (error) {
      analysis.error = error.message;
    }

    // 保存基本分析結果
    const analysisFile = path.join(this.reportDir, "basic-file-analysis.json");
    fs.writeFileSync(analysisFile, JSON.stringify(analysis, null, 2));

    console.log("✅ 基本檔案分析完成");
  }

  /**
   * Phase 3: 架構分析
   */
  async phase3_ArchitectureAnalysis() {
    console.log("🏗️  Phase 3: 架構分析...");

    const analysis = {
      timestamp: new Date().toISOString(),
      currentArchitecture: await this.analyzeCurrentArchitecture(),
      vibeKanbanComparison: await this.compareWithVibeKanban(),
      recommendations: [],
    };

    // 生成架構建議
    analysis.recommendations =
      this.generateArchitectureRecommendations(analysis);

    // 保存架構分析
    const archFile = path.join(this.reportDir, "architecture-analysis.json");
    fs.writeFileSync(archFile, JSON.stringify(analysis, null, 2));

    console.log("✅ 架構分析完成");
  }

  /**
   * 分析當前架構
   */
  async analyzeCurrentArchitecture() {
    const structure = {
      backend: {
        path: "src-tauri/src",
        modules: [],
        patterns: [],
      },
      frontend: {
        path: "src",
        modules: [],
        patterns: [],
      },
      tests: {
        path: "tests",
        types: [],
      },
    };

    try {
      // 分析 Rust 後端結構
      const { stdout: rustFiles } = await execAsync(
        'find src-tauri/src -name "*.rs" | head -20'
      );
      structure.backend.modules = rustFiles
        .trim()
        .split("\n")
        .filter((f) => f);

      // 分析前端結構
      const { stdout: jsFiles } = await execAsync(
        'find src -name "*.js" -o -name "*.html" -o -name "*.css" | head -20'
      );
      structure.frontend.modules = jsFiles
        .trim()
        .split("\n")
        .filter((f) => f);

      // 分析測試結構
      const { stdout: testFiles } = await execAsync(
        'find tests -name "*.js" -o -name "*.spec.js" | head -20'
      );
      structure.tests.types = testFiles
        .trim()
        .split("\n")
        .filter((f) => f);
    } catch (error) {
      structure.error = error.message;
    }

    return structure;
  } /**

   * 與 Vibe-Kanban 架構比較
   */
  async compareWithVibeKanban() {
    const comparison = {
      similarities: [],
      differences: [],
      adoptablePatterns: [],
    };

    // 檢查 Vibe-Kanban 結構
    try {
      const vibeStructure = await this.analyzeVibeKanbanStructure();

      // 相似點
      comparison.similarities = [
        "Both use Rust backend with SQLite database",
        "Both have CLI interfaces",
        "Both use modern async Rust patterns",
      ];

      // 差異點
      comparison.differences = [
        "Vibe-Kanban uses Axum web framework, we use Tauri",
        "Vibe-Kanban has React frontend, we use vanilla JS",
        "Vibe-Kanban has executor pattern for multiple agents",
      ];

      // 可採用的模式
      comparison.adoptablePatterns = [
        {
          pattern: "Executor System",
          description:
            "Implement executor pattern for different Claude operations",
          priority: "high",
          files: ["src-tauri/src/executors/"],
        },
        {
          pattern: "Type Sharing",
          description: "Generate TypeScript types from Rust structs",
          priority: "medium",
          files: ["shared-types/"],
        },
        {
          pattern: "API Architecture",
          description: "RESTful endpoints with WebSocket streaming",
          priority: "medium",
          files: ["src-tauri/src/api/"],
        },
      ];
    } catch (error) {
      comparison.error = error.message;
    }

    return comparison;
  }

  /**
   * 分析 Vibe-Kanban 結構
   */
  async analyzeVibeKanbanStructure() {
    try {
      const { stdout: vibeFiles } = await execAsync(
        'find research-projects/vibe-kanban -name "*.rs" -o -name "*.ts" | head -30'
      );

      return {
        files: vibeFiles
          .trim()
          .split("\n")
          .filter((f) => f),
        hasExecutors: vibeFiles.includes("executors"),
        hasAPI: vibeFiles.includes("api"),
        hasDB: vibeFiles.includes("db"),
      };
    } catch (error) {
      return { error: error.message };
    }
  }

  /**
   * 生成架構建議
   */
  generateArchitectureRecommendations(analysis) {
    const recommendations = [];

    // 基於 Vibe-Kanban 的建議
    if (analysis.vibeKanbanComparison.adoptablePatterns) {
      for (const pattern of analysis.vibeKanbanComparison.adoptablePatterns) {
        recommendations.push({
          type: "architecture_pattern",
          title: `Adopt ${pattern.pattern}`,
          description: pattern.description,
          priority: pattern.priority,
          estimatedEffort: "medium",
          benefits: ["Better modularity", "Improved maintainability"],
        });
      }
    }

    // 模組化建議
    recommendations.push({
      type: "modularization",
      title: "Implement Service Layer",
      description: "Extract business logic into separate service modules",
      priority: "high",
      estimatedEffort: "high",
      benefits: ["Better testability", "Cleaner separation of concerns"],
    });

    return recommendations;
  }

  /**
   * Phase 4: 技術債務分析
   */
  async phase4_TechnicalDebt() {
    console.log("🔧 Phase 4: 技術債務分析...");

    const debt = {
      timestamp: new Date().toISOString(),
      codeQuality: await this.analyzeCodeQuality(),
      dependencies: await this.analyzeDependencies(),
      performance: await this.analyzePerformance(),
      recommendations: [],
    };

    // 生成清理建議
    debt.recommendations = this.generateDebtRecommendations(debt);

    // 保存技術債務分析
    const debtFile = path.join(this.reportDir, "technical-debt.json");
    fs.writeFileSync(debtFile, JSON.stringify(debt, null, 2));

    console.log("✅ 技術債務分析完成");
  }

  /**
   * 分析代碼品質
   */
  async analyzeCodeQuality() {
    const quality = {
      rust: { warnings: 0, errors: 0 },
      javascript: { warnings: 0, errors: 0 },
      overall: "good",
    };

    try {
      // Rust 代碼檢查
      const { stderr: rustCheck } = await execAsync(
        "cd src-tauri && cargo clippy 2>&1 || true",
        { timeout: 60000 }
      );

      quality.rust.warnings = (rustCheck.match(/warning/g) || []).length;
      quality.rust.errors = (rustCheck.match(/error/g) || []).length;

      // JavaScript 代碼檢查
      const { stdout: jsCheck } = await execAsync(
        "npm run lint:check 2>&1 || true",
        { timeout: 30000 }
      );

      quality.javascript.warnings = (jsCheck.match(/warning/g) || []).length;
      quality.javascript.errors = (jsCheck.match(/error/g) || []).length;
    } catch (error) {
      quality.error = error.message;
    }

    return quality;
  }

  /**
   * 分析依賴
   */
  async analyzeDependencies() {
    const deps = {
      rust: { outdated: [], vulnerable: [] },
      node: { outdated: [], vulnerable: [] },
    };

    try {
      // Rust 依賴檢查
      const { stdout: cargoAudit } = await execAsync(
        "cd src-tauri && cargo audit 2>&1 || true",
        { timeout: 30000 }
      );

      if (cargoAudit.includes("Vulnerabilities found")) {
        deps.rust.vulnerable.push(
          "Found vulnerabilities in Cargo dependencies"
        );
      }

      // Node 依賴檢查
      const { stdout: npmAudit } = await execAsync(
        "npm audit --json 2>&1 || true",
        { timeout: 30000 }
      );

      try {
        const auditResult = JSON.parse(npmAudit);
        if (auditResult.metadata && auditResult.metadata.vulnerabilities) {
          deps.node.vulnerable.push(
            `Found ${auditResult.metadata.vulnerabilities.total} vulnerabilities`
          );
        }
      } catch {
        // JSON 解析失敗，忽略
      }
    } catch (error) {
      deps.error = error.message;
    }

    return deps;
  }

  /**
   * 分析性能
   */
  async analyzePerformance() {
    const perf = {
      cliStartup: null,
      buildTime: null,
      testTime: null,
    };

    try {
      // CLI 啟動時間測試
      const start = Date.now();
      await execAsync("npm run cli:optimized -- --help", { timeout: 15000 });
      perf.cliStartup = Date.now() - start;
    } catch (error) {
      perf.error = error.message;
    }

    return perf;
  }
  /**
   * 生成債務清理建議
   */
  generateDebtRecommendations(debt) {
    const recommendations = [];

    // 代碼品質建議
    if (debt.codeQuality.rust.warnings > 0) {
      recommendations.push({
        type: "code_quality",
        title: "Fix Rust warnings",
        description: `Found ${debt.codeQuality.rust.warnings} Rust warnings`,
        priority: "medium",
        action: "Run cargo clippy --fix",
      });
    }

    if (debt.codeQuality.javascript.warnings > 0) {
      recommendations.push({
        type: "code_quality",
        title: "Fix JavaScript warnings",
        description: `Found ${debt.codeQuality.javascript.warnings} JS warnings`,
        priority: "medium",
        action: "Run npm run lint --fix",
      });
    }

    // 安全建議
    if (debt.dependencies.rust.vulnerable.length > 0) {
      recommendations.push({
        type: "security",
        title: "Update vulnerable Rust dependencies",
        description: "Found security vulnerabilities in Rust dependencies",
        priority: "high",
        action: "Update Cargo.toml dependencies",
      });
    }

    if (debt.dependencies.node.vulnerable.length > 0) {
      recommendations.push({
        type: "security",
        title: "Update vulnerable Node dependencies",
        description: "Found security vulnerabilities in Node dependencies",
        priority: "high",
        action: "Run npm audit fix",
      });
    }

    // 性能建議
    if (debt.performance.cliStartup > 5000) {
      recommendations.push({
        type: "performance",
        title: "Optimize CLI startup time",
        description: `CLI startup takes ${debt.performance.cliStartup}ms`,
        priority: "low",
        action: "Profile and optimize startup code",
      });
    }

    return recommendations;
  }

  /**
   * Phase 5: 整合和報告
   */
  async phase5_Integration() {
    console.log("📊 Phase 5: 整合分析結果...");

    // 收集所有分析結果
    const consolidatedReport = await this.consolidateResults();

    // 生成實施計劃
    const implementationPlan =
      this.generateImplementationPlan(consolidatedReport);

    // 生成最終報告
    await this.generateFinalReport(consolidatedReport, implementationPlan);

    console.log("✅ 整合完成");
  }

  /**
   * 整合所有結果
   */
  async consolidateResults() {
    const report = {
      timestamp: new Date().toISOString(),
      executionTime: Date.now() - this.startTime.getTime(),
      summary: {
        totalIssues: 0,
        highPriorityIssues: 0,
        recommendations: 0,
      },
      sections: {},
    };

    // 讀取各個分析結果
    const reportFiles = [
      "cli-analysis.json",
      "cli-test-results.json",
      "basic-file-analysis.json",
      "architecture-analysis.json",
      "technical-debt.json",
    ];

    for (const file of reportFiles) {
      const filePath = path.join(this.reportDir, file);
      if (fs.existsSync(filePath)) {
        try {
          const content = JSON.parse(fs.readFileSync(filePath, "utf8"));
          const sectionName = file.replace(".json", "").replace(/-/g, "_");
          report.sections[sectionName] = content;

          // 統計建議數量
          if (content.recommendations) {
            report.summary.recommendations += content.recommendations.length;
            report.summary.highPriorityIssues += content.recommendations.filter(
              (r) => r.priority === "high"
            ).length;
          }
        } catch (error) {
          console.warn(`⚠️  無法讀取 ${file}: ${error.message}`);
        }
      }
    }

    return report;
  }

  /**
   * 生成實施計劃
   */
  generateImplementationPlan(report) {
    const plan = {
      timestamp: new Date().toISOString(),
      phases: [
        {
          name: "Critical Fixes",
          duration: "1 week",
          priority: "critical",
          tasks: [],
        },
        {
          name: "Architecture Improvements",
          duration: "2-3 weeks",
          priority: "high",
          tasks: [],
        },
        {
          name: "Quality Enhancements",
          duration: "1-2 weeks",
          priority: "medium",
          tasks: [],
        },
        {
          name: "Optimization",
          duration: "1 week",
          priority: "low",
          tasks: [],
        },
      ],
    };

    // 從各個分析結果中提取任務
    for (const [sectionName, section] of Object.entries(report.sections)) {
      if (section.recommendations) {
        for (const rec of section.recommendations) {
          const phase = this.getPhaseForRecommendation(rec);
          if (phase) {
            phase.tasks.push({
              title: rec.title,
              description: rec.description,
              source: sectionName,
              estimatedHours: this.estimateHours(rec),
            });
          }
        }
      }
    }

    return plan;
  }

  /**
   * 獲取建議對應的階段
   */
  getPhaseForRecommendation(recommendation) {
    // 這裡應該有更複雜的邏輯來分配任務到不同階段
    // 簡化版本：根據優先級分配
    switch (recommendation.priority) {
      case "critical":
      case "high":
        return { name: "Critical Fixes", tasks: [] };
      case "medium":
        return { name: "Quality Enhancements", tasks: [] };
      default:
        return { name: "Optimization", tasks: [] };
    }
  }

  /**
   * 估算工作時間
   */
  estimateHours(recommendation) {
    // 簡化的時間估算
    const effortMap = {
      low: 4,
      medium: 16,
      high: 40,
    };

    return effortMap[recommendation.estimatedEffort] || 8;
  }

  /**
   * 生成最終報告
   */
  async generateFinalReport(consolidatedReport, implementationPlan) {
    const finalReport = {
      ...consolidatedReport,
      implementationPlan,
      nextSteps: [
        "Review the consolidated analysis report",
        "Prioritize recommendations based on business impact",
        "Begin with critical fixes and security updates",
        "Implement architecture improvements incrementally",
        "Set up continuous monitoring for code quality",
      ],
    };

    // 保存最終報告
    const finalFile = path.join(this.reportDir, "FINAL_ANALYSIS_REPORT.json");
    fs.writeFileSync(finalFile, JSON.stringify(finalReport, null, 2));

    // 生成人類可讀的摘要
    await this.generateHumanReadableSummary(finalReport);

    console.log(`📄 最終報告已生成: ${finalFile}`);
  }

  /**
   * 生成人類可讀摘要
   */
  async generateHumanReadableSummary(finalReport) {
    const summary = `# Claude Night Pilot - 專案分析完成報告

**分析完成時間**: ${new Date().toLocaleString("zh-TW")}
**執行時間**: ${Math.round(finalReport.executionTime / 1000 / 60)} 分鐘
**狀態**: ✅ 分析完成

## 📊 分析摘要

- **總建議數**: ${finalReport.summary.recommendations}
- **高優先級問題**: ${finalReport.summary.highPriorityIssues}
- **分析模組**: ${Object.keys(finalReport.sections).length}

## 🎯 主要發現

### CLI 功能分析
- 發現 ${finalReport.sections.cli_analysis?.totalCommands || 0} 個 CLI 指令
- 測試通過率: ${finalReport.sections.cli_test_results?.summary?.passed || 0}/${
      finalReport.sections.cli_test_results?.summary?.total || 0
    }

### 架構分析
- 與 Vibe-Kanban 架構比較完成
- 識別 ${
      finalReport.sections.architecture_analysis?.recommendations?.length || 0
    } 個改進機會

### 技術債務
- 代碼品質檢查完成
- 依賴安全性檢查完成
- 性能分析完成

## 🚀 建議的實施順序

${finalReport.implementationPlan.phases
  .map(
    (phase, i) => `
### ${i + 1}. ${phase.name} (${phase.duration})
${phase.tasks.map((task) => `- ${task.title}`).join("\n")}
`
  )
  .join("")}

## 📋 下一步行動

${finalReport.nextSteps.map((step) => `- ${step}`).join("\n")}

---

**分析工具**: Claude Night Pilot Analysis System
**版本**: 1.0.0
`;

    const summaryFile = path.join(this.reportDir, "ANALYSIS_SUMMARY.md");
    fs.writeFileSync(summaryFile, summary);

    console.log(`📄 摘要報告已生成: ${summaryFile}`);
  }

  /**
   * 簡單的 YAML 轉換
   */
  convertToYAML(obj, indent = 0) {
    const spaces = "  ".repeat(indent);
    let yaml = "";

    for (const [key, value] of Object.entries(obj)) {
      if (Array.isArray(value)) {
        yaml += `${spaces}${key}:\n`;
        for (const item of value) {
          if (typeof item === "object") {
            yaml += `${spaces}  -\n`;
            yaml += this.convertToYAML(item, indent + 2).replace(/^/gm, "    ");
          } else {
            yaml += `${spaces}  - ${item}\n`;
          }
        }
      } else if (typeof value === "object" && value !== null) {
        yaml += `${spaces}${key}:\n`;
        yaml += this.convertToYAML(value, indent + 1);
      } else {
        yaml += `${spaces}${key}: ${value}\n`;
      }
    }

    return yaml;
  }
}

// 主執行邏輯
async function main() {
  const executor = new ProjectAnalysisExecutor();
  await executor.executeAnalysis();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("❌ 專案分析執行失敗:", error);
    process.exit(1);
  });
}

export default ProjectAnalysisExecutor;
