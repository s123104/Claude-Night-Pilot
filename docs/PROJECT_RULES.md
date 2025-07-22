以下文件是針對 **「Claude-Night-Pilot」**（暫名）的完整開發規劃與專案規範。
它以 **Rust ＋ Tauri 2  單一 Binary** 為核心，搭配 _htmx_ 與 Pico.css 做極小前端，實現：

- GUI  管理  Prompt  模板
- 同步／非同步（Cron）排程
- 自動偵測  Claude  冷卻時間並倒數顯示

---

# 1 專案概述

| 欄位     | 內容                                                                        |
| -------- | --------------------------------------------------------------------------- |
| 名稱     | Claude-Night-Pilot                                                          |
| 類型     | 本地桌面應用（macOS／Windows／Linux）                                       |
| 目標     | 讓使用者夜間自動執行多筆  Claude Prompt 工作流，並以 GUI 監控進度與冷卻倒數 |
| 授權     | MIT                                                                         |
| 最終包體 | ≤ 10 MB（Tauri 實測可低於 3 MB）([Tauri][1])                                |

---

# 2 核心功能

1. **Prompt 庫管理**

   - CRUD、標籤、快速搜尋。

2. **執行模式**

   - 同步：立即執行並回傳結果。
   - 非同步：Cron 排程，離開程式仍在背景執行。

3. **冷卻偵測**

   - 解析 `claude doctor --json` 與 CLI 429／503  訊息，顯示 ETA。([docs.anthropic.com][2])

4. **結果保存**

   - 每次回覆存  SQLite  並可匯出 .md 或 .json。

5. **日誌與錯誤**

   - stderr、exit code ≠ 0 會 toast 提示並寫入 `logs/`.

---

# 3 非功能需求

| 分類   | 需求                                                             |
| ------ | ---------------------------------------------------------------- |
| 效能   | 首啟動 < 1 s；記憶體佔用 < 150 MB。                              |
| 體積   | 安裝包 ≤ 10 MB。                                                 |
| 可攜   | 單檔  SQLite；搬移 `.db` 即完成備份。                            |
| 安全   | 不上雲、不收集 Prompt 內容；API Key 以 Tauri secure‑store 加密。 |
| 可維護 | 前後端皆採格式化（rustfmt／prettier）與 Git Hooks 檢查。         |

---

# 4 技術選型

| 層級     | 套件                                             | 理由                                                           |
| -------- | ------------------------------------------------ | -------------------------------------------------------------- |
| 桌面殼   | **Tauri 2**                                      | 600 KB–3 MB 二進位、原生 WebView。([Tauri][1])                 |
| 前端     | ***htmx* 1.9 + Pico.css**                        | htmx ≈ 14 kB gz。Pico 無 JS 依賴。([Medium][3], [Pico CSS][4]) |
| DB       | **SQLite + sqlx + Tauri SQL plugin**             | 零伺服器、官方支援。([Tauri][5])                               |
| 排程     | **tokio‑cron‑scheduler**                         | async Cron、熱插拔任務。([Crates][6])                          |
| CLI 橋接 | `std::process::Command` + `--output-format json` | 官方建議自動化用法。([docs.anthropic.com][7])                  |

---

# 5 高層架構圖

```
┌──────── GUI (Webview) ────────┐
│ index.html + htmx + Pico.css  │
│ ├─ /prompts    • CRUD         │
│ └─ /jobs       • ETA          │
└───────────┬───────────────────┘
            │ Tauri invoke (IPC)
┌───────────▼───────────┐
│  Rust Core (tokio)    │
│ ├─ sqlx (SQLite)      │
│ ├─ CronScheduler      │
│ ├─ ClaudeExecutor     │
│ └─ CooldownWatcher    │
└───────────┬───────────┘
            │ spawn
       ┌────▼────┐
       │ Claude  │
       │  CLI    │
       └─────────┘
```

---

# 6 主要流程

```
[UI 新增 Prompt] → [寫 SQLite]

[執行]
  │ 立即              │ 排程
  ▼                   ▼
[ClaudeExecutor]   [Scheduler]
  │ spawn              │ 時到
  ▼                    │
[CLI 回覆]  ←──────────┘
  │
[寫 SQLite → 刷新 UI]
```

---

# 7 資料模型

```sql
-- prompts
CREATE TABLE prompts (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  title     TEXT NOT NULL,
  content   TEXT NOT NULL,
  tags      TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- jobs
CREATE TABLE jobs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  prompt_id   INTEGER,
  cron_expr   TEXT,     -- '*' 即手動
  mode        TEXT,     -- 'sync' | 'async'
  status      TEXT,     -- 'pending' | 'running' | 'done' | 'error'
  eta_unix    INTEGER,  -- cooldown 倒數秒
  last_run_at DATETIME,
  FOREIGN KEY (prompt_id) REFERENCES prompts(id)
);

-- results
CREATE TABLE results (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id    INTEGER,
  content   TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id)
);
```

---

# 8 前後端  IPC API

| 名稱           | 方法 | 參數                                       | 回傳              |
| -------------- | ---- | ------------------------------------------ | ----------------- |
| `list_prompts` | GET  | –                                          | Prompt\[]         |
| `save_prompt`  | POST | {title, content, tags?}                    | id                |
| `run_prompt`   | POST | {promptId, mode: 'sync' \| 'async', cron?} | jobId             |
| `cancel_job`   | POST | {jobId}                                    | ok                |
| `get_results`  | GET  | {jobId}                                    | Result\[]         |
| `get_eta`      | GET  | –                                          | {seconds: number} |

---

