/**
 * Claude Night Pilot - Modern UI/UX JavaScript
 * 現代化的夜間自動打工仔前端應用程式
 */

// ===== Application State Management =====
class AppState {
  constructor() {
    this.theme = localStorage.getItem("theme") || "auto";
    this.currentTab = "prompts";
    this.prompts = [];
    this.jobs = [];
    this.results = [];
    this.isLoading = false;
    this.cooldownStatus = "checking";
  }

  setState(key, value) {
    this[key] = value;
    this.notifyStateChange(key, value);
  }

  notifyStateChange(key, value) {
    document.dispatchEvent(
      new CustomEvent("stateChange", {
        detail: { key, value },
      })
    );
  }
}

// ===== Theme Management =====
class ThemeManager {
  constructor() {
    this.init();
  }

  init() {
    this.applyTheme(appState.theme);
    this.bindEvents();
  }

  applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
    appState.setState("theme", theme);
    this.updateThemeIcon(theme);
  }

  updateThemeIcon(theme) {
    const themeToggle = document.getElementById("theme-toggle");
    const icon = themeToggle?.querySelector(".material-symbols-outlined");

    if (!icon) return;

    if (theme === "dark") {
      icon.textContent = "dark_mode";
    } else if (theme === "light") {
      icon.textContent = "light_mode";
    } else {
      icon.textContent = "brightness_auto";
    }
  }

  toggleTheme() {
    const themes = ["auto", "light", "dark"];
    const currentIndex = themes.indexOf(appState.theme);
    const nextTheme = themes[(currentIndex + 1) % themes.length];
    this.applyTheme(nextTheme);
  }

  bindEvents() {
    const themeToggle = document.getElementById("theme-toggle");
    themeToggle?.addEventListener("click", () => this.toggleTheme());
  }
}

// ===== Toast Notification System =====
class NotificationManager {
  constructor() {
    this.container = document.getElementById("toast-container");
    this.toasts = new Map();
  }

  show(message, type = "info", duration = 5000) {
    const id = `toast-${Date.now()}`;
    const toast = this.createToast(id, message, type);

    this.container.appendChild(toast);
    this.toasts.set(id, toast);

    // Auto remove
    if (duration > 0) {
      setTimeout(() => this.remove(id), duration);
    }

    return id;
  }

  createToast(id, message, type) {
    const toast = document.createElement("div");
    toast.className = `toast ${type}`;
    toast.setAttribute("data-toast-id", id);

    const icons = {
      success: "check_circle",
      error: "error",
      warning: "warning",
      info: "info",
    };

    toast.innerHTML = `
      <span class="material-symbols-outlined toast-icon">${
        icons[type] || icons.info
      }</span>
      <div class="toast-content">
        <div class="toast-message">${message}</div>
      </div>
      <button class="close-btn" onclick="notificationManager.remove('${id}')">
        <span class="material-symbols-outlined">close</span>
      </button>
    `;

    return toast;
  }

  remove(id) {
    const toast = this.toasts.get(id);
    if (toast) {
      toast.style.animation = "toastSlideOut 0.3s ease-in forwards";
      setTimeout(() => {
        toast.remove();
        this.toasts.delete(id);
      }, 300);
    }
  }

  success(message) {
    return this.show(message, "success");
  }

  error(message) {
    return this.show(message, "error");
  }

  warning(message) {
    return this.show(message, "warning");
  }

  info(message) {
    return this.show(message, "info");
  }
}

// ===== Tab Navigation System =====
class TabManager {
  constructor() {
    this.tabs = document.querySelectorAll(".tab-btn");
    this.panels = document.querySelectorAll(".tab-panel");
    this.init();
  }

  init() {
    this.tabs.forEach((tab) => {
      tab.addEventListener("click", (e) => {
        const targetTab = e.currentTarget.dataset.tab;
        this.switchTab(targetTab);
      });
    });
  }

  switchTab(tabName) {
    // Update tab buttons
    this.tabs.forEach((tab) => {
      if (tab.dataset.tab === tabName) {
        tab.classList.add("active");
      } else {
        tab.classList.remove("active");
      }
    });

    // Update tab panels
    this.panels.forEach((panel) => {
      if (panel.id === `${tabName}-tab`) {
        panel.classList.add("active");
      } else {
        panel.classList.remove("active");
      }
    });

    appState.setState("currentTab", tabName);

    // Load tab content if needed
    this.loadTabContent(tabName);
  }

