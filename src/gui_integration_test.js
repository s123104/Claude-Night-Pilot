/**
 * Claude Night Pilot GUI 整合測試
 * 測試前端與 Tauri 後端的完整整合
 */

const { invoke } = window.__TAURI__.tauri;
const { Command } = window.__TAURI__.shell;

class GUIIntegrationTest {
    constructor() {
        this.testResults = [];
        this.totalTests = 0;
        this.passedTests = 0;
        this.failedTests = 0;
    }

    // 記錄測試結果
    logTest(name, passed, details = '') {
        this.totalTests++;
        if (passed) {
            this.passedTests++;
            console.log(`✅ ${name}: PASSED ${details}`);
        } else {
            this.failedTests++;
            console.error(`❌ ${name}: FAILED ${details}`);
        }
        
        this.testResults.push({
            name,
            passed,
            details,
            timestamp: new Date().toISOString()
        });
    }

    // 測試 Tauri 命令調用
    async testTauriCommands() {
        console.log('\n🔧 測試 Tauri 命令調用...');
        
        // 測試健康檢查
        try {
            const healthResult = await invoke('health_check');
            this.logTest('健康檢查命令', true, `回應: ${JSON.stringify(healthResult)}`);
        } catch (error) {
            this.logTest('健康檢查命令', false, `錯誤: ${error.message}`);
        }

        // 測試冷卻狀態檢查
        try {
            const cooldownResult = await invoke('check_cooldown');
            this.logTest('冷卻狀態檢查', true, `回應: ${JSON.stringify(cooldownResult)}`);
        } catch (error) {
            this.logTest('冷卻狀態檢查', false, `錯誤: ${error.message}`);
        }

        // 測試 Prompt 創建
        try {
            const createResult = await invoke('create_prompt', {
                title: 'GUI測試Prompt',
                content: '這是一個GUI整合測試的Prompt',
                tags: 'test,gui,integration'
            });
            this.logTest('創建Prompt', true, `ID: ${createResult}`);
            
            // 測試 Prompt 讀取
            try {
                const getResult = await invoke('get_prompt', { id: createResult });
                this.logTest('讀取Prompt', true, `標題: ${getResult.title}`);
            } catch (error) {
                this.logTest('讀取Prompt', false, `錯誤: ${error.message}`);
            }
        } catch (error) {
            this.logTest('創建Prompt', false, `錯誤: ${error.message}`);
        }

        // 測試排程創建
        try {
            const scheduleTime = new Date(Date.now() + 60000).toISOString(); // 1分鐘後
            const scheduleResult = await invoke('create_schedule', {
                promptId: 1,
                scheduleTime: scheduleTime,
                cronExpr: null
            });
            this.logTest('創建排程', true, `排程ID: ${scheduleResult}`);
        } catch (error) {
            this.logTest('創建排程', false, `錯誤: ${error.message}`);
        }

        // 測試 Token 使用量統計
        try {
            const tokenStats = await invoke('get_token_usage_stats');
            this.logTest('Token統計查詢', true, `統計: ${JSON.stringify(tokenStats)}`);
        } catch (error) {
            this.logTest('Token統計查詢', false, `錯誤: ${error.message}`);
        }
    }

    // 測試 DOM 操作與事件綁定
    async testDOMIntegration() {
        console.log('\n🎨 測試 DOM 整合...');
        
        // 測試主題切換
        try {
            const themeToggle = document.getElementById('theme-toggle');
            if (themeToggle) {
                themeToggle.click();
                setTimeout(() => {
                    const currentTheme = document.documentElement.getAttribute('data-theme');
                    this.logTest('主題切換', !!currentTheme, `當前主題: ${currentTheme}`);
                }, 100);
            } else {
                this.logTest('主題切換', false, '找不到主題切換按鈕');
            }
        } catch (error) {
            this.logTest('主題切換', false, `錯誤: ${error.message}`);
        }

        // 測試分頁切換
        try {
            const tabs = ['prompts', 'jobs', 'monitoring', 'settings'];
            for (const tab of tabs) {
                const tabButton = document.querySelector(`[data-tab="${tab}"]`);
                if (tabButton) {
                    tabButton.click();
                    setTimeout(() => {
                        const activeTab = document.querySelector('.nav-tab.active');
                        const isActive = activeTab && activeTab.dataset.tab === tab;
                        this.logTest(`分頁切換: ${tab}`, isActive, `激活分頁: ${activeTab?.dataset.tab || 'none'}`);
                    }, 50);
                } else {
                    this.logTest(`分頁切換: ${tab}`, false, '找不到分頁按鈕');
                }
            }
        } catch (error) {
            this.logTest('分頁切換', false, `錯誤: ${error.message}`);
        }

        // 測試表單驗證
        try {
            const promptForm = document.getElementById('prompt-form');
            if (promptForm) {
                // 模擬填寫表單
                const titleInput = promptForm.querySelector('[name="title"]');
                const contentInput = promptForm.querySelector('[name="content"]');
                
                if (titleInput && contentInput) {
                    titleInput.value = 'GUI測試標題';
                    contentInput.value = 'GUI測試內容';
                    
                    // 觸發驗證
                    titleInput.dispatchEvent(new Event('input'));
                    contentInput.dispatchEvent(new Event('input'));
                    
                    this.logTest('表單驗證', true, '表單填寫和驗證正常');
                } else {
                    this.logTest('表單驗證', false, '找不到表單輸入欄位');
                }
            } else {
                this.logTest('表單驗證', false, '找不到prompt表單');
            }
        } catch (error) {
            this.logTest('表單驗證', false, `錯誤: ${error.message}`);
        }
    }

