# 🔧 [模組名稱] 開發者指南

> 為 [專案名稱] 貢獻代碼和擴展功能的完整指南

## 🎯 開發目標

本指南幫助開發者：
- ✅ 理解專案架構和設計原則
- ✅ 設定開發環境和工具鏈
- ✅ 遵循代碼規範和最佳實踐
- ✅ 實施測試驅動開發流程
- ✅ 貢獻高品質的代碼和文檔

## 🏗️ 系統架構

### 技術棧概覽
```
Frontend: [前端技術棧]
├── Framework: [主要框架]
├── State Management: [狀態管理]
├── Styling: [樣式系統]
└── Build Tools: [建置工具]

Backend: [後端技術棧]
├── Runtime: [執行環境]
├── Framework: [後端框架]
├── Database: [資料庫系統]
└── Authentication: [認證系統]

Infrastructure: [基礎設施]
├── Deployment: [部署平台]
├── Monitoring: [監控系統]
├── CI/CD: [持續整合]
└── Security: [安全工具]
```

### 目錄結構
```
project-root/
├── src/                    # 前端源碼
│   ├── components/         # 可複用組件
│   ├── pages/             # 頁面組件
│   ├── hooks/             # 自定義 Hooks
│   ├── utils/             # 工具函數
│   └── styles/            # 樣式檔案
├── src-tauri/             # 後端源碼
│   ├── src/               # Rust 源碼
│   ├── migrations/        # 資料庫遷移
│   └── tests/             # 後端測試
├── tests/                 # E2E 測試
├── docs/                  # 文檔
├── scripts/               # 開發腳本
└── config/                # 配置檔案
```

### 核心模組
#### [模組1名稱] (`src/[module1]/`)
**職責**：[模組功能描述]
**主要文件**：
- `[file1].js` - [檔案職責]
- `[file2].js` - [檔案職責]

#### [模組2名稱] (`src-tauri/src/[module2]/`)
**職責**：[模組功能描述]
**主要文件**：
- `[file1].rs` - [檔案職責]
- `[file2].rs` - [檔案職責]

## 🚀 開發環境設定

### 前置需求
確認安裝以下工具：
```bash
# Node.js (建議使用 LTS 版本)
node --version  # >= 18.0.0

# Rust (最新穩定版)
rustc --version  # >= 1.76.0

# 其他必要工具
npm --version
cargo --version
git --version
```

### 專案設定
```bash
# 1. 克隆專案
git clone https://github.com/[username]/[project-name].git
cd [project-name]

# 2. 安裝依賴
npm install
cd src-tauri && cargo build

# 3. 設定環境變數
cp .env.example .env
# 編輯 .env 檔案配置必要參數

# 4. 初始化資料庫
npm run db:init

# 5. 啟動開發服務
npm run dev
```

### IDE 設定建議
#### VS Code
推薦擴充功能：
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tauri-apps.tauri-vscode",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-playwright.playwright"
  ]
}
```

#### 設定檔案 (`.vscode/settings.json`)
```json
{
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "rust-analyzer.cargo.features": "all"
}
```

## 📝 代碼規範

### JavaScript/TypeScript 規範
遵循 [ESLint 配置](../.eslintrc.json) 和 [Prettier 設定](../.prettierrc)

**命名慣例**：
```javascript
// 變數和函數：camelCase
const userData = getUserData();
const handleSubmit = () => {};

// 常數：UPPER_SNAKE_CASE
const API_BASE_URL = 'https://api.example.com';

// 組件：PascalCase
const UserProfile = () => {};

// 檔案名稱：kebab-case
// user-profile.js, api-client.js
```

**函數撰寫原則**：
```javascript
// ✅ 好的函數設計
const calculateTotal = (items) => {
  return items.reduce((sum, item) => sum + item.price, 0);
};

// ❌ 避免的寫法
const calc = (x) => {
  let t = 0;
  for (let i = 0; i < x.length; i++) {
    t += x[i].p;
  }
  return t;
};
```

### Rust 代碼規範
遵循 [Rust 官方風格指南](https://doc.rust-lang.org/style-guide/)

**結構定義**：
```rust
// ✅ 良好的結構設計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: u64,
    pub status: ExecutionStatus,
    pub output: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}
```

**錯誤處理**：
```rust
// ✅ 使用 Result 和 anyhow
use anyhow::{Context, Result};

