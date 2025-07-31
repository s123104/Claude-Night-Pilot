/* global unifiedApiClient, snackbarManager */
/**
 * Claude Night Pilot Prompt執行器
 * 統一的Prompt執行和管理系統
 */

class PromptExecutor {
  constructor() {
    this.executionHistory = [];
    this.activeExecutions = new Map();
  }

  /**
   * 執行Prompt
   * @param {Object} prompt - Prompt對象
   * @param {Object} options - 執行選項
   * @returns {Promise<Object>} 執行結果
   */
  async executePrompt(prompt, options = {}) {
    const executionId = window.crypto?.randomUUID?.() || "exec_" + Date.now() + "_" + Math.random().toString(36).substr(2, 9);
    const startTime = Date.now();

    try {
      // 記錄執行開始
      this.activeExecutions.set(executionId, {
        prompt,
        options,
        startTime,
        status: "running",
      });

      // 顯示執行狀態
      this.showExecutionStatus(executionId, prompt.title);

      // 調用統一API執行
      const result = await unifiedApiClient.executeClaudeUnified(
        prompt.content,
        {
          mode: options.mode || "sync",
          retryEnabled: options.enableRetry !== false,
          cooldownCheck: options.checkCooldown !== false,
          workingDirectory: options.workingDirectory,
        },
      );

      // 記錄執行成功
      const execution = {
        id: executionId,
        prompt,
        options,
        result,
        startTime,
        endTime: Date.now(),
        status: "completed",
        duration: Date.now() - startTime,
      };

      this.executionHistory.unshift(execution);
      this.activeExecutions.delete(executionId);

      // 更新UI
      this.updateExecutionResult(executionId, result, "success");

      return result;
    } catch (error) {
      // 記錄執行失敗
      const execution = {
        id: executionId,
        prompt,
        options,
        error: error.message,
        startTime,
        endTime: Date.now(),
        status: "failed",
        duration: Date.now() - startTime,
      };

      this.executionHistory.unshift(execution);
      this.activeExecutions.delete(executionId);

      // 更新UI
      this.updateExecutionResult(executionId, error, "error");

      throw error;
    }
  }

  /**
   * 執行Prompt（通過ID）
   * @param {string|number} promptId - Prompt ID
   * @param {Object} options - 執行選項
   */
  async executePromptById(promptId, options = {}) {
    try {
      // 獲取Prompt詳情
      const prompts = await unifiedApiClient.invokeCommand("list_prompts");
      const prompt = prompts.find(
        (p) => p.id.toString() === promptId.toString(),
      );

      if (!prompt) {
        throw new Error(`Prompt ID ${promptId} 不存在`);
      }

      return await this.executePrompt(prompt, options);
    } catch (error) {
      console.error("執行Prompt失敗:", error);
      snackbarManager.error(`執行失敗：${error.message}`);
      throw error;
    }
  }

  /**
   * 批量執行Prompts
   * @param {Array} prompts - Prompt列表
   * @param {Object} options - 執行選項
   */
  async executeBatch(prompts, options = {}) {
    const results = [];
    const concurrent = options.concurrent || 1;

    if (concurrent === 1) {
      // 串行執行
      for (const prompt of prompts) {
        try {
          const result = await this.executePrompt(prompt, options);
          results.push({ prompt, result, status: "success" });
        } catch (error) {
          results.push({ prompt, error: error.message, status: "failed" });
        }
      }
    } else {
      // 並行執行
      const chunks = this.chunkArray(prompts, concurrent);
      for (const chunk of chunks) {
        const chunkPromises = chunk.map(async (prompt) => {
          try {
            const result = await this.executePrompt(prompt, options);
            return { prompt, result, status: "success" };
          } catch (error) {
            return { prompt, error: error.message, status: "failed" };
          }
        });

        const chunkResults = await Promise.all(chunkPromises);
        results.push(...chunkResults);
      }
    }

    return results;
  }

  /**
   * 顯示執行狀態
   */
  showExecutionStatus(executionId, promptTitle) {
    snackbarManager.info(`正在執行: ${promptTitle}`, 0); // 持續顯示直到完成

    // 如果有執行狀態面板，更新它
    const statusPanel = document.getElementById("execution-status-panel");
    if (statusPanel) {
      this.updateStatusPanel(executionId, promptTitle, "running");
    }
  }

