#!/usr/bin/env node

/**
 * Claude Night Pilot - 隱私檢測和檔案清理工具
 * 根據 CLAUDE.md 規則進行企業級隱私保護
 */

import fs from "fs";
import "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

class PrivacyAuditor {
  constructor() {
    this.sensitivePatterns = [
      // 用戶路徑
      /\/Users\/[^\/\s]+/g,
      /\/home\/[^\/\s]+/g,
      /C:\\Users\\[^\\]+/g,

      // 特定用戶名 (示例模式)
      /[a-zA-Z0-9]+\.eth/g,

      // 敏感檔案路徑 (示例模式)
      /\/Users\/[^\/\s]+\/[^\s]*/g,

      // 其他可能的敏感信息
      /localhost:\d+/g,
      /127\.0\.0\.1:\d+/g,

      // 臨時路徑
      /\/tmp\/[^\s]*/g,
      /\/var\/tmp\/[^\s]*/g,
    ];

    this.replacements = new Map([
      [/\/Users\/[^\/\s]+/g, "/Users/[USER]"],
      [/\/home\/[^\/\s]+/g, "/home/[USER]"],
      [/C:\\Users\\[^\\]+/g, "C:\\Users\\[USER]"],
      [/[a-zA-Z0-9]+\.eth/g, "[USER]"],
      [/\/Users\/[^\/\s]+\/[^\s]*/g, "/Users/[USER]/[PROJECT_PATH]"],
      [/localhost:\d+/g, "localhost:[PORT]"],
      [/127\.0\.0\.1:\d+/g, "127.0.0.1:[PORT]"],
      [/\/tmp\/[^\s]*/g, "/tmp/[TEMP_PATH]"],
      [/\/var\/tmp\/[^\s]*/g, "/var/tmp/[TEMP_PATH]"],
    ]);

    this.issues = [];
    this.fixedFiles = [];
  }

  async auditProject() {
    console.log("🔍 開始隱私檢測和檔案整理...\n");

    // 1. 檢查未提交的檔案
    await this.checkUncommittedFiles();

    // 2. 掃描敏感信息
    await this.scanSensitiveContent();

    // 3. 檢查 .gitignore 配置
    await this.validateGitignore();

    // 4. 清理臨時檔案
    await this.cleanupTempFiles();

    // 5. 生成報告
    await this.generateReport();

    console.log("✅ 隱私檢測和檔案整理完成！");
  }

  async checkUncommittedFiles() {
    console.log("📋 檢查未提交的檔案...");

    try {
      const { stdout } = await execAsync("git status --porcelain");
      const files = stdout
        .trim()
        .split("\n")
        .filter((line) => line.trim());

      console.log(`   發現 ${files.length} 個未提交的檔案`);

      for (const file of files) {
        const status = file.substring(0, 2);
        const filename = file.substring(3);

        // 檢查是否為敏感檔案
        if (this.isSensitiveFile(filename)) {
          this.issues.push({
            type: "sensitive_file",
            file: filename,
            status: status,
            severity: "high",
          });
        }
      }
    } catch (error) {
      console.log("   ⚠️  無法檢查 Git 狀態:", error.message);
    }
  }

  async scanSensitiveContent() {
    console.log("🔍 掃描敏感內容...");

    const filesToScan = [
      "test-analysis-report.md",
      "tests/test-schedule.yaml",
      "ERROR_FIXES_REPORT.md",
      "COMPLETE_ERROR_FIXES_REPORT.md",
      "TIMEOUT_FIXES_REPORT.md",
    ];

    for (const file of filesToScan) {
      if (fs.existsSync(file)) {
        await this.scanFile(file);
      }
    }
  }