pub async fn execute_prompt(prompt: &str) -> Result<ExecutionResult> {
    let result = claude_client
        .execute(prompt)
        .await
        .context("Failed to execute Claude prompt")?;
    
    Ok(ExecutionResult {
        status: ExecutionStatus::Completed,
        output: result,
        created_at: Utc::now(),
    })
}
```

### Git 提交規範
使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

```bash
# 格式
<type>(<scope>): <description>

# 範例
feat(cli): add prompt execution command
fix(database): resolve connection timeout issue
docs(api): update authentication guide
test(integration): add job scheduling tests
```

**提交類型**：
- `feat`: 新功能
- `fix`: 錯誤修復
- `docs`: 文檔變更
- `style`: 代碼格式調整
- `refactor`: 代碼重構
- `test`: 測試相關
- `chore`: 雜項工作

## 🧪 測試策略

### 測試金字塔
```
    E2E Tests (10%)
   ┌─────────────────┐
   │   Playwright    │
   └─────────────────┘
  Integration Tests (20%)
 ┌───────────────────────┐
 │    Rust + JS APIs     │
 └───────────────────────┘
Unit Tests (70%)
┌─────────────────────────────┐
│ Jest + Rust cargo test      │
└─────────────────────────────┘
```

### 單元測試
#### JavaScript 測試
```javascript
// tests/utils/date-formatter.test.js
import { formatDate } from '../src/utils/date-formatter';

describe('formatDate', () => {
  test('formats date correctly', () => {
    const date = new Date('2025-08-09T10:30:00Z');
    expect(formatDate(date)).toBe('2025-08-09');
  });

  test('handles invalid date', () => {
    expect(() => formatDate('invalid')).toThrow('Invalid date');
  });
});
```

#### Rust 測試
```rust
// src-tauri/src/executor.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_execute_prompt_success() {
        let executor = ClaudeExecutor::new();
        let result = executor.execute("test prompt").await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ExecutionStatus::Completed);
    }
    
    #[test]
    fn test_parse_cron_expression() {
        let expr = "0 9 * * *";
        let parsed = parse_cron(expr);
        
        assert!(parsed.is_ok());
    }
}
```

### 整合測試
```javascript
// tests/integration/api.test.js
import { test, expect } from '@playwright/test';

test.describe('API Integration', () => {
  test('prompt CRUD operations', async ({ request }) => {
    // Create
    const createResponse = await request.post('/api/prompts', {
      data: { name: 'Test Prompt', content: 'Hello world' }
    });
    expect(createResponse.ok()).toBeTruthy();
    
    const prompt = await createResponse.json();
    
    // Read
    const getResponse = await request.get(`/api/prompts/${prompt.id}`);
    expect(getResponse.ok()).toBeTruthy();
    
    // Delete
    const deleteResponse = await request.delete(`/api/prompts/${prompt.id}`);
    expect(deleteResponse.ok()).toBeTruthy();
  });
});
```

### E2E 測試
```javascript
// tests/e2e/prompt-management.spec.js
import { test, expect } from '@playwright/test';

test('完整 Prompt 管理工作流程', async ({ page }) => {
  // 導航到主頁
  await page.goto('/');
  
  // 創建新 Prompt
  await page.click('text=新增 Prompt');
  await page.fill('[data-testid=prompt-name]', 'E2E 測試 Prompt');
  await page.fill('[data-testid=prompt-content]', '這是測試內容');
  await page.click('text=儲存');
  
  // 驗證創建成功
  await expect(page.locator('text=E2E 測試 Prompt')).toBeVisible();
  
  // 執行 Prompt
  await page.click('[data-testid=execute-prompt]');
  await expect(page.locator('text=執行成功')).toBeVisible();
});
```

### 測試執行
```bash
# 單元測試
npm run test:unit
npm run test:rust

# 整合測試
npm run test:integration

# E2E 測試
npm run test:e2e
npm run test:e2e:headed

# 完整測試套件
npm run test:all

# 測試覆蓋率
npm run test:coverage
```

## 🔄 開發工作流程

### 功能開發流程
1. **創建分支**
   ```bash
   git checkout -b feat/prompt-templates
   ```

2. **開發功能**
   - 遵循 TDD：先寫測試，再實作功能
   - 定期提交：小步快進，頻繁提交
   - 運行測試：確保不破壞現有功能

3. **代碼審查**
   ```bash
   # 自我檢查
   npm run lint:check
   npm run test:all
   npm run typecheck
   
   # 提交 PR
   git push origin feat/prompt-templates
   ```

4. **合併代碼**
   - PR 審查通過
   - CI/CD 檢查通過
   - 合併到 main 分支

### 發布流程
```bash
# 1. 更新版本號
npm version patch|minor|major