  async loadTabContent(tabName) {
    switch (tabName) {
      case "prompts":
        await promptManager.loadPrompts();
        break;
      case "scheduler":
        await jobManager.loadJobs();
        break;
      case "results":
        await resultManager.loadResults();
        break;
      case "system":
        await systemManager.loadSystemInfo();
        break;
    }
  }
}

// ===== Modal Management =====
class ModalManager {
  constructor() {
    this.modals = new Map();
    this.init();
  }

  init() {
    // Setup modal triggers
    document
      .getElementById("create-prompt-btn")
      ?.addEventListener("click", () => {
        this.open("prompt-modal");
      });

    document.getElementById("create-job-btn")?.addEventListener("click", () => {
      this.open("job-modal");
    });

    // Setup close handlers
    document.querySelectorAll(".close-btn, [data-close]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const modal = e.target.closest(".modal");
        if (modal) this.close(modal.id);
      });
    });

    // Setup form submissions
    this.setupForms();
  }

  open(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      modal.showModal();
      this.modals.set(modalId, modal);

      // Focus first input
      const firstInput = modal.querySelector("input, textarea, select");
      if (firstInput) {
        setTimeout(() => firstInput.focus(), 100);
      }
    }
  }

  close(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      modal.close();
      this.modals.delete(modalId);
      this.resetForm(modal);
    }
  }

  resetForm(modal) {
    const form = modal.querySelector("form");
    if (form) {
      form.reset();
    }
  }

  setupForms() {
    // Prompt form
    const promptForm = document.getElementById("prompt-form");
    promptForm?.addEventListener("submit", async (e) => {
      e.preventDefault();
      await this.handlePromptSubmit(e.target);
    });

    // Job form
    const jobForm = document.getElementById("job-form");
    jobForm?.addEventListener("submit", async (e) => {
      e.preventDefault();
      await this.handleJobSubmit(e.target);
    });
  }

  async handlePromptSubmit(form) {
    const formData = new FormData(form);
    const promptData = {
      title: formData.get("prompt-title"),
      content: formData.get("prompt-content"),
      tags:
        formData
          .get("prompt-tags")
          ?.split(",")
          .map((tag) => tag.trim())
          .filter(Boolean) || [],
    };

    try {
      await promptManager.createPrompt(promptData);
      this.close("prompt-modal");
      notificationManager.success("Prompt 建立成功！");
    } catch (error) {
      notificationManager.error(`建立失敗：${error.message}`);
    }
  }

  async handleJobSubmit(form) {
    const formData = new FormData(form);
    const jobData = {
      promptId: formData.get("job-prompt"),
      cronExpression: formData.get("job-cron"),
    };

    try {
      await jobManager.createJob(jobData);
      this.close("job-modal");
      notificationManager.success("排程任務建立成功！");
    } catch (error) {
      notificationManager.error(`建立失敗：${error.message}`);
    }
  }
}

// ===== API Client =====
class APIClient {
  constructor() {
    this.baseURL = "";
  }

  async request(endpoint, options = {}) {
    const url = `${this.baseURL}${endpoint}`;
    const config = {
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
      ...options,
    };

    if (config.body && typeof config.body === "object") {
      config.body = JSON.stringify(config.body);
    }

    try {
      const response = await fetch(url, config);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const contentType = response.headers.get("content-type");
      if (contentType && contentType.includes("application/json")) {
        return await response.json();
      }

      return await response.text();
    } catch (error) {
      console.error("API Request failed:", error);
      throw error;
    }
  }

  // Tauri commands
  async invokeCommand(command, args = {}) {
    if (window.__TAURI_API__) {
      return await window.__TAURI_API__.invoke(command, args);
    }

    // Fallback for development
    return this.mockResponse(command, args);
  }

  mockResponse(command, args) {
    // Mock responses for development
    switch (command) {
      case "get_prompts":
        return [
          {
            id: "1",
            title: "範例 Prompt",
            content: "@README.md 請分析這個專案的架構",
            tags: ["example", "analysis"],
            created_at: new Date().toISOString(),
          },
        ];

      case "get_jobs":
        return [
          {
            id: "1",
            prompt_id: "1",
            cron_expression: "0 9 * * *",
            status: "active",
            created_at: new Date().toISOString(),
          },
        ];

      case "get_cooldown_status":
        return {
          status: "available",
          next_available: null,
          remaining_seconds: 0,
        };

      case "get_app_info":
        return {
          version: "0.1.0",
          tauri_version: "2.0",
          build_date: new Date().toISOString(),
        };

      default:
        return {};
    }
  }
}

// ===== Prompt Manager =====
class PromptManager {
  constructor() {
    this.prompts = [];
  }

