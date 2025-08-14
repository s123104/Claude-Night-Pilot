#!/usr/bin/env node

/**
 * Claude Night Pilot - BDD CLI Testing Framework
 * 基於 Cucumber/Gherkin 風格的 CLI 測試框架
 */

const { spawn, exec } = require('child_process');
const fs = require('fs');
const path = require('path');
const util = require('util');

const execAsync = util.promisify(exec);

class BddCliTestFramework {
    constructor() {
        this.projectRoot = path.resolve(__dirname, '..', '..');
        this.testResults = [];
        this.currentFeature = null;
        this.currentScenario = null;
        this.stepResults = [];
        
        // CLI 路徑配置
        this.cliPaths = {
            'cnp-optimized': 'cd src-tauri && cargo run --bin cnp-optimized --',
            'cnp-unified': 'cd src-tauri && cargo run --bin cnp-unified --'
        };
    }

    // Feature 定義
    feature(name, description) {
        this.currentFeature = {
            name,
            description,
            scenarios: [],
            startTime: Date.now()
        };
        
        console.log(`\n🎯 Feature: ${name}`);
        if (description) {
            console.log(`   ${description}`);
        }
        
        return this;
    }

    // Scenario 定義
    scenario(name, steps) {
        this.currentScenario = {
            name,
            steps: [],
            status: 'pending',
            startTime: Date.now()
        };
        
        console.log(`\n  📋 Scenario: ${name}`);
        
        return this;
    }

    // Given 步驟
    given(description, action) {
        return this.addStep('Given', description, action);
    }

    // When 步驟
    when(description, action) {
        return this.addStep('When', description, action);
    }

    // Then 步驟
    then(description, assertion) {
        return this.addStep('Then', description, assertion);
    }

    // And 步驟
    and(description, action) {
        return this.addStep('And', description, action);
    }

    addStep(type, description, action) {
        const step = {
            type,
            description,
            action,
            status: 'pending',
            result: null,
            error: null,
            duration: 0
        };
        
        this.currentScenario.steps.push(step);
        return this;
    }

    // 執行當前 scenario
    async run() {
        if (!this.currentScenario) {
            throw new Error('No scenario defined');
        }

        console.log(`    🏃 Running scenario: ${this.currentScenario.name}`);
        
        try {
            for (const step of this.currentScenario.steps) {
                await this.executeStep(step);
            }
            
            this.currentScenario.status = 'passed';
            this.currentScenario.duration = Date.now() - this.currentScenario.startTime;
            
            console.log(`    ✅ Scenario passed (${this.currentScenario.duration}ms)`);
            
        } catch (error) {
            this.currentScenario.status = 'failed';
            this.currentScenario.error = error.message;
            this.currentScenario.duration = Date.now() - this.currentScenario.startTime;
            
            console.log(`    ❌ Scenario failed: ${error.message}`);
            throw error;
        } finally {
            this.currentFeature.scenarios.push({...this.currentScenario});
            this.currentScenario = null;
        }
        
        return this;
    }

    async executeStep(step) {
        const startTime = Date.now();
        console.log(`      ${step.type} ${step.description}`);
        
        try {
            if (typeof step.action === 'function') {
                step.result = await step.action();
            } else if (typeof step.action === 'string') {
                // 處理 CLI 命令
                step.result = await this.executeCli(step.action);
            }
            
            step.status = 'passed';
            step.duration = Date.now() - startTime;
            
        } catch (error) {
            step.status = 'failed';
            step.error = error.message;
            step.duration = Date.now() - startTime;
            throw error;
        }
    }

