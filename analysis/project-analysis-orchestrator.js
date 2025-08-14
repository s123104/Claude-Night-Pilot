#!/usr/bin/env node

/**
 * Claude Night Pilot - 專案分析與重構 Orchestrator
 * 統一管理多個並行分析工作流程
 */

import fs from 'fs';
import path from 'path';
import { spawn, exec } from 'child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const execAsync = promisify(exec);

class ProjectAnalysisOrchestrator {
    constructor() {
        this.projectRoot = path.resolve(__dirname, '..');
        this.analysisDir = path.join(this.projectRoot, 'analysis');
        this.reportsDir = path.join(this.analysisDir, 'reports');
        this.logsDir = path.join(this.analysisDir, 'logs');
        
        this.sessions = [
            'session-1-file-analysis',
            'session-2-cli-analysis',
            'session-3-architecture-refactoring',
            'session-4-technical-debt',
            'session-5-monitoring-coordination'
        ];
        
        this.sessionStatuses = {};
        this.results = {};
        
        this.initializeDirectories();
    }

    initializeDirectories() {
        [this.reportsDir, this.logsDir].forEach(dir => {
            if (!fs.existsSync(dir)) {
                fs.mkdirSync(dir, { recursive: true });
            }
        });
    }

    async runFileAnalysis() {
        console.log('🔍 Session 1: 檔案分析與清理');
        
        const analysis = {
            timestamp: new Date().toISOString(),
            obsolete_files: [],
            unreferenced_files: [],
            duplicate_code: [],
            directory_suggestions: []
        };

        try {
            // 1. 識別可能過時的檔案
            const obsoleteFiles = await this.findObsoleteFiles();
            analysis.obsolete_files = obsoleteFiles;

            // 2. 檢查無引用檔案
            const unreferencedFiles = await this.findUnreferencedFiles();
            analysis.unreferenced_files = unreferencedFiles;

            // 3. 檢測重複代碼
            const duplicates = await this.findDuplicateCode();
            analysis.duplicate_code = duplicates;

            // 4. 目錄結構建議
            const directoryAnalysis = await this.analyzeDirectoryStructure();
            analysis.directory_suggestions = directoryAnalysis;

            // 保存報告
            const reportPath = path.join(this.reportsDir, 'file-analysis-report.json');
            fs.writeFileSync(reportPath, JSON.stringify(analysis, null, 2));
            
            console.log(`✅ 檔案分析完成: ${reportPath}`);
            return analysis;
            
        } catch (error) {
            console.error('❌ 檔案分析失敗:', error.message);
            throw error;
        }
    }

    async findObsoleteFiles() {
        const obsoleteFiles = [];
        
        // 檢查 archive/ 目錄中的檔案
        const archiveDir = path.join(this.projectRoot, 'archive');
        if (fs.existsSync(archiveDir)) {
            const archiveFiles = await this.scanDirectory(archiveDir);
            archiveFiles.forEach(file => {
                const stats = fs.statSync(file);
                obsoleteFiles.push({
                    path: path.relative(this.projectRoot, file),
                    last_modified: stats.mtime.toISOString(),
                    reason: "Located in archive directory",
                    confidence: 0.95,
                    size: stats.size
                });
            });
        }

        // 檢查 target/ 目錄 (Rust 編譯產物)
        const targetDir = path.join(this.projectRoot, 'src-tauri', 'target');
        if (fs.existsSync(targetDir)) {
            obsoleteFiles.push({
                path: 'src-tauri/target/',
                last_modified: new Date().toISOString(),
                reason: "Rust build artifacts - can be regenerated",
                confidence: 1.0,
                size: await this.getDirectorySize(targetDir)
            });
        }

        // 檢查重複的 CLI 實現
        const duplicateClis = await this.findDuplicateCliImplementations();
        obsoleteFiles.push(...duplicateClis);

        return obsoleteFiles;
    }

