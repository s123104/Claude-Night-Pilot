# Claude Night Pilot - MVP 開發指南 🚀

> **目標**: 以最小可行產品 (MVP) 原則快速上手，避免過度工程化

## 🎯 MVP 核心原則

### 優先順序

1. **功能優先**: 先實現核心功能，再優化
2. **簡單優先**: 使用最簡單的解決方案
3. **測試優先**: 確保功能可靠運作
4. **文檔優先**: 記錄重要決策和使用方式

### 不做的事情

- ❌ 過早優化效能
- ❌ 複雜的架構設計
- ❌ 非必要的功能
- ❌ 過度的抽象化

## 🛠️ MVP 開發流程

### 步驟 1: 環境設定 (5 分鐘)

```bash
# 克隆專案
git clone <repository-url>
cd claude-night-pilot

# 安裝依賴
npm install

# 啟動開發模式
npm run tauri dev
```

### 步驟 2: 功能驗證 (10 分鐘)

```bash
# 測試基本功能
1. 開啟應用程式
2. 建立一個 Prompt
3. 執行 Prompt（模擬模式）
4. 查看結果

# 測試 CLI 功能
cnp --help
cnp prompt list
cnp status
```

### 步驟 3: 自定義開發 (30 分鐘)

```bash
# 修改 Prompt 模板
1. 編輯 src/main.js
2. 調整 UI 佈局
3. 測試變更

# 添加新功能
1. 參考現有程式碼
2. 使用簡單的實作方式
3. 先寫測試再寫功能
```

## 📋 MVP 檢查清單

### 基本功能 ✅

- [x] Prompt 建立/編輯/刪除
- [x] 即時執行 Prompt
- [x] 查看執行結果
- [x] 基本的 UI 互動

### 進階功能 (可選)

- [ ] 排程執行
- [ ] 冷卻監控
- [ ] 結果匯出
- [ ] 標籤管理

### 部署就緒

- [ ] 所有測試通過
- [ ] 基本文檔完成
- [ ] 錯誤處理適當
- [ ] 使用者體驗流暢

## 🔧 常用開發指令

```bash
# 開發
npm run tauri dev          # 啟動開發伺服器
npm run lint              # 程式碼檢查
npm test                  # 執行測試

# 建置
npm run tauri build       # 建置應用程式
npm run cli:build         # 建置 CLI 工具

# 除錯
npm run tauri dev -- --verbose  # 詳細輸出
npx playwright test --ui         # 互動式測試
```

## 🚨 常見陷阱

### 避免過度設計

```javascript
// ❌ 過度複雜
class PromptManager extends EventEmitter {
  constructor(config) {
    super();
    this.strategy = new ExecutionStrategy(config);
    this.pipeline = new ProcessingPipeline();
  }
}

// ✅ 簡單直接
function createPrompt(title, content) {
  return { id: Date.now(), title, content };
}
```

### 避免過早優化

```rust
// ❌ 過早優化
async fn execute_with_retry_and_exponential_backoff() {
  // 複雜的重試邏輯
}

// ✅ 先實現基本功能
async fn execute_prompt(prompt: &str) -> Result<String> {
  Command::new("claude").arg(prompt).output()
}
```

## 📚 快速參考

### 文件結構

```
src/
├── index.html     # 主介面
├── main.js        # 核心邏輯
└── styles.css     # 樣式

src-tauri/src/
├── lib.rs         # 主程式
├── db.rs          # 資料庫
└── bin/cnp.rs     # CLI 工具
```

### API 快速參考

```javascript
// Tauri 命令
await invoke('list_prompts');
await invoke('create_prompt', { title, content });
await invoke('run_prompt_sync', { prompt_id });

// CLI 命令
cnp prompt list
cnp prompt create "title" "content"
cnp run 1
```

## 🎯 MVP 成功標準

### 技術標準

- ✅ 應用程式能正常啟動
- ✅ 基本功能運作正常
- ✅ 錯誤不會導致崩潰
- ✅ 回應時間 < 1 秒

### 使用者體驗

- ✅ 介面直觀易用
- ✅ 操作流程順暢
- ✅ 錯誤訊息清楚
- ✅ 符合預期行為

### 可維護性

- ✅ 程式碼結構清晰
- ✅ 關鍵功能有測試
- ✅ 基本文檔完整
- ✅ 容易擴展新功能

---

**記住**: MVP 的目標是快速驗證想法，而不是完美的產品。保持簡單，專注核心價值！ 🌟
