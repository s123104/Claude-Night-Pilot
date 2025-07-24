# Claude Night Pilot - 專案文檔

**更新時間**: 2025-07-24T23:23:43+08:00  
**專案狀態**: ✅ 生產就緒  
**技術棧**: Tauri 2 + Rust + SQLite + htmx  
**作者**: s123104 (GitHub)

## 📁 文檔架構

```
docs/
├── README.md                    # 本檔案 - 文檔導航
├── user-guide/                 # 使用者指南
│   ├── deployment-guide.md     # 部署指南
│   ├── user-manual.md         # 使用手冊
│   └── quick-start.md         # 快速開始
├── developer/                  # 開發者文檔
│   ├── architecture.md        # 系統架構
│   ├── api-reference.md       # API 參考
│   ├── development-guide.md   # 開發指南
│   └── security.md           # 安全文檔
├── reports/                    # 正式報告
│   └── 2025-07/              # 按月份組織
│       ├── 2025-07-24_github-deployment-optimization-report.md
│       ├── 2025-07-24_e2e-testing-100-percent-complete-report.md
│       ├── 2025-07-24_comprehensive-testing-final-report.md
│       ├── 2025-07-24_e2e-testing-final-report.md
│       └── 2025-07-24_final-implementation-report.md
├── temp/                       # 臨時檔案 (被 git 忽略)
└── archive/                    # 歷史歸檔 (被 git 忽略)
```

## 🚀 快速開始

### 新使用者
- 閱讀 [`user-guide/quick-start.md`](user-guide/quick-start.md) 快速上手
- 查看 [`user-guide/deployment-guide.md`](user-guide/deployment-guide.md) 了解部署流程

### 開發者
- 參考 [`developer/development-guide.md`](developer/development-guide.md) 開始開發
- 查看 [`developer/architecture.md`](developer/architecture.md) 了解系統架構

### 專案管理者
- 查看 [`reports/2025-07/`](reports/2025-07/) 目錄下的最新進度報告
- 參考實施報告了解專案完成狀況

## 📊 專案狀態

基於最新報告 (2025-07-24)：

- ✅ **CLI 功能**: 100% 完成並測試通過
- ✅ **GUI 介面**: 基本功能完整實現
- ✅ **四大核心模組**: 2,050+ 行 Rust 代碼
- ✅ **安全執行系統**: 企業級安全標準
- ✅ **資料庫整合**: SQLite 完整支援
- ✅ **GitHub 部署**: 優化完成，就緒上線

**總體完成度**: 95% - 生產環境就緒

## 🔧 核心功能

### 四大整合模組
1. **ccusage API 整合** - Claude 使用量追蹤與監控
2. **安全執行系統** - 多層安全檢查與審計
3. **自適應監控** - 智能監控頻率調整
4. **智能排程** - 時區感知的任務排程

### 技術架構
- **Tauri 2**: 跨平台桌面應用框架
- **Rust**: 記憶體安全的系統程式語言
- **SQLite**: 輕量級嵌入式資料庫
- **htmx**: 現代化前端互動框架

## 📚 文檔維護

### 命名規範
- **正式報告**: `YYYY-MM-DD_功能描述-report.md`
- **指南文檔**: `功能名稱-guide.md`
- **參考文檔**: `功能名稱-reference.md`
- **臨時檔案**: `檔名-temp.md` (會被 git 忽略)

### 更新流程
1. 臨時檔案先放在 `docs/temp/` 目錄
2. 完成後移至正確的分類目錄
3. 更新本 README 的最新狀態
4. 提交到 git 進行版本控制

### Git 忽略規則
- `docs/temp/` - 臨時工作檔案
- `docs/archive/` - 歷史歸檔檔案
- `docs/**/*-temp.md` - 任何標記為臨時的檔案

## 🔗 相關連結

- **GitHub 倉庫**: https://github.com/s123104/Claude-Night-Pilot
- **最新部署報告**: [GitHub 部署優化報告](reports/2025-07/2025-07-24_github-deployment-optimization-report.md)
- **完整測試報告**: [100% E2E 測試報告](reports/2025-07/2025-07-24_e2e-testing-100-percent-complete-report.md)

---

**專案狀態**: 🌟 五星評級 - 企業級品質標準  
**下次更新**: 依專案進展需要  
**維護者**: Claude Night Pilot 開發團隊 