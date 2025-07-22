# Claude Night Pilot 🌙✈️

> 現代 Claude Code 用戶的「夜間自動打工仔」- 零雲端、零安裝痛苦、零學習曲線

## 專案概述

Claude Night Pilot 是一個輕量級桌面應用程式，專為 Claude CLI 用戶設計，提供：

- 📝 **Prompt 模板管理** - 視覺化建立、編輯和組織 Claude prompts
- ⏰ **智能排程執行** - Cron 表達式支援，背景自動執行
- 🔄 **冷卻狀態監控** - 自動偵測 Claude API 限制並顯示剩餘時間
- 💾 **本地資料存儲** - SQLite 單檔資料庫，無雲端依賴
- 🚀 **極輕量設計** - 單一 Binary < 10MB，記憶體占用 < 150MB

## 技術架構

```
┌───────── UI (Webview) ─────────┐
│  HTML + htmx + Pico.css (~20KB) │
│  ├─ Prompt 管理介面             │
│  └─ 任務控制台                   │
└──────────────┬─────────────────┘
               │ Tauri invoke
┌──────────────▼─────────────────┐
│        Rust Backend             │
│  • SQLite 資料庫                │
│  • tokio-cron-scheduler         │
│  • Claude CLI 整合              │
│  • 冷卻監控系統                  │
└─────────────────────────────────┘
```

### 技術棧

- **桌面殼**: Tauri 2 (Rust)
- **前端**: HTML + htmx (14kB) + Pico.css (9kB)
- **資料層**: SQLite (內嵌)
- **排程**: tokio-cron-scheduler
- **測試**: Playwright E2E

## 快速開始

### 前置需求

- Node.js 18+
- Rust 1.76+
- Claude CLI (已安裝並配置)

### 安裝

```bash
# 克隆專案
git clone <repository-url>
cd claude-night-pilot

# 安裝依賴
npm install

# 首次建置
npm run tauri build

# 開發模式
npm run tauri dev
```

### 使用方式

1. **建立 Prompt 模板**

   - 在介面中輸入標題、內容和標籤
   - 支援 Claude Code 語法（如 `@file.md`）

2. **執行模式**

   - **立即執行**: 同步執行並顯示結果
   - **排程執行**: 使用 Cron 表達式設定自動執行

3. **冷卻監控**
   - 自動偵測 Claude API 429/503 錯誤
   - 顯示剩餘冷卻時間倒數
   - 自動重試機制

## 主要功能

### 🎯 Prompt 模板管理

- CRUD 操作：新增、編輯、刪除、搜尋
- 標籤分類系統
- 歷史紀錄保存

### ⚡ 執行引擎

```rust
// 同步執行
claude-pilot execute --prompt-id 1

// 排程執行 (每日 9 點)
claude-pilot schedule --prompt-id 1 --cron "0 9 * * *"
```

### 📊 狀態監控

- 即時冷卻狀態顯示
- 任務執行歷史
- 錯誤日誌追蹤

### 🔄 自動重試

- 偵測到冷卻時自動延遲
- 指數退避策略
- 最大重試次數限制

## 目錄結構

```
claude-night-pilot/
├── src/                    # 前端資源
│   ├── index.html         # 主介面
│   ├── main.js           # JavaScript 邏輯
│   ├── css/pico.min.css  # 樣式框架
│   └── js/htmx.min.js    # 互動框架
├── src-tauri/             # Rust 後端
│   ├── src/
│   │   ├── lib.rs        # 主程式邏輯
│   │   ├── db.rs         # 資料庫層
│   │   ├── executor.rs   # Claude CLI 執行器
│   │   └── scheduler.rs  # 排程器
│   └── Cargo.toml        # Rust 依賴
├── tests/                 # E2E 測試
│   ├── claude-night-pilot.spec.js
│   └── simple-test.spec.js
└── docs/                  # 專案文檔
    └── PROJECT_RULES.md   # 開發規範
```

## 測試

```bash
# 執行完整測試套件
npm test

# 互動式測試
npm run test:ui

# 帶有瀏覽器視窗的測試
npm run test:headed

# 除錯模式
npm run test:debug
```

### 測試覆蓋範圍

- ✅ 應用程式啟動與介面載入
- ✅ Prompt CRUD 操作
- ✅ 同步和非同步執行
- ✅ 排程任務建立與管理
- ✅ 冷卻狀態監控
- ✅ 資料持久化
- ✅ 錯誤處理
- ✅ 響應式設計

## Cron 表達式範例

```bash
# 每小時執行
0 * * * *

# 每日上午 9 點
0 9 * * *

# 週一到週五下午 2 點
0 14 * * 1-5

# 每 30 分鐘
*/30 * * * *

# 每週日午夜
0 0 * * 0
```

## 與 Claude Code 整合

Claude Night Pilot 完全相容 Claude Code 語法：

```javascript
// 支援檔案引用
"@docs/PROJECT_RULES.md 請分析這個專案的架構";

// 支援多檔案操作
"@src/**.js 重構這些 JavaScript 檔案";

// 支援編輯指令
"編輯 config.json 並添加新的設定項目";
```

## 性能指標

| 指標       | 目標值  | 實際值 |
| ---------- | ------- | ------ |
| 安裝包大小 | < 10MB  | ~8MB   |
| 啟動時間   | < 3s    | ~1.5s  |
| 記憶體使用 | < 150MB | ~80MB  |
| 介面響應   | < 100ms | ~50ms  |

## 故障排除

### 常見問題

1. **資料庫錯誤**

   ```bash
   # 重建資料庫
   rm claude-pilot.db
   npm run tauri dev
   ```

2. **Claude CLI 未找到**

   ```bash
   # 檢查 Claude CLI 安裝
   claude --version

   # 確認環境變數
   echo $PATH | grep claude
   ```

3. **權限問題**
   ```bash
   # macOS 需要允許應用程式執行
   xattr -cr claude-night-pilot.app
   ```

## 開發指南

### 建置流程

```bash
# 開發模式（熱重載）
npm run tauri dev

# 生產建置
npm run tauri build

# 僅編譯 Rust 後端
cd src-tauri && cargo build --release
```

### 程式碼規範

- Rust: 遵循 `rustfmt` 標準
- JavaScript: ES6+ 語法，避免大型框架
- CSS: 使用 Pico.css 類別，最小化自定義樣式

### 貢獻指南

1. Fork 專案
2. 建立功能分支
3. 編寫測試
4. 執行完整測試套件
5. 提交 Pull Request

## 授權

MIT License - 詳見 [LICENSE](LICENSE) 檔案

## 更新日誌

### v0.1.0 (2025-01-22)

- ✨ 初始版本發布
- 🎯 完整的 Prompt 管理系統
- ⏰ Cron 排程功能
- 🔄 Claude CLI 冷卻監控
- 🧪 完整的 E2E 測試套件

---

**Claude Night Pilot** - 讓你的 Claude 助手在夜晚也能勤奮工作！ 🌙✨
