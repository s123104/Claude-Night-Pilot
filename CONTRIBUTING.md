# 🤝 貢獻指南

> 歡迎為 Claude Night Pilot (夜間自動打工仔) 專案貢獻！

## 🎯 如何貢獻

我們歡迎各種形式的貢獻：
- 🐛 **錯誤回報** - 幫助我們發現和修復問題
- 💡 **功能建議** - 提出新功能和改進想法
- 📝 **文檔改進** - 完善使用指南和技術文檔
- 🔧 **代碼貢獻** - 修復錯誤、實現新功能、優化效能
- 🧪 **測試改進** - 增加測試覆蓋率、改進測試品質

## 🚀 快速開始

### 開發環境設定
```bash
# 1. Fork 並克隆專案
git clone https://github.com/your-username/claude-night-pilot.git
cd claude-night-pilot

# 2. 安裝依賴
npm install

# 3. 設定開發環境
cp .env.example .env
# 編輯 .env 檔案配置必要參數

# 4. 啟動開發模式
npm run tauri dev

# 5. 運行測試套件
npm test
```

### 確認環境正常
```bash
# 檢查代碼品質
npm run lint:check

# 檢查類型安全
npm run typecheck

# 運行完整測試
npm run test:all
```

## 📋 代碼規範

### Git 提交規範
採用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**提交類型 (type)**：
- `feat`: 新功能
- `fix`: 錯誤修復  
- `docs`: 文檔變更
- `style`: 代碼格式調整 (不影響功能)
- `refactor`: 代碼重構
- `test`: 測試相關
- `chore`: 建置過程或輔助工具變動

**範圍 (scope)**：
- `core`: 核心功能
- `gui`: 圖形介面
- `cli`: 命令列工具
- `db`: 資料庫
- `scheduler`: 排程系統
- `test`: 測試相關
- `docs`: 文檔
- `ci`: 持續整合

**範例**：
```bash
feat(cli): add prompt execution command
fix(database): resolve connection timeout issue
docs(api): update authentication guide
test(integration): add job scheduling tests
```

### 代碼風格