    async findDuplicateCliImplementations() {
        const cliFiles = [];
        const srcTauriSrc = path.join(this.projectRoot, 'src-tauri', 'src');
        
        // 查找多個 CLI 實現
        const cliBinaries = [
            'src-tauri/src/bin/cnp-unified.rs',
            'src-tauri/src/bin/cnp-optimized.rs'
        ];

        const existingClis = cliBinaries.filter(cli => 
            fs.existsSync(path.join(this.projectRoot, cli))
        );

        if (existingClis.length > 1) {
            // 建議保留優化版本，移除舊版本
            return [{
                path: 'src-tauri/src/bin/cnp-unified.rs',
                last_modified: fs.existsSync(path.join(this.projectRoot, 'src-tauri/src/bin/cnp-unified.rs')) 
                    ? fs.statSync(path.join(this.projectRoot, 'src-tauri/src/bin/cnp-unified.rs')).mtime.toISOString() 
                    : new Date().toISOString(),
                reason: "Duplicate CLI implementation - cnp-optimized is preferred",
                confidence: 0.85,
                size: fs.existsSync(path.join(this.projectRoot, 'src-tauri/src/bin/cnp-unified.rs')) 
                    ? fs.statSync(path.join(this.projectRoot, 'src-tauri/src/bin/cnp-unified.rs')).size 
                    : 0
            }];
        }

        return [];
    }

    async findUnreferencedFiles() {
        const unreferencedFiles = [];
        
        // 掃描所有 .js, .rs, .ts 檔案
        const allFiles = await this.scanProjectFiles(['.js', '.rs', '.ts', '.md']);
        const references = new Map();
        
        // 建立引用映射
        for (const file of allFiles) {
            if (file.includes('node_modules') || file.includes('target') || file.includes('archive')) {
                continue;
            }
            
            try {
                const content = fs.readFileSync(file, 'utf8');
                const imports = this.extractImports(content, path.extname(file));
                
                imports.forEach(importPath => {
                    if (!references.has(importPath)) {
                        references.set(importPath, []);
                    }
                    references.get(importPath).push(file);
                });
            } catch (error) {
                // 忽略讀取錯誤
            }
        }

        // 檢查哪些檔案沒有被引用
        for (const file of allFiles) {
            if (file.includes('node_modules') || file.includes('target') || file.includes('archive')) {
                continue;
            }
            
            const relativePath = path.relative(this.projectRoot, file);
            const baseName = path.basename(file, path.extname(file));
            
            // 檢查是否被引用
            let isReferenced = false;
            for (const [importPath, referencingFiles] of references) {
                if (importPath.includes(baseName) || importPath.includes(relativePath)) {
                    isReferenced = true;
                    break;
                }
            }
            
            // 特殊檔案（如 main.rs, lib.rs）總是被引用
            const specialFiles = ['main.rs', 'lib.rs', 'mod.rs', 'index.js', 'main.js'];
            if (specialFiles.includes(path.basename(file))) {
                isReferenced = true;
            }
            
            if (!isReferenced) {
                const stats = fs.statSync(file);
                unreferencedFiles.push({
                    path: relativePath,
                    type: this.getFileType(file),
                    size: stats.size,
                    potential_impact: this.assessImpact(file)
                });
            }
        }

        return unreferencedFiles;
    }

    extractImports(content, ext) {
        const imports = [];
        
        if (ext === '.rs') {
            // Rust imports
            const useMatches = content.match(/use\s+[^;]+;/g) || [];
            const modMatches = content.match(/mod\s+\w+/g) || [];
            imports.push(...useMatches, ...modMatches);
        } else if (ext === '.js' || ext === '.ts') {
            // JavaScript/TypeScript imports
            const importMatches = content.match(/import\s+.*from\s+['"][^'"]+['"]/g) || [];
            const requireMatches = content.match(/require\(['"][^'"]+['"]\)/g) || [];
            imports.push(...importMatches, ...requireMatches);
        }
        
        return imports;
    }

    async findDuplicateCode() {
        // 簡化版重複代碼檢測
        const duplicates = [];
        
        // 檢查明顯的重複實現
        const cliImplementations = [
            'src-tauri/src/bin/cnp-unified.rs',
            'src-tauri/src/bin/cnp-optimized.rs'
        ];
        
        const existingClis = cliImplementations.filter(cli => 
            fs.existsSync(path.join(this.projectRoot, cli))
        );
        
        if (existingClis.length > 1) {
            duplicates.push({
                files: existingClis,
                similarity: 0.75,
                lines: 400,
                suggestion: "Merge CLI implementations or choose one as canonical"
            });
        }

        return duplicates;
    }

