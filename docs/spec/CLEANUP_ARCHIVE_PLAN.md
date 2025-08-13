# 🧹 遺留模組清理與歸檔計劃

## 現況

- 仍有遺留呼叫與檔案：
  - `src-tauri/src/database_manager.rs`（Deprecated）
  - `simple_db.rs`（仍被大量使用，短期保留作相容層）
  - 新增但未納管：`src-tauri/src/core/database/best_practices_*` 與 `models/`、`tests/`

## 清理策略（兩階段）

- 階段一（相容保持）：
  - 保留 `simple_db.rs` 與 `database_manager_impl.rs`
  - 將 `database_manager.rs` 標註 deprecated 並在文件中註記替代方案
  - 新增 docs 與 TODO，規劃移轉到 `core/database/*` repository 模式
- 階段二（切換至新資料層）：
  - Services 改用新 Repository 與 `best_practices_manager`
  - 自動化遷移 SQL 與資料表索引
  - 刪除 `database_manager.rs`（歸檔到 `archive/`）

## 歸檔規則

- 具體檔案：
  - `src-tauri/src/database_manager.rs` → `archive/database_manager.rs.bak`
  - 相關測試 → `archive/tests_legacy_db/`
- 保留期限：一個小版本（minor）週期

## 追蹤清單

- [ ] 搜索移除所有 `use crate::database_manager` 與 `get_database_manager()` 引用
- [ ] Services 層改用新 `core/database` 入口
- [ ] Benchmarks 切換引用
- [ ] Playwright 測試過一輪（含 CLI/GUI/E2E/整合）

最後更新：2025-08-12 • 負責人：@s123104
