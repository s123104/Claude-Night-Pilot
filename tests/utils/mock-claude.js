/**
 * Claude CLI 模擬工具
 * 
 * 提供測試環境下的 Claude CLI 模擬功能
 */

/**
 * 模擬的 Claude CLI 回應
 */
export const mockClaudeResponses = {
  success: {
    status: 'success',
    output: 'Hello from Claude! This is a test response.',
    timestamp: new Date().toISOString(),
    usage: {
      input_tokens: 10,
      output_tokens: 15,
      total_tokens: 25
    }
  },
  
  error: {
    status: 'error',
    error: 'Command execution failed',
    code: 1,
    timestamp: new Date().toISOString()
  },
  
  cooldown: {
    status: 'cooldown',
    message: 'Rate limit exceeded',
    retry_after: 60,
    timestamp: new Date().toISOString()
  }
};

/**
 * 模擬的系統資訊
 */
export const mockSystemInfo = {
  platform: process.platform,
  arch: process.arch,
  node_version: process.version,
  claude_cli_available: true,
  database_status: 'connected',
  scheduler_status: 'running'
};

/**
 * 模擬的冷卻狀態
 */
export const mockCooldownStatus = {
  is_cooling_down: false,
  time_remaining: 0,
  next_available: null,
  rate_limit_info: {
    requests_per_minute: 50,
    requests_used: 15,
    reset_time: new Date(Date.now() + 60000).toISOString()
  }
};

/**
 * 設定 Page 的 Claude CLI 模擬
 * @param {Page} page - Playwright Page 物件
 */
export async function setupClaudeMock(page) {
  await page.addInitScript(() => {
    console.log('Setting up Claude mock...');
    
    // 模擬統一 API 客戶端
    window.unifiedApiClient = {
      async executeClaudeUnified(prompt, options = {}) {
        await new Promise(resolve => setTimeout(resolve, 200)); // Faster for tests
        
        if (prompt.includes('error')) {
          throw new Error('Simulated execution error');
        }
        
        return {
          status: 'success',
          completion: `Simulated response for: ${prompt.substring(0, 50)}...`,
          output: `Simulated response for: ${prompt.substring(0, 50)}...`,
          timestamp: new Date().toISOString(),
          execution_time: Math.random() * 500 + 100
        };
      },
      
      // Legacy method for backward compatibility
      async executePrompt(prompt, options = {}) {
        return this.executeClaudeUnified(prompt, options);
      },
      
      async getCooldownStatusUnified() {
        return {
          is_available: true,
          is_cooling: false,
          is_cooling_down: false,
          time_remaining: 0,
          remaining_seconds: 0,
          next_available: null,
          reset_time: null,
          status: 'available',
          human_readable: '可立即執行'
        };
      },
      
      async getSystemHealthUnified() {
        return {
          claude_cli_available: true,
          cooldown_detection_working: true,
          current_cooldown: null,
          active_processes: 0,
          platform: 'test',
          claude_cli_status: 'available',
          database_status: 'connected',
          scheduler_status: 'running',
          version: '0.1.0-test',
          last_check: new Date().toISOString(),
          timestamp: new Date().toISOString()
        };
      },
      
      // Legacy method
      async getSystemInfo() {
        return this.getSystemHealthUnified();
      },
      
      async createPromptService(title, content, tags = null) {
        await new Promise(resolve => setTimeout(resolve, 200));
        return Math.floor(Math.random() * 10000);
      },
      
      async createPrompt(promptData) {
        await new Promise(resolve => setTimeout(resolve, 200));
        return {
          id: Math.floor(Math.random() * 10000),
          ...promptData,
          created_at: new Date().toISOString()
        };
      },
      
      async deletePromptService(id) {
        await new Promise(resolve => setTimeout(resolve, 100));
        return true;
      },
      
      async deletePrompt(promptId) {
        await new Promise(resolve => setTimeout(resolve, 100));
        return { success: true, id: promptId };
      },
      
      async listPromptsService() {
        return [
          {
            id: 1,
            title: "測試 Prompt 1",
            content: "這是第一個測試 Prompt",
            tags: "test,sample",
            created_at: new Date().toISOString(),
            updated_at: null
          },
          {
            id: 2,
            title: "Claude Code 測試",
            content: "@README.md 分析這個專案的結構",
            tags: "claude-code,analysis",
            created_at: new Date().toISOString(),
            updated_at: null
          }
        ];
      },
      
      async listPrompts() {
        return this.listPromptsService();
      },
      
      async listJobsService() {
        return [
          {
            id: 1,
            prompt_id: 1,
            name: "測試任務",
            cron_expression: "0 9 * * *",
            status: "active",
            description: "測試用排程任務",
            last_run_at: new Date(Date.now() - 3600000).toISOString(),
            next_run_at: new Date(Date.now() + 86400000).toISOString(),
            created_at: new Date().toISOString(),
            updated_at: null
          }
        ];
      },
      
      async createJobService(promptId, name, cronExpression, description = null) {
        await new Promise(resolve => setTimeout(resolve, 200));
        return Math.floor(Math.random() * 1000) + 200;
      },
      
      async deleteJobService(id) {
        await new Promise(resolve => setTimeout(resolve, 100));
        return true;
      },
      
      async getSyncStatusService() {
        return {
          gui_cli_sync_enabled: true,
          last_sync_timestamp: new Date().toISOString(),
          pending_changes: 0,
          sync_conflicts: 0,
          performance_impact: 1.2,
          sync_health: "healthy",
        };
      },
      
      async triggerSyncService() {
        return "sync_" + Date.now();
      },
      
      async cleanup() {
        // 清理測試資料的模擬
        console.log('Mock cleanup executed');
        return { success: true };
      }
    };
    
    // Initialize API client
    if (window.unifiedApiClient) {
      console.log('UnifiedApiClient mock setup complete');
    }
    
    // 模擬 Tauri 2.0 API
    window.__TAURI__ = {
      core: {
        async invoke(command, args = {}) {
          console.log(`Mock Tauri command: ${command}`, args);
          
          // Delegate to unified API client if available
          if (window.unifiedApiClient && typeof window.unifiedApiClient.getMockResponse === 'function') {
            return window.unifiedApiClient.getMockResponse(command, args);
          }
          
          // Fallback responses
          switch (command) {
            case 'health_check':
              return { status: 'healthy', timestamp: new Date().toISOString() };
              
            case 'get_system_info':
            case 'get_app_info':
              return {
                platform: 'test',
                version: '0.1.0-test',
                claude_cli_available: true,
                database_status: 'connected'
              };
              
            case 'prompt_service_list_prompts':
            case 'list_prompts':
              return window.unifiedApiClient ? window.unifiedApiClient.listPromptsService() : [];
              
            case 'get_cooldown_status':
            case 'get_unified_cooldown_status':
              return {
                is_available: true,
                is_cooling: false,
                time_remaining: 0
              };
              
            default:
              console.warn(`Unknown mock command: ${command}`);
              return { success: true, command, args };
          }
        }
      }
    };
    
    // Legacy Tauri 1.x support
    window.__TAURI_API__ = window.__TAURI__.core;
    
    // Set up DOM ready state
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => {
        console.log('Mock: DOM ready');
        window.__APP_READY__ = true;
      });
    } else {
      console.log('Mock: DOM already ready');
      window.__APP_READY__ = true;
    }
  });
}

