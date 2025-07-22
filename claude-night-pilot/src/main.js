import { invoke } from "@tauri-apps/api/tauri";
import { message } from "@tauri-apps/api/dialog";

// 全域狀態
let currentPrompts = [];
let currentJobs = [];
let cooldownInterval = null;
let cliIntegrated = false;

// 初始化應用
document.addEventListener("DOMContentLoaded", async () => {
  await loadPrompts();
  await loadJobs();
  await updateCooldownStatus();
  await checkSystemInfo();
  startCooldownMonitor();
  setupCliIntegration();
});

// 檢查系統資訊和 CLI 整合狀態
async function checkSystemInfo() {
  try {
    const systemInfo = await invoke("get_system_info");
    console.log("系統資訊:", systemInfo);

    cliIntegrated = systemInfo.cli_integrated || false;

    if (cliIntegrated) {
      // 顯示 CLI 整合狀態
      const statusElement = document.getElementById("cli-status");
      if (statusElement) {
        statusElement.textContent = "CLI 工具已整合";
        statusElement.className = "success";
      }

      // 添加 CLI 功能按鈕
      addCliButtons();
    }

    // 更新版本資訊
    const versionElement = document.getElementById("version-info");
    if (versionElement) {
      versionElement.textContent = `v${systemInfo.version} (${systemInfo.platform}-${systemInfo.arch})`;
    }
  } catch (error) {
    console.error("無法獲取系統資訊:", error);
  }
}

// 設置 CLI 整合功能
function setupCliIntegration() {
  // 添加鍵盤快捷鍵
  document.addEventListener("keydown", (event) => {
    // Ctrl/Cmd + K: 快速 CLI 命令
    if ((event.ctrlKey || event.metaKey) && event.key === "k") {
      event.preventDefault();
      showCliCommandDialog();
    }

    // Ctrl/Cmd + Shift + P: 快速建立 Prompt
    if (
      (event.ctrlKey || event.metaKey) &&
      event.shiftKey &&
      event.key === "P"
    ) {
      event.preventDefault();
      focusPromptCreation();
    }
  });
}

// 添加 CLI 相關按鈕
function addCliButtons() {
  const container = document.querySelector(".container");
  if (!container) return;

  // 檢查是否已經添加過
  if (document.getElementById("cli-section")) return;

  const cliSection = document.createElement("section");
  cliSection.id = "cli-section";
  cliSection.innerHTML = `
    <h3>🖥️ CLI 工具整合</h3>
    <div class="cli-controls">
      <button onclick="showCliStatus()" class="secondary">CLI 狀態</button>
      <button onclick="showCliCommandDialog()" class="contrast">執行 CLI 命令</button>
      <button onclick="showCliHelp()" class="outline">CLI 說明</button>
    </div>
    <div id="cli-output" class="cli-output" style="display: none;"></div>
  `;

  // 插入到主要內容之前
  const mainContent = document.querySelector("#content");
  if (mainContent) {
    container.insertBefore(cliSection, mainContent);
  }
}

// 顯示 CLI 狀態
async function showCliStatus() {
  try {
    const result = await invoke("run_cli_command", {
      command: "status",
      args: [],
    });

    showCliOutput("CLI 狀態", result);
  } catch (error) {
    showCliOutput("CLI 狀態錯誤", error.toString());
  }
}

// 顯示 CLI 命令對話框
function showCliCommandDialog() {
  const dialog = document.createElement("dialog");
  dialog.innerHTML = `
    <article>
      <header>
        <button aria-label="Close" rel="prev" onclick="this.closest('dialog').close()"></button>
        <h3>🖥️ 執行 CLI 命令</h3>
      </header>
      <form id="cli-command-form">
        <fieldset>
          <label for="cli-command">命令</label>
          <select id="cli-command" required>
            <option value="">選擇命令...</option>
            <option value="status">status - 系統狀態</option>
            <option value="cooldown">cooldown - 冷卻狀態</option>
            <option value="prompt list">prompt list - 列出 Prompts</option>
            <option value="job list">job list - 列出任務</option>
            <option value="results">results - 執行結果</option>
          </select>
          
          <label for="cli-args">額外參數 (可選)</label>
          <input type="text" id="cli-args" placeholder="例: --limit 5" />
          
          <div id="cli-preview" class="cli-preview"></div>
        </fieldset>
      </form>
      <footer>
        <button class="secondary" onclick="this.closest('dialog').close()">取消</button>
        <button onclick="executeCLICommand()" class="primary">執行</button>
      </footer>
    </article>
  `;

  document.body.appendChild(dialog);
  dialog.showModal();

  // 命令預覽
  const commandSelect = dialog.querySelector("#cli-command");
  const argsInput = dialog.querySelector("#cli-args");
  const preview = dialog.querySelector("#cli-preview");

  function updatePreview() {
    const command = commandSelect.value;
    const args = argsInput.value.trim();
    if (command) {
      preview.textContent = `cnp ${command}${args ? " " + args : ""}`;
      preview.style.display = "block";
    } else {
      preview.style.display = "none";
    }
  }

  commandSelect.addEventListener("change", updatePreview);
  argsInput.addEventListener("input", updatePreview);

  // 關閉對話框時清理
  dialog.addEventListener("close", () => {
    dialog.remove();
  });
}

