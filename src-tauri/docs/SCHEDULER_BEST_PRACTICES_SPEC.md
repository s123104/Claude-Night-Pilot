# Claude Night Pilot — Scheduler Best Practices Spec and Verification Plan

建立時間: 2025-08-17T22:48:33+08:00 [time.now]
資料來源: [context7:/websites/rs_tokio], [context7:/tokio-rs/tracing]

---

## 目標

- 確保排程功能完整、可觀測、可維護、低技術債
- 統一資料庫整合，消除不一致
- 明確執行管線：創建 → 註冊 → 觸發 → 執行 → 記錄 → 監控
- 建立 E2E 與整合測試規格，避免回歸

## 範圍

- 排程核心模組與服務
- CLI/Service 接口
- 資料庫 Schema 與一致性
- 監控與健康檢查
- E2E/整合測試

---

## 一、檔案逐一檢視與責任歸屬

### A. 執行器與排程

- `src/scheduler/real_time_executor.rs`

  - 責任：企業級實時排程執行器；新增 `JobExecutionResult`、`execute_job_logic()`、`retrieve_prompt_content()`、`save_execution_result()`
  - 需符合：
    - 只能透過 `tokio::runtime::Handle::try_current()` 確認運行時存在
    - DB 操作以 `spawn_blocking` 包裝（Context7/Tokio 最佳實踐）
    - 結構化日誌與錯誤上下文（tracing + anyhow）
  - 檢查項：
    - [x] 啟動/停止流程
    - [x] Cron 任務閉包安全（捕捉 ID、狀態更新）
    - [x] 執行結果入庫至 `job_executions`

- `src/services/job_scheduler.rs`（若仍使用）

  - 責任：低層排程對接/舊版橋接
  - 檢查項：
    - [ ] 若同時存在多路排程器（real_time_executor + 本檔）需整合為單一路徑

- `src/core/scheduler.rs`
  - 責任：高階 Scheduler trait（注意 async fn in trait 警告）
  - 檢查項：
    - [ ] 規劃：未來改為 `fn -> impl Future` 以消除 lint 警告（保持相容，避免過度工程化）

### B. CLI 與 Adapter

- `src/bin/cnp-unified.rs`

  - 責任：CLI 主入口，`prompt`/`job` 指令
  - 變更：
    - [x] `job create` 實際入庫，並呼叫 `start_real_time_scheduler`
    - [x] 統一 DB 為 `claude-night-pilot.db`
  - 檢查項：
    - [x] prompt 存在性驗證改為正確資料庫
    - [x] 記錄排程註冊警告但不中斷（優雅降級）

- `src/interfaces/cli_adapter.rs`, `src/services/job_service.rs`, `src/services/prompt_service.rs`
  - 責任：供 CLI 使用的服務層封裝
  - 檢查項：
    - [x] 服務層使用的 `DatabaseManager::default()` 路徑需統一（已在 `database_manager_impl.rs` 改為 `claude-night-pilot.db`）
    - [ ] 後續將 SimpleSchedule 過渡到新 `jobs` 表（見重構計劃）

### C. 資料庫層

- `src/core/database/types.rs`
  - 變更：
    - [x] `DatabaseConfig::default().path` 改為 `claude-night-pilot.db`
- `src/database_manager_impl.rs`
  - 變更：
    - [x] 預設路徑改為 `claude-night-pilot.db`
- 現況 Schema：
  - 已存在：`prompts`, `schedules`, `execution_results`, `token_usage_stats`
  - 新增：`jobs`, `job_executions`
- 檢查項：
  - [x] WAL + FK 啟用
  - [x] 所有寫入包於交易/錯誤處理
  - [ ] `schedules` 與 `jobs` 關係：目前並存，需制定遷移策略

### D. 監控與健康

- `src/monitoring/health.rs`, `src/monitoring/logging.rs`
  - 責任：健康檢查、結構化日誌
  - 檢查項：
    - [x] Arc<RwLock> 並發安全
    - [x] tracing JSON/time 設定