#### JavaScript/TypeScript
遵循 [ESLint 配置](https://github.com/s123104/claude-night-pilot/blob/main/.eslintrc.json)：

```javascript
// ✅ 良好的代碼風格
const executePrompt = async (promptText) => {
  try {
    const result = await claudeClient.execute(promptText);
    return { success: true, data: result };
  } catch (error) {
    console.error('執行失敗:', error);
    return { success: false, error: error.message };
  }
};

// ❌ 避免的寫法
function exec(p){
  let r=claudeClient.execute(p);
  return r;
}
```

#### Rust
遵循 [Rust 官方風格指南](https://doc.rust-lang.org/style-guide/)：

```rust
// ✅ 良好的 Rust 代碼
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: u64,
    pub status: ExecutionStatus,
    pub output: String,
    pub created_at: DateTime<Utc>,
}

impl ExecutionResult {
    pub fn new(output: String) -> Self {
        Self {
            id: generate_id(),
            status: ExecutionStatus::Completed,
            output,
            created_at: Utc::now(),
        }
    }
}
```

## 🔄 開發流程

### 1. 創建功能分支
```bash
# 從最新的 main 分支創建
git checkout main
git pull origin main
git checkout -b feat/your-feature-name
```

### 2. 開發和測試
```bash
# 開發過程中定期運行檢查
npm run lint:check
npm run typecheck
npm run test

# 提交變更
git add .
git commit -m "feat(scope): add new feature description"
```

### 3. 推送和創建 PR
```bash
# 推送到您的 Fork
git push origin feat/your-feature-name

# 在 GitHub 上創建 Pull Request
```

### 4. Code Review 流程
1. **自動檢查**: CI 系統會自動運行測試和檢查
2. **代碼審查**: 維護者會審查您的代碼
3. **反饋處理**: 根據反饋修改代碼
4. **合併**: 審查通過後合併到 main 分支

## 🐛 錯誤回報

### 回報前的檢查
- [ ] 搜索現有 Issues，確認問題尚未被回報
- [ ] 在最新版本上確認問題仍存在
- [ ] 收集必要的除錯資訊

### 錯誤回報模板
```markdown
**問題描述**
簡潔清楚地描述遇到的問題。

**重現步驟**
1. 執行 '...'
2. 點擊 '....'
3. 捲動到 '....'
4. 看到錯誤

**預期行為**
描述您期望發生的行為。

**實際行為**
描述實際發生的行為。

**環境資訊**
- OS: [如 Windows 10, macOS 14, Ubuntu 22.04]
- Node.js 版本: [如 18.19.0]
- Rust 版本: [如 1.76.0]
- 專案版本: [如 0.1.0]

**額外資訊**
任何其他有助於問題分析的資訊。
```

## 💡 功能建議

### 建議前的考慮
- [ ] 確認功能符合專案目標
- [ ] 檢查是否已有類似建議
- [ ] 考慮實現的複雜度和維護成本

### 功能建議模板
```markdown
**功能概述**
簡潔描述建議的功能。

**問題陳述**
這個功能要解決什麼問題？

**建議解決方案**
詳細描述您建議的解決方案。

**替代方案**
是否考慮過其他替代方案？

**額外資訊**
任何其他相關資訊、範例或參考資料。
```

## 📝 文檔貢獻

### 文檔類型
- **用戶指南**: 幫助用戶使用軟體
- **開發者文檔**: 協助開發者貢獻代碼
- **API 文檔**: 程式介面參考
- **範例和教學**: 實際使用範例

### 文檔標準
請參閱 [品牌指南](docs/BRAND_GUIDE.md) 了解：
- 文檔格式標準
- 語言和風格指南
- Emoji 使用規範
- 品牌識別要求

### 文檔檢查清單
- [ ] 遵循品牌指南格式
- [ ] 內容準確且最新
- [ ] 連結有效可訪問
- [ ] 代碼範例可執行
- [ ] 語言表達清晰

## 🧪 測試指南

### 測試類型
- **單元測試**: 測試個別函數和模組
- **整合測試**: 測試模組間的互動
- **E2E 測試**: 測試完整的用戶工作流程

### 測試要求
- 新功能必須包含對應測試
- 錯誤修復應包含回歸測試
- 測試應該清晰描述測試場景
- 保持測試獨立性和穩定性

### 測試範例
```javascript
// tests/prompt-execution.test.js
import { test, expect } from '@playwright/test';

test.describe('Prompt 執行功能', () => {
  test('成功執行簡單 prompt', async ({ page }) => {
    await page.goto('/');
    await page.fill('[data-testid=prompt-input]', '測試 prompt');
    await page.click('[data-testid=execute-button]');
    
    await expect(page.locator('[data-testid=result]')).toContainText('執行成功');
  });
});
```

## 🛡️ 安全考量

### 安全回報
如果發現安全漏洞，請：
1. **不要** 公開報告安全問題
2. 發送詳細資訊到 [security email]
3. 等待安全團隊回應
4. 協助驗證修復方案

### 安全最佳實踐
- 不要在代碼中硬編碼密鑰或敏感資訊
- 驗證所有用戶輸入
- 使用安全的依賴套件
- 遵循最小權限原則

## 🎉 貢獻者認可

我們重視每一位貢獻者！您的貢獻將會：
- 列入 [貢獻者清單](CONTRIBUTORS.md)
- 在 CHANGELOG 中標註您的貢獻
- 成為專案社群的一員

### 貢獻類型認可
- 🐛 錯誤回報和修復
- 💡 功能建議和實現
- 📝 文檔改進
- 🎨 設計改善
- 🧪 測試增強
- 🔧 工具和基礎設施
- 💬 社群支援

## 📞 獲取幫助

如果您需要幫助：

- **GitHub Discussions**: [專案討論區](https://github.com/s123104/claude-night-pilot/discussions)
- **Issues**: [問題回報](https://github.com/s123104/claude-night-pilot/issues)
- **Documentation**: [完整文檔](docs/)

### 常見問題
- **Q**: 我應該如何開始貢獻？
  **A**: 從小問題開始，熟悉專案結構和流程。

- **Q**: 我的 PR 多久會被審查？
  **A**: 我們通常在 2-5 個工作日內回應 PR。

- **Q**: 如何知道什麼需要幫助？
  **A**: 查看標有 "good first issue" 或 "help wanted" 的 Issues。

## 📄 授權條款

貢獻本專案即表示您同意：
- 您的貢獻將以 [MIT License](LICENSE) 授權
- 您擁有貢獻內容的合法權利
- 遵守專案的 [行為準則](CODE_OF_CONDUCT.md)

---

**感謝您的貢獻！** 🙏

每一個貢獻都讓 Claude Night Pilot 變得更好，讓更多開發者受益。

---

**貢獻指南版本**: v1.0 • **最後更新**: 2025-08 • **維護者**: [Claude Night Pilot Team]