# 9 ClaudeExecutor 細節

```rust
pub async fn run(prompt: &str) -> Result<String> {
    let output = Command::new("claude")
        .arg("-p").arg(prompt)
        .arg("--output-format").arg("json")
        .output()
        .await?;
    if !output.status.success() {
        bail!("CLI error: {}", String::from_utf8_lossy(&output.stderr));
    }
    let reply = serde_json::from_slice::<CliJson>(&output.stdout)?;
    Ok(reply.completion)
}
```

- 若 stderr 含 `429` 或 `.+cooldown.+(\d+)s` → 更新 `eta_unix`.

---

# 10 Scheduler 行為

```rust
let sched = JobScheduler::new().await?;
sched.add(Job::new_async("0 2 * * *", |_id, _lock| async {
    run_prompt_async(job_id).await;
})?);
sched.start().await?;
```

- `mode=sync`：不入 Cron，直接 `run()`.
- `mode=async` 且 `cron_expr='*'`：立即 spawn 背景 Job。
- `cron_expr="0 2 * * *"`：凌晨 2  點執行。

---

# 11 冷卻監聽

```rust
async fn refresh_eta() -> u64 {
    let out = Command::new("claude")
        .arg("doctor").arg("--json")
        .output()
        .await?;
    let diag: Doctor = serde_json::from_slice(&out.stdout)?;
    diag.cooldown_secs.unwrap_or(0)
}
```

每  60  秒輪詢，UI 透過 IPC 取秒數 → 顯示「剩餘  mm\:ss」。

---

# 12 日誌與錯誤

- 成功：寫 `logs/claude-YYYYMMDD.log`.
- 失敗：寫 `logs/failed-YYYYMMDD.log` 並 Toast。
- 所有 `Command` 捕捉 exit code 與 stderr。

---

# 13 專案目錄

```
.
├─ src-tauri/
│  ├─ main.rs
│  ├─ db.rs
│  ├─ scheduler.rs
│  ├─ executor.rs
│  └─ watcher.rs
├─ src/
│  ├─ index.html
│  ├─ js/htmx.min.js
│  └─ css/pico.min.css
├─ sql/0001_init.sql
├─ logs/
└─ README.md
```

---

# 14 Git 工作流程

| 規範        | 說明                                                         |
| ----------- | ------------------------------------------------------------ |
| Flow        | GitHub PR ＋ Squash merge                                    |
| Branch 命名 | `feat/x`, `fix/x`, `docs/x`                                  |
| Commit      | `<type>(scope): subject`                                     |
| Lint        | rustfmt ＋ prettier；`cargo clippy` 強制 0 warning           |
| CI          | GitHub Actions：`cargo test` → `tauri build` → 上傳 artefact |

---

# 15 測試策略

- **單元**：Rust 模組邏輯 (`executor`, `scheduler`)。
- **整合**：啟動 Tauri `--dev`，用 Playwright 點擊 UI。
- **負載**：模擬 100  條 Cron Job，確保記憶體 < 300 MB。

---

# 16 打包與發布

1. `npm ci && cargo tauri build --target x86_64-pc-windows-msvc`
2. 產生 `.msi`, `.dmg`, `.AppImage`.
3. GitHub Release 動作自動上傳 artefacts。
4. 用 `tauri updater` 可選 OTA 更新。

---

# 17 安全與隱私

- API Key 僅存 Tauri secure‑store，無明文落 disk。
- 不回傳任何遙測。
- `.db` 與 `logs/` 皆在 `$APPDATA/claude-pilot/`, 可加 GitIgnore。

---

# 18 貢獻指南

1. Fork ➜ 建議 Issue → PR.
2. 通過 `npm test`、`cargo test` 與 `tauri build`.
3. 新功能需更新 README 與 `docs/CHANGELOG.md`.

---

# 19 路線圖

| 版本 | 重點                       |
| ---- | -------------------------- |
| v0.1 | MVP：Prompt CRUD, 同步執行 |
| v0.2 | Cron 背景任務＋ ETA 顯示   |
| v0.3 | 多  Claude  模型選擇       |
| v0.4 | TUI  模式（Ratatui）       |
| v1.0 | 自動更新、插件 API、國際化 |

---

## 結語

依照此文件，你能在 **5 個 Sprint** 內交付一個 < 10 MB、跨平台、開箱即用的  Claude-Night-Pilot。技術棧只有 **Rust ＋ Tauri ＋ SQLite ＋極小前端**，既輕巧又符合 2025  最佳實踐。祝開發一路順風 🚀

[1]: https://v2.tauri.app/?utm_source=chatgpt.com "Tauri 2.0 | Tauri"
[2]: https://docs.anthropic.com/en/docs/claude-code/cli-reference?utm_source=chatgpt.com "CLI reference - Anthropic API"
[3]: https://medium.com/%40ric.kanjilal/htmx-the-javascript-killer-revolutionizing-web-development-in-2025-8dcc13a8920f?utm_source=chatgpt.com "HTMX: The JavaScript Killer Revolutionizing Web Development in ..."
[4]: https://picocss.com/?utm_source=chatgpt.com "Pico CSS • Minimal CSS Framework for semantic HTML"
[5]: https://v2.tauri.app/plugin/sql/?utm_source=chatgpt.com "SQL - Tauri"
[6]: https://crates.io/crates/tokio-cron-scheduler?utm_source=chatgpt.com "tokio-cron-scheduler - crates.io: Rust Package Registry"
[7]: https://docs.anthropic.com/en/docs/agents/claude-code/introduction?utm_source=chatgpt.com "Claude Code overview - Anthropic API"
