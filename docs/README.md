# Claude Night Pilot 文檔中心

完整的 Claude Night Pilot 使用與開發文檔。

## 📖 文檔結構

### 用戶指南 ([user-guide/](user-guide/))
- **[安裝指南](user-guide/installation.md)** - 詳細安裝步驟與系統需求
- **[快速入門](user-guide/quick-start.md)** - 5分鐘快速上手
- **[GUI 使用指南](user-guide/gui-usage.md)** - 桌面應用程式完整操作
- **[CLI 使用指南](user-guide/cli-usage.md)** - 命令列工具參考
- **[排程建立指南](user-guide/schedule-creation.md)** - 任務排程設定
- **[部署指南](user-guide/DEPLOYMENT_GUIDE.md)** - 生產環境部署
- **[測試指南](user-guide/TESTING_GUIDE.md)** - 測試與品質保證

### 開發者資源 ([developer/](developer/))
- **[架構文檔](developer/architecture.md)** - 系統設計與架構說明
- **[API 參考](developer/api-reference.md)** - 完整 API 文檔
- **[整合指南](developer/INTEGRATION_IMPLEMENTATION_GUIDE.md)** - 第三方整合開發
- **[安全實作](developer/SECURITY_IMPLEMENTATION_REPORT.md)** - 安全機制說明
- **[專案規則](developer/PROJECT_RULES.md)** - 開發規範與最佳實踐

### 技術報告 ([reports/](reports/))
- **[測試框架報告](reports/testing-framework.md)** - 測試架構與策略
- **[效能分析報告](reports/FINAL_PERFORMANCE_ANALYSIS.md)** - 效能指標與優化
- **[資料庫重構報告](reports/database-refactoring.md)** - 資料庫架構改進
- **[GUI 測試結果](reports/gui-test-results.md)** - 使用介面測試分析

### 範例與教學 ([examples/](examples/))
- **[GUI 排程範例](examples/gui-schedule-demo.js)** - 排程功能示範
- **基本使用範例** - 常見使用場景
- **進階整合範例** - 複雜整合案例

## 🚀 快速導覽

### 新手用戶
1. [安裝指南](user-guide/installation.md) - 開始安裝
2. [快速入門](user-guide/quick-start.md) - 學習基本操作  
3. [GUI 使用指南](user-guide/gui-usage.md) - 圖形介面使用

### 進階用戶  
1. [CLI 使用指南](user-guide/cli-usage.md) - 命令列操作
2. [排程建立指南](user-guide/schedule-creation.md) - 自動化設定
3. [部署指南](user-guide/DEPLOYMENT_GUIDE.md) - 生產部署

### 開發者
1. [架構文檔](developer/architecture.md) - 了解系統設計
2. [API 參考](developer/api-reference.md) - 程式介面文檔
3. [整合指南](developer/INTEGRATION_IMPLEMENTATION_GUIDE.md) - 開發整合

## 🎯 專案狀態概覽

| 模組 | 狀態 | 測試覆蓋 | 備註 |
|------|------|----------|------|
| CLI工具 | ✅ 完成 | 92% | 生產就緒，效能優化完成 |
| GUI界面 | ✅ 完成 | 80% | Material Design 3.0 |
| 資料庫 | ✅ 完成 | 95% | SQLite + 最佳實踐 |
| 文檔系統 | ✅ 完成 | 100% | 遵循 vibe-kanban 最佳實踐 |

## 🔍 文檔導航

**按功能分類**:
- **安裝設定**: [安裝](user-guide/installation.md) → [快速入門](user-guide/quick-start.md)
- **使用操作**: [GUI](user-guide/gui-usage.md) → [CLI](user-guide/cli-usage.md) → [排程](user-guide/schedule-creation.md)
- **開發整合**: [架構](developer/architecture.md) → [API](developer/api-reference.md) → [整合](developer/INTEGRATION_IMPLEMENTATION_GUIDE.md)
- **問題解決**: [常見問題](faq.md) → [測試指南](user-guide/TESTING_GUIDE.md)

**按用戶角色分類**:
- **終端用戶**: [快速入門](user-guide/quick-start.md) → [GUI 指南](user-guide/gui-usage.md) → [常見問題](faq.md)
- **系統管理員**: [安裝指南](user-guide/installation.md) → [部署指南](user-guide/DEPLOYMENT_GUIDE.md) → [安全實作](developer/SECURITY_IMPLEMENTATION_REPORT.md)  
- **開發者**: [架構文檔](developer/architecture.md) → [API 參考](developer/api-reference.md) → [專案規則](developer/PROJECT_RULES.md)

## 📋 參考資源

- **[主專案 README](../README.md)** - 專案總覽
- **[CLAUDE.md](../CLAUDE.md)** - Claude Code 專用開發指南
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - 貢獻指南
- **[常見問題 FAQ](faq.md)** - 問題解答
- **[變更日誌](../CHANGELOG.md)** - 版本更新記錄

## 🔄 文檔維護

文檔遵循以下原則：
- **時效性**: 隨專案更新同步維護
- **一致性**: 統一格式與術語規範  
- **完整性**: 涵蓋所有核心功能
- **實用性**: 提供具體可操作的指導

**最後更新**: 2025-08 • **文檔版本**: v1.0

---

如有文檔問題或建議，歡迎提交 [GitHub Issue](https://github.com/s123104/claude-night-pilot/issues)。 