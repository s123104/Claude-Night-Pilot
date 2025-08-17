# Claude Night Pilot — Scheduler Deep Audit and Migration Plan

建立時間: 2025-08-17T22:48:33+08:00 [time.now]
來源: [context7:/websites/rs_tokio:2025-08-17T22:48:33+08:00], [context7:/tokio-rs/tracing:2025-08-17T22:48:33+08:00]

---

## 0. 執行摘要（Executive Summary）

- 我們對專案中所有與排程（scheduler）相關的檔案、服務層、CLI 與資料庫整合進行了逐一審視。
- 已發現並修復關鍵不一致（資料庫路徑、執行引擎缺失），但仍存在「重複排程實作」與「表結構並存（schedules vs jobs）」兩大技術債。
- 本規劃提出一套分階段（Phased）遷移與驗證策略，確保在不過度工程化的前提下，恢復穩定、提升可維護性與一致性，並建立 E2E 測試基準。

---

## 1. 範圍與目標

- 範圍：排程相關的所有程式碼、CLI 指令、服務層、資料庫結構與整合測試。
- 目標：
  - 完整梳理與收斂「排程器」實作，消除重複路徑與不一致。
  - 統一與強化資料庫整合，制定從 `schedules` 過渡至 `jobs` 的安全遷移計畫。
  - 建立企業級觀測能力（tracing/health）與 E2E 驗證規格。

---

## 2. 逐檔案審視（Per-file Deep Audit）

### 2.1 執行器與排程

- `src/scheduler/real_time_executor.rs`

  - 現況：已引入 `JobExecutionResult`、`execute_job_logic()`、`retrieve_prompt_content()`、`save_execution_result()`。
  - 風險/觀察：
    - Cron 註冊警告（`Failed to create cron job`）仍存在，疑似 `tokio-cron-scheduler` 與 Runtime 組態／閉包生命週期問題。
  - 行動：
    - 保持 `start()` 前後的 runtime 檢查與詳盡日誌。
    - 為每次註冊新增更精細的錯誤分類（表達式、Runtime、內部異常）。
    - 短期保底：在註冊失敗時，將任務加入「保底計時器（tokio::time::interval）」輪詢清單，避免掛失。

- `src/services/job_scheduler.rs`

  - 現況：可能為較舊或平行的排程器封裝。
  - 風險：與 `real_time_executor` 職責重疊。
  - 行動：將其狀態標註為「過渡相容層」，後續逐步下線，統一到 `real_time_executor`。

- `src/services/simple_job_manager.rs`

  - 現況：以 `tokio_cron_scheduler::JobScheduler` 簡化管理；與 `JobService` 整合執行。
  - 風險：與 `real_time_executor` 重複；兩個排程入口造成複雜性與潛在不一致。
  - 行動：
    - 短期：標註為實驗/相容路徑，不預設啟用。
    - 中期：將其能力融入 `real_time_executor`（保留必要功能後移除）。

- `src/core/scheduler.rs`
  - 現況：定義高階 Scheduler trait（存在 `async fn in trait` 警告）。
  - 行動：
    - 中期修改為 `fn -> impl Future` 以消除 Lint 警告；避免公開 API 破壞性變更前，先以內部使用為主。

### 2.2 CLI 與 Adapter

- `src/bin/cnp-unified.rs`

  - 現況：CLI 主入口；`job create` 完成實際入庫與後續註冊呼叫；資料庫路徑已統一。
  - 行動：
    - 針對「CLI 層」新增 `--dry-run` 與 `--no-register` 選項，利於測試。

- `src/interfaces/cli_adapter.rs`、`src/services/job_service.rs`、`src/services/prompt_service.rs`
  - 現況：服務層以 `SimpleSchedule` 為輸出型別（來自 `simple_db.rs`）。
  - 風險：新 `jobs`/`job_executions` 結構與舊 `schedules` 共存，資料模型分裂。
  - 行動：
    - 提供過渡型 Mapper（`SimpleSchedule` <-> `Job`）以維持相容，並規劃逐步淘汰 `schedules`。

### 2.3 資料庫層

- `src/core/database/types.rs`
  - 現況：`DatabaseConfig::default().path = claude-night-pilot.db`（已統一）。
- `src/database_manager_impl.rs`
  - 現況：預設路徑已統一為 `claude-night-pilot.db`。
- `src/core/database/mod.rs`

  - 現況：`DEFAULT_DATABASE_PATH` 仍為 `claude-pilot.db`（不一致）。
  - 行動：
    - 短期：調整為 `claude-night-pilot.db`，同步更新相關測試斷言。

- `src/simple_db.rs`
  - 現況：仍以 `schedules` 表為核心；測試齊全，作為穩定相容層。
  - 行動：
    - 中期：保留簡化查詢與 API 風格，底層改為讀寫 `jobs`/`job_executions`（或提供聚合視圖）。

---

## 3. 資料庫一致性與遷移策略

### 3.1 權威資料庫與連線策略

- 權威路徑：`claude-night-pilot.db`
- 連線最佳實踐（[context7:/websites/rs_tokio]）：
  - 所有阻塞 I/O 包裹於 `tokio::task::spawn_blocking`。
  - 啟用 `PRAGMA foreign_keys=ON`、`journal_mode=WAL`、適度 `busy_timeout`。
  - 明確交易邊界，保證 ACID。

### 3.2 Schema 共存與過渡