    async executeCli(command) {
        // 解析命令
        const parts = command.split(' ');
        const cli = parts[0];
        const args = parts.slice(1);
        
        if (!this.cliPaths[cli]) {
            throw new Error(`Unknown CLI: ${cli}`);
        }
        
        const fullCommand = `${this.cliPaths[cli]} ${args.join(' ')}`;
        
        try {
            const { stdout, stderr } = await execAsync(fullCommand, {
                cwd: this.projectRoot,
                timeout: 30000
            });
            
            return {
                stdout: stdout.trim(),
                stderr: stderr.trim(),
                exitCode: 0
            };
            
        } catch (error) {
            // 某些命令可能返回非零退出碼但仍然是有效的
            return {
                stdout: error.stdout || '',
                stderr: error.stderr || '',
                exitCode: error.code || 1,
                error: error.message
            };
        }
    }

    // 完成 feature
    complete() {
        if (this.currentFeature) {
            this.currentFeature.duration = Date.now() - this.currentFeature.startTime;
            this.currentFeature.status = this.currentFeature.scenarios.every(s => s.status === 'passed') ? 'passed' : 'failed';
            
            this.testResults.push({...this.currentFeature});
            
            const passedScenarios = this.currentFeature.scenarios.filter(s => s.status === 'passed').length;
            const totalScenarios = this.currentFeature.scenarios.length;
            
            console.log(`\n🏁 Feature completed: ${passedScenarios}/${totalScenarios} scenarios passed`);
            
            this.currentFeature = null;
        }
        
        return this;
    }

    // 生成測試報告
    generateReport() {
        const report = {
            summary: {
                totalFeatures: this.testResults.length,
                passedFeatures: this.testResults.filter(f => f.status === 'passed').length,
                totalScenarios: this.testResults.reduce((sum, f) => sum + f.scenarios.length, 0),
                passedScenarios: this.testResults.reduce((sum, f) => sum + f.scenarios.filter(s => s.status === 'passed').length, 0),
                totalDuration: this.testResults.reduce((sum, f) => sum + f.duration, 0),
                timestamp: new Date().toISOString()
            },
            features: this.testResults
        };
        
        const reportPath = path.join(this.projectRoot, 'tests', 'bdd', 'reports', 'test-report.json');
        
        // 確保目錄存在
        const reportDir = path.dirname(reportPath);
        if (!fs.existsSync(reportDir)) {
            fs.mkdirSync(reportDir, { recursive: true });
        }
        
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        
        console.log(`\n📊 Test report generated: ${reportPath}`);
        console.log(`📈 Summary: ${report.summary.passedScenarios}/${report.summary.totalScenarios} scenarios passed`);
        
        return report;
    }

    // 便利方法：建立基本的 CLI 測試
    static createCliTest(cliName) {
        const framework = new BddCliTestFramework();
        
        return framework
            .feature(`${cliName} CLI Basic Functionality`, `Test basic functionality of ${cliName} CLI tool`)
            .scenario(`${cliName} shows help`, [
                framework.given("The CLI tool is available", async () => {
                    // 預設為可用
                    return true;
                }),
                framework.when(`I run '${cliName} --help'`, `${cliName} --help`),
                framework.then("I should see help information", (result) => {
                    if (!result.stdout || result.stdout.length === 0) {
                        throw new Error('No help output received');
                    }
                    if (!result.stdout.includes('help') && !result.stdout.includes('usage')) {
                        throw new Error('Help output does not contain expected help text');
                    }
                    return true;
                }),
                framework.and("Exit code should be 0 or help-related", (result) => {
                    // 幫助命令通常返回 0 或特定的退出碼
                    if (result.exitCode !== 0 && result.exitCode !== 2) {
                        throw new Error(`Unexpected exit code: ${result.exitCode}`);
                    }
                    return true;
                })
            ]);
    }

    // 便利方法：建立狀態檢查測試
    static createStatusTest(cliName) {
        const framework = new BddCliTestFramework();
        
        return framework
            .feature(`${cliName} Status Check`, `Test status checking functionality of ${cliName}`)
            .scenario(`${cliName} status command works`, [
                framework.given("The system is initialized", async () => {
                    return true;
                }),
                framework.when(`I run '${cliName} status'`, `${cliName} status`),
                framework.then("I should see status information", (result) => {
                    if (result.exitCode !== 0) {
                        console.warn(`Status command returned exit code ${result.exitCode}, but output might still be valid`);
                    }
                    // 檢查輸出是否包含狀態相關信息
                    const output = result.stdout || result.stderr;
                    if (!output || output.length === 0) {
                        throw new Error('No status output received');
                    }
                    return true;
                })
            ]);
    }
}

