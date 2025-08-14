#!/usr/bin/env node

/**
 * Claude Night Pilot - 並行任務執行器
 * 管理多個 Claude Code sessions 的並行執行和監控
 */

const fs = require('fs');
const path = require('path');
const { spawn, exec } = require('child_process');
const util = require('util');

const execAsync = util.promisify(exec);

class ParallelTaskExecutor {
    constructor() {
        this.projectRoot = path.resolve(__dirname, '..');
        this.sessionsDir = path.join(__dirname, 'sessions');
        this.logsDir = path.join(__dirname, 'logs');
        this.tempDir = path.join(__dirname, 'temp');
        
        this.activeSessions = new Map();
        this.sessionResults = new Map();
        this.sessionLogs = new Map();
        
        this.initializeDirectories();
        
        // Claude Code 執行配置
        this.claudeConfig = {
            executable: 'npx',
            args: ['@anthropic-ai/claude-code@latest'],
            baseOptions: ['-p', '--output-format=stream-json'],
            maxConcurrentSessions: 5,
            sessionTimeout: 300000 // 5 minutes
        };
    }

    initializeDirectories() {
        [this.logsDir, this.tempDir].forEach(dir => {
            if (!fs.existsSync(dir)) {
                fs.mkdirSync(dir, { recursive: true });
            }
        });
    }

    /**
     * 創建並執行並行任務
     */
    async executeParallelTasks() {
        console.log('🚀 啟動並行任務執行器');
        console.log(`📁 專案根目錄: ${this.projectRoot}`);
        
        const tasks = [
            {
                id: 'session-1-file-analysis',
                name: '檔案分析與清理',
                prompt: this.generateFileAnalysisPrompt(),
                priority: 1,
                estimatedTime: 120000 // 2 minutes
            },
            {
                id: 'session-2-cli-analysis',
                name: 'CLI 指令分析',
                prompt: this.generateCliAnalysisPrompt(),
                priority: 2,
                estimatedTime: 180000 // 3 minutes
            },
            {
                id: 'session-3-architecture-refactoring',
                name: '架構重構分析',
                prompt: this.generateArchitectureAnalysisPrompt(),
                priority: 3,
                estimatedTime: 240000 // 4 minutes
            },
            {
                id: 'session-4-technical-debt',
                name: '技術債務清理',
                prompt: this.generateTechnicalDebtPrompt(),
                priority: 4,
                estimatedTime: 180000 // 3 minutes
            },
            {
                id: 'session-5-monitoring-coordination',
                name: '監控與協調',
                prompt: this.generateMonitoringPrompt(),
                priority: 5,
                estimatedTime: 120000 // 2 minutes
            }
        ];

        try {
            // 啟動監控器
            this.startMonitor();
            
            // 分批執行任務（避免超載）
            const batchSize = Math.min(this.claudeConfig.maxConcurrentSessions, tasks.length);
            const batches = this.chunkArray(tasks, batchSize);
            
            for (let i = 0; i < batches.length; i++) {
                console.log(`📦 執行批次 ${i + 1}/${batches.length}`);
                await this.executeBatch(batches[i]);
                
                if (i < batches.length - 1) {
                    console.log('⏱️ 批次間等待 10 秒...');
                    await this.sleep(10000);
                }
            }
            
            // 等待所有任務完成
            await this.waitForAllTasksCompletion();
            
            // 生成整合報告
            const report = await this.generateIntegratedReport();
            
            console.log('🎉 所有並行任務執行完成！');
            return report;
            
        } catch (error) {
            console.error('❌ 並行任務執行失敗:', error.message);
            await this.cleanupSessions();
            throw error;
        }
    }

    async executeBatch(tasks) {
        const promises = tasks.map(task => this.executeTask(task));
        
        try {
            await Promise.allSettled(promises);
        } catch (error) {
            console.error(`❌ 批次執行錯誤: ${error.message}`);
        }
    }

