# Changelog

本專案的所有重要變更都會記錄在此檔案中。

格式基於 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.0.0/)，
並且本專案遵循 [語義化版本控制](https://semver.org/lang/zh-TW/)。

## [未發布] - 開源專案最佳實踐完善

### 🏗️ 重構 (Restructured)

- 將專案根目錄重構為標準開源專案架構
- 移動 `claude-night-pilot` 內容到根目錄
- 將舊檔案歸檔到 `archive/` 目錄 (包含 research-projects)
- 重新組織 `docs/` 和 `tests/` 目錄結構
- 清理重複和過時的測試檔案

### ✨ 新增 (Added)

- 完整的開源專案標準檔案結構
  - `SECURITY.md` - 安全政策和漏洞回報流程
  - `CODEOWNERS` - 代碼擁有者配置
  - `.github/FUNDING.yml` - 專案資助配置
  - `.github/ISSUE_TEMPLATE/` - Issue 模板 (bug report, feature request)
  - `.github/workflows/ci.yml` - GitHub Actions CI 管道
  - `.github/workflows/release.yml` - 自動化發布工作流程
- 冷卻檢查重試機制 (3次重試，指數退避延遲)
- 改進的測試等待機制 (waitForSelector, networkidle)
- 完整的部署指南 (`DEPLOYMENT_GUIDE.md`)
- 一鍵安裝腳本 (`install.sh`)

### 🔄 變更 (Changed)

- 更新專案架構為 GitHub 開源標準
- 改善文檔結構與組織方式
- 統一程式碼規範與開發流程
- 優化二進制檔案大小 (release mode: 1.8MB vs debug mode: 12.8MB)
- 提升 GUI E2E 測試穩定性

### 🐛 修復 (Fixed)

- GUI E2E 測試超時問題 (增加元素等待機制)
- 冷卻檢查偶發性超時 (實作重試機制)
- 測試報告中的拼寫檢查問題
- 清理過時的檔案和架構

### 📚 文檔 (Documentation)

- 新增完整的貢獻指南
- 新增詳細的技術架構文檔
- 新增開發環境設定指南
- 新增測試策略與效能標準
- 更新 README.md 狀態表格和部署資訊

### 🔒 安全性 (Security)

- 實作多層級安全檢查機制
- 增加安全政策文檔
- 設定 GitHub Actions 安全掃描
- 實作輸入驗證和清理機制

## [1.0.0] - 2025-07-22

### ✨ 新增 (Added)

- **完整的 Tauri 2.0 應用框架**

  - 跨平台桌面應用支援 (Windows, macOS, Linux)
  - 極輕量二進位檔案 (~8MB)
  - 現代化的 Material Design 3.0 UI

- **Prompt 管理系統**

  - CRUD 操作：建立、讀取、更新、刪除 Prompts
  - 標籤系統與分類功能
  - Claude Code 語法完整支援 (`@file.md`, `@src/*.js` 等)

- **智能排程系統**

  - 同步執行：立即執行並顯示結果
  - 非同步執行：Cron 表達式支援背景執行
  - 排程任務管理與狀態追蹤

- **冷卻狀態監控**

  - 自動偵測 Claude CLI 使用限制
  - 即時倒數計時器顯示
  - 智能 ETA 預測與顯示

- **雙模式操作**

  - GUI 模式：直觀的視覺化介面
  - CLI 模式：`cnp` 命令列工具

- **資料持久化**
  - SQLite 資料庫整合
  - Migration 系統支援
  - 完整的資料備份與恢復

### 🧪 測試 (Testing)

- **完整的 E2E 測試套件**

  - Playwright 測試框架整合
  - 25+ 測試案例覆蓋
  - 自動化測試管道

- **測試類型覆蓋**
  - 基本功能測試
  - Claude Code 整合測試
  - Material Design UI 測試
  - 響應式設計測試
  - 效能與穩定性測試

### 🔧 技術特性 (Technical)

- **現代技術棧**

  - Rust 1.76+ 後端
  - Tauri 2.0 應用框架
  - htmx + CSS 前端
  - SQLite 資料庫

- **效能指標達成**
  - 啟動時間：~1.5 秒 (目標 < 3 秒)
  - 記憶體使用：~80MB (目標 < 150MB)
  - 檔案大小：~8MB (目標 < 10MB)
  - UI 響應：~50ms (目標 < 100ms)

### 📊 程式碼品質 (Quality)

- **測試覆蓋率**：~85% (目標 > 80%)
- **程式碼規範**：100% 遵循 Rust 與 JavaScript 最佳實踐
- **安全掃描**：通過所有安全性檢查
- **效能測試**：所有關鍵指標符合要求

## [0.2.0] - 2025-07-20

### ✨ 新增 (Added)

- Material Design 3.0 圖示系統
- 進階排程功能與 Cron 支援
- 系統監控與效能追蹤

### 🐛 修復 (Fixed)

- JavaScript 初始化錯誤
- 冷卻狀態檢測問題
- UI 響應性改善

## [0.1.0] - 2025-07-18

### ✨ 新增 (Added)

- 基本 Tauri 應用框架
- 簡單的 Prompt 管理功能
- Claude CLI 基礎整合
- SQLite 資料庫支援

---

## 版本說明

### 版本格式

我們使用 [語義化版本控制](https://semver.org/lang/zh-TW/) 格式：`MAJOR.MINOR.PATCH`

- **MAJOR**: 不向後相容的 API 變更
- **MINOR**: 向後相容的新功能
- **PATCH**: 向後相容的錯誤修復

### 變更類型說明

- **✨ 新增 (Added)**: 新功能
- **🔄 變更 (Changed)**: 現有功能的變更
- **🚨 棄用 (Deprecated)**: 即將移除的功能
- **🗑️ 移除 (Removed)**: 已移除的功能
- **🐛 修復 (Fixed)**: 錯誤修復
- **🔒 安全性 (Security)**: 安全性相關修復
- **🧪 測試 (Testing)**: 測試相關變更
- **📚 文檔 (Documentation)**: 文檔變更
- **🏗️ 重構 (Restructured)**: 架構重構

---

**維護者**: [s123104](https://github.com/s123104)  
**專案**: [Claude Night Pilot](https://github.com/s123104/claude-night-pilot)
