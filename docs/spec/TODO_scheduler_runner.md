# ✅ Scheduler Runner 實作待辦清單

- [ ] 設計：審閱 `docs/spec/SCHEDULER_RUNNER_SPEC.md`，確認欄位/查詢條件
- [ ] 啟動：於 `src-tauri/src/lib.rs` 的 `setup` 階段 spawn 背景任務（受 `AppConfig` 控制）
- [ ] 核心：新增 `src-tauri/src/core/scheduler_runner.rs`
  - [ ] `poll_pending()`：查詢 `pending/active` 且 `next_run_at <= now` 或首次 `schedule_time <= now`
  - [ ] `dispatch()`：調用 `EnhancedClaudeExecutor` 執行並回寫 `last_run_at/next_run_at/status`
  - [ ] `apply_retry()`：對失敗結果套用 `RetryOrchestrator` 計算下次時間
  - [ ] `missed_replay()`：啟動時補執行過期任務
  - [ ] 併發控制：尊重 `ExecutorConfig.max_concurrent_executions`
- [ ] 設定：在 `AppConfig` 加入 `scheduler.enabled/poll_interval_seconds/max_concurrent_jobs`
- [ ] 日誌：重要事件 tracing/info，錯誤含 context
- [ ] 測試：
  - [ ] Rust 單元：查詢條件與 `next_run_at` 計算
  - [ ] Playwright 整合：冷卻中建立 → 解除後自動完成；3 分鐘內排程 →3 秒完成
- [ ] 文件：README/USER GUIDE 補充背景 Runner 行為與限制

負責人：@s123104 • 最後更新：2025-08-12