### E. 測試與樣例

- `src-tauri/tests/integration/*.rs` + `src/core/tests_new/*`
  - 檢查項：
    - [ ] 新增針對 `jobs`/`job_executions` 的 E2E
    - [ ] 覆蓋排程器啟動/任務登記/執行/入庫全鏈路

---

## 二、資料庫整合一致性規範

- 統一路徑：`claude-night-pilot.db`
- 連線策略：
  - 所有阻塞 I/O → `spawn_blocking`（Context7/Tokio）
  - 開啟 `PRAGMA foreign_keys=ON`、`journal_mode=WAL`、適度的 `busy_timeout`
- Schema 標準：
  - `jobs(id TEXT PK, prompt_id TEXT, cron_expression TEXT, status TEXT, ... timestamps)`
  - `job_executions(id TEXT PK, job_id TEXT, prompt_id TEXT, status TEXT, start_time TEXT, end_time TEXT, duration_ms INTEGER, output TEXT, error_message TEXT, created_at TEXT)`
- 遷移策略：
  - 短期：`schedules` 作為相容層沿用
  - 中期：將 `schedules` → `jobs` 的投影與同步器
  - 長期：移除 `schedules` / SimpleSchedule（避免雙源）

---

## 三、排程器註冊告警解決路線

- 現狀："Failed to create cron job"（註冊告警）
- 可能原因：
  1. tokio-cron-scheduler 與當前 Runtime 設定不符
  2. Cron 表達式解析或時區設定問題
  3. 任務閉包生命週期/捕捉導致 panic
- 修復步驟：
  1. 升級 tokio-cron-scheduler 至最新版，確認 API 差異
  2. 在 `RealTimeExecutor::start()` 前後加入 runtime/層級檢查與日誌
  3. 以 `tokio::time::interval` 建立保底觸發器（短期降級方案，不影響資料保存）
  4. 若仍不穩定，改用 `cron` crate + 自研簡單分發器

---

## 四、E2E 驗證規格

### 測試目標

1. 任務創建 → 正確入庫 `jobs`/`schedules`
2. 排程啟動 → 無告警（或可容忍但不影響執行）
3. 時間到達 → `execute_job_logic()` 被呼叫
4. 執行結果 → 正確入庫 `job_executions`
5. 監控 → 健康檢查返回運行中

### 測試案例

- `tests/integration/scheduler_e2e.rs`
  - `creates_job_and_persists_to_db`
  - `registers_job_with_executor`
  - `triggers_and_records_execution`
  - `handles_empty_prompt_safely`
  - `records_error_and_does_not_panic`

### 資料庫檢查斷言

- `SELECT count(*) FROM jobs WHERE id = ?`
- `SELECT count(*) FROM job_executions WHERE job_id = ? AND status='Completed'`
- `SELECT content FROM prompts WHERE id = ?`

---

## 五、重構與去重路線圖

- Phase 1（已完成）

  - 統一路徑：`claude-night-pilot.db`
  - 新增 `jobs` / `job_executions`
  - 實作 `execute_job_logic`

- Phase 2（本週）

  - 將 `schedules` 替換為 `jobs` 管道
  - `JobService` 從 SimpleSchedule 過渡到 `jobs`
  - 移除重複排程器實作（保留 `real_time_executor`）

- Phase 3（下週）
  - 建立視覺化監控接口（可選）
  - 分散式鎖/多實例安全（可選）

---

## 六、最佳實踐對齊（Context7）

- Tokio（/websites/rs_tokio）
  - 阻塞 I/O -> `spawn_blocking`
  - Runtime 檢查 -> `Handle::try_current()`
- tracing（/tokio-rs/tracing）
  - 事件/Span 覆蓋，要點節點記錄（排程註冊、執行開始/結束、DB 寫入）

標註時間: 2025-08-17T22:48:33+08:00