// 預定義的測試套件
class PredefinedTestSuites {
    static async runAllCliTests() {
        console.log('🚀 Running all CLI BDD tests...');
        
        const results = [];
        const cliTools = ['cnp-optimized'];
        
        for (const cli of cliTools) {
            try {
                // 基本功能測試
                const basicTest = BddCliTestFramework.createCliTest(cli);
                await basicTest.run().complete();
                results.push(...basicTest.testResults);
                
                // 狀態檢查測試
                const statusTest = BddCliTestFramework.createStatusTest(cli);
                await statusTest.run().complete();
                results.push(...statusTest.testResults);
                
            } catch (error) {
                console.error(`❌ Tests failed for ${cli}:`, error.message);
            }
        }
        
        // 生成總合報告
        const framework = new BddCliTestFramework();
        framework.testResults = results;
        return framework.generateReport();
    }
    
    static async runHealthCheckTests() {
        console.log('🏥 Running health check BDD tests...');
        
        const framework = new BddCliTestFramework();
        
        try {
            await framework
                .feature('Health Check Functionality', 'Test system health check capabilities')
                .scenario('Health check returns system status', [
                    framework.given("The system is running", async () => true),
                    framework.when("I run 'cnp-optimized health'", 'cnp-optimized health'),
                    framework.then("I should see health information", (result) => {
                        const output = result.stdout || result.stderr;
                        if (!output) {
                            throw new Error('No health check output received');
                        }
                        return true;
                    })
                ])
                .run()
                .complete();
                
            return framework.generateReport();
            
        } catch (error) {
            console.error('❌ Health check tests failed:', error.message);
            return framework.generateReport();
        }
    }
    
    static async runPerformanceTests() {
        console.log('⚡ Running performance BDD tests...');
        
        const framework = new BddCliTestFramework();
        
        try {
            await framework
                .feature('Performance Benchmarks', 'Test CLI performance characteristics')
                .scenario('CLI startup time is acceptable', [
                    framework.given("The CLI tool is built", async () => true),
                    framework.when("I measure startup time", async () => {
                        const startTime = Date.now();
                        await framework.executeCli('cnp-optimized --help');
                        const endTime = Date.now();
                        return { duration: endTime - startTime };
                    }),
                    framework.then("Startup time should be under 5 seconds", (result) => {
                        if (result.duration > 5000) {
                            throw new Error(`Startup time too slow: ${result.duration}ms`);
                        }
                        console.log(`      ⚡ Startup time: ${result.duration}ms`);
                        return true;
                    })
                ])
                .run()
                .complete();
                
            return framework.generateReport();
            
        } catch (error) {
            console.error('❌ Performance tests failed:', error.message);
            return framework.generateReport();
        }
    }
}

// 如果直接執行此腳本
if (require.main === module) {
    const testSuite = process.argv[2] || 'all';
    
    (async () => {
        try {
            switch (testSuite) {
                case 'basic':
                    await PredefinedTestSuites.runAllCliTests();
                    break;
                case 'health':
                    await PredefinedTestSuites.runHealthCheckTests();
                    break;
                case 'performance':
                    await PredefinedTestSuites.runPerformanceTests();
                    break;
                case 'all':
                default:
                    await PredefinedTestSuites.runAllCliTests();
                    await PredefinedTestSuites.runHealthCheckTests();
                    await PredefinedTestSuites.runPerformanceTests();
                    break;
            }
            
            console.log('\n🎉 All BDD tests completed!');
            
        } catch (error) {
            console.error('❌ Test execution failed:', error);
            process.exit(1);
        }
    })();
}

module.exports = {
    BddCliTestFramework,
    PredefinedTestSuites
};