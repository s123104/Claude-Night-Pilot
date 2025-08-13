#!/usr/bin/env node

/**
 * Claude Night Pilot - 專案分析主執行器
 * 協調多個 Claude Code sessions 進行全面專案分析
 */

import fs from "fs";
import path from "path";
import SessionExecutor from "./tools/session-executor.js";
import CLIAnalyzer from "./tools/cli-analyzer.js";

class ProjectAnalysisOrchestrator {
  constructor() {
    this.startTime = new Date();
    this.logFile = path.join(
      process.cwd(),
      "analysis",
      "logs",
      "orchestrator.log"
    );
    this.statusFile = path.join(
      process.cwd(),
      "analysis",
      "logs",
      "analysis-status.json"
    );

    // 確保目錄存在
    fs.mkdirSync(path.dirname(this.logFile), { recursive: true });
  }

  /**
   * 記錄日誌
   */
  log(message) {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] ${message}\\n`;

    console.log(message);
    fs.appendFileSync(this.logFile, logEntry);
  }

  /**
   * 更新分析狀態
   */
  updateStatus(phase, progress, details = {}) {
    const status = {
      timestamp: new Date().toISOString(),
      startTime: this.startTime.toISOString(),
      phase,
      progress,
      details,
    };

    fs.writeFileSync(this.statusFile, JSON.stringify(status, null, 2));
  }

  /**
   * 執行預分析階段
   */
  async preAnalysisPhase() {
    this.log("🚀 Starting Pre-Analysis Phase...");
    this.updateStatus("pre-analysis", 0);

    // 1. 檢查專案結構
    this.log("📁 Checking project structure...");
    await this.checkProjectStructure();
    this.updateStatus("pre-analysis", 20);

    // 2. 執行 CLI 分析
    this.log("🔧 Running CLI analysis...");
    const cliAnalyzer = new CLIAnalyzer();
    await cliAnalyzer.analyzeAllBinaries();
    this.updateStatus("pre-analysis", 50);

    // 3. 檢查依賴和工具
    this.log("🔍 Checking dependencies and tools...");
    await this.checkDependencies();
    this.updateStatus("pre-analysis", 80);

    // 4. 準備 Claude Code prompts
    this.log("📝 Preparing Claude Code prompts...");
    await this.preparePrompts();
    this.updateStatus("pre-analysis", 100);

    this.log("✅ Pre-Analysis Phase completed");
  }

  /**
   * 檢查專案結構
   */
  async checkProjectStructure() {
    const requiredPaths = [
      "src-tauri/src",
      "src/js",
      "tests/e2e",
      "scripts",
      "docs",
    ];

    const structure = {
      valid: true,
      missing: [],
      present: [],
    };

    for (const requiredPath of requiredPaths) {
      if (fs.existsSync(requiredPath)) {
        structure.present.push(requiredPath);
      } else {
        structure.missing.push(requiredPath);
        structure.valid = false;
      }
    }

    if (!structure.valid) {
      this.log(`⚠️  Missing required paths: ${structure.missing.join(", ")}`);
    } else {
      this.log("✅ Project structure is valid");
    }

    // 保存結構分析
    const structureFile = path.join(
      process.cwd(),
      "analysis",
      "reports",
      "project-structure.json"
    );
    fs.mkdirSync(path.dirname(structureFile), { recursive: true });
    fs.writeFileSync(structureFile, JSON.stringify(structure, null, 2));
  }

  /**
   * 檢查依賴和工具
   */
  async checkDependencies() {
    const dependencies = {
      rust: await this.checkRustEnvironment(),
      node: await this.checkNodeEnvironment(),
      tools: await this.checkAnalysisTools(),
    };

    const depsFile = path.join(
      process.cwd(),
      "analysis",
      "reports",
      "dependencies.json"
    );
    fs.writeFileSync(depsFile, JSON.stringify(dependencies, null, 2));
  }

  /**
   * 檢查 Rust 環境
   */
  async checkRustEnvironment() {
    try {
      const { exec } = await import("child_process");
      const { promisify } = await import("util");
      const execAsync = promisify(exec);

      const [cargoVersion, rustcVersion] = await Promise.all([
        execAsync("cargo --version").catch(() => ({ stdout: "not found" })),
        execAsync("rustc --version").catch(() => ({ stdout: "not found" })),
      ]);

      return {
        cargo: cargoVersion.stdout.trim(),
        rustc: rustcVersion.stdout.trim(),
        available: !cargoVersion.stdout.includes("not found"),
      };
    } catch (error) {
      return {
        cargo: "not available",
        rustc: "not available",
        available: false,
        error: error.message,
      };
    }
  }

  /**
   * 檢查 Node.js 環境
   */
  async checkNodeEnvironment() {
    return {
      node: process.version,
      npm: process.env.npm_version || "unknown",
      available: true,
    };
  }

  /**
   * 檢查分析工具
   */
  async checkAnalysisTools() {
    const tools = ["git", "find", "grep"];
    const results = {};

    for (const tool of tools) {
      try {
        const { exec } = await import("child_process");
        const { promisify } = await import("util");
        const execAsync = promisify(exec);

        const result = await execAsync(`which ${tool}`);
        results[tool] = {
          available: true,
          path: result.stdout.trim(),
        };
      } catch (error) {
        results[tool] = {
          available: false,
          error: error.message,
        };
      }
    }

    return results;
  }

  /**
   * 準備 Claude Code prompts
   */
  async preparePrompts() {
    const promptsDir = path.join(process.cwd(), "analysis", "prompts");
    fs.mkdirSync(promptsDir, { recursive: true });

    // 為每個 session 生成優化的 prompt
    const sessions = [
      "session-1-file-analysis",
      "session-2-cli-analysis",
      "session-3-architecture-refactoring",
      "session-4-technical-debt",
      "session-5-monitoring-coordination",
    ];

    for (const sessionId of sessions) {
      const sessionFile = path.join(
        process.cwd(),
        "analysis",
        "sessions",
        `${sessionId}.md`
      );
      const promptFile = path.join(promptsDir, `${sessionId}-prompt.md`);

      if (fs.existsSync(sessionFile)) {
        const sessionContent = fs.readFileSync(sessionFile, "utf8");
        const enhancedPrompt = this.enhancePrompt(sessionId, sessionContent);
        fs.writeFileSync(promptFile, enhancedPrompt);
      }
    }
  }

  /**
   * 增強 prompt 內容
   */
  enhancePrompt(sessionId, baseContent) {
    const projectContext = this.getProjectContext();

    return `# Enhanced Claude Code Prompt for ${sessionId}

## Project Context
${projectContext}

## Your Specific Task
${baseContent}

## Additional Instructions
- Focus on actionable, implementable recommendations
- Provide confidence scores (0-1) for each recommendation
- Include risk assessment for proposed changes
- Generate machine-readable JSON outputs where specified
- Consider the parallel execution context - other sessions are running simultaneously

## Output Structure
Please structure your analysis results as follows:
1. Executive Summary (human-readable)
2. Detailed Analysis (structured data)
3. Recommendations (prioritized list)
4. Implementation Plan (step-by-step)
5. Risk Assessment (potential issues and mitigations)

Begin your analysis now.
`;
  }

  /**
   * 獲取專案上下文
   */
  getProjectContext() {
    return `
**Project**: Claude Night Pilot
**Type**: Tauri Desktop Application + CLI Tool
**Tech Stack**: Rust (backend), JavaScript (frontend), SQLite (database)
**Goal**: Professional, maintainable, enterprise-grade automation tool
**Current Phase**: Analysis and refactoring for production readiness

**Key Directories**:
- src-tauri/: Rust backend code
- src/: Frontend JavaScript/HTML/CSS
- tests/: E2E and integration tests
- scripts/: Development and build scripts
- docs/: Documentation

**Architecture Reference**: research-projects/vibe-kanban (similar Rust + web architecture)
`;
  }

  /**
   * 執行主分析階段
   */
  async mainAnalysisPhase() {
    this.log("🔬 Starting Main Analysis Phase...");
    this.updateStatus("main-analysis", 0);

    // 啟動 session 執行器
    const executor = new SessionExecutor();

    // 註冊所有 sessions
    const sessions = [
      {
        id: "session-1-file-analysis",
        role: "File Analysis Specialist",
        priority: 1,
      },
      {
        id: "session-2-cli-analysis",
        role: "CLI Testing Specialist",
        priority: 1,
      },
      {
        id: "session-3-architecture-refactoring",
        role: "Architecture Specialist",
        priority: 2,
      },
      {
        id: "session-4-technical-debt",
        role: "Code Quality Specialist",
        priority: 2,
      },
      {
        id: "session-5-monitoring-coordination",
        role: "Project Coordinator",
        priority: 3,
      },
    ];

    for (const session of sessions) {
      executor.registerSession(session.id, session);
    }

    this.updateStatus("main-analysis", 20);

    // 啟動所有 sessions
    await executor.startAllSessions();
    this.updateStatus("main-analysis", 50);

    // 監控執行
    await executor.monitorSessions();
    this.updateStatus("main-analysis", 100);

    this.log("✅ Main Analysis Phase completed");
  }

  /**
   * 執行後分析階段
   */
  async postAnalysisPhase() {
    this.log("📊 Starting Post-Analysis Phase...");
    this.updateStatus("post-analysis", 0);

    // 1. 收集所有結果
    this.log("📥 Collecting analysis results...");
    const results = await this.collectResults();
    this.updateStatus("post-analysis", 25);

    // 2. 生成統一報告
    this.log("📋 Generating unified report...");
    await this.generateUnifiedReport(results);
    this.updateStatus("post-analysis", 50);

    // 3. 創建實施計劃
    this.log("📅 Creating implementation plan...");
    await this.createImplementationPlan(results);
    this.updateStatus("post-analysis", 75);

    // 4. 生成最終摘要
    this.log("📄 Generating final summary...");
    await this.generateFinalSummary();
    this.updateStatus("post-analysis", 100);

    this.log("✅ Post-Analysis Phase completed");
  }

  /**
   * 收集分析結果
   */
  async collectResults() {
    const resultsDir = path.join(process.cwd(), "analysis", "reports");
    const results = {
      sessions: {},
      consolidated: null,
    };

    // 讀取整合報告
    const consolidatedFile = path.join(resultsDir, "consolidated-report.json");
    if (fs.existsSync(consolidatedFile)) {
      results.consolidated = JSON.parse(
        fs.readFileSync(consolidatedFile, "utf8")
      );
    }

    // 讀取各 session 的詳細報告
    const sessionDirs = fs
      .readdirSync(resultsDir, { withFileTypes: true })
      .filter(
        (dirent) => dirent.isDirectory() && dirent.name.startsWith("session-")
      )
      .map((dirent) => dirent.name);

    for (const sessionDir of sessionDirs) {
      const reportFile = path.join(resultsDir, sessionDir, "report.json");
      if (fs.existsSync(reportFile)) {
        results.sessions[sessionDir] = JSON.parse(
          fs.readFileSync(reportFile, "utf8")
        );
      }
    }

    return results;
  }

  /**
   * 生成統一報告
   */
  async generateUnifiedReport(results) {
    const report = {
      timestamp: new Date().toISOString(),
      analysisStartTime: this.startTime.toISOString(),
      analysisEndTime: new Date().toISOString(),
      duration: new Date() - this.startTime,
      summary: {
        totalSessions: Object.keys(results.sessions).length,
        completedSessions: Object.values(results.sessions).filter(
          (s) => s.status === "completed"
        ).length,
        totalRecommendations: 0,
        highPriorityIssues: 0,
      },
      results,
      recommendations: [],
      implementationPlan: {},
      riskAssessment: {},
    };

    const reportFile = path.join(
      process.cwd(),
      "analysis",
      "reports",
      "unified-analysis-report.json"
    );
    fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  }

  /**
   * 創建實施計劃
   */
  async createImplementationPlan(results) {
    const plan = {
      timestamp: new Date().toISOString(),
      phases: [
        {
          name: "Foundation",
          duration: "1-2 weeks",
          priority: "critical",
          tasks: [],
        },
        {
          name: "Structure",
          duration: "2-3 weeks",
          priority: "high",
          tasks: [],
        },
        {
          name: "Quality",
          duration: "1-2 weeks",
          priority: "medium",
          tasks: [],
        },
        {
          name: "Polish",
          duration: "1 week",
          priority: "low",
          tasks: [],
        },
      ],
    };

    const planFile = path.join(
      process.cwd(),
      "analysis",
      "reports",
      "implementation-plan.json"
    );
    fs.writeFileSync(planFile, JSON.stringify(plan, null, 2));
  }

  /**
   * 生成最終摘要
   */
  async generateFinalSummary() {
    const endTime = new Date();
    const duration = endTime - this.startTime;

    const summary = `# Claude Night Pilot - 專案分析完成報告

**分析完成時間**: ${endTime.toLocaleString("zh-TW")}
**總執行時間**: ${Math.round(duration / 1000 / 60)} 分鐘
**狀態**: ✅ 分析完成

## 📊 分析摘要

本次分析透過 5 個並行的 Claude Code sessions 對專案進行了全面評估：

1. **檔案分析與清理** - 識別過時和無用檔案
2. **CLI 指令分析** - 建立完整的 CLI 文檔和測試
3. **架構重構分析** - 評估和改進專案架構
4. **技術債務清理** - 識別和解決代碼品質問題
5. **監控與協調** - 整合所有分析結果

## 📋 主要成果

- ✅ **完整的專案分析報告**
- ✅ **詳細的實施計劃**
- ✅ **BDD 測試場景**
- ✅ **架構改進建議**
- ✅ **技術債務清理方案**

## 🚀 下一步行動

請查看以下檔案以了解詳細結果：

- \`analysis/reports/unified-analysis-report.json\` - 統一分析報告
- \`analysis/reports/implementation-plan.json\` - 實施計劃
- \`analysis/reports/cli-analysis/\` - CLI 分析結果
- \`analysis/logs/\` - 執行日誌

建議按照實施計劃分階段執行改進措施。

---

**分析工具**: Claude Night Pilot Analysis Orchestrator
**版本**: 1.0.0
`;

    const summaryFile = path.join(
      process.cwd(),
      "analysis",
      "ANALYSIS_COMPLETE.md"
    );
    fs.writeFileSync(summaryFile, summary);

    this.log("📄 Final summary saved to: analysis/ANALYSIS_COMPLETE.md");
  }

  /**
   * 主執行方法
   */
  async run() {
    try {
      this.log("🎯 Starting Claude Night Pilot Project Analysis...");
      this.updateStatus("starting", 0);

      await this.preAnalysisPhase();
      await this.mainAnalysisPhase();
      await this.postAnalysisPhase();

      this.log("🎉 Project Analysis completed successfully!");
      this.updateStatus("completed", 100);
    } catch (error) {
      this.log(`❌ Analysis failed: ${error.message}`);
      this.updateStatus("failed", -1, { error: error.message });
      throw error;
    }
  }
}

// 主執行邏輯
async function main() {
  const orchestrator = new ProjectAnalysisOrchestrator();
  await orchestrator.run();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error("❌ Analysis orchestrator failed:", error);
    process.exit(1);
  });
}

export default ProjectAnalysisOrchestrator;
