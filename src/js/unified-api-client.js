/**
 * Claude Night Pilot 統一API客戶端
 * 提供GUI和CLI一致的API介面
 */

class UnifiedApiClient {
  constructor() {
    this.isProduction = this.detectTauriEnvironment();
    this.initializeAPI();
  }

  detectTauriEnvironment() {
    // Check for Tauri 2.0 API first
    if (window.__TAURI__ && window.__TAURI__.core) {
      console.log("Tauri 2.0 detected");
      return true;
    }
    // Fallback to Tauri 1.x
    if (window.__TAURI_API__) {
      console.log("Tauri 1.x detected");
      return true;
    }
    // Development/test mode
    console.log("Development mode detected");
    return false;
  }

  async initializeAPI() {
    if (this.isProduction) {
      try {
        // Test basic API connectivity
        await this.invokeCommand("health_check").catch(() => {
          console.warn(
            "Tauri API health check failed, falling back to development mode",
          );
          this.isProduction = false;
        });
      } catch (error) {
        console.warn("API initialization failed:", error);
        this.isProduction = false;
      }
    }
  }

  async invokeCommand(command, args = {}) {
    if (this.isProduction) {
      try {
        // Try Tauri 2.0 API first
        if (
          window.__TAURI__ &&
          window.__TAURI__.core &&
          window.__TAURI__.core.invoke
        ) {
          return await window.__TAURI__.core.invoke(command, args);
        }
        // Fallback to Tauri 1.x
        else if (window.__TAURI_API__ && window.__TAURI_API__.invoke) {
          return await window.__TAURI_API__.invoke(command, args);
        }
      } catch (error) {
        console.error(`Tauri command '${command}' failed:`, error);
        // Fall through to mock
      }
    }

    // Use mock responses in development or when Tauri fails
    return this.getMockResponse(command, args);
  }

  getMockResponse(command, args) {
    console.log(`Using mock response for: ${command}`);
    return this.mockResponse(command, args);
  }

  /**
   * 統一的Claude執行命令 - 使用直接調用
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

    return await this.invokeCommandDirect("execute_unified_claude", {
      prompt,
      options: unifiedOptions,
    });
  }

  /**
   * 統一的冷卻狀態檢查 - 使用直接調用
   * @returns {Promise<Object>} 冷卻狀態信息
   */
  async getCooldownStatusUnified() {
    return await this.invokeCommandDirect("get_unified_cooldown_status");
  }

  /**
   * 統一的系統健康檢查 - 使用直接調用
   * @returns {Promise<Object>} 系統健康狀態
   */
  async getSystemHealthUnified() {
    return await this.invokeCommandDirect("get_unified_system_health");
  }

  /**
   * 取得代理清單（Catalog） - 使用直接調用
   * @returns {Promise<Object>} 代理清單
   */
  async getAgentsCatalog() {
    return await this.invokeCommandDirect("get_agents_catalog");
  }

  /**
   * 新的共享服務API調用
   */

  // Prompt服務API - 使用直接調用避免遞歸
  async listPromptsService() {
    return await this.invokeCommandDirect("prompt_service_list_prompts");
  }

  async createPromptService(title, content, tags = null) {
    return await this.invokeCommandDirect("prompt_service_create_prompt", {
      title,
      content,
      tags,
    });
  }

  async deletePromptService(id) {
    return await this.invokeCommandDirect("prompt_service_delete_prompt", {
      id,
    });
  }

  // Job服務API - 使用直接調用避免遞歸
  async listJobsService() {
    return await this.invokeCommandDirect("job_service_list_jobs");
  }

  async createJobService(promptId, name, cronExpression, description = null) {
    return await this.invokeCommandDirect("job_service_create_job", {
      prompt_id: promptId,
      name,
      cron_expression: cronExpression,
      description,
    });
  }

  async deleteJobService(id) {
    return await this.invokeCommandDirect("job_service_delete_job", { id });
  }

  // 同步服務API - 使用直接調用避免遞歸
  async getSyncStatusService() {
    return await this.invokeCommandDirect("sync_service_get_status");
  }

  async triggerSyncService() {
    return await this.invokeCommandDirect("sync_service_trigger_sync");
  }