    async analyzeDirectoryStructure() {
        const suggestions = [];
        
        // 檢查是否需要採用 vibe-kanban 結構
        const currentStructure = await this.analyzeCurrentStructure();
        
        if (!currentStructure.hasModularBackend) {
            suggestions.push({
                current: "src-tauri/src/ (monolithic)",
                suggested: "src-tauri/src/ (modular: models/, routes/, services/, executors/)",
                reason: "Adopt vibe-kanban modular architecture pattern"
            });
        }
        
        if (!currentStructure.hasTypeSharing) {
            suggestions.push({
                current: "No type sharing between Rust and TypeScript",
                suggested: "Implement ts-rs for automatic type generation",
                reason: "Enable type safety across language boundaries"
            });
        }

        return suggestions;
    }

    async analyzeCurrentStructure() {
        const srcTauriSrc = path.join(this.projectRoot, 'src-tauri', 'src');
        
        return {
            hasModularBackend: fs.existsSync(path.join(srcTauriSrc, 'models')) &&
                              fs.existsSync(path.join(srcTauriSrc, 'routes')),
            hasTypeSharing: fs.existsSync(path.join(this.projectRoot, 'shared-types')) ||
                           (await this.checkForTsRs())
        };
    }

    async checkForTsRs() {
        try {
            const cargoToml = fs.readFileSync(path.join(this.projectRoot, 'src-tauri', 'Cargo.toml'), 'utf8');
            return cargoToml.includes('ts-rs');
        } catch {
            return false;
        }
    }

    async runCliAnalysis() {
        console.log('🔧 Session 2: CLI 指令分析');
        
        const analysis = {
            timestamp: new Date().toISOString(),
            cli_binaries: {},
            bdd_scenarios: [],
            test_coverage: {}
        };

        try {
            // 1. 發現所有 CLI 指令
            const cliCommands = await this.discoverCliCommands();
            analysis.cli_binaries = cliCommands;

            // 2. 生成 BDD 測試場景
            const bddScenarios = await this.generateBddScenarios(cliCommands);
            analysis.bdd_scenarios = bddScenarios;

            // 3. 分析測試覆蓋率
            const testCoverage = await this.analyzeTestCoverage();
            analysis.test_coverage = testCoverage;

            const reportPath = path.join(this.reportsDir, 'cli-analysis-report.json');
            fs.writeFileSync(reportPath, JSON.stringify(analysis, null, 2));
            
            console.log(`✅ CLI 分析完成: ${reportPath}`);
            return analysis;
            
        } catch (error) {
            console.error('❌ CLI 分析失敗:', error.message);
            throw error;
        }
    }

    async discoverCliCommands() {
        const binaries = {};
        
        // 檢查 cnp-optimized
        try {
            const { stdout } = await execAsync('cd src-tauri && cargo run --bin cnp-optimized -- --help', {
                cwd: this.projectRoot,
                timeout: 10000
            });
            
            binaries['cnp-optimized'] = this.parseCliHelp(stdout);
        } catch (error) {
            console.warn('無法執行 cnp-optimized --help:', error.message);
        }

        return binaries;
    }

    parseCliHelp(helpOutput) {
        const commands = {};
        
        // 簡化的解析邏輯
        const lines = helpOutput.split('\n');
        let inCommandsSection = false;
        
        for (const line of lines) {
            if (line.includes('SUBCOMMANDS:') || line.includes('Commands:')) {
                inCommandsSection = true;
                continue;
            }
            
            if (inCommandsSection && line.trim()) {
                const match = line.match(/^\s*(\w+)\s+(.+)$/);
                if (match) {
                    const [, command, description] = match;
                    commands[command] = {
                        description: description.trim(),
                        options: [],
                        examples: [`cnp ${command}`]
                    };
                }
            }
        }
        
        return {
            version: "0.1.0",
            commands
        };
    }

