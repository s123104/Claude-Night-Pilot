#!/usr/bin/env node

/**
 * Claude Night Pilot - 檔案分析工具
 * 檢測過時檔案、無引用檔案和重複代碼
 */

import fs from "fs";
import path from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

class FileAnalyzer {
  constructor() {
    this.projectRoot = process.cwd();
    this.outputDir = path.join(
      this.projectRoot,
      "analysis",
      "reports",
      "file-analysis"
    );
    this.excludeDirs = [
      "node_modules",
      "target",
      ".git",
      "coverage",
      "test-results",
    ];

    fs.mkdirSync(this.outputDir, { recursive: true });
  }

  /**
   * 執行完整的檔案分析
   */
  async analyzeProject() {
    console.log("🔍 Starting comprehensive file analysis...");

    const analysis = {
      timestamp: new Date().toISOString(),
      projectRoot: this.projectRoot,
      summary: {},
      obsoleteFiles: [],
      unreferencedFiles: [],
      duplicateCode: [],
      directoryAnalysis: {},
      recommendations: [],
    };

    // 1. 建立檔案清單
    console.log("📋 Building file inventory...");
    const fileInventory = await this.buildFileInventory();
    analysis.summary.totalFiles = fileInventory.length;

    // 2. 檢測過時檔案
    console.log("⏰ Detecting obsolete files...");
    analysis.obsoleteFiles = await this.detectObsoleteFiles(fileInventory);

    // 3. 檢測無引用檔案
    console.log("🔗 Detecting unreferenced files...");
    analysis.unreferencedFiles = await this.detectUnreferencedFiles(
      fileInventory
    );

    // 4. 檢測重複代碼
    console.log("📄 Detecting duplicate code...");
    analysis.duplicateCode = await this.detectDuplicateCode(fileInventory);

    // 5. 分析目錄結構
    console.log("📁 Analyzing directory structure...");
    analysis.directoryAnalysis = await this.analyzeDirectoryStructure();

    // 6. 生成建議
    console.log("💡 Generating recommendations...");
    analysis.recommendations = this.generateRecommendations(analysis);

    // 7. 保存結果
    await this.saveAnalysis(analysis);

    console.log("✅ File analysis completed!");
    return analysis;
  }

  /**
   * 建立檔案清單
   */
  async buildFileInventory() {
    const files = [];

    const scanDirectory = (dir) => {
      const items = fs.readdirSync(dir, { withFileTypes: true });

      for (const item of items) {
        const fullPath = path.join(dir, item.name);
        const relativePath = path.relative(this.projectRoot, fullPath);

        // 跳過排除的目錄
        if (
          this.excludeDirs.some((exclude) => relativePath.startsWith(exclude))
        ) {
          continue;
        }

        if (item.isDirectory()) {
          scanDirectory(fullPath);
        } else {
          const stats = fs.statSync(fullPath);
          files.push({
            path: relativePath,
            fullPath,
            size: stats.size,
            mtime: stats.mtime,
            extension: path.extname(item.name),
            type: this.getFileType(item.name),
          });
        }
      }
    };

    scanDirectory(this.projectRoot);
    return files;
  }

  /**
   * 獲取檔案類型
   */
  getFileType(filename) {
    const ext = path.extname(filename).toLowerCase();
    const typeMap = {
      ".js": "javascript",
      ".ts": "typescript",
      ".rs": "rust",
      ".html": "html",
      ".css": "css",
      ".md": "markdown",
      ".json": "json",
      ".yaml": "yaml",
      ".yml": "yaml",
      ".toml": "toml",
      ".sh": "shell",
      ".py": "python",
    };

    return typeMap[ext] || "other";
  }

  /**
   * 檢測過時檔案
   */
  async detectObsoleteFiles(fileInventory) {
    const obsoleteFiles = [];
    const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);

    for (const file of fileInventory) {
      // 檢查檔案是否超過 30 天未修改
      if (file.mtime < thirtyDaysAgo) {
        // 檢查是否在 Git 歷史中有最近的活動
        try {
          const { stdout } = await execAsync(
            `git log --since="3 months ago" --name-only -- "${file.path}"`,
            { cwd: this.projectRoot }
          );

          if (!stdout.trim()) {
            obsoleteFiles.push({
              ...file,
              reason: "No recent Git activity and old modification time",
              confidence: 0.8,
            });
          }
        } catch (error) {
          // Git 命令失敗，僅基於修改時間判斷
          obsoleteFiles.push({
            ...file,
            reason: "Old modification time (Git check failed)",
            confidence: 0.6,
          });
        }
      }
    }