// 執行 CLI 命令
async function executeCLICommand() {
  const dialog = document.querySelector("dialog");
  const command = dialog.querySelector("#cli-command").value;
  const argsInput = dialog.querySelector("#cli-args").value.trim();

  if (!command) {
    await message("請選擇一個命令", "CLI 錯誤");
    return;
  }

  // 解析命令和參數
  const commandParts = command.split(" ");
  const baseCommand = commandParts[0];
  const baseArgs = commandParts.slice(1);

  // 解析額外參數
  const extraArgs = argsInput
    ? argsInput.split(" ").filter((arg) => arg.trim())
    : [];
  const allArgs = [...baseArgs, ...extraArgs];

  dialog.close();

  try {
    showCliOutput(
      "執行中...",
      `cnp ${command}${argsInput ? " " + argsInput : ""}`
    );

    const result = await invoke("run_cli_command", {
      command: baseCommand,
      args: allArgs,
    });

    showCliOutput(`命令執行結果: cnp ${command}`, result);

    // 如果是會改變資料的命令，重新載入相關資料
    if (command.startsWith("prompt") || command.startsWith("job")) {
      await loadPrompts();
      await loadJobs();
    }
  } catch (error) {
    showCliOutput("命令執行錯誤", error.toString());
  }
}

// 顯示 CLI 輸出
function showCliOutput(title, content) {
  const outputDiv = document.getElementById("cli-output");
  if (!outputDiv) return;

  outputDiv.innerHTML = `
    <h4>${title}</h4>
    <pre class="cli-result">${content}</pre>
    <button onclick="document.getElementById('cli-output').style.display='none'" class="outline">隱藏</button>
  `;
  outputDiv.style.display = "block";

  // 滾動到輸出位置
  outputDiv.scrollIntoView({ behavior: "smooth" });
}

// 顯示 CLI 說明
function showCliHelp() {
  const helpContent = `
Claude Night Pilot CLI 工具 (cnp) 說明

基本命令:
  cnp init                 初始化資料庫
  cnp status              顯示系統狀態
  cnp cooldown            檢查冷卻狀態

Prompt 管理:
  cnp prompt list         列出所有 Prompts
  cnp prompt create       建立新 Prompt
  cnp prompt show <id>    顯示 Prompt 詳情
  cnp prompt edit <id>    編輯 Prompt
  cnp prompt delete <id>  刪除 Prompt

任務管理:
  cnp job list            列出所有任務
  cnp job show <id>       顯示任務詳情
  cnp job cancel <id>     取消任務

執行命令:
  cnp run -p <prompt>     執行 Prompt
  cnp results             顯示執行結果

快捷鍵:
  Ctrl+K                  開啟 CLI 命令對話框
  Ctrl+Shift+P           聚焦到 Prompt 建立

範例:
  cnp prompt create -t "測試" -c "Hello Claude"
  cnp run -p 1 -m sync
  cnp job list --status done
  `;

  showCliOutput("CLI 工具說明", helpContent);
}

// 聚焦到 Prompt 建立
function focusPromptCreation() {
  const titleInput = document.getElementById("prompt-title");
  if (titleInput) {
    titleInput.focus();
    titleInput.scrollIntoView({ behavior: "smooth" });
  }
}

// 載入 Prompts
async function loadPrompts() {
  try {
    const prompts = await invoke("list_prompts");
    currentPrompts = prompts;
    renderPromptList(prompts);
  } catch (error) {
    console.error("載入 Prompts 失敗:", error);
    await message(`載入 Prompts 失敗: ${error}`, "錯誤");
  }
}

// 載入任務
async function loadJobs() {
  try {
    const jobs = await invoke("list_jobs");
    currentJobs = jobs;
    renderJobList(jobs);
  } catch (error) {
    console.error("載入任務失敗:", error);
  }
}