    async executeTask(task) {
        console.log(`🏃 啟動任務: ${task.name} (${task.id})`);
        
        const sessionInfo = {
            id: task.id,
            name: task.name,
            status: 'starting',
            startTime: Date.now(),
            process: null,
            output: [],
            error: null
        };
        
        this.activeSessions.set(task.id, sessionInfo);
        
        try {
            // 創建臨時 prompt 檔案
            const promptFile = path.join(this.tempDir, `${task.id}-prompt.md`);
            fs.writeFileSync(promptFile, task.prompt);
            
            // 啟動 Claude Code session
            const process = await this.startClaudeCodeSession(task.id, promptFile);
            sessionInfo.process = process;
            sessionInfo.status = 'running';
            
            // 設置超時
            const timeout = setTimeout(() => {
                console.warn(`⏰ 任務 ${task.id} 超時，正在終止...`);
                this.terminateSession(task.id);
            }, task.estimatedTime + 60000); // 額外 1 分鐘緩衝
            
            // 等待完成
            await new Promise((resolve, reject) => {
                process.on('close', (code) => {
                    clearTimeout(timeout);
                    sessionInfo.status = code === 0 ? 'completed' : 'failed';
                    sessionInfo.endTime = Date.now();
                    sessionInfo.duration = sessionInfo.endTime - sessionInfo.startTime;
                    
                    console.log(`${code === 0 ? '✅' : '❌'} 任務 ${task.id} ${sessionInfo.status} (${sessionInfo.duration}ms)`);
                    resolve();
                });
                
                process.on('error', (error) => {
                    clearTimeout(timeout);
                    sessionInfo.status = 'error';
                    sessionInfo.error = error.message;
                    sessionInfo.endTime = Date.now();
                    sessionInfo.duration = sessionInfo.endTime - sessionInfo.startTime;
                    
                    console.error(`❌ 任務 ${task.id} 錯誤: ${error.message}`);
                    reject(error);
                });
            });
            
        } catch (error) {
            sessionInfo.status = 'error';
            sessionInfo.error = error.message;
            sessionInfo.endTime = Date.now();
            sessionInfo.duration = sessionInfo.endTime - sessionInfo.startTime;
            
            console.error(`❌ 任務 ${task.id} 執行錯誤: ${error.message}`);
        } finally {
            // 保存結果
            this.sessionResults.set(task.id, sessionInfo);
        }
    }

    async startClaudeCodeSession(sessionId, promptFile) {
        const logFile = path.join(this.logsDir, `${sessionId}.log`);
        
        // 構建 Claude Code 命令
        const args = [
            ...this.claudeConfig.args,
            ...this.claudeConfig.baseOptions,
            '--file', promptFile,
            '--working-directory', this.projectRoot
        ];
        
        const process = spawn(this.claudeConfig.executable, args, {
            cwd: this.projectRoot,
            stdio: ['pipe', 'pipe', 'pipe']
        });
        
        // 設置輸出處理
        const logStream = fs.createWriteStream(logFile);
        
        process.stdout.on('data', (data) => {
            const output = data.toString();
            logStream.write(`[STDOUT] ${output}`);
            
            const sessionInfo = this.activeSessions.get(sessionId);
            if (sessionInfo) {
                sessionInfo.output.push({
                    type: 'stdout',
                    timestamp: Date.now(),
                    content: output
                });
            }
        });
        
        process.stderr.on('data', (data) => {
            const output = data.toString();
            logStream.write(`[STDERR] ${output}`);
            
            const sessionInfo = this.activeSessions.get(sessionId);
            if (sessionInfo) {
                sessionInfo.output.push({
                    type: 'stderr',
                    timestamp: Date.now(),
                    content: output
                });
            }
        });
        
        process.on('close', () => {
            logStream.end();
        });
        
        return process;
    }

    startMonitor() {
        console.log('📊 啟動任務監控器');
        
        this.monitorInterval = setInterval(() => {
            this.printSessionStatus();
        }, 30000); // 每 30 秒監控一次
    }

    printSessionStatus() {
        console.log('\n📈 任務狀態報告:');
        console.log('═'.repeat(60));
        
        this.activeSessions.forEach((session, id) => {
            const duration = Date.now() - session.startTime;
            const status = this.getStatusIcon(session.status);
            
            console.log(`${status} ${session.name}`);
            console.log(`   ID: ${id}`);
            console.log(`   狀態: ${session.status}`);
            console.log(`   運行時間: ${Math.round(duration / 1000)}s`);
            
            if (session.error) {
                console.log(`   錯誤: ${session.error}`);
            }
        });
        
        console.log('═'.repeat(60));
    }