    return obsoleteFiles;
  }

  /**
   * 檢測無引用檔案
   */
  async detectUnreferencedFiles(fileInventory) {
    const unreferencedFiles = [];

    // 建立所有檔案的引用映射
    const references = new Map();

    // 掃描所有代碼檔案中的引用
    const codeFiles = fileInventory.filter((f) =>
      ["javascript", "typescript", "rust", "html", "css"].includes(f.type)
    );

    for (const file of codeFiles) {
      try {
        const content = fs.readFileSync(file.fullPath, "utf8");
        const refs = this.extractReferences(content, file.type);

        for (const ref of refs) {
          if (!references.has(ref)) {
            references.set(ref, []);
          }
          references.get(ref).push(file.path);
        }
      } catch (error) {
        console.warn(`⚠️  Could not read file: ${file.path}`);
      }
    }

    // 檢查哪些檔案沒有被引用
    for (const file of fileInventory) {
      const filename = path.basename(file.path);
      const filenameWithoutExt = path.basename(
        file.path,
        path.extname(file.path)
      );

      const isReferenced =
        references.has(file.path) ||
        references.has(filename) ||
        references.has(filenameWithoutExt) ||
        this.isSpecialFile(file.path);

      if (!isReferenced && file.type !== "other") {
        unreferencedFiles.push({
          ...file,
          reason: "No references found in codebase",
          confidence: 0.7,
        });
      }
    }

    return unreferencedFiles;
  }

  /**
   * 提取檔案中的引用
   */
  extractReferences(content, fileType) {
    const references = [];

    switch (fileType) {
      case "javascript":
      case "typescript":
        // import/require 語句
        const jsImports = content.match(
          /(?:import.*from\s+['"]([^'"]+)['"]|require\s*\(\s*['"]([^'"]+)['"]\s*\))/g
        );
        if (jsImports) {
          jsImports.forEach((imp) => {
            const match = imp.match(/['"]([^'"]+)['"]/);
            if (match) references.push(match[1]);
          });
        }
        break;

      case "rust":
        // use 和 mod 語句
        const rustUses = content.match(/(?:use\s+[^;]+|mod\s+\w+)/g);
        if (rustUses) {
          rustUses.forEach((use) => {
            const match = use.match(/(?:use\s+|mod\s+)([^;:\s]+)/);
            if (match) references.push(match[1]);
          });
        }
        break;

      case "html":
        // script, link, img 等標籤
        const htmlRefs = content.match(/(?:src|href)=['"]([^'"]+)['"]/g);
        if (htmlRefs) {
          htmlRefs.forEach((ref) => {
            const match = ref.match(/['"]([^'"]+)['"]/);
            if (match) references.push(match[1]);
          });
        }
        break;

      case "css":
        // @import 和 url() 引用
        const cssRefs = content.match(
          /(?:@import\s+['"]([^'"]+)['"]|url\s*\(\s*['"]?([^'")\s]+)['"]?\s*\))/g
        );
        if (cssRefs) {
          cssRefs.forEach((ref) => {
            const match = ref.match(/['"]?([^'")\s]+)['"]?/);
            if (match) references.push(match[1]);
          });
        }
        break;
    }

    return references;
  }

  /**
   * 檢查是否為特殊檔案（不應被刪除）
   */
  isSpecialFile(filePath) {
    const specialFiles = [
      "package.json",
      "Cargo.toml",
      "README.md",
      "LICENSE",
      ".gitignore",
      "index.html",
      "main.js",
      "lib.rs",
      "main.rs",
    ];

    const filename = path.basename(filePath);
    return (
      specialFiles.includes(filename) ||
      filePath.startsWith("docs/") ||
      filePath.includes("test") ||
      filePath.includes("spec")
    );
  }

  /**
   * 檢測重複代碼
   */
  async detectDuplicateCode(fileInventory) {
    const duplicates = [];

    // 簡單的重複檢測：比較檔案內容的 hash
    const contentHashes = new Map();

    const codeFiles = fileInventory.filter(
      (f) =>
        ["javascript", "typescript", "rust"].includes(f.type) && f.size > 100
    );

    for (const file of codeFiles) {
      try {
        const content = fs.readFileSync(file.fullPath, "utf8");
        const normalizedContent = this.normalizeCode(content);
        const hash = this.simpleHash(normalizedContent);

        if (contentHashes.has(hash)) {
          const existingFile = contentHashes.get(hash);
          duplicates.push({
            files: [existingFile.path, file.path],
            similarity: 1.0,
            lines: content.split("\n").length,
            suggestion:
              "Files appear to be identical - consider removing duplicate",
          });
        } else {
          contentHashes.set(hash, file);
        }
      } catch (error) {
        console.warn(`⚠️  Could not analyze file for duplicates: ${file.path}`);
      }
    }

    return duplicates;
  }

  /**
   * 標準化代碼（移除空白和註釋）
   */
  normalizeCode(content) {
    return content
      .replace(/\/\*[\s\S]*?\*\//g, "") // 移除多行註釋
      .replace(/\/\/.*$/gm, "") // 移除單行註釋
      .replace(/\s+/g, " ") // 標準化空白
      .trim();
  }

  /**
   * 簡單的字串 hash 函數
   */
  simpleHash(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // 轉換為 32 位整數
    }
    return hash;
  }

  /**
   * 分析目錄結構
   */
  async analyzeDirectoryStructure() {
    const structure = {
      depth: {},
      emptyDirectories: [],
      suggestions: [],
    };

    const analyzeDir = (dir, depth = 0) => {
      if (!structure.depth[depth]) structure.depth[depth] = 0;
      structure.depth[depth]++;

      try {
        const items = fs.readdirSync(dir, { withFileTypes: true });
        const relativePath = path.relative(this.projectRoot, dir);

        // 檢查空目錄
        if (items.length === 0 && depth > 0) {
          structure.emptyDirectories.push(relativePath);
        }

        // 遞歸分析子目錄
        for (const item of items) {
          if (item.isDirectory()) {
            const fullPath = path.join(dir, item.name);
            const relPath = path.relative(this.projectRoot, fullPath);

            if (
              !this.excludeDirs.some((exclude) => relPath.startsWith(exclude))
            ) {
              analyzeDir(fullPath, depth + 1);
            }
          }
        }
      } catch (error) {
        console.warn(`⚠️  Could not analyze directory: ${dir}`);
      }
    };

    analyzeDir(this.projectRoot);

    // 生成結構建議
    if (Object.keys(structure.depth).length > 6) {
      structure.suggestions.push({
        type: "deep_nesting",
        message: "Directory structure is deeply nested - consider flattening",
        priority: "medium",
      });
    }

    if (structure.emptyDirectories.length > 0) {
      structure.suggestions.push({
        type: "empty_directories",
        message: `Found ${structure.emptyDirectories.length} empty directories`,
        directories: structure.emptyDirectories,
        priority: "low",
      });
    }

    return structure;
  }

  /**
   * 生成建議
   */
  generateRecommendations(analysis) {
    const recommendations = [];

    // 過時檔案建議
    if (analysis.obsoleteFiles.length > 0) {
      recommendations.push({
        type: "cleanup",
        priority: "high",
        title: "Remove obsolete files",
        description: `Found ${analysis.obsoleteFiles.length} potentially obsolete files`,
        action: "Review and remove files that are no longer needed",
        files: analysis.obsoleteFiles.map((f) => f.path),
        estimatedImpact: "low",
        confidence: 0.8,
      });
    }

    // 無引用檔案建議
    if (analysis.unreferencedFiles.length > 0) {
      recommendations.push({
        type: "cleanup",
        priority: "medium",
        title: "Remove unreferenced files",
        description: `Found ${analysis.unreferencedFiles.length} files with no apparent references`,
        action: "Verify these files are not needed and remove them",
        files: analysis.unreferencedFiles.map((f) => f.path),
        estimatedImpact: "low",
        confidence: 0.7,
      });
    }

    // 重複代碼建議
    if (analysis.duplicateCode.length > 0) {
      recommendations.push({
        type: "refactoring",
        priority: "medium",
        title: "Remove duplicate code",
        description: `Found ${analysis.duplicateCode.length} instances of duplicate code`,
        action: "Extract common code into shared modules",
        duplicates: analysis.duplicateCode,
        estimatedImpact: "medium",
        confidence: 0.9,
      });
    }

    // 目錄結構建議
    if (analysis.directoryAnalysis.suggestions.length > 0) {
      recommendations.push({
        type: "structure",
        priority: "low",
        title: "Improve directory structure",
        description: "Directory structure can be optimized",
        suggestions: analysis.directoryAnalysis.suggestions,
        estimatedImpact: "low",
        confidence: 0.6,
      });
    }

    return recommendations;
  }

  /**
   * 保存分析結果
   */
  async saveAnalysis(analysis) {
    // 保存完整分析結果
    const reportFile = path.join(this.outputDir, "file-analysis-report.json");
    fs.writeFileSync(reportFile, JSON.stringify(analysis, null, 2));

    // 生成清理腳本
    await this.generateCleanupScript(analysis);

    // 生成摘要報告
    await this.generateSummaryReport(analysis);

    console.log(`📊 Analysis report saved to: ${reportFile}`);
  }

  /**
   * 生成清理腳本
   */
  async generateCleanupScript(analysis) {
    const script = `#!/bin/bash

# Claude Night Pilot - Automated File Cleanup Script
# Generated on: ${new Date().toISOString()}

echo "🧹 Starting automated file cleanup..."

# Backup directory
BACKUP_DIR="./file-cleanup-backup-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP_DIR"

echo "📦 Creating backup in $BACKUP_DIR"

# Remove obsolete files
echo "🗑️  Removing obsolete files..."
${analysis.obsoleteFiles
  .map(
    (f) => `
# ${f.reason} (confidence: ${f.confidence})
if [ -f "${f.path}" ]; then
  cp "${f.path}" "$BACKUP_DIR/"
  rm "${f.path}"
  echo "  Removed: ${f.path}"
fi`
  )
  .join("")}

# Remove unreferenced files (with lower confidence)
echo "🔗 Removing unreferenced files..."
${analysis.unreferencedFiles
  .filter((f) => f.confidence > 0.8)
  .map(
    (f) => `
# ${f.reason} (confidence: ${f.confidence})
if [ -f "${f.path}" ]; then
  cp "${f.path}" "$BACKUP_DIR/"
  rm "${f.path}"
  echo "  Removed: ${f.path}"
fi`
  )
  .join("")}

# Remove empty directories
echo "📁 Removing empty directories..."
${analysis.directoryAnalysis.emptyDirectories
  .map(
    (dir) => `
if [ -d "${dir}" ] && [ -z "$(ls -A "${dir}")" ]; then
  rmdir "${dir}"
  echo "  Removed empty directory: ${dir}"
fi`
  )
  .join("")}

echo "✅ Cleanup completed!"
echo "📦 Backup available at: $BACKUP_DIR"
echo "🔄 To restore files: cp $BACKUP_DIR/* ./"
`;

    const scriptFile = path.join(this.outputDir, "cleanup-files.sh");
    fs.writeFileSync(scriptFile, script);

    // 使腳本可執行
    try {
      await execAsync(`chmod +x "${scriptFile}"`);
    } catch (error) {
      console.warn("⚠️  Could not make cleanup script executable");
    }

    console.log(`🧹 Cleanup script saved to: ${scriptFile}`);
  }

  /**
   * 生成摘要報告
   */
  async generateSummaryReport(analysis) {
    const summary = `# File Analysis Summary

**Analysis Date**: ${new Date().toLocaleString()}
**Project**: Claude Night Pilot

## 📊 Overview

- **Total Files Analyzed**: ${analysis.summary.totalFiles}
- **Obsolete Files Found**: ${analysis.obsoleteFiles.length}
- **Unreferenced Files Found**: ${analysis.unreferencedFiles.length}
- **Duplicate Code Instances**: ${analysis.duplicateCode.length}
- **Empty Directories**: ${analysis.directoryAnalysis.emptyDirectories.length}

## 🎯 Recommendations

${analysis.recommendations
  .map(
    (rec) => `
### ${rec.title} (${rec.priority} priority)
${rec.description}

**Action**: ${rec.action}
**Estimated Impact**: ${rec.estimatedImpact}
**Confidence**: ${rec.confidence}
`
  )
  .join("")}

## 📋 Next Steps

1. Review the detailed analysis in \`file-analysis-report.json\`
2. Run the cleanup script: \`./cleanup-files.sh\`
3. Verify the changes and commit the cleanup
4. Monitor for any issues after cleanup

---
Generated by Claude Night Pilot File Analyzer
`;

    const summaryFile = path.join(this.outputDir, "ANALYSIS_SUMMARY.md");
    fs.writeFileSync(summaryFile, summary);

    console.log(`📄 Summary report saved to: ${summaryFile}`);
  }
}

// 主執行邏輯
async function main() {
  const analyzer = new FileAnalyzer();
  await analyzer.analyzeProject();
  console.log("🎉 File analysis completed successfully!");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export default FileAnalyzer;
