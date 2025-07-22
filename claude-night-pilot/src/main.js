/**
 * Claude Night Pilot - Material Design 3.0 JavaScript
 * 現代化的夜間自動打工仔前端應用程式
 * 基於 Material Design 3.0 設計系統
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

// ===== Material Design Theme Management =====
class MaterialThemeManager {
  constructor() {
    this.init();
  }

  init() {
    this.applyTheme(appState.theme);
    this.bindEvents();
    this.setupSystemThemeDetection();
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

    const iconMap = {
      dark: "dark_mode",
      light: "light_mode",
      auto: "brightness_auto",
    };

    icon.textContent = iconMap[theme] || iconMap.auto;

    // Add ripple effect
    this.addRippleEffect(themeToggle);
  }

  addRippleEffect(element) {
    if (!element) return;

    element.addEventListener("click", (e) => {
      const ripple = document.createElement("span");
      const rect = element.getBoundingClientRect();
      const size = Math.max(rect.width, rect.height);
      const x = e.clientX - rect.left - size / 2;
      const y = e.clientY - rect.top - size / 2;

      ripple.style.cssText = `
        position: absolute;
        width: ${size}px;
        height: ${size}px;
        left: ${x}px;
        top: ${y}px;
        background: radial-gradient(circle, rgba(255,255,255,0.6) 0%, transparent 70%);
        border-radius: 50%;
        pointer-events: none;
        animation: ripple 0.6s ease-out;
      `;

      element.style.position = "relative";
      element.style.overflow = "hidden";
      element.appendChild(ripple);

      setTimeout(() => ripple.remove(), 600);
    });
  }

  setupSystemThemeDetection() {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaQuery.addEventListener("change", () => {
      if (appState.theme === "auto") {
        this.applyTheme("auto");
      }
    });
  }

  toggleTheme() {
    const themes = ["auto", "light", "dark"];
    const currentIndex = themes.indexOf(appState.theme);
    const nextTheme = themes[(currentIndex + 1) % themes.length];
    this.applyTheme(nextTheme);

    // Show feedback
    if (window.snackbarManager) {
      const themeNames = {
        auto: "自動主題",
        light: "淺色主題",
        dark: "深色主題",
      };
      snackbarManager.show(`已切換至${themeNames[nextTheme]}`, "info", 2000);
    }
  }

  bindEvents() {
    const themeToggle = document.getElementById("theme-toggle");
    themeToggle?.addEventListener("click", () => this.toggleTheme());
  }
}

// ===== Material Design Snackbar System =====
class MaterialSnackbarManager {
  constructor() {
    this.container = document.getElementById("snackbar-container");
    this.snackbars = new Map();
    this.queue = [];
    this.maxVisible = 3;
  }

  show(message, type = "info", duration = 5000) {
    const id = `snackbar-${Date.now()}`;

    if (this.snackbars.size >= this.maxVisible) {
      this.queue.push({ id, message, type, duration });
      return id;
    }

    const snackbar = this.createSnackbar(id, message, type);
    this.container.appendChild(snackbar);
    this.snackbars.set(id, snackbar);

    // Auto remove
    if (duration > 0) {
      setTimeout(() => this.remove(id), duration);
    }

    return id;
  }

  createSnackbar(id, message, type) {
    const snackbar = document.createElement("div");
    snackbar.className = `md-snackbar ${type}`;
    snackbar.setAttribute("data-snackbar-id", id);

    const icons = {
      success: "check_circle",
      error: "error",
      warning: "warning",
      info: "info",
    };

    snackbar.innerHTML = `
      <span class="material-symbols-outlined snackbar-icon">${
        icons[type] || icons.info
      }</span>
      <div class="snackbar-content">
        <div class="snackbar-message">${message}</div>
      </div>
      <button class="md-icon-button" onclick="snackbarManager.remove('${id}')">
        <span class="material-symbols-outlined">close</span>
      </button>
    `;

    return snackbar;
  }

  remove(id) {
    const snackbar = this.snackbars.get(id);
    if (snackbar) {
      snackbar.style.animation = "snackbarSlideOut 0.3s ease-in forwards";
      setTimeout(() => {
        snackbar.remove();
        this.snackbars.delete(id);
        this.processQueue();
      }, 300);
    }
  }

  processQueue() {
    if (this.queue.length > 0 && this.snackbars.size < this.maxVisible) {
      const next = this.queue.shift();
      this.show(next.message, next.type, next.duration);
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

// ===== Material Design Navigation Rail Management =====
class MaterialNavigationManager {
  constructor() {
    this.railItems = document.querySelectorAll(".md-navigation-rail-item");
    this.panels = document.querySelectorAll(".md-tab-panel");
    this.init();
  }

  init() {
    this.railItems.forEach((item) => {
      item.addEventListener("click", (e) => {
        const targetTab = e.currentTarget.dataset.tab;
        this.switchTab(targetTab);
      });

      // Add ripple effect
      themeManager.addRippleEffect(item);
    });
  }

  switchTab(tabName) {
    // Update navigation rail items
    this.railItems.forEach((item) => {
      if (item.dataset.tab === tabName) {
        item.classList.add("active");
        item.setAttribute("aria-selected", "true");
      } else {
        item.classList.remove("active");
        item.setAttribute("aria-selected", "false");
      }
    });

    // Update tab panels with Material Motion
    this.panels.forEach((panel) => {
      if (panel.id === `${tabName}-tab`) {
        panel.classList.add("active");
        panel.setAttribute("aria-hidden", "false");
      } else {
        panel.classList.remove("active");
        panel.setAttribute("aria-hidden", "true");
      }
    });

    appState.setState("currentTab", tabName);

    // Load tab content if needed
    this.loadTabContent(tabName);
  }

  async loadTabContent(tabName) {
    try {
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
    } catch (error) {
      console.error(`Failed to load ${tabName} content:`, error);
      snackbarManager.error(`載入 ${tabName} 內容失敗`);
    }
  }
}

// ===== Material Design Modal Management =====
class MaterialModalManager {
  constructor() {
    this.modals = new Map();
    this.init();
  }

  init() {
    // Setup FAB triggers
    document
      .getElementById("create-prompt-fab")
      ?.addEventListener("click", () => {
        this.open("prompt-modal");
      });

    document.getElementById("create-job-fab")?.addEventListener("click", () => {
      this.open("job-modal");
    });

    // Setup close handlers
    document.querySelectorAll(".close-btn, [data-close]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const modal = e.target.closest(".md-dialog");
        if (modal) this.close(modal.id);
      });
    });

    // Close on backdrop click
    document.querySelectorAll(".md-dialog").forEach((modal) => {
      modal.addEventListener("click", (e) => {
        if (e.target === modal) {
          this.close(modal.id);
        }
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
      document.body.style.overflow = "hidden";

      // Focus first input with Material Design focus behavior
      const firstInput = modal.querySelector("input, textarea, select");
      if (firstInput) {
        setTimeout(() => {
          firstInput.focus();
          firstInput.parentElement?.classList.add("md-focused");
        }, 150);
      }
    }
  }

  close(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
      modal.close();
      this.modals.delete(modalId);
      document.body.style.overflow = "";
      this.resetForm(modal);
    }
  }

  resetForm(modal) {
    const form = modal.querySelector("form");
    if (form) {
      form.reset();
      // Reset Material Design field states
      form.querySelectorAll(".md-form-group").forEach((group) => {
        group.classList.remove("md-focused", "md-filled");
      });
    }
  }

  setupForms() {
    // Setup Material Design text field behavior
    document.querySelectorAll(".md-text-field").forEach((field) => {
      const group = field.closest(".md-form-group");

      field.addEventListener("focus", () => {
        group?.classList.add("md-focused");
      });

      field.addEventListener("blur", () => {
        group?.classList.remove("md-focused");
        if (field.value.trim()) {
          group?.classList.add("md-filled");
        } else {
          group?.classList.remove("md-filled");
        }
      });

      // Initial state
      if (field.value.trim()) {
        group?.classList.add("md-filled");
      }
    });

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
      snackbarManager.success("Prompt 建立成功！");
    } catch (error) {
      snackbarManager.error(`建立失敗：${error.message}`);
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
      snackbarManager.success("排程任務建立成功！");
    } catch (error) {
      snackbarManager.error(`建立失敗：${error.message}`);
    }
  }
}

// ===== Enhanced API Client =====
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
    // Mock responses for development with more realistic data
    switch (command) {
      case "get_prompts":
        return [
          {
            id: "1",
            title: "架構分析 Prompt",
            content:
              "@README.md @src/ 請分析這個專案的整體架構，包括前端、後端和資料庫設計，並提供改進建議。",
            tags: ["architecture", "analysis", "code-review"],
            created_at: new Date(Date.now() - 86400000).toISOString(),
          },
          {
            id: "2",
            title: "程式碼品質檢查",
            content:
              "@src/**/*.js @src/**/*.ts 檢查程式碼品質，找出潛在的bug和效能問題。",
            tags: ["quality", "performance", "debugging"],
            created_at: new Date(Date.now() - 172800000).toISOString(),
          },
          {
            id: "3",
            title: "文檔生成助手",
            content: "根據程式碼自動生成API文檔和使用說明。",
            tags: ["documentation", "api", "automation"],
            created_at: new Date(Date.now() - 259200000).toISOString(),
          },
        ];

      case "get_jobs":
        return [
          {
            id: "1",
            prompt_id: "1",
            prompt_title: "架構分析 Prompt",
            cron_expression: "0 9 * * *",
            status: "active",
            next_run: new Date(Date.now() + 86400000).toISOString(),
            created_at: new Date(Date.now() - 86400000).toISOString(),
          },
          {
            id: "2",
            prompt_id: "2",
            prompt_title: "程式碼品質檢查",
            cron_expression: "0 12,18 * * *",
            status: "paused",
            next_run: null,
            created_at: new Date(Date.now() - 172800000).toISOString(),
          },
        ];

      case "get_results":
        return [
          {
            id: "1",
            prompt_id: "1",
            prompt_title: "架構分析 Prompt",
            status: "success",
            output:
              "專案架構分析完成。\n\n✅ 前端使用 Material Design 3.0\n✅ 後端採用 Rust + Tauri\n✅ 資料庫使用 SQLite\n\n建議改進：\n- 加強錯誤處理機制\n- 增加單元測試覆蓋率\n- 優化載入效能",
            execution_time: 2340,
            created_at: new Date(Date.now() - 3600000).toISOString(),
          },
          {
            id: "2",
            prompt_id: "2",
            prompt_title: "程式碼品質檢查",
            status: "error",
            output:
              "執行過程中發生錯誤：\n\nError: Connection timeout\n請檢查網路連接或 Claude API 配置。",
            execution_time: 5000,
            created_at: new Date(Date.now() - 7200000).toISOString(),
          },
        ];

      case "get_cooldown_status":
        const random = Math.random();
        if (random < 0.3) {
          return {
            status: "cooldown",
            next_available: new Date(Date.now() + 45000).toISOString(),
            remaining_seconds: 45,
          };
        } else if (random < 0.1) {
          return {
            status: "error",
            message: "API 連接失敗",
            next_available: null,
            remaining_seconds: 0,
          };
        } else {
          return {
            status: "available",
            next_available: null,
            remaining_seconds: 0,
          };
        }

      case "get_app_info":
        return {
          version: "0.2.0",
          tauri_version: "2.0.0",
          build_date: new Date().toISOString(),
          platform: navigator.platform,
          user_agent: navigator.userAgent,
        };

      case "get_performance_info":
        return {
          memory_usage: `${Math.floor(Math.random() * 50 + 30)}MB`,
          cpu_usage: `${Math.floor(Math.random() * 15 + 5)}%`,
          uptime: `${Math.floor(Math.random() * 24 + 1)} 小時`,
          prompts_executed: Math.floor(Math.random() * 100 + 50),
          success_rate: `${Math.floor(Math.random() * 10 + 90)}%`,
        };

      default:
        return {};
    }
  }
}