    getStatusIcon(status) {
        const icons = {
            'starting': '🚀',
            'running': '🔄',
            'completed': '✅',
            'failed': '❌',
            'error': '💥'
        };
        
        return icons[status] || '❓';
    }

    async waitForAllTasksCompletion() {
        console.log('⏳ 等待所有任務完成...');
        
        while (this.activeSessions.size > 0) {
            const incompleteSessions = Array.from(this.activeSessions.values())
                .filter(session => !['completed', 'failed', 'error'].includes(session.status));
            
            if (incompleteSessions.length === 0) {
                break;
            }
            
            await this.sleep(5000); // 等待 5 秒後再檢查
        }
        
        // 停止監控器
        if (this.monitorInterval) {
            clearInterval(this.monitorInterval);
        }
        
        console.log('🏁 所有任務已完成');
    }

    terminateSession(sessionId) {
        const sessionInfo = this.activeSessions.get(sessionId);
        
        if (sessionInfo && sessionInfo.process) {
            sessionInfo.process.kill('SIGTERM');
            sessionInfo.status = 'terminated';
            
            setTimeout(() => {
                if (sessionInfo.process && !sessionInfo.process.killed) {
                    sessionInfo.process.kill('SIGKILL');
                }
            }, 5000); // 5 秒後強制終止
        }
    }

    async cleanupSessions() {
        console.log('🧹 清理會話...');
        
        this.activeSessions.forEach((session, id) => {
            if (session.process && !session.process.killed) {
                session.process.kill('SIGTERM');
            }
        });
        
        // 清理臨時檔案
        try {
            const tempFiles = fs.readdirSync(this.tempDir);
            tempFiles.forEach(file => {
                if (file.endsWith('-prompt.md')) {
                    fs.unlinkSync(path.join(this.tempDir, file));
                }
            });
        } catch (error) {
            console.warn('清理臨時檔案時發生錯誤:', error.message);
        }
    }

    async generateIntegratedReport() {
        console.log('📊 生成整合報告...');
        
        const report = {
            summary: {
                totalTasks: this.sessionResults.size,
                completedTasks: 0,
                failedTasks: 0,
                errorTasks: 0,
                totalDuration: 0,
                timestamp: new Date().toISOString()
            },
            tasks: [],
            recommendations: [],
            nextSteps: []
        };
        
        // 分析結果
        this.sessionResults.forEach((session, id) => {
            const taskReport = {
                id: session.id,
                name: session.name,
                status: session.status,
                duration: session.duration || 0,
                outputSize: session.output.length,
                hasError: !!session.error
            };
            
            report.tasks.push(taskReport);
            
            switch (session.status) {
                case 'completed':
                    report.summary.completedTasks++;
                    break;
                case 'failed':
                    report.summary.failedTasks++;
                    break;
                case 'error':
                    report.summary.errorTasks++;
                    break;
            }
            
            report.summary.totalDuration += taskReport.duration;
        });
        
        // 生成建議
        if (report.summary.completedTasks > 0) {
            report.recommendations.push('基於完成的分析任務，建議開始實施清理和重構');
        }
        
        if (report.summary.failedTasks > 0) {
            report.recommendations.push('檢查失敗的任務日誌，解決相關問題');
        }
        
        // 保存報告
        const reportPath = path.join(__dirname, 'reports', 'parallel-execution-report.json');
        const reportDir = path.dirname(reportPath);
        
        if (!fs.existsSync(reportDir)) {
            fs.mkdirSync(reportDir, { recursive: true });
        }
        
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        
        console.log(`📋 整合報告已生成: ${reportPath}`);
        console.log(`📈 成功率: ${Math.round((report.summary.completedTasks / report.summary.totalTasks) * 100)}%`);
        
        return report;
    }

    // Prompt 生成方法
    generateFileAnalysisPrompt() {
        return `# 檔案分析與清理任務

請分析 Claude Night Pilot 專案並完成以下任務：

## 目標
識別並清理過時檔案、無引用檔案和重複代碼，改善專案結構。

## 具體任務
1. 掃描專案目錄，識別以下類型的檔案：
   - archive/ 目錄中的過時檔案
   - src-tauri/target/ 編譯產物
   - 無引用的 .rs, .js, .ts 檔案
   - 重複的配置檔案

2. 分析檔案引用關係：
   - 檢查 Rust 的 use 和 mod 聲明
   - 檢查 JavaScript/TypeScript 的 import/require
   - 識別孤立的檔案

3. 生成清理建議：
   - 可安全刪除的檔案清單
   - 風險評估等級
   - 自動化清理腳本

## 輸出格式
請以 JSON 格式輸出分析結果，包含：
- obsolete_files: 過時檔案列表
- unreferenced_files: 無引用檔案列表
- cleanup_script: 清理腳本內容

專案根目錄: ${this.projectRoot}`;
    }