- 目前表：`prompts`, `schedules`, `execution_results`, `token_usage_stats`；新增 `jobs`, `job_executions`。
- 過渡策略：
  1. 短期：保持 `schedules` 可讀寫，新增將 `jobs` 映射回 `schedules` 的兼容讀接口（或 View/Query）。
  2. 中期：所有新寫入轉向 `jobs`，`schedules` 僅讀。
  3. 長期：移除 `schedules` 寫入點與相關 Mapper；資料留存或以 View 供舊接口查詢。

### 3.3 遷移工具與驗證

- 提供一次性 SQL：
  - `INSERT INTO jobs (...) SELECT ... FROM schedules ...`（字段對齊後分批遷移）。
- 建立對帳報表：
  - `COUNT(schedules)` vs `COUNT(jobs)`，及關鍵字段（prompt_id、cron_expr）采樣核對。

---

## 4. 排程器註冊告警（tokio-cron-scheduler）修復路徑

### 4.1 可能原因分類

- Runtime 組態異常或未初始化。
- Cron 表達式或時區處理差異。
- 任務閉包移動/生命週期導致 Panic 被吞。

### 4.2 修復步驟

1. 升級 `tokio-cron-scheduler` 至最新版，查核 API 變動與相容性說明。
2. 在 `RealTimeExecutor::start()` 與 `add_job()` 增加更精細錯誤分類與 Span。
3. 增加保底計時器：對註冊失敗的任務，加入 `tokio::time::interval` 監測，時間達時手動觸發執行（不中斷主流程）。
4. 若仍不穩定：切換為 `cron` + 自製單點分發器；由 `interval` 驅動 `cron` 計算下一次執行時間。

---

## 5. 研究專案參考（research-projects/vibe-kanban）

- `backend/src/execution_monitor.rs`：

  - 採「執行監視 + 服務」模型，將執行過程事件化、記錄化；建議借鑑：
    - 統一 `AppState` 與監控通道（events/metrics）。
    - 嚴格分離「排程」與「執行」兩層，排程只負責觸發，執行層負責重試、記錄與通知。

- 統整建議：
  - 我們沿用 `real_time_executor` 作為唯一排程入口；執行層抽象為 `Executor`，對接 Claude/本地腳本等。

---

## 6. E2E 與整合測試規格

### 6.1 測試檔案結構

- `src-tauri/tests/integration/scheduler_e2e.rs`
- `src-tauri/tests/integration/database_consistency.rs`
- `src-tauri/tests/integration/cli_scheduler.rs`

### 6.2 主要用例

- creates_job_and_persists_to_db
  - CLI: `job create` → 斷言 `jobs`（或 `schedules`）新增成功。
- registers_job_with_executor
  - 啟動排程器 → 驗證註冊成功（或降級策略生效）。
- triggers_and_records_execution
  - 模擬近時執行 → 斷言 `job_executions` 新記錄與時長。
- handles_empty_prompt_safely
  - 空內容 → 斷言安全失敗與記錄錯誤。
- db_path_consistency
  - 斷言所有路徑最終指向 `claude-night-pilot.db`。

### 6.3 典型資料庫斷言

- `SELECT COUNT(*) FROM jobs WHERE id = ?`（或 schedules 相容層）
- `SELECT COUNT(*) FROM job_executions WHERE job_id = ? AND status = 'Completed'`
- `SELECT content FROM prompts WHERE id = ?`

---

## 7. 分階段重構與時間表

### Phase 1（已完成）

- 統一路徑：`claude-night-pilot.db`
- 新增 `jobs` / `job_executions`
- 實作 `execute_job_logic` 與結果落庫

### Phase 2（本週）

- 將 `schedules` 寫入點轉為只讀；新寫入使用 `jobs`
- `JobService` 資料模型過渡（SimpleSchedule -> JobResponse）
- 標註/凍結 `simple_job_manager.rs` 與 `job_scheduler.rs`

### Phase 3（下週）

- 完成 E2E 與整合測試；建立可重放測試基準
- 排程器註冊告警關閉或以保底機制完全覆蓋
- 技術債清單收尾：刪除未用路徑、移除重複抽象

---

## 8. 風險與回滾

- 風險：遷移期間雙表並存引發資料不一致。
  - 緩解：所有寫入單路口（jobs），`schedules` 僅讀或視圖層。
- 風險：註冊警告導致預期時間未觸發。
  - 緩解：保底計時器與手動觸發指令（CLI: `job trigger <id>`）。
- 回滾：保留 `schedules` 寫路徑（隱藏旗標），必要時快速回退。

---

## 9. 驗收標準（Acceptance Criteria）

- 功能：所有 `job create` / 觸發 / 記錄功能可用，無資料遺失。
- 可觀測：tracing 事件覆蓋註冊、觸發、執行、落庫關鍵點。
- 資料庫：單一權威路徑，無 `claude-pilot.db` 遺留用於生產路徑。
- 測試：E2E/整合測試全部綠燈；覆蓋主要 Happy Path 與錯誤處理。

---

## 10. 備註（Context7 對齊）

- Tokio 最佳實踐（[context7:/websites/rs_tokio:2025-08-17T22:48:33+08:00]）
  - 阻塞 I/O 使用 `spawn_blocking`；Runtime 檢查使用 `Handle::try_current()`。
- tracing 最佳實踐（[context7:/tokio-rs/tracing:2025-08-17T22:48:33+08:00]）
  - 關鍵操作皆以 Span/Events 標記，結構化輸出，支援 JSON。