# 2. 更新 CHANGELOG
npm run changelog

# 3. 建置發布版本
npm run build:release

# 4. 創建 Git 標籤
git tag v1.2.3
git push --tags

# 5. 發布到平台
npm run release:github
```

## 🔧 除錯和優化

### 除錯工具
#### 前端除錯
```javascript
// 開發模式日誌
console.debug('Debug info:', data);
console.info('Information:', status);
console.warn('Warning:', warning);
console.error('Error:', error);

// 效能監控
console.time('expensive-operation');
performExpensiveOperation();
console.timeEnd('expensive-operation');
```

#### 後端除錯
```rust
// 使用 tracing crate
use tracing::{debug, info, warn, error};

#[tracing::instrument]
async fn execute_command(cmd: &str) -> Result<String> {
    debug!("Executing command: {}", cmd);
    
    let result = run_command(cmd).await?;
    
    info!("Command executed successfully, output length: {}", result.len());
    Ok(result)
}
```

### 效能優化
#### 前端優化
```javascript
// Code Splitting
const LazyComponent = lazy(() => import('./LazyComponent'));

// Memoization
const ExpensiveComponent = memo(({ data }) => {
  const processedData = useMemo(() => 
    heavyProcessing(data), [data]
  );
  
  return <div>{processedData}</div>;
});
```

#### 後端優化
```rust
// 資料庫連線池
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;

// 非同步處理
use tokio::task::JoinSet;

let mut set = JoinSet::new();
for item in items {
    set.spawn(process_item(item));
}
```

## 📦 部署指南

### 本地建置
```bash
# 前端建置
npm run build:frontend

# 後端建置
npm run build:backend

# 完整應用程式建置
npm run tauri build
```

### Docker 部署
```dockerfile
# Dockerfile
FROM node:18-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --only=production
COPY src/ ./src/
RUN npm run build

FROM rust:1.76-slim AS backend
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src-tauri/ ./src-tauri/
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=backend /app/target/release/app /usr/local/bin/
CMD ["app"]
```

### CI/CD 配置
```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run lint:check
      - run: npm run test:all
      
  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm run tauri build
```

## 🤝 貢獻指南

### 開始貢獻
1. **Fork 專案** 到您的 GitHub 帳戶
2. **創建分支** 進行功能開發
3. **遵循代碼規範** 和測試要求
4. **提交 Pull Request** 並描述變更內容
5. **參與 Code Review** 並根據反饋調整

### PR 檢查清單
- [ ] 代碼遵循專案規範
- [ ] 包含適當的測試覆蓋
- [ ] 文檔更新 (如適用)
- [ ] 通過所有 CI 檢查
- [ ] PR 描述清晰詳細

### 代碼審查指南
#### 審查者
- 檢查代碼邏輯正確性
- 確認測試覆蓋充分
- 驗證效能影響
- 確保文檔完整性

#### 提交者
- 接受建設性回饋
- 及時回應審查意見
- 解釋設計決策
- 保持禮貌和專業

## 🔗 開發資源

### 內部文檔
- [API 參考](../api-reference.md) - 完整 API 文檔
- [架構決策](../adr/) - 重要設計決策記錄
- [故障排除](../troubleshooting.md) - 常見問題解決

### 外部資源
- [Tauri 文檔](https://tauri.app/v1/guides/)
- [Rust 學習資源](https://doc.rust-lang.org/book/)
- [JavaScript 最佳實踐](https://github.com/airbnb/javascript)
- [Testing Library](https://testing-library.com/)

### 開發工具
- [Rust Analyzer](https://rust-analyzer.github.io/) - Rust 語言服務
- [Playwright](https://playwright.dev/) - E2E 測試框架
- [SQLx](https://docs.rs/sqlx/) - 資料庫工具包

---

**指南版本**: v1.0 • **最後更新**: [DATE] • **維護者**: [MAINTAINER_TEAM]

<!-- 
使用說明:
1. 根據具體專案調整技術棧和架構說明
2. 更新目錄結構以反映實際專案結構
3. 確保所有命令和範例可執行
4. 保持開發工具和資源連結的時效性
5. 定期更新版本資訊和貢獻者名單
-->