  /**
   * 向後兼容的API調用 - Fixed infinite recursion
   * @param {string} command - 命令名稱
   * @param {Object} args - 參數
   * @returns {Promise<any>} 調用結果
   */
  async invokeCommand(command, args = {}) {
    // 映射舊命令到新的服務API
    switch (command) {
    case "list_prompts":
    case "get_prompts":
      return this.listPromptsService();

    case "create_prompt":
      return this.createPromptService(args.title, args.content, args.tags);

    case "delete_prompt":
      return this.deletePromptService(args.id);

    case "list_jobs":
    case "get_jobs":
      return this.listJobsService();

    case "create_job":
      return this.createJobService(
        args.prompt_id || args.promptId,
        args.name || args.job_name || "新任務",
        args.cron_expression || args.cronExpression,
        args.description,
      );

    case "delete_job":
      return this.deleteJobService(args.id);

    case "run_prompt_sync":
      return this.executeClaudeUnified(args.prompt || args.content, {
        mode: args.mode || "sync",
        cronExpr: args.cron_expr,
      });

    case "get_cooldown_status":
      return this.getCooldownStatusUnified();

    case "get_system_info":
    case "get_app_info":
    case "get_performance_info":
      return this.getSystemHealthUnified();

    default:
      // 使用統一調用方式 - 修復無限遞歸
      return await this.invokeCommandDirect(command, args);
    }
  }

