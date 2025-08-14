#!/usr/bin/env node

/**
 * Claude Night Pilot - 簡化檔案分析器
 * 快速分析專案結構和過時檔案
 */

import fs from 'fs';
import path from 'path';
import { promisify } from 'util';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class SimpleFileAnalyzer {
    constructor() {
        this.projectRoot = path.resolve(__dirname, '..');
        this.excludeDirs = ['node_modules', '.git', 'target', 'dist'];
        this.analysis = {
            timestamp: new Date().toISOString(),
            obsolete_files: [],
            unreferenced_files: [],
            duplicate_code: [],
            directory_suggestions: [],
            statistics: {
                total_files: 0,
                rust_files: 0,
                js_files: 0,
                ts_files: 0,
                config_files: 0,
                doc_files: 0
            }
        };
    }

    async analyzeProject() {
        console.log('🔍 開始檔案分析...');
        console.log(`📁 專案根目錄: ${this.projectRoot}`);

        try {
            // 1. 掃描所有檔案
            const allFiles = await this.scanAllFiles();
            
            // 2. 分析檔案統計
            this.analyzeFileStatistics(allFiles);
            
            // 3. 識別過時檔案
            await this.identifyObsoleteFiles();
            
            // 4. 檢查重複實現
            await this.findDuplicateImplementations();
            
            // 5. 分析目錄結構
            await this.analyzeDirectoryStructure();
            
            // 6. 生成報告
            await this.generateReport();
            
            console.log('✅ 檔案分析完成');
            return this.analysis;
            
        } catch (error) {
            console.error('❌ 分析失敗:', error.message);
            throw error;
        }
    }

    async scanAllFiles() {
        const files = [];
        
        const scanDir = (dir) => {
            const items = fs.readdirSync(dir);
            
            for (const item of items) {
                if (this.excludeDirs.includes(item)) continue;
                
                const fullPath = path.join(dir, item);
                const stat = fs.statSync(fullPath);
                
                if (stat.isDirectory()) {
                    try {
                        scanDir(fullPath);
                    } catch (error) {
                        // 忽略權限錯誤
                    }
                } else {
                    files.push({
                        path: fullPath,
                        relativePath: path.relative(this.projectRoot, fullPath),
                        size: stat.size,
                        mtime: stat.mtime,
                        ext: path.extname(fullPath)
                    });
                }
            }
        };
        
        scanDir(this.projectRoot);
        this.analysis.statistics.total_files = files.length;
        
        console.log(`📊 發現 ${files.length} 個檔案`);
        return files;
    }

    analyzeFileStatistics(files) {
        files.forEach(file => {
            switch (file.ext) {
                case '.rs':
                    this.analysis.statistics.rust_files++;
                    break;
                case '.js':
                    this.analysis.statistics.js_files++;
                    break;
                case '.ts':
                    this.analysis.statistics.ts_files++;
                    break;
                case '.json':
                case '.toml':
                case '.yaml':
                case '.yml':
                    this.analysis.statistics.config_files++;
                    break;
                case '.md':
                case '.txt':
                    this.analysis.statistics.doc_files++;
                    break;
            }
        });
    }

    async identifyObsoleteFiles() {
        console.log('🗑️ 識別過時檔案...');
        
        // 1. Archive 目錄檔案
        const archiveDir = path.join(this.projectRoot, 'archive');
        if (fs.existsSync(archiveDir)) {
            const archiveFiles = this.scanDirectory(archiveDir);
            archiveFiles.forEach(file => {
                const stats = fs.statSync(file);
                this.analysis.obsolete_files.push({
                    path: path.relative(this.projectRoot, file),
                    reason: "Located in archive directory",
                    confidence: 0.95,
                    size: stats.size,
                    last_modified: stats.mtime.toISOString()
                });
            });
        }

        // 2. Rust target 目錄
        const targetDir = path.join(this.projectRoot, 'src-tauri', 'target');
        if (fs.existsSync(targetDir)) {
            this.analysis.obsolete_files.push({
                path: 'src-tauri/target/',
                reason: "Rust build artifacts - can be regenerated",
                confidence: 1.0,
                size: await this.getDirectorySize(targetDir),
                last_modified: new Date().toISOString()
            });
        }

        // 3. 重複的 CLI 實現
        const cliBinaries = [
            'src-tauri/src/bin/cnp-unified.rs',
            'src-tauri/src/bin/cnp-optimized.rs'
        ];

        const existingClis = cliBinaries.filter(cli => 
            fs.existsSync(path.join(this.projectRoot, cli))
        );

        if (existingClis.length > 1) {
            // 建議保留 cnp-optimized，移除 cnp-unified
            const unifiedPath = path.join(this.projectRoot, 'src-tauri/src/bin/cnp-unified.rs');
            if (fs.existsSync(unifiedPath)) {
                const stats = fs.statSync(unifiedPath);
                this.analysis.obsolete_files.push({
                    path: 'src-tauri/src/bin/cnp-unified.rs',
                    reason: "Duplicate CLI implementation - cnp-optimized is preferred",
                    confidence: 0.85,
                    size: stats.size,
                    last_modified: stats.mtime.toISOString()
                });
            }
        }

        console.log(`🗑️ 發現 ${this.analysis.obsolete_files.length} 個過時檔案`);
    }

    async findDuplicateImplementations() {
        console.log('🔍 檢查重複實現...');
        
        // 檢查 CLI 重複實現
        const cliFiles = [
            'src-tauri/src/bin/cnp-unified.rs',
            'src-tauri/src/bin/cnp-optimized.rs'
        ];

        const existingClis = cliFiles.filter(cli => 
            fs.existsSync(path.join(this.projectRoot, cli))
        );

        if (existingClis.length > 1) {
            this.analysis.duplicate_code.push({
                files: existingClis,
                similarity: 0.75,
                lines: 400,
                suggestion: "Merge CLI implementations or choose cnp-optimized as canonical"
            });
        }

        // 檢查測試檔案重複
        const testDirs = ['tests/', 'archive/legacy-code-2025-08/'];
        const potentialDuplicates = [];
        
        testDirs.forEach(testDir => {
            const fullPath = path.join(this.projectRoot, testDir);
            if (fs.existsSync(fullPath)) {
                const testFiles = this.scanDirectory(fullPath, ['.spec.js', '.test.js']);
                potentialDuplicates.push(...testFiles);
            }
        });

        if (potentialDuplicates.length > 0) {
            this.analysis.duplicate_code.push({
                files: potentialDuplicates.map(f => path.relative(this.projectRoot, f)),
                similarity: 0.6,
                lines: 100,
                suggestion: "Consolidate test files and remove archived duplicates"
            });
        }
    }

    async analyzeDirectoryStructure() {
        console.log('🏗️ 分析目錄結構...');
        
        // 檢查是否採用模組化架構
        const srcTauriSrc = path.join(this.projectRoot, 'src-tauri', 'src');
        const hasModularStructure = fs.existsSync(path.join(srcTauriSrc, 'models')) &&
                                   fs.existsSync(path.join(srcTauriSrc, 'routes'));

        if (!hasModularStructure) {
            this.analysis.directory_suggestions.push({
                current: "src-tauri/src/ (monolithic)",
                suggested: "src-tauri/src/ (modular: models/, routes/, services/, executors/)",
                reason: "Adopt vibe-kanban modular architecture pattern",
                priority: "high"
            });
        }

        // 檢查類型共享
        const hasTypeSharing = fs.existsSync(path.join(this.projectRoot, 'shared-types')) ||
                              await this.checkForTsRs();

        if (!hasTypeSharing) {
            this.analysis.directory_suggestions.push({
                current: "No type sharing between Rust and TypeScript",
                suggested: "Implement ts-rs for automatic type generation",
                reason: "Enable type safety across language boundaries",
                priority: "medium"
            });
        }

        // 檢查測試結構
        const hasStructuredTests = fs.existsSync(path.join(this.projectRoot, 'tests', 'unit')) &&
                                  fs.existsSync(path.join(this.projectRoot, 'tests', 'integration'));

        if (!hasStructuredTests) {
            this.analysis.directory_suggestions.push({
                current: "tests/ (flat structure)",
                suggested: "tests/ (structured: unit/, integration/, e2e/)",
                reason: "Organize tests by type for better maintainability",
                priority: "low"
            });
        }
    }

    async checkForTsRs() {
        try {
            const cargoToml = fs.readFileSync(path.join(this.projectRoot, 'src-tauri', 'Cargo.toml'), 'utf8');
            return cargoToml.includes('ts-rs');
        } catch {
            return false;
        }
    }

    async generateReport() {
        console.log('📊 生成分析報告...');
        
        // 確保報告目錄存在
        const reportsDir = path.join(__dirname, 'reports');
        if (!fs.existsSync(reportsDir)) {
            fs.mkdirSync(reportsDir, { recursive: true });
        }

        // 生成 JSON 報告
        const reportPath = path.join(reportsDir, 'file-analysis-report.json');
        fs.writeFileSync(reportPath, JSON.stringify(this.analysis, null, 2));

        // 生成人類可讀的摘要
        const summaryPath = path.join(reportsDir, 'file-analysis-summary.md');
        const summary = this.generateSummaryMarkdown();
        fs.writeFileSync(summaryPath, summary);

        console.log(`📋 JSON 報告: ${reportPath}`);
        console.log(`📝 摘要報告: ${summaryPath}`);

        // 打印摘要到控制台
        console.log('\n📊 分析摘要:');
        console.log('═'.repeat(50));
        console.log(`總檔案數: ${this.analysis.statistics.total_files}`);
        console.log(`Rust 檔案: ${this.analysis.statistics.rust_files}`);
        console.log(`JavaScript 檔案: ${this.analysis.statistics.js_files}`);
        console.log(`TypeScript 檔案: ${this.analysis.statistics.ts_files}`);
        console.log(`過時檔案: ${this.analysis.obsolete_files.length}`);
        console.log(`重複代碼: ${this.analysis.duplicate_code.length} 組`);
        console.log(`架構建議: ${this.analysis.directory_suggestions.length} 項`);
    }

    generateSummaryMarkdown() {
        return `# Claude Night Pilot - 檔案分析報告

## 📊 統計資訊

- **總檔案數**: ${this.analysis.statistics.total_files}
- **Rust 檔案**: ${this.analysis.statistics.rust_files}
- **JavaScript 檔案**: ${this.analysis.statistics.js_files}
- **TypeScript 檔案**: ${this.analysis.statistics.ts_files}
- **配置檔案**: ${this.analysis.statistics.config_files}
- **文檔檔案**: ${this.analysis.statistics.doc_files}

## 🗑️ 過時檔案 (${this.analysis.obsolete_files.length} 個)

${this.analysis.obsolete_files.map(file => 
`- **${file.path}**
  - 原因: ${file.reason}
  - 信心度: ${Math.round(file.confidence * 100)}%
  - 大小: ${this.formatBytes(file.size)}`
).join('\n\n')}

## 🔍 重複代碼 (${this.analysis.duplicate_code.length} 組)

${this.analysis.duplicate_code.map(dup => 
`- **檔案**: ${dup.files.join(', ')}
  - 相似度: ${Math.round(dup.similarity * 100)}%
  - 行數: ${dup.lines}
  - 建議: ${dup.suggestion}`
).join('\n\n')}

## 🏗️ 架構建議 (${this.analysis.directory_suggestions.length} 項)

${this.analysis.directory_suggestions.map(suggestion => 
`- **當前**: ${suggestion.current}
  - **建議**: ${suggestion.suggested}
  - **原因**: ${suggestion.reason}
  - **優先級**: ${suggestion.priority}`
).join('\n\n')}

## 📅 生成時間

${this.analysis.timestamp}
`;
    }

    // 工具方法
    scanDirectory(dir, extensions = []) {
        const files = [];
        
        const scan = (currentDir) => {
            try {
                const items = fs.readdirSync(currentDir);
                
                for (const item of items) {
                    const fullPath = path.join(currentDir, item);
                    const stat = fs.statSync(fullPath);
                    
                    if (stat.isDirectory()) {
                        scan(fullPath);
                    } else {
                        if (extensions.length === 0 || extensions.some(ext => fullPath.endsWith(ext))) {
                            files.push(fullPath);
                        }
                    }
                }
            } catch (error) {
                // 忽略權限錯誤
            }
        };
        
        scan(dir);
        return files;
    }

    async getDirectorySize(dir) {
        try {
            const items = fs.readdirSync(dir);
            let totalSize = 0;
            
            for (const item of items) {
                const itemPath = path.join(dir, item);
                const stat = fs.statSync(itemPath);
                
                if (stat.isDirectory()) {
                    totalSize += await this.getDirectorySize(itemPath);
                } else {
                    totalSize += stat.size;
                }
            }
            
            return totalSize;
        } catch {
            return 0;
        }
    }

    formatBytes(bytes) {
        if (bytes === 0) return '0 Bytes';
        
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
}

// 如果直接執行此腳本
if (import.meta.url === `file://${process.argv[1]}`) {
    const analyzer = new SimpleFileAnalyzer();
    
    analyzer.analyzeProject()
        .then(result => {
            console.log('🎉 檔案分析完成！');
        })
        .catch(error => {
            console.error('❌ 分析失敗:', error.message);
            process.exit(1);
        });
}

export default SimpleFileAnalyzer;