  /**
   * 更新執行結果
   */
  updateExecutionResult(executionId, result, status) {
    // 關閉loading snackbar
    snackbarManager.clear();

    if (status === "success") {
      snackbarManager.success("執行完成！");
      this.showResultDialog(result);
    } else {
      snackbarManager.error(`執行失敗：${result.message}`);
    }

    // 更新狀態面板
    const statusPanel = document.getElementById("execution-status-panel");
    if (statusPanel) {
      this.updateStatusPanel(executionId, null, status);
    }

    // 刷新執行歷史
    this.refreshExecutionHistory();
  }

  /**
   * 顯示執行結果對話框
   */
  showResultDialog(result) {
    const dialog = document.createElement("div");
    dialog.className = "result-dialog";
    dialog.innerHTML = `
            <div class="dialog-backdrop" onclick="this.parentElement.remove()">
                <div class="dialog-content" onclick="event.stopPropagation()">
                    <div class="dialog-header">
                        <h3>執行結果</h3>
                        <button class="dialog-close" onclick="this.closest('.result-dialog').remove()">
                            <i class="material-icons">close</i>
                        </button>
                    </div>
                    <div class="dialog-body">
                        <div class="result-content">
                            <h4>回應內容</h4>
                            <div class="response-text">${this.escapeHtml(
    result.completion,
  )}</div>
                        </div>
                        ${
  result.usage
    ? `
                            <div class="usage-info">
                                <h4>使用統計</h4>
                                <div class="usage-stats">
                                    <span>輸入Token: ${
  result.usage.input_tokens || 0
}</span>
                                    <span>輸出Token: ${
  result.usage.output_tokens || 0
}</span>
                                </div>
                            </div>
                        `
    : ""
}
                        ${
  result.execution_metadata
    ? `
                            <div class="execution-metadata">
                                <h4>執行信息</h4>
                                <div class="metadata-info">
                                    <div>執行ID: ${
  result.execution_metadata.execution_id
}</div>
                                    <div>重試次數: ${
  result.execution_metadata
    .total_attempts || 1
}</div>
                                    <div>排程器: ${
  result.execution_metadata
    .scheduler_used || "default"
}</div>
                                </div>
                            </div>
                        `
    : ""
}
                    </div>
                    <div class="dialog-actions">
                        <button class="btn-secondary" onclick="this.closest('.result-dialog').remove()">
                            關閉
                        </button>
                        <button class="btn-primary" onclick="this.copyResult('${this.escapeHtml(
    result.completion,
  )}')">
                            複製結果
                        </button>
                    </div>
                </div>
            </div>
        `;

    document.body.appendChild(dialog);

    // 添加複製功能
    dialog.copyResult = (text) => {
      navigator.clipboard.writeText(text).then(() => {
        snackbarManager.success("結果已複製到剪貼板");
      });
    };
  }

  /**
   * 更新狀態面板
   */
  updateStatusPanel(executionId, title, status) {
    // 實現狀態面板更新邏輯
    console.log(`執行狀態更新: ${executionId} - ${status}`);
  }

  /**
   * 刷新執行歷史
   */
  refreshExecutionHistory() {
    const historyContainer = document.getElementById("execution-history");
    if (historyContainer) {
      this.renderExecutionHistory(historyContainer);
    }
  }

  /**
   * 渲染執行歷史
   */
  renderExecutionHistory(container) {
    const recentExecutions = this.executionHistory.slice(0, 10);

    container.innerHTML = recentExecutions
      .map(
        (execution) => `
            <div class="execution-item ${execution.status}">
                <div class="execution-header">
                    <h4>${this.escapeHtml(execution.prompt.title)}</h4>
                    <span class="execution-time">${this.formatDuration(
    execution.duration,
  )}</span>
                </div>
                <div class="execution-status">
                    <span class="status-badge ${execution.status}">
                        ${execution.status === "completed" ? "完成" : "失敗"}
                    </span>
                    <span class="execution-date">
                        ${new Date(execution.endTime).toLocaleString()}
                    </span>
                </div>
            </div>
        `,
      )
      .join("");
  }

  // 工具方法
  escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }

  formatDuration(ms) {
    if (ms < 1000) {return `${ms}ms`;}
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) {return `${seconds}s`;}
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ${seconds % 60}s`;
  }

  chunkArray(array, size) {
    const chunks = [];
    for (let i = 0; i < array.length; i += size) {
      chunks.push(array.slice(i, i + size));
    }
    return chunks;
  }
}

// 創建全局實例
window.promptExecutor = new PromptExecutor();