// ===== Enhanced Prompt Manager =====
class PromptManager {
  constructor() {
    this.prompts = [];
  }

  async loadPrompts() {
    try {
      this.showMaterialLoading("prompts-list");
      this.prompts = await apiClient.invokeCommand("get_prompts");
      this.renderPrompts();
    } catch (error) {
      snackbarManager.error(`載入 Prompts 失敗：${error.message}`);
    } finally {
      this.hideMaterialLoading("prompts-list");
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
      snackbarManager.success("Prompt 已刪除");
    } catch (error) {
      snackbarManager.error(`刪除失敗：${error.message}`);
    }
  }

  async executePrompt(id) {
    try {
      snackbarManager.info("正在執行 Prompt...");
      const result = await apiClient.invokeCommand("execute_prompt", { id });
      snackbarManager.success("Prompt 執行成功");
      return result;
    } catch (error) {
      snackbarManager.error(`執行失敗：${error.message}`);
    }
  }

  renderPrompts() {
    const container = document.getElementById("prompts-list");
    if (!container) return;

    if (this.prompts.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="material-symbols-outlined">chat</span>
          <h3 class="md-typescale-headline-small">尚無 Prompts</h3>
          <p class="md-typescale-body-medium">建立您的第一個 Prompt 開始使用</p>
          <button class="md-filled-button" onclick="modalManager.open('prompt-modal')">
            <span class="material-symbols-outlined">add</span>
            <span>建立 Prompt</span>
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = this.prompts
      .map(
        (prompt) => `
      <div class="md-card md-elevation-level1" data-prompt-id="${prompt.id}">
        <div class="md-card-header">
          <span class="material-symbols-outlined">chat</span>
          <h3>${prompt.title}</h3>
        </div>
        <div class="md-card-content">
          <p class="md-typescale-body-medium">${this.truncateText(
            prompt.content,
            150
          )}</p>
          ${
            prompt.tags.length > 0
              ? `
            <div class="md-chip-set" style="margin-top: 16px;">
              ${prompt.tags
                .map((tag) => `<span class="md-assist-chip">${tag}</span>`)
                .join("")}
            </div>
          `
              : ""
          }
          <div class="md-card-footer" style="margin-top: 24px; display: flex; justify-content: space-between; align-items: center;">
            <span class="md-typescale-body-small" style="color: var(--md-sys-color-on-surface-variant);">
              ${this.formatDate(prompt.created_at)}
            </span>
            <div style="display: flex; gap: 8px;">
              <button class="md-filled-button" onclick="promptManager.executePrompt('${
                prompt.id
              }')">
                <span class="material-symbols-outlined">play_arrow</span>
                <span>執行</span>
              </button>
              <button class="md-text-button" onclick="promptManager.deletePrompt('${
                prompt.id
              }')">
                <span class="material-symbols-outlined">delete</span>
                <span>刪除</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    `
      )
      .join("");
  }

  truncateText(text, maxLength) {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
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

  showMaterialLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="md-loading-skeleton">
          <div class="md-skeleton-card"></div>
          <div class="md-skeleton-card"></div>
          <div class="md-skeleton-card"></div>
        </div>
      `;
    }
  }

  hideMaterialLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== Enhanced Job Manager =====
class JobManager {
  constructor() {
    this.jobs = [];
  }

  async loadJobs() {
    try {
      this.showMaterialLoading("jobs-list");
      this.jobs = await apiClient.invokeCommand("get_jobs");
      this.renderJobs();
      await this.populatePromptSelect();
    } catch (error) {
      snackbarManager.error(`載入任務失敗：${error.message}`);
    } finally {
      this.hideMaterialLoading("jobs-list");
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
      snackbarManager.success("任務已刪除");
    } catch (error) {
      snackbarManager.error(`刪除失敗：${error.message}`);
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
          <h3 class="md-typescale-headline-small">尚無排程任務</h3>
          <p class="md-typescale-body-medium">建立排程任務實現自動化執行</p>
          <button class="md-filled-button" onclick="modalManager.open('job-modal')">
            <span class="material-symbols-outlined">add_task</span>
            <span>建立任務</span>
          </button>
        </div>
      `;
      return;
    }

    container.innerHTML = this.jobs
      .map(
        (job) => `
      <div class="md-list-item" data-job-id="${job.id}">
        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
          <div style="flex: 1;">
            <h4 class="md-typescale-title-medium" style="margin: 0 0 8px;">${
              job.prompt_title || this.getPromptTitle(job.prompt_id)
            }</h4>
            <p style="display: flex; align-items: center; gap: 8px; margin: 0; font-family: 'Roboto Mono', monospace; font-size: 14px; color: var(--md-sys-color-on-surface-variant);">
              <span class="material-symbols-outlined" style="font-size: 16px;">schedule</span>
              ${job.cron_expression}
            </p>
            ${
              job.next_run
                ? `
              <p style="margin: 8px 0 0; font-size: 12px; color: var(--md-sys-color-on-surface-variant);">
                下次執行：${this.formatDate(job.next_run)}
              </p>
            `
                : ""
            }
          </div>
          <div style="display: flex; align-items: center; gap: 12px;">
            <span class="md-status-chip ${this.getStatusClass(job.status)}">
              ${this.getStatusText(job.status)}
            </span>
            <button class="md-icon-button" onclick="jobManager.deleteJob('${
              job.id
            }')" title="刪除任務">
              <span class="material-symbols-outlined">delete</span>
            </button>
          </div>
        </div>
      </div>
    `
      )
      .join("");
  }