/**
 * 設定錯誤情境模擬
 * @param {Page} page - Playwright Page 物件
 * @param {string} scenario - 錯誤情境類型
 */
export async function setupErrorScenario(page, scenario) {
  await page.addInitScript((scenario) => {
    const originalClient = window.unifiedApiClient;
    
    window.unifiedApiClient = {
      ...originalClient,
      
      async executePrompt(prompt, options = {}) {
        switch (scenario) {
          case 'network_error':
            throw new Error('Network connection failed');
            
          case 'rate_limit':
            throw new Error('Rate limit exceeded');
            
          case 'invalid_prompt':
            throw new Error('Invalid prompt format');
            
          case 'claude_unavailable':
            throw new Error('Claude CLI not available');
            
          default:
            return originalClient.executePrompt(prompt, options);
        }
      },
      
      async getCooldownStatusUnified() {
        if (scenario === 'cooldown_error') {
          throw new Error('Failed to check cooldown status');
        }
        
        if (scenario === 'rate_limit') {
          return {
            is_cooling_down: true,
            time_remaining: 300,
            next_available: new Date(Date.now() + 300000).toISOString()
          };
        }
        
        return originalClient.getCooldownStatusUnified();
      }
    };
  }, scenario);
}

/**
 * 驗證模擬設定
 * @param {Page} page - Playwright Page 物件
 */
export async function validateMockSetup(page) {
  const hasClient = await page.evaluate(() => {
    return typeof window.unifiedApiClient !== 'undefined';
  });
  
  const hasAppReady = await page.evaluate(() => {
    return window.__APP_READY__ === true;
  });
  
  const hasTauri = await page.evaluate(() => {
    return typeof window.__TAURI__ !== 'undefined';
  });
  
  return {
    unifiedApiClient: hasClient,
    appReady: hasAppReady,
    tauriApi: hasTauri
  };
}

/**
 * 重置模擬狀態
 * @param {Page} page - Playwright Page 物件
 */
export async function resetMockState(page) {
  await page.evaluate(() => {
    if (window.unifiedApiClient && window.unifiedApiClient.cleanup) {
      window.unifiedApiClient.cleanup();
    }
    
    // 重置應用狀態
    window.__APP_READY__ = true;
  });
}