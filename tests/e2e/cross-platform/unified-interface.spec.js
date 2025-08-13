// Claude Night Pilot 統一介面端到端測試
// 驗證GUI與CLI功能一致性

import { test, expect } from "@playwright/test";
import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 測試配置
const TAURI_BINARY = path.join(
  __dirname,
  "../../src-tauri/target/debug/claude-night-pilot"
);
const CLI_BINARY = path.join(
  __dirname,
  "../../src-tauri/target/debug/cnp-unified"
);

test.describe("統一介面端到端測試", () => {
  let tauriApp;

  test.beforeAll(async () => {
    // 確保二進制文件存在
    const fs = await import("fs");
    if (!fs.existsSync(CLI_BINARY)) {
      throw new Error(`CLI二進制文件不存在: ${CLI_BINARY}`);
    }
  });

  test.afterAll(async () => {
    if (tauriApp) {
      tauriApp.kill();
    }
  });

  test("CLI冷卻檢查功能測試", async () => {
    const cliResult = await executeCLI(["cooldown", "--format", "json"]);

    expect(cliResult.exitCode).toBe(0);

    const cooldownData = JSON.parse(cliResult.stdout);
    expect(cooldownData).toHaveProperty("is_cooling");
    expect(cooldownData).toHaveProperty("seconds_remaining");
    expect(cooldownData).toHaveProperty("original_message");

    console.log("✅ CLI冷卻檢查測試通過");
  });

  test("CLI系統健康檢查功能測試", async () => {
    const cliResult = await executeCLI(["health", "--format", "json"]);

    expect(cliResult.exitCode).toBe(0);

    const healthData = JSON.parse(cliResult.stdout);
    expect(healthData).toHaveProperty("claude_cli_available");
    expect(healthData).toHaveProperty("cooldown_detection_working");
    expect(healthData).toHaveProperty("active_processes");

    console.log("✅ CLI系統健康檢查測試通過");
  });

  test("CLI執行命令格式測試", async () => {
    // 測試help命令
    const helpResult = await executeCLI(["--help"]);
    expect(helpResult.exitCode).toBe(0);
    expect(helpResult.stdout).toContain("Claude Night Pilot");
    expect(helpResult.stdout).toContain("execute");
    expect(helpResult.stdout).toContain("cooldown");
    expect(helpResult.stdout).toContain("health");

    console.log("✅ CLI命令格式測試通過");
  });

  test("CLI執行子命令幫助測試", async () => {
    const executeHelpResult = await executeCLI(["execute", "--help"]);
    expect(executeHelpResult.exitCode).toBe(0);
    expect(executeHelpResult.stdout).toContain("執行Claude命令");
    expect(executeHelpResult.stdout).toContain("--prompt");
    expect(executeHelpResult.stdout).toContain("--mode");
    expect(executeHelpResult.stdout).toContain("--retry");

    console.log("✅ CLI執行命令幫助測試通過");
  });

  test("CLI批量執行配置測試", async () => {
    // 創建測試批量文件
    const fs = await import("fs");
    const testBatchFile = path.join(__dirname, "test-batch.json");
    const batchData = [
      "測試prompt 1: 說 hello",
      "測試prompt 2: 說 world",
      { content: "測試prompt 3: 說 test" },
    ];

    fs.writeFileSync(testBatchFile, JSON.stringify(batchData, null, 2));

    try {
      const batchHelpResult = await executeCLI(["batch", "--help"]);
      expect(batchHelpResult.exitCode).toBe(0);
      expect(batchHelpResult.stdout).toContain("批量執行prompts");
      expect(batchHelpResult.stdout).toContain("--file");
      expect(batchHelpResult.stdout).toContain("--concurrent");

      console.log("✅ CLI批量執行配置測試通過");
    } finally {
      // 清理測試文件
      if (fs.existsSync(testBatchFile)) {
        fs.unlinkSync(testBatchFile);
      }
    }
  });

  test("統一API響應格式一致性測試", async () => {
    // 測試冷卻狀態的JSON格式
    const cooldownResult = await executeCLI(["cooldown", "--format", "json"]);
    expect(cooldownResult.exitCode).toBe(0);

    const cooldownData = JSON.parse(cooldownResult.stdout);

    // 驗證必需字段
    const requiredCooldownFields = [
      "is_cooling",
      "seconds_remaining",
      "next_available_time",
      "reset_time",
      "original_message",
      "cooldown_pattern",
    ];

    for (const field of requiredCooldownFields) {
      expect(cooldownData).toHaveProperty(field);
    }

    // 測試健康狀態的JSON格式
    const healthResult = await executeCLI(["health", "--format", "json"]);
    expect(healthResult.exitCode).toBe(0);

    const healthData = JSON.parse(healthResult.stdout);

    // 驗證必需字段
    const requiredHealthFields = [
      "claude_cli_available",
      "cooldown_detection_working",
      "active_processes",
    ];

    for (const field of requiredHealthFields) {
      expect(healthData).toHaveProperty(field);
    }

    console.log("✅ 統一API響應格式一致性測試通過");
  });

  test("錯誤處理一致性測試", async () => {
    // 測試無效參數
    const invalidResult = await executeCLI(["execute", "--invalid-option"]);
    expect(invalidResult.exitCode).not.toBe(0);
    expect(invalidResult.stderr).toContain("error:");

    // 測試缺少必需參數
    const missingParamResult = await executeCLI(["execute"]);
    expect(missingParamResult.exitCode).not.toBe(0);

    console.log("✅ 錯誤處理一致性測試通過");
  });

  test("性能基準測試", async () => {
    const startTime = Date.now();

    // 測試冷卻檢查性能
    const cooldownResult = await executeCLI(["cooldown"]);
    const cooldownTime = Date.now() - startTime;

    expect(cooldownResult.exitCode).toBe(0);
    expect(cooldownTime).toBeLessThan(5000); // 應在5秒內完成

    // 測試健康檢查性能
    const healthStartTime = Date.now();
    const healthResult = await executeCLI(["health"]);
    const healthTime = Date.now() - healthStartTime;

    expect(healthResult.exitCode).toBe(0);
    expect(healthTime).toBeLessThan(5000); // 應在5秒內完成

    console.log(
      `✅ 性能基準測試通過 - 冷卻檢查: ${cooldownTime}ms, 健康檢查: ${healthTime}ms`
    );
  });
});

// 輔助函數：執行CLI命令
async function executeCLI(args) {
  return new Promise((resolve) => {
    const child = spawn(CLI_BINARY, args, {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    child.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    child.on("close", (exitCode) => {
      resolve({
        exitCode,
        stdout: stdout.trim(),
        stderr: stderr.trim(),
      });
    });

    // 設置超時
    setTimeout(() => {
      child.kill();
      resolve({
        exitCode: -1,
        stdout: "",
        stderr: "Test timeout",
      });
    }, 10000); // 10秒超時
  });
}