// 渲染 Prompt 列表
function renderPromptList(prompts) {
  const container = document.getElementById("prompt-list");
  if (!container) return;

  if (prompts.length === 0) {
    container.innerHTML = "<p>尚無 Prompt 記錄</p>";
    return;
  }

  container.innerHTML = prompts
    .map(
      (prompt) => `
      <div class="prompt-item" data-id="${prompt.id}">
        <h4>${prompt.title}</h4>
        <p class="prompt-content">${prompt.content.substring(0, 100)}${
        prompt.content.length > 100 ? "..." : ""
      }</p>
        ${
          prompt.tags
            ? `<div class="tags">${prompt.tags
                .split(",")
                .map((tag) => `<span class="tag">${tag.trim()}</span>`)
                .join("")}</div>`
            : ""
        }
        <div class="prompt-actions">
          <button onclick="runPromptSync(${
            prompt.id
          })" class="primary">立即執行</button>
          <button onclick="showScheduleDialog(${
            prompt.id
          })" class="secondary">排程執行</button>
          <button onclick="editPrompt(${
            prompt.id
          })" class="outline">編輯</button>
          <button onclick="deletePrompt(${
            prompt.id
          })" class="outline contrast">刪除</button>
        </div>
        <small>建立時間: ${new Date(prompt.created_at).toLocaleString()}</small>
      </div>
    `
    )
    .join("");
}

// 渲染任務列表
function renderJobList(jobs) {
  const container = document.getElementById("job-list");
  if (!container) return;

  if (jobs.length === 0) {
    container.innerHTML = "<p>尚無任務記錄</p>";
    return;
  }

  container.innerHTML = jobs
    .map(
      (job) => `
      <div class="job-item" data-id="${job.id}">
        <div class="job-header">
          <h5>任務 #${job.id}</h5>
          <span class="status ${job.status.toLowerCase()}">${getStatusText(
        job.status
      )}</span>
        </div>
        <p>Prompt ID: ${job.prompt_id} | 模式: ${job.mode}</p>
        ${job.cron_expr !== "*" ? `<p>Cron: ${job.cron_expr}</p>` : ""}
        ${
          job.eta_unix && job.eta_unix > 0
            ? `<p class="eta">冷卻倒數: ${job.eta_unix} 秒</p>`
            : ""
        }
        <div class="job-actions">
          <button onclick="viewJobResults(${
            job.id
          })" class="secondary">查看結果</button>
          ${
            job.status === "pending" || job.status === "running"
              ? `<button onclick="cancelJob(${job.id})" class="outline">取消</button>`
              : ""
          }
        </div>
        ${
          job.last_run_at
            ? `<small>最後執行: ${new Date(
                job.last_run_at
              ).toLocaleString()}</small>`
            : ""
        }
      </div>
    `
    )
    .join("");
}

// 獲取狀態文字
function getStatusText(status) {
  const statusMap = {
    pending: "等待中",
    running: "執行中",
    done: "已完成",
    error: "錯誤",
  };
  return statusMap[status] || status;
}

// 更新冷卻狀態
async function updateCooldownStatus() {
  try {
    const cooldownInfo = await invoke("get_cooldown_status");
    const statusElement = document.getElementById("cooldown-status");

    if (statusElement) {
      if (cooldownInfo.is_cooling) {
        statusElement.textContent = `冷卻中，剩餘 ${cooldownInfo.seconds_remaining} 秒`;
        statusElement.className = "eta-display cooling";
      } else {
        statusElement.textContent = "Claude CLI 可用";
        statusElement.className = "eta-display available";
      }
    }
  } catch (error) {
    console.error("更新冷卻狀態失敗:", error);
    const statusElement = document.getElementById("cooldown-status");
    if (statusElement) {
      statusElement.textContent = "狀態未知";
      statusElement.className = "eta-display unknown";
    }
  }
}

// 開始冷卻監控
function startCooldownMonitor() {
  if (cooldownInterval) {
    clearInterval(cooldownInterval);
  }

  cooldownInterval = setInterval(async () => {
    await updateCooldownStatus();
    await loadJobs(); // 同時更新任務狀態
  }, 5000); // 每 5 秒更新一次
}

// 建立 Prompt
async function createPrompt() {
  const title = document.getElementById("prompt-title").value.trim();
  const content = document.getElementById("prompt-content").value.trim();
  const tags = document.getElementById("prompt-tags").value.trim();

  if (!title || !content) {
    await message("請填寫 Prompt 標題和內容", "輸入錯誤");
    return;
  }

  try {
    const promptId = await invoke("create_prompt", {
      title,
      content,
      tags: tags || null,
    });

    await message("Prompt 建立成功！", "成功");

    // 清空表單
    document.getElementById("prompt-title").value = "";
    document.getElementById("prompt-content").value = "";
    document.getElementById("prompt-tags").value = "";

    // 重新載入列表
    await loadPrompts();
  } catch (error) {
    console.error("建立 Prompt 失敗:", error);
    await message(`建立 Prompt 失敗: ${error}`, "錯誤");
  }
}

