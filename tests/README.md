# Claude Night Pilot 測試架構

本文檔描述了重構後的測試檔案結構，參考 vibe-kanban 的最佳實踐。

## 📁 目錄結構

```
tests/
├── e2e/                          # E2E 測試
│   ├── gui/                     # GUI 功能測試
│   │   ├── material-design-ui.spec.js      # Material Design 介面測試
│   │   ├── prompt-management.spec.js       # Prompt 管理功能
│   │   ├── core-functionality.spec.js      # 核心功能測試
│   │   └── frontend-features.spec.js       # 前端功能測試
│   ├── cli/                     # CLI 功能測試
│   │   ├── basic-commands.spec.js          # 基本 CLI 命令測試
│   │   └── stress-testing.spec.js          # CLI 壓力測試
│   └── cross-platform/          # 跨平台整合測試
│       ├── gui-cli-consistency.spec.js     # GUI-CLI 一致性測試
│       └── unified-interface.spec.js       # 統一介面測試
├── integration/                  # 整合測試
│   ├── claude-integration.spec.js          # Claude Code 整合測試
│   ├── database-integration.spec.js        # 資料庫整合測試
│   ├── module-integration.spec.js          # 模組整合測試
│   ├── system-integration.spec.js          # 系統整合測試
│   └── error-handling.spec.js              # 錯誤處理測試
├── fixtures/                     # 測試夾具和資料
│   ├── mock-data/               # 模擬資料
│   │   ├── sample-prompts.json             # 範例 Prompt 資料
│   │   └── system-status.json              # 系統狀態模擬資料
│   └── test-configs/            # 測試配置
│       └── playwright-test.config.js       # 測試專用配置
├── utils/                        # 共享測試工具
│   ├── test-helpers.js                     # 測試輔助函數
│   ├── mock-claude.js                      # Claude CLI 模擬工具
│   ├── db-setup.js                         # 資料庫測試設定
│   ├── global-setup.js                     # 全域測試初始化
│   └── global-teardown.js                  # 全域測試清理
├── demos/                        # 演示和除錯測試
│   ├── gui-schedule-demo.js                # GUI 排程演示
│   ├── debug-demo.spec.js                  # 除錯演示測試
│   └── simple-debug.spec.js                # 簡單除錯測試
└── README.md                     # 本文檔
```

## 🎯 測試分類

### E2E 測試 (`e2e/`)
端到端測試，驗證完整的使用者工作流程。

#### GUI 測試 (`e2e/gui/`)
- **Material Design UI**: 驗證 Material Design 3.0 元件和主題
- **Prompt Management**: 測試 Prompt 的建立、編輯、刪除功能
- **Core Functionality**: 核心應用功能測試
- **Frontend Features**: 前端特定功能測試

#### CLI 測試 (`e2e/cli/`)
- **Basic Commands**: 測試基本 CLI 命令（init、status、cooldown 等）
- **Stress Testing**: CLI 工具的併發和壓力測試

#### 跨平台測試 (`e2e/cross-platform/`)
- **GUI-CLI Consistency**: 驗證 GUI 和 CLI 功能的一致性
- **Unified Interface**: 統一介面 API 測試

### 整合測試 (`integration/`)
測試各模組之間的整合和互動。

- **Claude Integration**: 與 Claude Code 的整合測試
- **Database Integration**: 資料庫層整合測試
- **Module Integration**: 核心模組間的整合測試
- **System Integration**: 系統級整合測試
- **Error Handling**: 錯誤處理和復原機制測試

### 測試工具 (`utils/`)
共享的測試工具和輔助函數。

- **test-helpers.js**: 通用測試輔助函數
- **mock-claude.js**: Claude CLI 模擬工具
- **db-setup.js**: 資料庫測試環境設定
- **global-setup.js**: 全域測試初始化
- **global-teardown.js**: 全域測試清理

### 測試夾具 (`fixtures/`)
測試資料和配置檔案。

- **mock-data/**: 模擬資料檔案
- **test-configs/**: 測試專用配置

### 演示測試 (`demos/`)
演示腳本和除錯用測試。

## 🚀 執行測試

### 基本命令
```bash
# 執行所有測試
npm test

# 分類執行測試
npm run test:gui              # GUI 測試
npm run test:cli              # CLI 測試
npm run test:integration      # 整合測試
npm run test:cross-platform   # 跨平台測試
npm run test:mobile          # 行動裝置測試

# 開發和除錯
npm run test:ui              # 互動式測試介面
npm run test:headed          # 顯示瀏覽器執行
npm run test:debug           # 除錯模式
npm run test:demos           # 演示測試

# Rust 測試
npm run test:rust            # 所有 Rust 測試
npm run test:rust:unit       # 單元測試
npm run test:rust:integration # 整合測試
```

### 進階用法
```bash
# 只執行特定測試檔案
npx playwright test tests/e2e/gui/prompt-management.spec.js

# 執行特定瀏覽器
npx playwright test --project=mobile-chrome

# 產生覆蓋率報告
npm run test:coverage
```

## 📊 測試報告

測試執行後會在 `coverage/` 目錄生成以下報告：

- `playwright-report/` - HTML 測試報告
- `test-results.json` - JSON 格式測試結果
- `test-summary.md` - 測試摘要報告
- `test-metrics.json` - 測試執行指標

## 🏗️ vibe-kanban 整合參考

本測試架構參考了 vibe-kanban 專案的以下最佳實踐：

### 1. 測試分層架構
- **單元測試**: Rust 源碼內嵌測試 (`#[cfg(test)]`)
- **整合測試**: `src-tauri/tests/` 和 `tests/integration/`
- **E2E 測試**: `tests/e2e/`

### 2. 工具共享
- 共享測試工具在 `tests/utils/`
- 標準化的測試夾具和模擬資料
- 統一的設定和清理流程

### 3. 報告整合
- 多格式測試報告輸出
- 自動化測試摘要生成
- CI/CD 整合支援

## 🔧 配置說明

### Playwright 配置
- 主配置檔案: `playwright.config.js`
- 測試專用配置: `tests/fixtures/test-configs/playwright-test.config.js`

### 環境變數
- `TEST_MODE=true` - 測試模式標記
- `USE_MOCK_CLI=true` - 使用模擬 CLI（當實際 CLI 不可用時）
- `INCLUDE_DEMOS=true` - 包含演示測試

### 全域設定
- **Setup**: 自動初始化測試資料庫和環境
- **Teardown**: 自動清理測試資料和生成報告

## 🚨 注意事項

1. **資料庫隔離**: 測試使用獨立的測試資料庫，不會影響開發資料
2. **並行執行**: E2E 測試支援並行執行，但要注意資源競爭
3. **Mock 策略**: CLI 不可用時自動啟用模擬模式
4. **覆蓋率**: 目標維持 90%+ 的測試覆蓋率

## 📚 相關文檔

- [CLAUDE.md](../CLAUDE.md) - 專案開發指南
- [Testing Guide](../docs/user-guide/TESTING_GUIDE.md) - 詳細測試指南
- [vibe-kanban 整合報告](../research-projects/vibe-kanban/Claude-Night-Pilot-Vibe-Kanban-完整整合報告.md)

---

## 🤝 貢獻指南

當添加新測試時：

1. 選擇合適的分類目錄
2. 使用共享的測試工具和夾具
3. 遵循現有的命名慣例
4. 添加適當的文檔說明
5. 確保測試能夠獨立執行

測試架構遵循「測試金字塔」原則：
- **底層**: 大量快速的單元測試
- **中層**: 適量的整合測試
- **頂層**: 少量但全面的 E2E 測試