  getStatusClass(status) {
    const classes = {
      active: "status-active",
      paused: "status-paused",
      error: "status-error",
    };
    return classes[status] || "";
  }

  getStatusText(status) {
    const texts = {
      active: "運行中",
      paused: "已暫停",
      error: "錯誤",
    };
    return texts[status] || status;
  }

  getPromptTitle(promptId) {
    const prompt = promptManager?.prompts?.find((p) => p.id === promptId);
    return prompt ? prompt.title : "未知 Prompt";
  }

  formatDate(dateString) {
    return new Date(dateString).toLocaleDateString("zh-TW", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  showMaterialLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="md-loading-skeleton">
          <div class="md-skeleton-list-item"></div>
          <div class="md-skeleton-list-item"></div>
          <div class="md-skeleton-list-item"></div>
        </div>
      `;
    }
  }

  hideMaterialLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== Enhanced Result Manager =====
class ResultManager {
  constructor() {
    this.results = [];
  }

  async loadResults() {
    try {
      this.showMaterialLoading("results-list");
      this.results = await apiClient.invokeCommand("get_results");
      this.renderResults();
    } catch (error) {
      snackbarManager.error(`載入結果失敗：${error.message}`);
    } finally {
      this.hideMaterialLoading("results-list");
    }
  }

  renderResults() {
    const container = document.getElementById("results-list");
    if (!container) return;

    if (this.results.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <span class="material-symbols-outlined">analytics</span>
          <h3 class="md-typescale-headline-small">尚無執行結果</h3>
          <p class="md-typescale-body-medium">執行 Prompts 後結果將顯示在這裡</p>
        </div>
      `;
      return;
    }

    container.innerHTML = this.results
      .map(
        (result) => `
      <div class="md-list-item" data-result-id="${result.id}">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px;">
          <h4 class="md-typescale-title-medium" style="margin: 0;">${
            result.prompt_title
          }</h4>
          <span class="md-status-chip ${this.getStatusClass(result.status)}">
            <span class="material-symbols-outlined">${this.getStatusIcon(
              result.status
            )}</span>
            <span>${this.getStatusText(result.status)}</span>
          </span>
        </div>
        <div style="background: var(--md-sys-color-surface-variant); border-radius: 12px; padding: 16px; margin-bottom: 16px;">
          <pre style="margin: 0; white-space: pre-wrap; font-family: 'Roboto Mono', monospace; font-size: 14px; line-height: 1.5;">${
            result.output
          }</pre>
        </div>
        <div style="display: flex; justify-content: space-between; align-items: center; font-size: 12px; color: var(--md-sys-color-on-surface-variant);">
          <span>${this.formatDate(result.created_at)}</span>
          <span>執行時間：${result.execution_time}ms</span>
        </div>
      </div>
    `
      )
      .join("");
  }