  async scanFile(filePath) {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      let hasIssues = false;
      let cleanedContent = content;

      for (const [pattern, replacement] of this.replacements) {
        if (pattern.test(content)) {
          hasIssues = true;
          cleanedContent = cleanedContent.replace(pattern, replacement);

          this.issues.push({
            type: "sensitive_content",
            file: filePath,
            pattern: pattern.toString(),
            severity: "medium",
          });
        }
      }

      if (hasIssues) {
        // 備份原檔案
        fs.writeFileSync(`${filePath}.backup`, content);

        // 寫入清理後的內容
        fs.writeFileSync(filePath, cleanedContent);

        this.fixedFiles.push(filePath);
        console.log(`   ✅ 已清理: ${filePath}`);
      }
    } catch (error) {
      console.log(`   ❌ 無法處理檔案 ${filePath}:`, error.message);
    }
  }

  async validateGitignore() {
    console.log("📝 檢查 .gitignore 配置...");

    const requiredIgnores = [
      "*.db",
      "*.db-wal",
      "*.db-shm",
      "test-results/",
      "playwright-report/",
      "coverage/",
      "*.log",
      "logs/",
      ".env",
      ".env.local",
      "node_modules/",
      "target/",
      "src-tauri/target/",
      "*.backup",
      "*-temp.md",
      "*-draft.md",
      "temp/",
      "tmp/",
    ];

    try {
      const gitignoreContent = fs.readFileSync(".gitignore", "utf8");
      const missingRules = [];

      for (const rule of requiredIgnores) {
        if (!gitignoreContent.includes(rule)) {
          missingRules.push(rule);
        }
      }

      if (missingRules.length > 0) {
        console.log(`   ⚠️  缺少 ${missingRules.length} 個 .gitignore 規則`);
        this.issues.push({
          type: "missing_gitignore_rules",
          rules: missingRules,
          severity: "medium",
        });
      } else {
        console.log("   ✅ .gitignore 配置完整");
      }
    } catch (error) {
      console.log("   ❌ 無法檢查 .gitignore:", error.message);
    }
  }

  async cleanupTempFiles() {
    console.log("🧹 清理臨時檔案...");

    const tempPatterns = [
      "*.backup",
      "*.tmp",
      "*.temp",
      "*~",
      ".DS_Store",
      "Thumbs.db",
      "*.log",
      "claude-pilot.db*",
    ];

    let cleanedCount = 0;

    for (const pattern of tempPatterns) {
      try {
        const { stdout } = await execAsync(`find . -name "${pattern}" -type f`);
        const files = stdout
          .trim()
          .split("\n")
          .filter((f) => f.trim());

        for (const file of files) {
          if (file && fs.existsSync(file)) {
            fs.unlinkSync(file);
            cleanedCount++;
            console.log(`   🗑️  已刪除: ${file}`);
          }
        }
      } catch (error) {
        // 忽略 find 命令的錯誤（可能是沒有找到檔案）
      }
    }

    console.log(`   ✅ 已清理 ${cleanedCount} 個臨時檔案`);
  }

  isSensitiveFile(filename) {
    const sensitiveFiles = [
      "claude-pilot.db",
      "*.db-wal",
      "*.db-shm",
      "*.log",
      "*.backup",
      "test-results/",
      "playwright-report/",
    ];

    return sensitiveFiles.some((pattern) => {
      if (pattern.includes("*")) {
        const regex = new RegExp(pattern.replace("*", ".*"));
        return regex.test(filename);
      }
      return filename.includes(pattern);
    });
  }

  async generateReport() {
    console.log("📊 生成隱私檢測報告...");

    const report = `# Claude Night Pilot - 隱私檢測報告

**檢測時間**: ${new Date().toLocaleString("zh-TW")}
**檢測範圍**: 全專案隱私和安全檢測

## 🎯 檢測摘要

- **發現問題**: ${this.issues.length} 個
- **修復檔案**: ${this.fixedFiles.length} 個
- **檢測狀態**: ${this.issues.length === 0 ? "✅ 通過" : "⚠️ 需要注意"}

## 🔍 發現的問題

${
  this.issues.length === 0
    ? "✅ 沒有發現隱私問題"
    : this.issues
        .map(
          (issue, i) => `
### ${i + 1}. ${issue.type}
- **檔案**: ${issue.file || "N/A"}
- **嚴重程度**: ${issue.severity}
- **詳情**: ${issue.pattern || issue.rules?.join(", ") || "詳見上下文"}
`
        )
        .join("")
}

## 🔧 已修復的檔案

${
  this.fixedFiles.length === 0
    ? "沒有需要修復的檔案"
    : this.fixedFiles.map((file) => `- ${file} (已創建備份)`).join("\n")
}

## 📋 建議的後續行動

### 立即行動
1. **檢查修復結果**: 確認清理後的檔案內容正確
2. **提交變更**: 將清理後的檔案提交到 Git
3. **刪除備份**: 確認無誤後刪除 .backup 檔案

### 長期維護
1. **定期檢測**: 每次提交前運行隱私檢測
2. **團隊培訓**: 確保團隊成員了解隱私保護規範
3. **自動化**: 將隱私檢測集成到 CI/CD 流程

## 🛡️ 隱私保護規範

### 禁止提交的內容
- 用戶路徑 (/Users/username, /home/username)
- 個人識別信息 (用戶名、郵箱等)
- 本地端口和 IP 地址
- 臨時檔案和日誌
- 資料庫檔案

### 推薦的替代方案
- 使用環境變數或配置檔案
- 使用相對路徑而非絕對路徑
- 使用佔位符 ([USER], [PROJECT_PATH] 等)
- 確保 .gitignore 規則完整

---

**檢測完成**: ${new Date().toLocaleString("zh-TW")}
**狀態**: ${
      this.issues.length === 0 ? "✅ 隱私保護合規" : "⚠️ 需要處理發現的問題"
    }
`;

    fs.writeFileSync("PRIVACY_AUDIT_REPORT.md", report);
    console.log("   ✅ 報告已生成: PRIVACY_AUDIT_REPORT.md");
  }
}

// 執行隱私檢測
if (import.meta.url === `file://${process.argv[1]}`) {
  const auditor = new PrivacyAuditor();
  auditor.auditProject().catch(console.error);
}

export default PrivacyAuditor;