// 刪除 Prompt
async function deletePrompt(id) {
  try {
    const confirmed = await message(
      `確定要刪除 Prompt #${id}？這個操作無法復原。`,
      "確認刪除"
    );

    const success = await invoke("delete_prompt", { id });

    if (success) {
      await message("Prompt 已刪除", "成功");
      await loadPrompts();
    } else {
      await message("刪除失敗", "錯誤");
    }
  } catch (error) {
    console.error("刪除 Prompt 失敗:", error);
    await message(`刪除失敗: ${error}`, "錯誤");
  }
}

// 同步執行 Prompt
async function runPromptSync(promptId) {
  try {
    await message("開始執行 Prompt...", "執行中");

    const result = await invoke("run_prompt_sync", {
      promptId,
      mode: "sync",
      cronExpr: null,
    });

    await message("執行成功！", "成功");
    console.log("執行結果:", result);

    // 重新載入任務列表
    await loadJobs();
  } catch (error) {
    console.error("執行 Prompt 失敗:", error);
    await message(`執行失敗: ${error}`, "錯誤");
  }
}

// 手動執行任務
async function executeManualJob(promptId) {
  return await runPromptSync(promptId);
}

// 顯示排程對話框
function showScheduleDialog(promptId) {
  const dialog = document.getElementById("schedule-dialog");
  dialog.dataset.promptId = promptId;
  dialog.showModal();
}

// 建立排程任務
async function createScheduledJob() {
  const dialog = document.getElementById("schedule-dialog");
  const promptId = parseInt(dialog.dataset.promptId);
  const cronExpression = document
    .getElementById("cron-expression")
    .value.trim();
  const mode = document.getElementById("execution-mode").value;

  if (!cronExpression) {
    await message("請輸入 Cron 表達式", "輸入錯誤");
    return;
  }

  try {
    const result = await invoke("run_prompt_sync", {
      promptId,
      mode,
      cronExpr: cronExpression,
    });

    await message("排程任務建立成功！", "成功");
    console.log("排程結果:", result);

    // 關閉對話框並清空表單
    dialog.close();
    document.getElementById("cron-expression").value = "";
    document.getElementById("execution-mode").value = "async";

    // 重新載入任務列表
    await loadJobs();
  } catch (error) {
    console.error("建立排程任務失敗:", error);
    await message(`建立排程任務失敗: ${error}`, "錯誤");
  }
}

// 查看任務結果
async function viewJobResults(jobId) {
  try {
    const results = await invoke("get_job_results", { jobId });

    if (results.length === 0) {
      await message("此任務尚無執行結果", "資訊");
      return;
    }

    // 顯示結果對話框
    const dialog = document.createElement("dialog");
    dialog.innerHTML = `
      <article>
        <header>
          <button aria-label="Close" rel="prev" onclick="this.closest('dialog').close()"></button>
          <h3>任務 #${jobId} 的執行結果</h3>
        </header>
        <div class="results-container">
          ${results
            .map(
              (result, index) => `
            <div class="result-item">
              <h5>結果 #${index + 1}</h5>
              <small>時間: ${new Date(
                result.created_at
              ).toLocaleString()}</small>
              <pre class="result-content">${result.content}</pre>
            </div>
          `
            )
            .join("")}
        </div>
        <footer>
          <button onclick="this.closest('dialog').close()">關閉</button>
        </footer>
      </article>
    `;

    document.body.appendChild(dialog);
    dialog.showModal();

    // 關閉對話框時清理
    dialog.addEventListener("close", () => {
      dialog.remove();
    });
  } catch (error) {
    console.error("載入任務結果失敗:", error);
    await message(`載入任務結果失敗: ${error}`, "錯誤");
  }
}

// 關閉對話框
function closeDialog(dialogId) {
  const dialog = document.getElementById(dialogId);
  if (dialog) {
    dialog.close();
  }
}

// 掛載到全域
window.createPrompt = createPrompt;
window.deletePrompt = deletePrompt;
window.editPrompt = editPrompt;
window.runPromptSync = runPromptSync;
window.executeManualJob = executeManualJob;
window.showScheduleDialog = showScheduleDialog;
window.createScheduledJob = createScheduledJob;
window.viewJobResults = viewJobResults;
window.closeDialog = closeDialog;
window.showCliStatus = showCliStatus;
window.showCliCommandDialog = showCliCommandDialog;
window.executeCLICommand = executeCLICommand;
window.showCliHelp = showCliHelp;

// 編輯 Prompt (佔位符)
function editPrompt(id) {
  message("編輯功能開發中...", "Claude Night Pilot");
}

console.log("🚀 Claude Night Pilot 前端初始化完成");