  getStatusClass(status) {
    const classes = {
      success: "status-success",
      error: "status-error",
      pending: "status-pending",
    };
    return classes[status] || "";
  }

  getStatusIcon(status) {
    const icons = {
      success: "check_circle",
      error: "error",
      pending: "schedule",
    };
    return icons[status] || "info";
  }

  getStatusText(status) {
    const texts = {
      success: "成功",
      error: "錯誤",
      pending: "進行中",
    };
    return texts[status] || status;
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

  showMaterialLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="md-loading-skeleton">
          <div class="md-skeleton-list-item"></div>
          <div class="md-skeleton-list-item"></div>
        </div>
      `;
    }
  }

  hideMaterialLoading(containerId) {
    // Loading will be replaced by actual content
  }
}

// ===== Enhanced System Manager =====
class SystemManager {
  constructor() {
    this.systemInfo = {};
  }

  async loadSystemInfo() {
    try {
      this.showMaterialLoading("app-info");
      this.showMaterialLoading("performance-info");

      const appInfo = await apiClient.invokeCommand("get_app_info");
      const performanceInfo = await apiClient.invokeCommand(
        "get_performance_info"
      );

      this.renderAppInfo(appInfo);
      this.renderPerformanceInfo(performanceInfo);
    } catch (error) {
      snackbarManager.error(`載入系統資訊失敗：${error.message}`);
    }
  }

  renderAppInfo(info) {
    const container = document.getElementById("app-info");
    if (!container) return;

    container.innerHTML = `
      <div class="info-item">
        <label class="md-typescale-label-medium">版本</label>
        <span class="md-typescale-body-medium">${info.version || "0.2.0"}</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">Tauri 版本</label>
        <span class="md-typescale-body-medium">${
          info.tauri_version || "2.0.0"
        }</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">平台</label>
        <span class="md-typescale-body-medium">${info.platform || "未知"}</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">建置日期</label>
        <span class="md-typescale-body-medium">${this.formatDate(
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
        <label class="md-typescale-label-medium">記憶體使用</label>
        <span class="md-typescale-body-medium">${
          info.memory_usage || "未知"
        }</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">CPU 使用率</label>
        <span class="md-typescale-body-medium">${
          info.cpu_usage || "未知"
        }</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">執行時間</label>
        <span class="md-typescale-body-medium">${info.uptime || "未知"}</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">已執行 Prompts</label>
        <span class="md-typescale-body-medium">${
          info.prompts_executed || "0"
        }</span>
      </div>
      <div class="info-item">
        <label class="md-typescale-label-medium">成功率</label>
        <span class="md-typescale-body-medium">${
          info.success_rate || "未知"
        }</span>
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

  showMaterialLoading(containerId) {
    const container = document.getElementById(containerId);
    if (container) {
      container.innerHTML = `
        <div class="md-loading-skeleton">
          <div class="md-skeleton-text"></div>
          <div class="md-skeleton-text"></div>
          <div class="md-skeleton-text"></div>
        </div>
      `;
    }
  }
}

// ===== Enhanced Cooldown Status Manager =====
class CooldownManager {
  constructor() {
    this.countdownInterval = null;
    this.resetTime = null;
    this.statusCheckInterval = null;
  }

  async init() {
    // Initialize cooldown status polling
    console.log("Initializing CooldownManager...");
    await this.checkCooldownStatus();
    
    // Set up periodic status checks (every 30 seconds)
    this.statusCheckInterval = setInterval(() => {
      this.checkCooldownStatus();
    }, 30000);
    
    console.log("CooldownManager initialized successfully");
  }

  async checkCooldownStatus() {
    try {
      // Assuming showLoadingState and hideLoadingState are defined elsewhere or will be added.
      // For now, we'll just log the status.
      console.log("Checking Claude CLI cooldown status...");

      const response = await apiClient.invokeCommand("get_cooldown_status");

      if (response.is_available) {
        this.displayAvailableStatus(response);
      } else {
        this.displayCooldownStatus(response);
      }

      // Assuming hideLoadingState is defined elsewhere or will be added.
      // For now, we'll just log the status.
      console.log("Cooldown status checked.");
    } catch (error) {
      console.error("冷卻狀態檢查失敗:", error);
      this.displayErrorStatus(error);
      // Assuming hideLoadingState is defined elsewhere or will be added.
      // For now, we'll just log the status.
      console.log("Cooldown status check failed.");
    }
  }

  displayAvailableStatus(response) {
    const statusElement = document.getElementById("cooldown-status");
    if (statusElement) {
      statusElement.innerHTML = `
        <div class="status-card available">
          <span class="material-symbols-rounded status-icon">check_circle</span>
          <div class="status-info">
            <h3>Claude API 可用</h3>
            <p>最後檢查: ${new Date().toLocaleString("zh-TW")}</p>
            <div class="version-info">
              版本: ${response.version || "Claude CLI 1.0.57"}
            </div>
          </div>
        </div>
      `;
    }
  }

  displayCooldownStatus(response) {
    const statusElement = document.getElementById("cooldown-status");
    if (!statusElement) return;

    if (response.reset_time) {
      this.resetTime = new Date(response.reset_time);
      this.startCountdown(statusElement);
    } else {
      statusElement.innerHTML = `
        <div class="status-card cooldown">
          <span class="material-symbols-rounded status-icon">schedule</span>
          <div class="status-info">
            <h3>Claude API 使用限制</h3>
            <p>API 已達到使用限制，請稍後再試</p>
            <div class="suggestion">
              <span class="material-symbols-rounded">lightbulb</span> 建議稍後再次檢查
            </div>
          </div>
        </div>
      `;
    }
  }

  startCountdown(statusElement) {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
    }

    const updateCountdown = () => {
      const now = new Date();
      const difference = this.resetTime - now;

      if (difference <= 0) {
        // 冷卻時間已過
        statusElement.innerHTML = `
          <div class="status-card ready">
            <span class="material-symbols-rounded status-icon">check_circle</span>
            <div class="status-info">
              <h3>冷卻時間已過</h3>
              <p>可以重新嘗試使用 Claude API</p>
              <button onclick="cooldownManager.checkCooldownStatus()" class="btn-primary">
                <span class="material-symbols-rounded">refresh</span>
                重新檢查
              </button>
            </div>
          </div>
        `;

        if (this.countdownInterval) {
          clearInterval(this.countdownInterval);
          this.countdownInterval = null;
        }
        return;
      }

      // 計算剩餘時間
      const hours = Math.floor(difference / (1000 * 60 * 60));
      const minutes = Math.floor((difference % (1000 * 60 * 60)) / (1000 * 60));
      const seconds = Math.floor((difference % (1000 * 60)) / 1000);

      const resetTimeStr = this.resetTime.toLocaleString("zh-TW");

      let timeDisplay;
      let suggestion;

      if (hours > 0) {
        timeDisplay = `${hours}小時 ${minutes}分鐘 ${seconds}秒`;
        suggestion = `建議在 ${this.resetTime.toLocaleTimeString("zh-TW", {
          hour: "2-digit",
          minute: "2-digit",
        })} 後再次嘗試`;
      } else if (minutes > 0) {
        timeDisplay = `${minutes}分鐘 ${seconds}秒`;
        suggestion = `約 ${minutes + 1} 分鐘後恢復`;
      } else {
        timeDisplay = `${seconds}秒`;
        suggestion = "即將恢復";
      }

      statusElement.innerHTML = `
        <div class="status-card cooldown">
          <span class="material-symbols-rounded status-icon">timer</span>
          <div class="status-info">
            <h3>Claude API 使用限制</h3>
            <div class="countdown-display">
              <div class="time-remaining">
                <span class="label">剩餘時間：</span>
                <span class="time">${timeDisplay}</span>
              </div>
              <div class="reset-time">
                <span class="label">預計解鎖：</span>
                <span class="time">${resetTimeStr}</span>
              </div>
            </div>
            <div class="suggestion">
              <span class="material-symbols-rounded">lightbulb</span> ${suggestion}
            </div>
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${this.calculateProgress()}%"></div>
            </div>
          </div>
        </div>
      `;
    };

    updateCountdown();
    this.countdownInterval = setInterval(updateCountdown, 1000);
  }

  calculateProgress() {
    if (!this.resetTime) return 0;

    const now = new Date();
    const total = this.resetTime - (this.resetTime - 60 * 60 * 1000); // 假設冷卻時間為 1 小時
    const remaining = this.resetTime - now;

    return Math.max(0, Math.min(100, ((total - remaining) / total) * 100));
  }

  displayErrorStatus(error) {
    const statusElement = document.getElementById("cooldown-status");
    if (statusElement) {
      statusElement.innerHTML = `
        <div class="status-card error">
          <span class="material-symbols-rounded status-icon">error</span>
          <div class="status-info">
            <h3>檢查失敗</h3>
            <p>無法檢查 Claude CLI 狀態</p>
            <div class="error-details">
              錯誤: ${error.message || error}
            </div>
            <button onclick="cooldownManager.checkCooldownStatus()" class="btn-secondary">
              <span class="material-symbols-rounded">refresh</span>
              重試
            </button>
          </div>
        </div>
      `;
    }
  }

  cleanup() {
    if (this.countdownInterval) {
      clearInterval(this.countdownInterval);
      this.countdownInterval = null;
    }
    if (this.statusCheckInterval) {
      clearInterval(this.statusCheckInterval);
      this.statusCheckInterval = null;
    }
  }
}

// ===== Enhanced App Initialization with Material Design Loading =====
class MaterialAppInitializer {
  constructor() {
    this.loadingOverlay = document.getElementById("app-loader");
    this.appContainer = document.getElementById("app");
    this.loadingSteps = document.querySelectorAll(".step-indicator");
    this.currentStep = 0;
  }

  async init() {
    try {
      // Show progressive loading steps
      await this.executeLoadingSteps();

      // Initialize managers
      await this.initializeManagers();

      // Hide loading overlay and show app
      this.showApp();

      // Load initial data
      await this.loadInitialData();
    } catch (error) {
      console.error("App initialization failed:", error);
      if (window.snackbarManager) {
        snackbarManager.error("應用程式初始化失敗");
      }
    }
  }

  async executeLoadingSteps() {
    const steps = [
      { name: "CLI 整合檢查", duration: 600 },
      { name: "資料庫初始化", duration: 800 },
      { name: "API 連接測試", duration: 700 },
      { name: "界面準備完成", duration: 500 },
    ];

    for (let i = 0; i < steps.length; i++) {
      await this.activateStep(i);
      await new Promise((resolve) => setTimeout(resolve, steps[i].duration));
    }
  }

  async activateStep(stepIndex) {
    // Deactivate all steps
    this.loadingSteps.forEach((step, index) => {
      if (index < stepIndex) {
        step.classList.remove("active");
        step.classList.add("completed");
        step.querySelector(".material-symbols-outlined").textContent = "check";
      } else if (index === stepIndex) {
        step.classList.add("active");
      } else {
        step.classList.remove("active", "completed");
      }
    });

    this.currentStep = stepIndex;
  }

  async initializeManagers() {
    // Initialize all managers with Material Design
    window.themeManager = new MaterialThemeManager();
    window.snackbarManager = new MaterialSnackbarManager();
    window.navigationManager = new MaterialNavigationManager();
    window.modalManager = new MaterialModalManager();
    window.apiClient = new APIClient();
    window.promptManager = new PromptManager();
    window.jobManager = new JobManager();
    window.resultManager = new ResultManager();
    window.systemManager = new SystemManager();
    window.cooldownManager = new CooldownManager();

    // Initialize cooldown status polling
    await cooldownManager.init();

    // Setup refresh system button
    const refreshBtn = document.getElementById("refresh-system-btn");
    refreshBtn?.addEventListener("click", () => {
      systemManager.loadSystemInfo();
      snackbarManager.info("正在刷新系統資訊...");
    });
  }

  showApp() {
    // Complete all loading steps
    this.loadingSteps.forEach((step) => {
      step.classList.remove("active");
      step.classList.add("completed");
      step.querySelector(".material-symbols-outlined").textContent = "check";
    });

    // Smooth transition to app
    setTimeout(() => {
      this.loadingOverlay.style.animation = "fadeOut 0.5s ease-out forwards";
      setTimeout(() => {
        this.loadingOverlay.style.display = "none";
        this.appContainer.style.display = "flex";
        this.appContainer.style.animation = "fadeIn 0.5s ease-out";
      }, 500);
    }, 300);
  }

  async loadInitialData() {
    // Load data for the current tab
    const currentTab = appState.currentTab;
    await navigationManager.loadTabContent(currentTab);
  }
}

// ===== Global State and Initialization =====
const appState = new AppState();

// Add CSS for fadeOut animation
const style = document.createElement("style");
style.textContent = `
  @keyframes fadeOut {
    from { opacity: 1; }
    to { opacity: 0; }
  }
  
  .info-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
  }
  
  .info-item:last-child {
    border-bottom: none;
  }
  
  .md-chip-set {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .md-assist-chip {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .empty-state {
    text-align: center;
    padding: 64px 32px;
    color: var(--md-sys-color-on-surface-variant);
  }
  
  .empty-state .material-symbols-outlined {
    font-size: 4rem;
    color: var(--md-sys-color-outline);
    margin-bottom: 24px;
  }
  
  .empty-state h3 {
    margin: 0 0 16px;
    color: var(--md-sys-color-on-surface);
  }
  
  .empty-state p {
    margin: 0 0 32px;
    max-width: 400px;
    margin-left: auto;
    margin-right: auto;
  }
  
  .status-active {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .status-paused {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .status-error {
    background: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .status-success {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  
  .status-pending {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }

  @keyframes ripple {
    to {
      transform: scale(2);
      opacity: 0;
    }
  }
`;
document.head.appendChild(style);

// Initialize app when DOM is loaded
document.addEventListener("DOMContentLoaded", async () => {
  const appInitializer = new MaterialAppInitializer();
  await appInitializer.init();
});

// Handle app cleanup
window.addEventListener("beforeunload", () => {
  if (window.cooldownManager) {
    cooldownManager.cleanup();
  }
});

// Global error handler
window.addEventListener("error", (event) => {
  console.error("Global error:", event.error);
  if (window.snackbarManager) {
    snackbarManager.error("發生未預期的錯誤");
  }
});

// Handle visibility change for performance
document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "hidden") {
    // Pause polling when app is not visible
    if (window.cooldownManager) {
      cooldownManager.cleanup(); // Changed from stopPolling to cleanup
    }
  } else {
    // Resume polling when app becomes visible
    if (window.cooldownManager) {
      cooldownManager.checkCooldownStatus(); // Changed from startPolling to checkCooldownStatus
    }
  }
});

// Expose managers globally for debugging
window.appState = appState;
