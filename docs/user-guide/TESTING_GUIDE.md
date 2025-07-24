# Claude Night Pilot - 測試指南 🧪

> **目標**: 確保應用程式品質與可靠性的完整測試策略

## 🎯 測試哲學

### 測試金字塔

```
       /\
      /  \     E2E Tests (少量但全面)
     /____\
    /      \   Integration Tests (中等數量)
   /________\
  /          \ Unit Tests (大量但快速)
 /__________\
```

### 測試原則

1. **快速回饋**: 測試應該快速執行
2. **可靠性**: 測試結果一致且可重現
3. **可讀性**: 測試案例清楚易懂
4. **維護性**: 易於更新和擴展

## 🧪 測試分層

### E2E 測試 (Playwright)

- **目標**: 驗證完整的使用者工作流程
- **工具**: Playwright
- **範圍**: GUI 互動、API 整合、CLI 工具

### 整合測試 (Rust)

- **目標**: 驗證模組間的協作
- **工具**: Rust 內建測試框架
- **範圍**: 資料庫操作、API 呼叫

### 單元測試 (Rust + JavaScript)

- **目標**: 驗證單個函數的正確性
- **工具**: Rust 測試 + Jest (如需要)
- **範圍**: 純函數、業務邏輯

## 🚀 快速開始

### 環境設定

```bash
# 安裝測試依賴
npm install

# 檢查測試環境
npx playwright --version
npm run tauri dev --help
```

### 執行測試

```bash
# 執行所有測試
npm test

# 互動式測試（推薦）
npm run test:ui

# 特定測試檔案
npx playwright test tests/claude-night-pilot.spec.js

# 調試模式
npm run test:debug
```

## 📝 測試案例規劃

### GUI 功能測試

- **應用程式啟動**

  - [x] 視窗正確載入
  - [x] 初始狀態正確
  - [x] UI 元素可見

- **Prompt 管理**

  - [x] 建立新 Prompt
  - [x] 編輯現有 Prompt
  - [x] 刪除 Prompt
  - [x] 搜尋 Prompt

- **執行功能**
  - [x] 同步執行
  - [x] 結果顯示
  - [x] 錯誤處理

### CLI 功能測試

- **基本指令**

  - [x] `cnp --help`
  - [x] `cnp prompt list`
  - [x] `cnp status`

- **Prompt 操作**

  - [x] `cnp prompt create`
  - [x] `cnp prompt show`
  - [x] `cnp prompt delete`

- **執行與監控**
  - [x] `cnp run`
  - [x] `cnp cooldown`
  - [x] `cnp results`

### Claude Code 整合測試

- **冷卻檢測**

  - [x] 解析錯誤訊息
  - [x] 時間倒數顯示
  - [x] 自動重試機制

- **語法支援**
  - [x] 檔案引用 (`@file.md`)
  - [x] 多檔案操作
  - [x] 編輯指令

## 📊 測試報告

### 涵蓋率目標

- **E2E 測試**: 100% 主要使用者流程
- **整合測試**: 80% 模組交互
- **單元測試**: 90% 核心邏輯

### 效能指標

- **測試執行時間**: < 5 分鐘 (完整套件)
- **啟動時間**: < 3 秒
- **回應時間**: < 100ms (UI 操作)

## 🔧 測試工具配置

### Playwright 設定

```javascript
// playwright.config.js
module.exports = {
  testDir: "./tests",
  timeout: 30000,
  expect: {
    timeout: 5000,
  },
  use: {
    headless: true,
    viewport: { width: 1280, height: 720 },
    actionTimeout: 0,
    ignoreHTTPSErrors: true,
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
};
```

### Rust 測試設定

```toml
# Cargo.toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
serial_test = "3.0"
```

## 🎭 Mock 策略

### 開發模式 Mock

```javascript
// src-tauri/src/lib.rs
#[cfg(debug_assertions)]
#[tauri::command]
async fn list_prompts() -> Result<Vec<Prompt>, String> {
    // 返回模擬資料
    Ok(vec![
        Prompt {
            id: 1,
            title: "範例 Prompt".to_string(),
            content: "分析這個專案".to_string(),
            tags: Some("測試".to_string()),
            created_at: Some("2025-07-22T22:14:06+08:00".to_string()),
        }
    ])
}
```

### Claude CLI Mock

```rust
// 模擬 Claude CLI 回應
async fn mock_claude_response(prompt: &str) -> String {
    format!("模擬回應：{}", prompt)
}
```

## 🔍 調試技巧

### Playwright 調試

```bash
# 開啟調試器
npx playwright test --debug

# 慢動作執行
npx playwright test --slow-mo=1000

# 產生測試錄製
npx playwright codegen http://localhost:1420
```

### Rust 調試

```bash
# 詳細測試輸出
cargo test -- --nocapture

# 執行特定測試
cargo test test_create_prompt

# 測試覆蓋率
cargo tarpaulin --out Html
```

## 📋 測試檢查清單

### 測試前檢查

- [ ] 開發環境正常運作
- [ ] 依賴安裝完成
- [ ] 資料庫狀態乾淨
- [ ] 環境變數設定正確

### 測試執行

- [ ] 所有測試通過
- [ ] 無測試跳過
- [ ] 效能指標達標
- [ ] 錯誤處理正確

### 測試後檢查

- [ ] 測試報告產生
- [ ] 涵蓋率達標
- [ ] 清理測試資料
- [ ] 記錄測試結果

## ⚠️ 常見問題

### 測試不穩定

```bash
# 檢查測試環境
npm run test:debug

# 確認時間設定
date

# 清理測試資料
rm -f test-results/
```

### Tauri 應用無法啟動

```bash
# 檢查 Rust 編譯
cd src-tauri && cargo check

# 檢查 Node.js 依賴
npm install

# 檢查連接埠占用
lsof -i :1420
```

### Claude CLI 測試失敗

```bash
# 確認 Claude CLI 安裝
claude --version

# 檢查 API 金鑰
echo $ANTHROPIC_API_KEY

# 使用模擬模式
export CLAUDE_MOCK_MODE=true
```

## 📈 持續改進

### 測試指標追蹤

- 測試執行時間趨勢
- 失敗率統計
- 涵蓋率變化
- 效能回歸檢測

### 測試策略優化

- 定期檢視測試案例
- 移除重複測試
- 增加邊界案例
- 改善測試效率

---

**記住**: 好的測試不只是驗證程式碼正確性，更是確保使用者體驗的品質保證！ ✨
