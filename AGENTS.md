# 儲存庫指南

## 語言與在地化
- 預設輸出語言：繁體中文（zh-Hant）。所有 Agent 回覆、CLI 日誌、UI 文案與新文件皆以繁體中文撰寫，除非議題明確要求其他語言。
- 測試：Playwright 斷言與測試名稱優先使用繁體中文；Rust 測試訊息亦採繁體中文。
- 英文僅用於程式碼識別子、對外 API 參數或必要技術名詞；對使用者顯示內容以繁體中文為主。

## 開發指令
### 核心開發
- `npm run dev` — 啟動 Tauri 應用（GUI + 後端）。
- `npm run dev:frontend` — 僅啟動前端靜態伺服器。
- `npm run cli -- <args>` — 執行優化版 CLI；建置用 `npm run cli:build`。

### 測試與驗證
- `npm run lint:check && npm test` — 快速驗證（ESLint + Playwright）。
- `npm run test:rust` — 於 `src-tauri` 執行 Rust 測試（單元/整合）。
- `npm run test:ui` / `npm run test:headed` — E2E 偵錯；覆蓋率 `npm run test:coverage`。

### 建置
- `npm run build` / `npm run tauri build` — 產出桌面應用。
- `npm run cli:build` / `npm run cli:install` — 產出/安裝 CLI。

## 架構總覽
### 技術棧
- 前端：vanilla JS + htmx + Material 風格
- 後端：Rust + Tauri 2.0；SQLite（sqlx）
- 測試：Playwright（GUI/CLI）、Rust `cargo test`

### 專案結構
```
src/                # HTML/CSS/JS（index.html, main.js）
src-tauri/          # Rust、Tauri、CLI（cnp-optimized）、bench、tests
tests/              # Playwright 規格（gui/cli/e2e/integration）
scripts/            # 連接埠工具、precompile、驗證腳本
```
核心模組：`src-tauri/src/*`（executor、scheduler、agents_registry、database_manager），CLI 於 `src-tauri/src/bin/`。

## 開發指南
- 程式風格：JS 採 ESLint（2 空白縮排、避免未使用/未定義），Rust 採 `rustfmt` 與 `clippy -D warnings`。
- 提交規範：Conventional Commits（由 commitlint 驗證）。格式：`<type>(<scope>): <subject>`。
  - 類型：feat、fix、docs、style、refactor、perf、test、build、ci、chore、revert
  - 範圍：core、gui、cli、db、scheduler、executor、security、test、docs、deps、config、ci、release
- 分支命名：`feat/...`、`fix/...`；PR 聚焦且附說明。
- 環境變數：使用 `.env` 管理敏感資料；連接埠工具：`npm run port:status|port:cleanup`。

### Git Hooks 與 lint-staged（最佳實踐）
- pre-commit：使用 `lint-staged` 僅檢查暫存區檔案（JS 執行 `eslint --fix`；Rust 執行 `cargo fmt` 與 `cargo clippy --fix -- -D warnings`）。
- pre-push：於推送前執行完整驗證（`npm run test:rust` 與 `npm test`）。
- 目的：加速 commit 階段、把較耗時的驗證移到 push，提升開發流暢度且確保品質。

## 測試策略
- Playwright：規格於 `tests/**`（`*.spec.js`）；可用 `playwright test tests/integration/` 或以 `package.json` 專案旗標篩選。
- Rust：`cd src-tauri && cargo test`；整合測試位於 `src-tauri/tests/**`。
- 產物：報表輸出於 `playwright-report/`、`test-results/`；覆蓋率採 tarpaulin。

## Commit 與 PR 要求
- 本地須通過：`npm run lint:check && npm run test:all`。
- PR 需含：變更摘要、關聯 Issue、測試步驟；GUI 變更請附截圖/錄影。
- CI 會驗證：lint、fmt/clippy、Rust 測試、Playwright、Tauri 建置（參見 `.github/workflows/ci.yml`）。

### PR 檢查清單（繁體中文）
- 文字與 UI 文案皆為繁體中文。
- `npm run lint:check && npm run test:all` 通過。
- Rust：`cd src-tauri && cargo fmt -- --check && cargo clippy -- -D warnings`。
- 附測試步驟；GUI 變更附截圖或短影片。
- 必要時更新 README/CLAUDE/AGENTS 並連結 Issue。

## 環境設定
- Node.js 18.x、npm；Rust stable（含 rustfmt、clippy）。
- Playwright 瀏覽器：`npx playwright install --with-deps`（CI 同步安裝）。
- Linux 需安裝 WebKit/Tauri 依賴：`libwebkit2gtk-4.0-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xvfb`（參考 CI）。
- 設定 `.env`（Tauri/CLI 所需金鑰或路徑），避免提交敏感資訊。

## 關鍵相依
- Tauri 2.x：桌面應用與 IPC。
- sqlx + SQLite：型別安全查詢與輕量資料。
- tokio：非同步執行與排程。
- Playwright：E2E 測試。
- ESLint、Husky、Commitlint：程式碼品質與提交規範。

## 安全與建置注意
- 稽核：`cd src-tauri && cargo audit`、`npm audit`。
- Linux 依賴需與 CI 一致以避免建置失敗。