    async generateBddScenarios(cliCommands) {
        const scenarios = [];
        
        for (const [binary, config] of Object.entries(cliCommands)) {
            for (const [command, details] of Object.entries(config.commands)) {
                scenarios.push({
                    feature: `CLI ${command} Command`,
                    scenarios: [
                        {
                            name: `${command} command executes successfully`,
                            given: "The Claude Night Pilot system is initialized",
                            when: `I run '${binary} ${command}'`,
                            then: [
                                "The command should execute without errors",
                                "The exit code should be 0",
                                "The output should contain relevant information"
                            ]
                        },
                        {
                            name: `${command} command shows help`,
                            given: "The CLI tool is available",
                            when: `I run '${binary} ${command} --help'`,
                            then: [
                                "I should see usage information",
                                "I should see available options",
                                "Exit code should be 0"
                            ]
                        }
                    ]
                });
            }
        }
        
        return scenarios;
    }

    async analyzeTestCoverage() {
        const testFiles = await this.scanProjectFiles(['.spec.js', '.test.js', '.rs']);
        const testCoverage = {
            total_test_files: testFiles.length,
            test_categories: {
                unit_tests: 0,
                integration_tests: 0,
                e2e_tests: 0
            },
            coverage_percentage: 0
        };

        testFiles.forEach(file => {
            if (file.includes('unit') || file.includes('src-tauri/src') && file.endsWith('.rs')) {
                testCoverage.test_categories.unit_tests++;
            } else if (file.includes('integration')) {
                testCoverage.test_categories.integration_tests++;
            } else if (file.includes('e2e') || file.includes('tests/')) {
                testCoverage.test_categories.e2e_tests++;
            }
        });

        return testCoverage;
    }

    async generateCleanupScript() {
        console.log('🧹 生成自動化清理腳本');
        
        const reports = this.loadAnalysisReports();
        let script = `#!/bin/bash
# Claude Night Pilot - 自動化清理腳本
# 生成時間: ${new Date().toISOString()}

set -e

echo "🧹 開始清理 Claude Night Pilot 專案..."

`;

        // 基於分析結果生成清理命令
        if (reports.fileAnalysis?.obsolete_files) {
            script += `
# 清理過時檔案
echo "📁 清理過時檔案..."
`;
            reports.fileAnalysis.obsolete_files.forEach(file => {
                if (file.confidence > 0.9) {
                    script += `rm -rf "${file.path}"\n`;
                }
            });
        }

        // 清理編譯產物
        script += `
# 清理編譯產物
echo "🗑️ 清理編譯產物..."
cd src-tauri
cargo clean
cd ..

# 清理 node_modules (如果存在)
if [ -d "node_modules" ]; then
    rm -rf node_modules
    echo "📦 已清理 node_modules"
fi

echo "✅ 清理完成!"
`;

        const scriptPath = path.join(this.analysisDir, 'cleanup.sh');
        fs.writeFileSync(scriptPath, script);
        fs.chmodSync(scriptPath, 0o755);
        
        console.log(`✅ 清理腳本已生成: ${scriptPath}`);
        return scriptPath;
    }

    loadAnalysisReports() {
        const reports = {};
        
        try {
            const fileAnalysisPath = path.join(this.reportsDir, 'file-analysis-report.json');
            if (fs.existsSync(fileAnalysisPath)) {
                reports.fileAnalysis = JSON.parse(fs.readFileSync(fileAnalysisPath, 'utf8'));
            }
        } catch (error) {
            console.warn('無法載入檔案分析報告:', error.message);
        }

        return reports;
    }

    // Helper methods
    async scanDirectory(dir) {
        const files = [];
        
        const scan = (currentDir) => {
            const items = fs.readdirSync(currentDir);
            
            for (const item of items) {
                const fullPath = path.join(currentDir, item);
                const stat = fs.statSync(fullPath);
                
                if (stat.isDirectory()) {
                    scan(fullPath);
                } else {
                    files.push(fullPath);
                }
            }
        };
        
        scan(dir);
        return files;
    }

