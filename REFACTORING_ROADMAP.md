# Claude Night Pilot - 重構路線圖

**版本**: 1.0.0  
**建立時間**: 2025-08-14  
**狀態**: 已完成分析，準備實施

## 🎯 重構目標

將 Claude Night Pilot 從功能原型轉換為企業級、可維護的現代應用程式，遵循 vibe-kanban 架構模式和最佳實踐。

## 📊 當前狀態分析

### ✅ 已完成的分析
- [x] 專案結構和 CLI 指令分析
- [x] 過時檔案和無引用檔案識別
- [x] vibe-kanban 架構模式研究
- [x] BDD 測試框架設計
- [x] 並行任務執行系統設計
- [x] 自動化清理腳本實施

### 🔍 分析發現

#### 過時檔案識別
- **archive/ 目錄**: 8 個檔案，建議安全清理
- **src-tauri/target/**: 編譯產物，可重新生成
- **重複 CLI 實現**: cnp-unified.rs vs cnp-optimized.rs

#### CLI 功能驗證
```bash
✅ cnp-optimized --help          # 功能完整
✅ cnp-optimized status          # 正常運作
✅ cnp-optimized health          # 系統健康檢查
✅ cnp-optimized benchmark       # 性能測試
✅ cnp-optimized cooldown        # 冷卻檢查
```

#### 架構缺口
- ❌ 缺乏模組化後端結構 (models/, routes/, services/)
- ❌ 無 Rust↔TypeScript 類型共享
- ❌ 無統一 API 響應格式
- ❌ 測試結構不完整
- ❌ 缺乏 pnpm workspace 整合

## 🚀 實施計劃

### Phase 1: 基礎清理與準備 (1-2 天)

#### 1.1 執行自動化清理
```bash
# 安全模式清理
./scripts/automated-cleanup.sh

# 檢查清理結果
./scripts/automated-cleanup.sh --dry-run
```

#### 1.2 CLI 整合
- [x] 保留 cnp-optimized.rs 作為標準實現
- [ ] 移除或備份 cnp-unified.rs
- [ ] 更新 package.json 腳本指向 cnp-optimized

#### 1.3 Git 整理
```bash
# 清理未追蹤檔案
git clean -fd

# 垃圾回收優化
git gc --aggressive --prune=now
```

### Phase 2: 架構重構 (3-5 天)

#### 2.1 採用 pnpm Workspace 結構
```yaml
# 建議的新結構
claude-night-pilot/
├── backend/                    # Rust backend (重新命名 src-tauri/)
│   ├── src/
│   │   ├── models/            # 資料模型
│   │   ├── routes/            # API 端點
│   │   ├── services/          # 業務邏輯
│   │   ├── executors/         # Claude 整合
│   │   └── utils/             # 工具函數
│   ├── migrations/            # 資料庫遷移
│   └── Cargo.toml
├── frontend/                   # Web 前端
│   ├── src/
│   └── package.json
├── shared-types/               # 共享類型定義
├── pnpm-workspace.yaml
└── package.json
```

#### 2.2 實施 ts-rs 類型共享
```toml
# backend/Cargo.toml
[dependencies]
ts-rs = "8.1"
serde = { version = "1.0", features = ["derive"] }
```

```rust
// backend/src/models/mod.rs
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Prompt {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

#### 2.3 統一 API 響應格式
```rust
// backend/src/models/api_response.rs
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### Phase 3: 開發工作流程現代化 (2-3 天)

#### 3.1 統一開發腳本
```json
{
  "scripts": {
    "dev": "concurrently \"npm run backend:dev\" \"npm run frontend:dev\"",
    "backend:dev": "cd backend && cargo watch -x run",
    "frontend:dev": "cd frontend && vite dev",
    "generate-types": "cd backend && cargo run --bin generate_types",
    "check": "npm run backend:check && npm run frontend:check",
    "backend:check": "cd backend && cargo check && cargo clippy",
    "frontend:check": "cd frontend && tsc --noEmit"
  }
}
```

#### 3.2 類型生成自動化
```rust
// backend/src/bin/generate_types.rs
use ts_rs::export_all_to_string;

fn main() {
    // 自動生成 TypeScript 類型到 shared-types/
    export_all_to_string!("../shared-types/");
}
```

#### 3.3 建置流程優化
```rust
// backend/build.rs
fn main() {
    // 確保前端 dist 目錄存在
    std::fs::create_dir_all("../frontend/dist").ok();
    
    // 自動觸發類型生成
    println!("cargo:rerun-if-changed=src/models/");
}
```

### Phase 4: 測試策略實施 (2-3 天)

#### 4.1 BDD 測試框架部署
```bash
# 執行 CLI 功能測試
node tests/bdd/cli-testing-framework.js basic

# 執行健康檢查測試
node tests/bdd/cli-testing-framework.js health

# 執行性能測試
node tests/bdd/cli-testing-framework.js performance
```

#### 4.2 測試結構化
```
tests/
├── unit/                      # 單元測試
│   ├── backend/              # Rust 單元測試
│   └── frontend/             # TypeScript 單元測試
├── integration/              # 整合測試
│   ├── api/                  # API 測試
│   └── database/             # 資料庫測試
├── e2e/                      # 端到端測試
│   ├── cli/                  # CLI 測試
│   └── gui/                  # GUI 測試
└── bdd/                      # BDD 測試
    ├── features/             # Gherkin 功能檔案
    └── step-definitions/     # 步驟定義
```

#### 4.3 CI/CD 整合
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install pnpm
        uses: pnpm/action-setup@v2
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'pnpm'
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: pnpm install
      - name: Run checks
        run: pnpm run check
      - name: Run tests
        run: pnpm run test
```

### Phase 5: 高級功能增強 (3-4 天)

#### 5.1 資料庫遷移系統
```sql
-- backend/migrations/001_init.sql
PRAGMA foreign_keys = ON;

CREATE TABLE prompts (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    tags        TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE jobs (
    id          TEXT PRIMARY KEY,
    prompt_id   TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','running','completed','failed')),
    cron_expr   TEXT,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);
```

#### 5.2 WebSocket 即時更新
```rust
// backend/src/routes/stream.rs
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    // 實時任務狀態更新
}
```

#### 5.3 進階 CLI 功能
```rust
// backend/src/bin/cnp.rs (統一 CLI)
#[derive(Parser)]
#[command(about = "Claude Night Pilot - Professional CLI Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(long, global = true)]
    config: Option<String>,
    
    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Prompt(PromptCommands),
    Job(JobCommands),
    Config(ConfigCommands),
    Doctor,
    Version,
}
```

## 📋 實施檢查清單

### Phase 1: 基礎清理 ✅
- [x] 執行自動化清理腳本
- [x] CLI 功能驗證
- [x] Git 儲存庫整理
- [ ] 移除重複 CLI 實現
- [ ] 更新 package.json 腳本

### Phase 2: 架構重構 🔄
- [ ] 重新組織目錄結構
- [ ] 實施 pnpm workspace
- [ ] 添加 ts-rs 依賴
- [ ] 重構 Rust 代碼為模組化結構
- [ ] 建立統一 API 響應格式

### Phase 3: 開發工作流程 ⏳
- [ ] 設置並行開發腳本
- [ ] 實施自動類型生成
- [ ] 優化建置流程
- [ ] 建立開發環境設置腳本

### Phase 4: 測試策略 ⏳
- [ ] 部署 BDD 測試框架
- [ ] 重新組織測試結構
- [ ] 實施 CI/CD 管道
- [ ] 建立測試覆蓋率報告

### Phase 5: 高級功能 ⏳
- [ ] 資料庫遷移系統
- [ ] WebSocket 即時更新
- [ ] 進階 CLI 功能
- [ ] 性能監控和優化

## 🎯 預期成果

### 量化指標
- **代碼品質**: ESLint/Clippy 警告 < 5
- **測試覆蓋率**: > 85%
- **建置時間**: < 30 秒
- **啟動時間**: < 100ms (CLI)
- **型別安全**: 100% TypeScript/Rust 類型覆蓋

### 質化改進
- **開發體驗**: 統一的開發工作流程
- **可維護性**: 模組化架構和清晰的關注點分離
- **可擴展性**: 插件式執行器系統
- **文檔品質**: 完整的 API 文檔和使用指南
- **企業就緒**: 生產級的錯誤處理和監控

## 🚧 風險評估

### 高風險項目
- **資料遷移**: 可能需要備份現有資料
- **API 變更**: 可能影響現有整合
- **依賴更新**: 可能引入相容性問題

### 緩解策略
- **增量實施**: 分階段重構，保持功能性
- **備份策略**: 在重大變更前建立備份
- **回滾計劃**: 準備快速回滾機制
- **測試優先**: 在重構前建立全面測試

## 📅 時間規劃

- **總計**: 11-17 天
- **Phase 1**: 1-2 天 (立即開始)
- **Phase 2**: 3-5 天 (架構重構)
- **Phase 3**: 2-3 天 (工作流程)
- **Phase 4**: 2-3 天 (測試)
- **Phase 5**: 3-4 天 (高級功能)

## 🎉 完成後狀態

Claude Night Pilot 將成為：
- ✨ **現代化**: 採用最新技術棧和最佳實踐
- 🏗️ **模組化**: 清晰的架構和關注點分離
- 🔒 **型別安全**: 跨語言的編譯時型別檢查
- 🧪 **測試完備**: 全面的測試覆蓋和 CI/CD
- 📚 **文檔齊全**: 完整的開發者和使用者文檔
- 🚀 **企業就緒**: 生產級的性能和可靠性

---

**下一步**: 開始執行 Phase 1 - 基礎清理與準備