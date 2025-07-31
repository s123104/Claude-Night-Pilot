/**
 * Claude Night Pilot 統一API客戶端
 * 提供GUI和CLI一致的API介面
 */

class UnifiedApiClient {
  constructor() {
    this.isProduction = !!window.__TAURI_API__;
  }

  /**
   * 統一的Claude執行命令
   * @param {string} prompt - 要執行的prompt
   * @param {Object} options - 執行選項
   * @returns {Promise<Object>} 執行結果
   */
  async executeClaudeUnified(prompt, options = {}) {
    const unifiedOptions = {
      mode: options.mode || "sync",
      cron_expr: options.cronExpr || null,
      retry_enabled: options.retryEnabled !== false,
      cooldown_check: options.cooldownCheck !== false,
      working_directory: options.workingDirectory || null,
    };

    if (this.isProduction) {
      return await window.__TAURI_API__.invoke("execute_unified_claude", {
        prompt,
        options: unifiedOptions,
      });
    }

    // 開發環境模擬
    return this.mockExecuteResponse(prompt, unifiedOptions);
  }

  /**
   * 統一的冷卻狀態檢查
   * @returns {Promise<Object>} 冷卻狀態信息
   */
  async getCooldownStatusUnified() {
    if (this.isProduction) {
      return await window.__TAURI_API__.invoke("get_unified_cooldown_status");
    }

    return this.mockCooldownResponse();
  }

  /**
   * 統一的系統健康檢查
   * @returns {Promise<Object>} 系統健康狀態
   */
  async getSystemHealthUnified() {
    if (this.isProduction) {
      return await window.__TAURI_API__.invoke("get_unified_system_health");
    }

    return this.mockHealthResponse();
  }

  /**
   * 向後兼容的API調用
   * @param {string} command - 命令名稱
   * @param {Object} args - 參數
   * @returns {Promise<any>} 調用結果
   */
  async invokeCommand(command, args = {}) {
    // 映射舊命令到新的統一命令
    switch (command) {
    case "run_prompt_sync":
      return this.executeClaudeUnified(args.prompt || args.content, {
        mode: args.mode || "sync",
        cronExpr: args.cron_expr,
      });

    case "get_cooldown_status":
      return this.getCooldownStatusUnified();

    case "get_system_info":
      return this.getSystemHealthUnified();

    default:
      // 其他命令保持原有調用方式
      if (this.isProduction) {
        return await window.__TAURI_API__.invoke(command, args);
      }
      return this.mockResponse(command, args);
    }
  }

  // 開發環境模擬響應
  mockExecuteResponse(prompt, options) {
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve({
          completion: `模擬回應: ${prompt.substring(0, 50)}...的執行結果`,
          model: "claude-3-sonnet",
          usage: {
            input_tokens: 150,
            output_tokens: 300,
          },
          execution_metadata: {
            execution_id: window.crypto?.randomUUID?.() || "exec_" + Date.now(),
            start_time: new Date().toISOString(),
            end_time: new Date(Date.now() + 2000).toISOString(),
            total_attempts: 1,
            cooldown_detected: null,
            process_handle: window.crypto?.randomUUID?.() || "proc_" + Date.now(),
            scheduler_used: options.mode,
          },
        });
      }, 1000);
    });
  }

  mockCooldownResponse() {
    return Promise.resolve({
      is_cooling: false,
      remaining_seconds: 0,
      cooldown_type: "none",
      eta_minutes: 0,
      human_readable: "可立即執行",
      last_check: new Date().toISOString(),
      pattern_detected: "none",
    });
  }

  mockHealthResponse() {
    return Promise.resolve({
      claude_cli_available: true,
      cooldown_detection_working: true,
      current_cooldown: null,
      active_processes: 0,
      last_check: new Date().toISOString(),
      timestamp: new Date().toISOString(),
    });
  }

  mockResponse(command, _args) {
    // 兼容原有的mock響應系統
    switch (command) {
    case "list_prompts":
      return [
        {
          id: 1,
          title: "測試 Prompt",
          content: "這是一個測試用的 prompt 內容",
          tags: "test,demo",
          created_at: new Date().toISOString(),
        },
      ];

    case "create_prompt":
      return 999;

    case "delete_prompt":
      return true;

    case "list_jobs":
      return [
        {
          id: 1,
          prompt_id: 1,
          job_name: "每日自動分析",
          cron_expr: "0 9 * * *",
          status: "active",
          last_run_at: new Date(Date.now() - 86400000).toISOString(),
          next_run_at: new Date(Date.now() + 86400000).toISOString(),
          created_at: new Date().toISOString(),
        },
      ];

    default:
      return { success: true, message: `Mock response for ${command}` };
    }
  }
}

// 創建全局實例
window.unifiedApiClient = new UnifiedApiClient();

// 向後兼容：保持原有的apiClient
window.apiClient = {
  invokeCommand: (command, args) =>
    window.unifiedApiClient.invokeCommand(command, args),
};