  async loadPrompts() {
    try {
      this.showLoading("prompts-list");
      this.prompts = await apiClient.invokeCommand("get_prompts");
      this.renderPrompts();
    } catch (error) {
      notificationManager.error(`載入 Prompts 失敗：${error.message}`);
    } finally {
      this.hideLoading("prompts-list");
    }
  }

  async createPrompt(promptData) {
    try {
      const newPrompt = await apiClient.invokeCommand(
        "create_prompt",
        promptData
      );
      this.prompts.push(newPrompt);
      this.renderPrompts();
      return newPrompt;
    } catch (error) {
      throw new Error(`建立 Prompt 失敗：${error.message}`);
    }
  }

  async deletePrompt(id) {
    try {
      await apiClient.invokeCommand("delete_prompt", { id });
      this.prompts = this.prompts.filter((p) => p.id !== id);
      this.renderPrompts();
      notificationManager.success("Prompt 已刪除");
    } catch (error) {
      notificationManager.error(`刪除失敗：${error.message}`);
    }
  }

  async executePrompt(id) {
    try {
      const result = await apiClient.invokeCommand("execute_prompt", { id });
      notificationManager.success("Prompt 執行成功");
      return result;
    } catch (error) {
      notificationManager.error(`執行失敗：${error.message}`);
    }
  }

  renderPrompts() {
    const container = document.getElementById("prompts-list");
    if (!container) return;

    if (this.prompts.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="material-symbols-outlined">chat</span>
          <h3>尚無 Prompts</h3>
          <p>建立您的第一個 Prompt 開始使用</p>
          <button class="btn btn-primary" onclick="modalManager.open('prompt-modal')">
            <span class="material-symbols-outlined">add</span>
            建立 Prompt
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = this.prompts
      .map(
        (prompt) => `
      <div class="card" data-prompt-id="${prompt.id}">
        <div class="card-header">
          <span class="material-symbols-outlined">chat</span>
          <h3>${prompt.title}</h3>
        </div>
        <div class="card-content">
          <p>${prompt.content}</p>
          ${
            prompt.tags.length > 0
              ? `
            <div class="tags">
              ${prompt.tags
                .map((tag) => `<span class="tag">${tag}</span>`)
                .join("")}
            </div>
          `
              : ""
          }
        </div>
        <div class="card-footer">
          <span class="text-secondary">${this.formatDate(
            prompt.created_at
          )}</span>
          <div class="card-actions">
            <button class="btn btn-primary btn-sm" onclick="promptManager.executePrompt('${
              prompt.id
            }')">
              <span class="material-symbols-outlined">play_arrow</span>
              執行
            </button>
            <button class="btn btn-secondary btn-sm" onclick="promptManager.deletePrompt('${
              prompt.id
            }')">
              <span class="material-symbols-outlined">delete</span>
              刪除
            </button>
          </div>
        </div>
      </div>
    `
      )
      .join("");
  }

  formatDate(dateString) {
    return new Date(dateString).toLocaleDateString("zh-TW", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  showLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="loading-skeleton">
          <div class="skeleton-card"></div>
          <div class="skeleton-card"></div>
          <div class="skeleton-card"></div>
        </div>
      `;
    }
  }

  hideLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== Job Manager =====
class JobManager {
  constructor() {
    this.jobs = [];
  }

  async loadJobs() {
    try {
      this.showLoading("jobs-list");
      this.jobs = await apiClient.invokeCommand("get_jobs");
      this.renderJobs();
      await this.populatePromptSelect();
    } catch (error) {
      notificationManager.error(`載入任務失敗：${error.message}`);
    } finally {
      this.hideLoading("jobs-list");
    }
  }

  async createJob(jobData) {
    try {
      const newJob = await apiClient.invokeCommand("create_job", jobData);
      this.jobs.push(newJob);
      this.renderJobs();
      return newJob;
    } catch (error) {
      throw new Error(`建立任務失敗：${error.message}`);
    }
  }

  async deleteJob(id) {
    try {
      await apiClient.invokeCommand("delete_job", { id });
      this.jobs = this.jobs.filter((j) => j.id !== id);
      this.renderJobs();
      notificationManager.success("任務已刪除");
    } catch (error) {
      notificationManager.error(`刪除失敗：${error.message}`);
    }
  }

  async populatePromptSelect() {
    const select = document.getElementById("job-prompt");
    if (!select) return;

    const prompts = await apiClient.invokeCommand("get_prompts");
    select.innerHTML = `
      <option value="">請選擇 Prompt</option>
      ${prompts
        .map(
          (prompt) => `
        <option value="${prompt.id}">${prompt.title}</option>
      `
        )
        .join("")}
    `;
  }

  renderJobs() {
    const container = document.getElementById("jobs-list");
    if (!container) return;

    if (this.jobs.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="material-symbols-outlined">schedule</span>
          <h3>尚無排程任務</h3>
          <p>建立排程任務實現自動化執行</p>
          <button class="btn btn-primary" onclick="modalManager.open('job-modal')">
            <span class="material-symbols-outlined">add_task</span>
            建立任務
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = this.jobs
      .map(
        (job) => `
      <div class="job-item" data-job-id="${job.id}">
        <div class="job-header">
          <div class="job-info">
            <h4>${this.getPromptTitle(job.prompt_id)}</h4>
            <p class="cron-expression">
              <span class="material-symbols-outlined">schedule</span>
              ${job.cron_expression}
            </p>
          </div>
          <div class="job-status status-${job.status}">
            ${job.status}
          </div>
        </div>
        <div class="job-actions">
          <button class="btn btn-secondary btn-sm" onclick="jobManager.deleteJob('${
            job.id
          }')">
            <span class="material-symbols-outlined">delete</span>
            刪除
          </button>
        </div>
      </div>
    `
      )
      .join("");
  }

  getPromptTitle(promptId) {
    const prompt = promptManager.prompts.find((p) => p.id === promptId);
    return prompt ? prompt.title : "未知 Prompt";
  }

  showLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="loading-skeleton">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
      `;
    }
  }

  hideLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== Result Manager =====
class ResultManager {
  constructor() {
    this.results = [];
  }

  async loadResults() {
    try {
      this.showLoading("results-list");
      this.results = await apiClient.invokeCommand("get_results");
      this.renderResults();
    } catch (error) {
      notificationManager.error(`載入結果失敗：${error.message}`);
    } finally {
      this.hideLoading("results-list");
    }
  }

  renderResults() {
    const container = document.getElementById("results-list");
    if (!container) return;

    if (this.results.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="material-symbols-outlined">analytics</span>
          <h3>尚無執行結果</h3>
          <p>執行 Prompts 後結果將顯示在這裡</p>
        </div>
      `;
      return;
    }

    container.innerHTML = this.results
      .map(
        (result) => `
      <div class="result-item" data-result-id="${result.id}">
        <div class="result-header">
          <h4>${result.prompt_title}</h4>
          <span class="status status-${result.status}">${result.status}</span>
        </div>
        <div class="result-content">
          <pre>${result.output}</pre>
        </div>
        <div class="result-footer">
          <span class="text-secondary">${this.formatDate(
            result.created_at
          )}</span>
        </div>
      </div>
    `
      )
      .join("");
  }

  formatDate(dateString) {
    return new Date(dateString).toLocaleDateString("zh-TW", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  showLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="loading-skeleton">
          <div class="skeleton-row"></div>
          <div class="skeleton-row"></div>
        </div>
      `;
    }
  }

  hideLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== System Manager =====
class SystemManager {
  constructor() {
    this.systemInfo = {};
  }

  async loadSystemInfo() {
    try {
      this.showLoading("app-info");
      this.showLoading("performance-info");

      const appInfo = await apiClient.invokeCommand("get_app_info");
      const performanceInfo = await apiClient.invokeCommand(
        "get_performance_info"
      );

      this.renderAppInfo(appInfo);
      this.renderPerformanceInfo(performanceInfo);
    } catch (error) {
      notificationManager.error(`載入系統資訊失敗：${error.message}`);
    }
  }

  renderAppInfo(info) {
    const container = document.getElementById("app-info");
    if (!container) return;

    container.innerHTML = `
      <div class="info-item">
        <label>版本</label>
        <span>${info.version || "0.1.0"}</span>
      </div>
      <div class="info-item">
        <label>Tauri 版本</label>
        <span>${info.tauri_version || "2.0"}</span>
      </div>
      <div class="info-item">
        <label>建置日期</label>
        <span>${this.formatDate(
          info.build_date || new Date().toISOString()
        )}</span>
      </div>
    `;
  }

  renderPerformanceInfo(info) {
    const container = document.getElementById("performance-info");
    if (!container) return;

    container.innerHTML = `
      <div class="info-item">
        <label>記憶體使用</label>
        <span>${info.memory_usage || "未知"}</span>
      </div>
      <div class="info-item">
        <label>CPU 使用率</label>
        <span>${info.cpu_usage || "未知"}</span>
      </div>
      <div class="info-item">
        <label>執行時間</label>
        <span>${info.uptime || "未知"}</span>
      </div>
    `;
  }