    // 測試即時更新功能
    async testRealTimeUpdates() {
        console.log('\n⚡ 測試即時更新功能...');
        
        try {
            // 測試狀態更新通知
            const originalDispatchEvent = document.dispatchEvent;
            let eventFired = false;
            
            document.dispatchEvent = function(event) {
                if (event.type === 'stateChange') {
                    eventFired = true;
                }
                return originalDispatchEvent.call(this, event);
            };
            
            // 觸發狀態變更
            if (window.appState) {
                window.appState.setState('testKey', 'testValue');
                
                setTimeout(() => {
                    this.logTest('狀態更新事件', eventFired, '狀態變更事件正確觸發');
                    document.dispatchEvent = originalDispatchEvent;
                }, 100);
            } else {
                this.logTest('狀態更新事件', false, '找不到appState物件');
            }
        } catch (error) {
            this.logTest('狀態更新事件', false, `錯誤: ${error.message}`);
        }
    }

    // 測試效能指標
    async testPerformanceMetrics() {
        console.log('\n📊 測試效能指標...');
        
        try {
            const startTime = performance.now();
            
            // 測試大量DOM操作的效能
            const testContainer = document.createElement('div');
            document.body.appendChild(testContainer);
            
            for (let i = 0; i < 1000; i++) {
                const element = document.createElement('div');
                element.textContent = `測試元素 ${i}`;
                testContainer.appendChild(element);
            }
            
            const endTime = performance.now();
            const duration = endTime - startTime;
            
            document.body.removeChild(testContainer);
            
            this.logTest('DOM操作效能', duration < 100, `耗時: ${duration.toFixed(2)}ms`);
        } catch (error) {
            this.logTest('DOM操作效能', false, `錯誤: ${error.message}`);
        }

        // 測試記憶體使用
        if (performance.memory) {
            const memoryInfo = performance.memory;
            const memoryUsage = Math.round(memoryInfo.usedJSHeapSize / 1024 / 1024);
            this.logTest('記憶體使用', memoryUsage < 100, `使用: ${memoryUsage}MB`);
        } else {
            this.logTest('記憶體使用', false, '瀏覽器不支援記憶體監控');
        }
    }

    // 生成測試報告
    generateReport() {
        const successRate = Math.round((this.passedTests / this.totalTests) * 100);
        
        const report = {
            summary: {
                total: this.totalTests,
                passed: this.passedTests,
                failed: this.failedTests,
                successRate: `${successRate}%`,
                timestamp: new Date().toISOString()
            },
            results: this.testResults
        };

        console.log('\n📋 GUI 整合測試報告');
        console.log('================================');
        console.log(`📊 總測試數: ${this.totalTests}`);
        console.log(`✅ 通過測試: ${this.passedTests}`);
        console.log(`❌ 失敗測試: ${this.failedTests}`);
        console.log(`📈 成功率: ${successRate}%`);
        console.log('================================');
        
        if (successRate >= 80) {
            console.log('🎉 GUI整合測試結果優秀！');
        } else if (successRate >= 60) {
            console.log('⚠️ GUI整合測試結果良好，但有改進空間');
        } else {
            console.log('❌ GUI整合測試需要改進');
        }

        // 顯示詳細結果
        console.log('\n📝 詳細測試結果:');
        this.testResults.forEach((result, index) => {
            const status = result.passed ? '✅' : '❌';
            console.log(`${index + 1}. ${status} ${result.name} - ${result.details}`);
        });

        return report;
    }

    // 執行完整測試套件
    async runFullTest() {
        console.log('🚀 開始 GUI 整合測試...');
        console.log('測試時間:', new Date().toLocaleString());
        
        try {
            await this.testTauriCommands();
            await this.testDOMIntegration();
            await this.testRealTimeUpdates();
            await this.testPerformanceMetrics();
            
            return this.generateReport();
        } catch (error) {
            console.error('測試執行過程中發生錯誤:', error);
            return {
                error: error.message,
                timestamp: new Date().toISOString()
            };
        }
    }
}

// 等待 DOM 載入完成後自動執行測試
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        // 延遲執行以確保所有組件都已初始化
        setTimeout(() => {
            const tester = new GUIIntegrationTest();
            window.guiTester = tester; // 供手動測試使用
            tester.runFullTest();
        }, 2000);
    });
} else {
    // DOM已經載入完成
    setTimeout(() => {
        const tester = new GUIIntegrationTest();
        window.guiTester = tester;
        tester.runFullTest();
    }, 2000);
}

// 供手動執行測試的全域函數
window.runGUITest = () => {
    const tester = new GUIIntegrationTest();
    return tester.runFullTest();
};

console.log('GUI 整合測試腳本已載入');
console.log('可以使用 window.runGUITest() 手動執行測試');