    generateCliAnalysisPrompt() {
        return `# CLI 指令分析任務

請分析 Claude Night Pilot 的 CLI 工具並建立完整測試規格。

## 目標
建立完整的 CLI 指令目錄和 BDD 測試場景。

## 具體任務
1. 分析 CLI 實現：
   - 檢查 src-tauri/src/bin/ 中的 CLI 檔案
   - 提取所有指令和選項
   - 分析指令功能和用法

2. 建立 BDD 測試場景：
   - 為每個指令設計 Given-When-Then 場景
   - 包含正面和負面測試用例
   - 測試錯誤處理和邊界條件

3. 測試自動化：
   - 設計測試執行框架
   - 建立 CI/CD 整合方案

## 輸出格式
請以 JSON 格式輸出：
- cli_commands: 指令目錄
- bdd_scenarios: BDD 測試場景
- test_automation: 自動化測試建議

專案根目錄: ${this.projectRoot}`;
    }

    generateArchitectureAnalysisPrompt() {
        return `# 架構重構分析任務

基於 vibe-kanban 架構模式，分析並重構 Claude Night Pilot 專案架構。

## 目標
將專案重構為現代化、模組化的架構設計。

## 參考架構
請參考 research-projects/vibe-kanban/ 的架構模式：
- 模組化 Rust 後端結構
- ts-rs 類型共享系統
- 統一的 API 設計模式
- 現代化建置工作流程

## 具體任務
1. 分析當前架構：
   - 評估代碼組織結構
   - 識別架構缺陷
   - 分析可維護性

2. 設計重構方案：
   - 採用模組化結構
   - 實施類型共享
   - 改善 API 設計

3. 實施計劃：
   - 分階段重構策略
   - 風險評估
   - 向後兼容性

## 輸出格式
請以詳細報告形式輸出重構建議和實施計劃。

專案根目錄: ${this.projectRoot}`;
    }

    generateTechnicalDebtPrompt() {
        return `# 技術債務清理任務

識別並清理 Claude Night Pilot 專案中的技術債務。

## 目標
降低技術債務，提升代碼品質和可維護性。

## 具體任務
1. 代碼品質分析：
   - 識別代碼異味
   - 分析複雜度
   - 檢查重複代碼

2. 依賴管理：
   - 檢查過時依賴
   - 分析安全漏洞
   - 優化依賴樹

3. 性能分析：
   - 識別性能瓶頸
   - 分析記憶體使用
   - 優化編譯時間

## 輸出格式
請以結構化報告輸出技術債務清理建議。

專案根目錄: ${this.projectRoot}`;
    }

    generateMonitoringPrompt() {
        return `# 監控與協調任務

協調所有分析任務的結果，生成統一的改進方案。

## 目標
整合所有分析結果，提供統一的專案改進方案。

## 具體任務
1. 結果整合：
   - 收集所有任務結果
   - 識別衝突和重疊
   - 統一改進建議

2. 優先級排序：
   - 評估改進影響
   - 制定實施順序
   - 資源分配建議

3. 監控機制：
   - 設計進度追蹤
   - 建立品質指標
   - 持續改進流程

## 輸出格式
請生成完整的專案改進方案和實施路線圖。

專案根目錄: ${this.projectRoot}`;
    }

    // 工具方法
    chunkArray(array, size) {
        const chunks = [];
        for (let i = 0; i < array.length; i += size) {
            chunks.push(array.slice(i, i + size));
        }
        return chunks;
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// 如果直接執行此腳本
if (require.main === module) {
    const executor = new ParallelTaskExecutor();
    
    executor.executeParallelTasks()
        .then(report => {
            console.log('🎉 並行任務執行完成！');
            process.exit(0);
        })
        .catch(error => {
            console.error('❌ 執行失敗:', error.message);
            process.exit(1);
        });
}

module.exports = ParallelTaskExecutor;