  formatDate(dateString) {
    return new Date(dateString).toLocaleDateString("zh-TW", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  showLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="loading-skeleton">
          <div class="skeleton-text"></div>
          <div class="skeleton-text"></div>
        </div>
      `;
    }
  }
}

// ===== Cooldown Status Manager =====
class CooldownManager {
  constructor() {
    this.statusElement = document.getElementById("cooldown-status");
    this.statusInterval = null;
  }

  async init() {
    await this.updateStatus();
    this.startPolling();
  }

  async updateStatus() {
    try {
      const status = await apiClient.invokeCommand("get_cooldown_status");
      this.renderStatus(status);
    } catch (error) {
      console.error("Failed to update cooldown status:", error);
      this.renderStatus({ status: "error", message: "檢查失敗" });
    }
  }

  renderStatus(status) {
    const statusText = this.statusElement?.querySelector(".status-text");
    const statusIcon = this.statusElement?.querySelector(".status-icon");

    if (!statusText || !statusIcon) return;

    // Remove all status classes
    this.statusElement.classList.remove("ready", "error", "cooldown");

    switch (status.status) {
      case "available":
        statusText.textContent = "API 可用";
        statusIcon.textContent = "check_circle";
        this.statusElement.classList.add("ready");
        break;

      case "cooldown":
        statusText.textContent = `冷卻中 (${status.remaining_seconds}s)`;
        statusIcon.textContent = "schedule";
        this.statusElement.classList.add("cooldown");
        break;

      case "error":
        statusText.textContent = status.message || "檢查失敗";
        statusIcon.textContent = "error";
        this.statusElement.classList.add("error");
        break;

      default:
        statusText.textContent = "檢查中...";
        statusIcon.textContent = "schedule";
        break;
    }
  }

  startPolling() {
    this.statusInterval = setInterval(() => {
      this.updateStatus();
    }, 5000); // Update every 5 seconds
  }

  stopPolling() {
    if (this.statusInterval) {
      clearInterval(this.statusInterval);
      this.statusInterval = null;
    }
  }
}

// ===== App Initialization =====
class AppInitializer {
  constructor() {
    this.loadingOverlay = document.getElementById("app-loader");
    this.appContainer = document.getElementById("app");
  }

  async init() {
    try {
      // Simulate loading process
      await this.simulateLoading();

      // Initialize managers
      await this.initializeManagers();

      // Hide loading overlay and show app
      this.showApp();

      // Load initial data
      await this.loadInitialData();
    } catch (error) {
      console.error("App initialization failed:", error);
      notificationManager.error("應用程式初始化失敗");
    }
  }

  async simulateLoading() {
    // Simulate loading steps
    const steps = [
      "載入系統元件...",
      "初始化資料庫連接...",
      "檢查 CLI 整合狀態...",
      "載入使用者設定...",
      "準備使用者介面...",
    ];

    for (let i = 0; i < steps.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 400));
      // Could update loading text here if needed
    }
  }

  async initializeManagers() {
    // Initialize all managers
    window.themeManager = new ThemeManager();
    window.notificationManager = new NotificationManager();
    window.tabManager = new TabManager();
    window.modalManager = new ModalManager();
    window.apiClient = new APIClient();
    window.promptManager = new PromptManager();
    window.jobManager = new JobManager();
    window.resultManager = new ResultManager();
    window.systemManager = new SystemManager();
    window.cooldownManager = new CooldownManager();

    // Initialize cooldown status polling
    await cooldownManager.init();
  }

  showApp() {
    this.loadingOverlay.style.display = "none";
    this.appContainer.style.display = "flex";
  }

  async loadInitialData() {
    // Load data for the current tab
    const currentTab = appState.currentTab;
    await tabManager.loadTabContent(currentTab);
  }
}

// ===== Global State and Initialization =====
const appState = new AppState();

// Initialize app when DOM is loaded
document.addEventListener("DOMContentLoaded", async () => {
  const appInitializer = new AppInitializer();
  await appInitializer.init();
});

// Handle app cleanup
window.addEventListener("beforeunload", () => {
  if (window.cooldownManager) {
    cooldownManager.stopPolling();
  }
});

// Global error handler
window.addEventListener("error", (event) => {
  console.error("Global error:", event.error);
  if (window.notificationManager) {
    notificationManager.error("發生未預期的錯誤");
  }
});

// Expose managers globally for debugging
window.appState = appState;
