# 🤝 貢獻指南 - Claude Night Pilot

> **歡迎貢獻！** 感謝您對 Claude Night Pilot 的興趣。本指南將協助您順利參與專案貢獻。

## 📋 目錄

1. [開始貢獻](#開始貢獻)
2. [開發環境設定](#開發環境設定)
3. [專案架構](#專案架構)
4. [開發工作流程](#開發工作流程)
5. [程式碼規範](#程式碼規範)
6. [測試指南](#測試指南)
7. [提交 Pull Request](#提交-pull-request)
8. [報告問題](#報告問題)

---

## 🚀 開始貢獻

### 貢獻類型

我們歡迎以下類型的貢獻：

- 🐛 **Bug 修復** - 發現並修復程式錯誤
- ✨ **新功能** - 實作有價值的新功能
- 📚 **文檔改善** - 改善文檔品質
- 🎨 **UI/UX 改善** - 提升使用者體驗
- ⚡ **效能優化** - 提升應用效能
- 🧪 **測試覆蓋** - 增加測試案例

### 貢獻者行為準則

- **友善互助** - 尊重所有貢獻者
- **建設性討論** - 提供具體、有幫助的回饋
- **品質導向** - 確保程式碼品質與文檔完整性
- **學習分享** - 分享知識，互相學習

---

## 🛠️ 開發環境設定

### 系統需求

| 工具        | 版本要求 | 說明                  |
| ----------- | -------- | --------------------- |
| **Rust**    | 1.70+    | 後端邏輯與 Tauri 應用 |
| **Node.js** | 18+      | 前端開發與工具鏈      |
| **npm**     | 9+       | 套件管理              |
| **Git**     | 2.40+    | 版本控制              |

### 快速開始

```bash
# 1. Fork 並克隆專案
git clone https://github.com/YOUR_USERNAME/claude-night-pilot.git
cd claude-night-pilot

# 2. 安裝依賴
npm install

# 3. 安裝 Rust (如果尚未安裝)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 4. 啟動開發環境
npm run dev

# 5. 執行測試
npm test
```

### 開發工具推薦

- **IDE**: VS Code + Rust 擴充套件
- **除錯**: Tauri DevTools
- **測試**: Playwright Test Runner
- **格式化**: Rustfmt + Prettier

---

## 🏗️ 專案架構

### 目錄結構

```
claude-night-pilot/
├── 📁 src/                    # 前端靜態檔案
│   ├── index.html             # 主要 HTML 檔案
│   ├── style.css              # 樣式檔案
│   └── script.js              # JavaScript 邏輯
├── 📁 src-tauri/              # Tauri 後端
│   ├── src/                   # Rust 原始碼
│   ├── Cargo.toml             # Rust 專案配置
│   └── tauri.conf.json        # Tauri 配置
├── 📁 tests/                  # E2E 測試
├── 📁 docs/                   # 專案文檔
├── 📁 archive/                # 歸檔檔案
├── README.md                  # 專案說明
├── CONTRIBUTING.md            # 本檔案
├── CHANGELOG.md               # 變更日誌
└── PROJECT_RULES.md           # 專案規則
```

### 技術棧

- **前端**: HTML + CSS + JavaScript (htmx)
- **後端**: Rust + Tauri 2.0
- **資料庫**: SQLite
- **測試**: Playwright
- **建置**: Cargo + npm

---

## 🔄 開發工作流程

### Git 工作流程

```bash
# 1. 建立功能分支
git checkout -b feature/your-feature-name

# 2. 進行開發
# ... 編輯檔案 ...

# 3. 提交變更
git add .
git commit -m "feat: add your feature description"

# 4. 推送分支
git push origin feature/your-feature-name

# 5. 建立 Pull Request
```

### 分支命名規範

| 類型     | 格式                   | 範例                             |
| -------- | ---------------------- | -------------------------------- |
| **功能** | `feature/description`  | `feature/add-export-function`    |
| **修復** | `fix/description`      | `fix/database-connection-error`  |
| **文檔** | `docs/description`     | `docs/update-api-guide`          |
| **重構** | `refactor/description` | `refactor/reorganize-components` |

### 提交訊息規範

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 規範：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**範例**:

```
feat(prompt): add prompt export functionality

- Add CSV export option
- Add JSON export option
- Update UI with export buttons

Closes #123
```

**提交類型**:

- `feat`: 新功能
- `fix`: Bug 修復
- `docs`: 文檔變更
- `style`: 格式化變更
- `refactor`: 重構
- `test`: 測試相關
- `chore`: 建置或輔助工具變更

---

## 📏 程式碼規範

### Rust 程式碼規範

```rust
// ✅ 良好的範例
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Prompt {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

impl Prompt {
    /// 建立新的 Prompt 實例
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: 0, // 將由資料庫分配
            title,
            content,
            tags: Vec::new(),
        }
    }
}
```

**Rust 規範重點**:

- 使用 `rustfmt` 自動格式化
- 遵循 Rust 命名慣例 (snake_case, CamelCase)
- 提供完整的文檔註解
- 處理所有 `Result` 類型
- 避免 `unwrap()`，使用適當的錯誤處理

### JavaScript 程式碼規範

```javascript
// ✅ 良好的範例
class PromptManager {
  constructor() {
    this.prompts = new Map();
    this.init();
  }

  /**
   * 初始化 Prompt 管理器
   */
  async init() {
    try {
      await this.loadPrompts();
      this.setupEventListeners();
    } catch (error) {
      console.error("初始化失敗:", error);
    }
  }

  /**
   * 載入所有 Prompts
   * @returns {Promise<Array>} Prompts 陣列
   */
  async loadPrompts() {
    // 實作邏輯
  }
}
```

**JavaScript 規範重點**:

- 使用 ES6+ 語法
- 提供 JSDoc 註解
- 適當的錯誤處理
- 清晰的變數命名
- 避免全域變數

---

## 🧪 測試指南

### 測試類型

1. **單元測試** (Rust)

```bash
cd src-tauri
cargo test
```

2. **E2E 測試** (Playwright)

```bash
npm test
```

3. **手動測試**

```bash
npm run dev
```

### 測試撰寫指南

```javascript
// E2E 測試範例
test("建立新 Prompt", async ({ page }) => {
  // 前往應用
  await page.goto("http://localhost:1420");

  // 點擊建立按鈕
  await page.click('[data-testid="create-prompt-btn"]');

  // 填寫表單
  await page.fill('[data-testid="prompt-title"]', "測試 Prompt");
  await page.fill('[data-testid="prompt-content"]', "測試內容");

  // 提交表單
  await page.click('[data-testid="submit-btn"]');

  // 驗證結果
  await expect(page.locator('[data-testid="prompt-list"]')).toContainText(
    "測試 Prompt"
  );
});
```

---

## 🚀 提交 Pull Request

### PR 檢查清單

- [ ] **程式碼品質**

  - [ ] 遵循程式碼規範
  - [ ] 通過所有測試
  - [ ] 無 lint 錯誤
  - [ ] 適當的錯誤處理

- [ ] **文檔**

  - [ ] 更新相關文檔
  - [ ] 添加/更新註解
  - [ ] 更新 CHANGELOG.md

- [ ] **測試**

  - [ ] 添加對應測試案例
  - [ ] 現有測試通過
  - [ ] 手動測試驗證

- [ ] **提交品質**
  - [ ] 清晰的提交訊息
  - [ ] 合理的變更範圍
  - [ ] 解決相關 issue

### PR 模板

建立 PR 時請使用以下模板：

```markdown
## 📝 變更摘要

簡要描述此 PR 的變更內容

## 🎯 相關 Issue

- Closes #123
- Related to #456

## 🧪 測試

- [ ] 單元測試通過
- [ ] E2E 測試通過
- [ ] 手動測試驗證

## 📸 截圖 (如適用)

[添加截圖展示 UI 變更]

## 📋 檢查清單

- [ ] 遵循程式碼規範
- [ ] 更新相關文檔
- [ ] 添加適當測試
- [ ] 通過所有檢查
```

---

## 🐛 報告問題

### Issue 類型

1. **Bug 報告** - 程式錯誤
2. **功能請求** - 新功能建議
3. **問題詢問** - 使用上的疑問
4. **文檔改善** - 文檔相關建議

### Bug 報告範本

```markdown
## 🐛 Bug 描述

清楚簡潔地描述 bug

## 🔄 重現步驟

1. 前往 '...'
2. 點擊 '...'
3. 向下滾動到 '...'
4. 看到錯誤

## ✅ 預期行為

描述您預期應該發生的情況

## 📸 截圖

如果適用，請添加截圖來協助解釋問題

## 💻 環境資訊

- OS: [e.g. macOS 13.0]
- 版本: [e.g. v1.0.0]
- 瀏覽器: [e.g. Chrome 118]

## 📄 額外資訊

添加任何其他與問題相關的資訊
```

---

## 🎉 感謝貢獻！

感謝您花時間閱讀本指南並考慮為 Claude Night Pilot 貢獻。您的每一個貢獻都讓這個專案變得更好！

如有任何問題，歡迎在 [GitHub Issues](https://github.com/s123104/claude-night-pilot/issues) 中提出。

---

**Happy Coding! 🚀**