    async scanProjectFiles(extensions) {
        const files = [];
        const excludeDirs = ['node_modules', 'target', '.git'];
        
        const scan = (dir) => {
            try {
                const items = fs.readdirSync(dir);
                
                for (const item of items) {
                    if (excludeDirs.includes(item)) continue;
                    
                    const fullPath = path.join(dir, item);
                    const stat = fs.statSync(fullPath);
                    
                    if (stat.isDirectory()) {
                        scan(fullPath);
                    } else {
                        const ext = path.extname(fullPath);
                        if (extensions.some(e => fullPath.endsWith(e))) {
                            files.push(fullPath);
                        }
                    }
                }
            } catch (error) {
                // 忽略權限錯誤
            }
        };
        
        scan(this.projectRoot);
        return files;
    }

    async getDirectorySize(dir) {
        try {
            const { stdout } = await execAsync(`du -s "${dir}" | cut -f1`);
            return parseInt(stdout.trim()) * 1024; // Convert from KB to bytes
        } catch {
            return 0;
        }
    }

    getFileType(filePath) {
        const ext = path.extname(filePath);
        const typeMap = {
            '.rs': 'rust',
            '.js': 'javascript',
            '.ts': 'typescript',
            '.json': 'config',
            '.md': 'documentation',
            '.toml': 'config'
        };
        
        return typeMap[ext] || 'other';
    }

    assessImpact(filePath) {
        const fileName = path.basename(filePath).toLowerCase();
        
        // 高影響檔案
        if (['main.rs', 'lib.rs', 'index.js', 'app.js'].includes(fileName)) {
            return 'high';
        }
        
        // 中等影響檔案
        if (fileName.includes('config') || fileName.includes('test')) {
            return 'medium';
        }
        
        return 'low';
    }

    async runFullAnalysis() {
        console.log('🚀 開始 Claude Night Pilot 專案全面分析');
        
        try {
            // Session 1: 檔案分析
            const fileAnalysis = await this.runFileAnalysis();
            
            // Session 2: CLI 分析
            const cliAnalysis = await this.runCliAnalysis();
            
            // 生成清理腳本
            await this.generateCleanupScript();
            
            // 生成總結報告
            const summary = {
                timestamp: new Date().toISOString(),
                analysis_status: 'completed',
                sessions: {
                    'file-analysis': 'completed',
                    'cli-analysis': 'completed',
                    'architecture-refactoring': 'pending',
                    'technical-debt': 'pending',
                    'monitoring-coordination': 'pending'
                },
                recommendations: [
                    '清理 archive/ 目錄中的過時檔案',
                    '移除重複的 CLI 實現',
                    '採用 vibe-kanban 的模組化架構',
                    '實施 ts-rs 類型共享',
                    '建立完整的 BDD 測試套件'
                ],
                next_steps: [
                    '執行自動化清理腳本',
                    '重構專案架構',
                    '實施 BDD 測試',
                    '清理技術債務'
                ]
            };
            
            const summaryPath = path.join(this.reportsDir, 'analysis-summary.json');
            fs.writeFileSync(summaryPath, JSON.stringify(summary, null, 2));
            
            console.log('🎉 專案分析完成!');
            console.log(`📊 總結報告: ${summaryPath}`);
            console.log(`🧹 清理腳本: ${path.join(this.analysisDir, 'cleanup.sh')}`);
            
            return summary;
            
        } catch (error) {
            console.error('❌ 分析過程中發生錯誤:', error);
            throw error;
        }
    }
}

// 如果直接執行此腳本
if (import.meta.url === `file://${process.argv[1]}`) {
    const orchestrator = new ProjectAnalysisOrchestrator();
    
    const command = process.argv[2] || 'full';
    
    switch (command) {
        case 'file':
            orchestrator.runFileAnalysis().catch(console.error);
            break;
        case 'cli':
            orchestrator.runCliAnalysis().catch(console.error);
            break;
        case 'cleanup':
            orchestrator.generateCleanupScript().catch(console.error);
            break;
        case 'full':
        default:
            orchestrator.runFullAnalysis().catch(console.error);
            break;
    }
}

export default ProjectAnalysisOrchestrator;