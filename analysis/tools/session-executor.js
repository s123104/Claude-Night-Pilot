#!/usr/bin/env node

/**
 * Claude Night Pilot - Session 執行器
 * 管理多個 Claude Code sessions 的並行執行
 */

import fs from "fs";
import path from "path";
import { spawn } from "child_process";
import { promisify } from "util";

const sleep = promisify(setTimeout);

class SessionExecutor {
  constructor() {
    this.sessions = new Map();
    this.logDir = path.join(process.cwd(), "analysis", "logs");
    this.reportDir = path.join(process.cwd(), "analysis", "reports");

    // 確保目錄存在
    fs.mkdirSync(this.logDir, { recursive: true });
    fs.mkdirSync(this.reportDir, { recursive: true });
  }

  /**
   * 註冊一個 session
   */
  registerSession(sessionId, config) {
    this.sessions.set(sessionId, {
      id: sessionId,
      status: "pending",
      progress: 0,
      startTime: null,
      endTime: null,
      config,
      process: null,
      logFile: path.join(this.logDir, `${sessionId}.log`),
      statusFile: path.join(this.logDir, `${sessionId}.status`),
      reportFile: path.join(this.reportDir, `${sessionId}`, "report.json"),
    });
  }

  /**
   * 啟動一個 session
   */
  async startSession(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session ${sessionId} not found`);
    }

    session.status = "starting";
    session.startTime = new Date();
    this.updateSessionStatus(sessionId);

    console.log(`🚀 Starting session: ${sessionId}`);

    // 創建 Claude Code prompt
    const prompt = this.generatePrompt(session);

    // 模擬 Claude Code 執行 (實際應該調用 Claude Code API)
    session.process = spawn(
      "node",
      [
        "-e",
        `
      console.log('Simulating Claude Code execution for ${sessionId}');
      console.log('Prompt:', ${JSON.stringify(prompt)});
      
      // 模擬進度更新
      let progress = 0;
      const interval = setInterval(() => {
        progress += Math.random() * 20;
        if (progress >= 100) {
          progress = 100;
          clearInterval(interval);
          console.log('Session completed');
          process.exit(0);
        }
        console.log(\`Progress: \${Math.round(progress)}%\`);
      }, 2000);
    `,
      ],
      {
        stdio: ["pipe", "pipe", "pipe"],
      }
    );

    // 處理輸出
    session.process.stdout.on("data", (data) => {
      const output = data.toString();
      fs.appendFileSync(session.logFile, output);

      // 解析進度
      const progressMatch = output.match(/Progress: (\\d+)%/);
      if (progressMatch) {
        session.progress = parseInt(progressMatch[1]);
        this.updateSessionStatus(sessionId);
      }
    });

    session.process.stderr.on("data", (data) => {
      fs.appendFileSync(session.logFile, `ERROR: ${data.toString()}`);
    });

    session.process.on("close", (code) => {
      session.status = code === 0 ? "completed" : "failed";
      session.endTime = new Date();
      session.progress = 100;
      this.updateSessionStatus(sessionId);

      if (code === 0) {
        this.generateSessionReport(sessionId);
      }

      console.log(`✅ Session ${sessionId} ${session.status}`);
    });

    session.status = "running";
    this.updateSessionStatus(sessionId);
  }

  /**
   * 生成 Claude Code prompt
   */
  generatePrompt(session) {
    const sessionFile = path.join(
      process.cwd(),
      "analysis",
      "sessions",
      `${session.id}.md`
    );
    const sessionContent = fs.readFileSync(sessionFile, "utf8");

    return `
# Claude Code Session: ${session.id}

You are working on the Claude Night Pilot project analysis. Your specific task is defined below:

${sessionContent}

## Project Context
- Project: Claude Night Pilot (Tauri + Rust + JavaScript)
- Goal: Comprehensive project analysis and refactoring
- Your role: ${session.config.role}

## Instructions
1. Read and understand your specific task requirements
2. Analyze the current project structure
3. Generate detailed analysis reports in JSON format
4. Provide actionable recommendations
5. Create implementation scripts where applicable

## Output Requirements
- Save analysis results to: analysis/reports/${session.id}/
- Use structured JSON format for machine-readable results
- Include human-readable summaries
- Provide confidence scores for recommendations

Please begin your analysis now.
`;
  }

  /**
   * 更新 session 狀態
   */
  updateSessionStatus(sessionId) {
    const session = this.sessions.get(sessionId);
    const status = {
      id: sessionId,
      status: session.status,
      progress: session.progress,
      startTime: session.startTime?.toISOString(),
      endTime: session.endTime?.toISOString(),
      lastUpdate: new Date().toISOString(),
    };

    fs.writeFileSync(session.statusFile, JSON.stringify(status, null, 2));
  }

  /**
   * 生成 session 報告
   */
  generateSessionReport(sessionId) {
    const session = this.sessions.get(sessionId);
    const reportDir = path.dirname(session.reportFile);

    fs.mkdirSync(reportDir, { recursive: true });

    // 模擬報告生成
    const report = {
      sessionId,
      timestamp: new Date().toISOString(),
      status: "completed",
      duration: session.endTime - session.startTime,
      results: {
        // 這裡應該包含實際的分析結果
        summary: `Analysis completed for ${sessionId}`,
        recommendations: [],
        metrics: {},
      },
    };

    fs.writeFileSync(session.reportFile, JSON.stringify(report, null, 2));
  }

  /**
   * 啟動所有 sessions
   */
  async startAllSessions() {
    console.log("🎯 Starting all analysis sessions...");

    // 按依賴順序啟動
    const startOrder = [
      "session-1-file-analysis",
      "session-2-cli-analysis",
      "session-3-architecture-refactoring",
      "session-4-technical-debt",
      "session-5-monitoring-coordination",
    ];

    for (const sessionId of startOrder) {
      if (this.sessions.has(sessionId)) {
        await this.startSession(sessionId);
        await sleep(1000); // 間隔啟動
      }
    }
  }

  /**
   * 監控所有 sessions
   */
  async monitorSessions() {
    console.log("📊 Monitoring sessions...");

    const interval = setInterval(() => {
      const statuses = Array.from(this.sessions.values()).map((session) => ({
        id: session.id,
        status: session.status,
        progress: session.progress,
      }));

      console.clear();
      console.log("📊 Session Status Dashboard");
      console.log("=".repeat(50));

      statuses.forEach((status) => {
        const progressBar =
          "█".repeat(Math.floor(status.progress / 5)) +
          "░".repeat(20 - Math.floor(status.progress / 5));
        console.log(
          `${status.id.padEnd(30)} [${progressBar}] ${status.progress}% ${
            status.status
          }`
        );
      });

      // 檢查是否所有 sessions 都完成
      const allCompleted = statuses.every(
        (s) => s.status === "completed" || s.status === "failed"
      );
      if (allCompleted) {
        clearInterval(interval);
        console.log("\n✅ All sessions completed!");
        this.generateConsolidatedReport();
      }
    }, 2000);
  }

  /**
   * 生成整合報告
   */
  generateConsolidatedReport() {
    console.log("📋 Generating consolidated report...");

    const consolidatedReport = {
      timestamp: new Date().toISOString(),
      sessions: {},
      summary: {
        totalSessions: this.sessions.size,
        completedSessions: 0,
        failedSessions: 0,
      },
      recommendations: [],
      nextSteps: [],
    };

    // 收集各 session 的結果
    for (const [sessionId, session] of this.sessions) {
      if (fs.existsSync(session.reportFile)) {
        const report = JSON.parse(fs.readFileSync(session.reportFile, "utf8"));
        consolidatedReport.sessions[sessionId] = report;

        if (report.status === "completed") {
          consolidatedReport.summary.completedSessions++;
        } else {
          consolidatedReport.summary.failedSessions++;
        }
      }
    }

    const consolidatedFile = path.join(
      this.reportDir,
      "consolidated-report.json"
    );
    fs.writeFileSync(
      consolidatedFile,
      JSON.stringify(consolidatedReport, null, 2)
    );

    console.log(`📄 Consolidated report saved to: ${consolidatedFile}`);
  }
}

// 主執行邏輯
async function main() {
  const executor = new SessionExecutor();

  // 註冊所有 sessions
  executor.registerSession("session-1-file-analysis", {
    role: "File Analysis Specialist",
    priority: 1,
  });

  executor.registerSession("session-2-cli-analysis", {
    role: "CLI Testing Specialist",
    priority: 1,
  });

  executor.registerSession("session-3-architecture-refactoring", {
    role: "Architecture Specialist",
    priority: 2,
  });

  executor.registerSession("session-4-technical-debt", {
    role: "Code Quality Specialist",
    priority: 2,
  });

  executor.registerSession("session-5-monitoring-coordination", {
    role: "Project Coordinator",
    priority: 3,
  });

  // 啟動所有 sessions
  await executor.startAllSessions();

  // 開始監控
  await executor.monitorSessions();
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export default SessionExecutor;