  /**
   * 直接調用Tauri命令（避免無限遞歸）
   * @param {string} command - 命令名稱
   * @param {Object} args - 參數
   * @returns {Promise<any>} 調用結果
   */
  async invokeCommandDirect(command, args = {}) {
    if (this.isProduction) {
      try {
        // Try Tauri 2.0 API first
        if (
          window.__TAURI__ &&
          window.__TAURI__.core &&
          window.__TAURI__.core.invoke
        ) {
          return await window.__TAURI__.core.invoke(command, args);
        }
        // Fallback to Tauri 1.x
        else if (window.__TAURI_API__ && window.__TAURI_API__.invoke) {
          return await window.__TAURI_API__.invoke(command, args);
        }
      } catch (error) {
        console.error(`Tauri command '${command}' failed:`, error);
        // Fall through to mock
      }
    }

    // Use mock responses in development or when Tauri fails
    return this.getMockResponse(command, args);
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
            process_handle:
              window.crypto?.randomUUID?.() || "proc_" + Date.now(),
            scheduler_used: options.mode,
          },
        });
      }, 100); // Faster for tests
    });
  }

  mockCooldownResponse() {
    const now = new Date();
    return Promise.resolve({
      is_available: true,
      is_cooling: false,
      // 保持向後相容：同時提供 remaining_seconds 與 seconds_remaining
      remaining_seconds: 0,
      seconds_remaining: 0,
      next_available_time: null,
      cooldown_type: "none",
      eta_minutes: 0,
      human_readable: "可立即執行",
      last_check: now.toISOString(),
      pattern_detected: "none",
      reset_time: null,
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

  // 增強的Mock響應系統 - 使用服務格式
  mockPromptsResponse() {
    return [
      {
        id: 1,
        title: "架構分析 Prompt",
        content:
          "@README.md @src/ 請分析這個專案的整體架構，包括前端、後端和資料庫設計，並提供改進建議。",
        tags: "architecture,analysis,code-review",
        created_at: new Date(Date.now() - 86400000).toISOString(),
        updated_at: null,
      },
      {
        id: 2,
        title: "程式碼品質檢查",
        content:
          "@src/**/*.js @src/**/*.ts 檢查程式碼品質，找出潛在的bug和效能問題。",
        tags: "quality,performance,debugging",
        created_at: new Date(Date.now() - 172800000).toISOString(),
        updated_at: null,
      },
      {
        id: 3,
        title: "文檔生成助手",
        content: "根據程式碼自動生成API文檔和使用說明。",
        tags: "documentation,api,automation",
        created_at: new Date(Date.now() - 259200000).toISOString(),
        updated_at: null,
      },
    ];
  }

  mockJobsResponse() {
    return [
      {
        id: 1,
        prompt_id: 1,
        name: "每日架構分析",
        cron_expression: "0 9 * * *",
        status: "active",
        description: "每天早上9點自動執行架構分析",
        last_run_at: new Date(Date.now() - 3600000).toISOString(),
        next_run_at: new Date(Date.now() + 82800000).toISOString(),
        created_at: new Date(Date.now() - 86400000).toISOString(),
        updated_at: null,
      },
      {
        id: 2,
        prompt_id: 2,
        name: "程式碼品質檢查",
        cron_expression: "0 12,18 * * *",
        status: "paused",
        description: "每天中午12點和下午6點檢查程式碼品質",
        last_run_at: new Date(Date.now() - 172800000).toISOString(),
        next_run_at: null,
        created_at: new Date(Date.now() - 172800000).toISOString(),
        updated_at: null,
      },
    ];
  }

  mockSyncStatusResponse() {
    return {
      gui_cli_sync_enabled: true,
      last_sync_timestamp: new Date().toISOString(),
      pending_changes: Math.floor(Math.random() * 5),
      sync_conflicts: 0,
      performance_impact: 2.3,
      sync_health: "healthy",
    };
  }

  mockResponse(command, _args) {
    // 兼容原有的mock響應系統
    switch (command) {
    case "prompt_service_list_prompts":
    case "list_prompts":
      return this.mockPromptsResponse();

    case "prompt_service_create_prompt":
    case "create_prompt":
      return Math.floor(Math.random() * 1000) + 100;

    case "prompt_service_delete_prompt":
    case "delete_prompt":
      return true;

    case "job_service_list_jobs":
    case "list_jobs":
      return this.mockJobsResponse();

    case "job_service_create_job":
    case "create_job":
      return Math.floor(Math.random() * 1000) + 200;

    case "job_service_delete_job":
    case "delete_job":
      return true;

    case "sync_service_get_status":
      return this.mockSyncStatusResponse();

    case "sync_service_trigger_sync":
      return "sync_" + Date.now();

    case "execute_unified_claude":
      return this.mockExecuteResponse(
        _args?.prompt || "test prompt",
        _args?.options || {},
      );

    case "get_unified_cooldown_status":
    case "get_cooldown_status":
      return this.mockCooldownResponse();

    case "get_unified_system_health":
    case "get_system_info":
    case "get_app_info":
    case "get_performance_info":
      return this.mockHealthResponse();

    case "get_agents_catalog":
      return { version: "dev", departments: [] };

    case "health_check":
      return { status: "healthy", timestamp: new Date().toISOString() };

    case "get_results":
      return [
        {
          id: 1,
          prompt_id: 1,
          prompt_title: "架構分析 Prompt",
          status: "success",
          output:
              "專案架構分析完成。\n\n✅ 前端使用 Material Design 3.0\n✅ 後端採用 Rust + Tauri\n✅ 資料庫使用 SQLite\n\n建議改進：\n- 加強錯誤處理機制\n- 增加單元測試覆蓋率\n- 優化載入效能",
          execution_time: 2340,
          created_at: new Date(Date.now() - 3600000).toISOString(),
        },
        {
          id: 2,
          prompt_id: 2,
          prompt_title: "程式碼品質檢查",
          status: "error",
          output:
              "執行過程中發生錯誤：\n\nError: Connection timeout\n請檢查網路連接或 Claude API 配置。",
          execution_time: 5000,
          created_at: new Date(Date.now() - 7200000).toISOString(),
        },
      ];

    default:
      console.log(`Unknown mock command: ${command}`);
      return {
        success: true,
        message: `Mock response for ${command}`,
        command,
      };
    }
  }
}

// 創建全局實例
window.unifiedApiClient = new UnifiedApiClient();

// 向後兼容：保持原有的apiClient
window.apiClient = {
  invokeCommand: (command, args) =>
    window.unifiedApiClient.invokeCommand(command, args),
  mockResponse: (command, args) =>
    window.unifiedApiClient.mockResponse(command, args),
};

// Ensure the unified API client is available globally for debugging
console.log("UnifiedApiClient initialized:", !!window.unifiedApiClient);
