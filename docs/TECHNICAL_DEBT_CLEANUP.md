# 🧹 技術債務清理計劃

## 🚨 立即處理 (紅色警告)

### 1. 廢棄模組清理

#### 問題識別
```rust
// 當前 lib.rs 中的問題程式碼
#[deprecated(note = "請使用 database_manager_impl 代替")]
pub mod database_manager;           // ❌ 廢棄但仍在使用
pub mod simple_database_manager;    // ❌ 重複功能
pub mod simple_db;                  // ❌ 聽名混亂
```

#### 清理動作
```bash
# 立即刪除這些檔案
rm src-tauri/src/database_manager.rs
rm src-tauri/src/simple_database_manager.rs  
rm src-tauri/src/simple_db.rs
rm src-tauri/src/database_error.rs           # 合併到統一錯誤處理
```

#### 影響評估
- **風險級別**: 低 (已標記廢棄)
- **依賴關係**: 檢查 lib.rs 和 CLI 中的引用
- **測試影響**: 需更新相關測試

### 2. lib.rs 肥大問題

#### 問題識別
```rust
// 當前 lib.rs: 506 行，包含：
- 20+ Tauri 命令實現      // 應該在介面層
- 資料庫初始化邏輯    // 應該在基礎設施層
- 業務邏輯實現         // 應該在服務層
- Mock 資料的實現       // 應該在測試模組
```

#### 重構目標
```rust
// 新的 lib.rs (目標 < 50 行)
pub mod core;
pub mod services; 
pub mod interfaces;
pub mod infrastructure;
pub mod utils;

// 重新導出核心類型
pub use core::types::*;
pub use infrastructure::error::*;

// Tauri 應用程式入口 (簡化版)
pub fn run() {
    tauri::Builder::default()
        .setup(infrastructure::setup::initialize_app)
        .invoke_handler(
            tauri::generate_handler![
                interfaces::tauri_commands::get_all_commands
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 🟡 中等優先級 (黃色警告)

### 1. 模組依賴混乱

#### 問題診斷
```bash
# 當前模組依賴圖
enhanced_executor.rs -> unified_interface.rs -> lib.rs
                    \-> database_manager_impl.rs
                     \-> claude_cooldown_detector.rs
                      \-> core::*
```

#### 重構計劃
```rust
// 目標依賴關係 (分層清晰)
interfaces/ -> services/ -> core/ -> infrastructure/
    │           │         │
    v           v         v
 Tauri      Business   Domain
Commands     Logic     Models
```

### 2. 測試結構混亂

#### 當前問題
```rust
src/
├── core/tests_new/          // 殊命名不清
│   ├── cooldown_tests.rs    // 散佈在各處
│   └── integration_tests.rs // 混合單元/整合測試
```

#### 重組目標
```rust
tests/
├── unit/                    // 單元測試
│   ├── core/
│   ├── services/
│   └── utils/
├── integration/             // 整合測試
│   ├── database/
│   └── executor/
└── e2e/                     // E2E 測試 (Playwright)
```

### 3. 配置管理散佈

#### 當前狀態
```rust
// 配置的佈在各處
static DB_MANAGER: OnceCell<Arc<DatabaseManager>> = OnceCell::const_new();
let database_url = "sqlite:claude-pilot.db";  // 硬編碼
let claude_cli_path = "npx @anthropic-ai/claude-code@latest";  // 散佈
```

#### 重構目標
```rust
// infrastructure/config/app_config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub claude: ClaudeConfig,
    pub scheduler: SchedulerConfig,
    pub logging: LoggingConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // 從環境變數/配置檔/命令列參數載入
    }
}
```

## 🟢 低優先級 (綠色優化)

### 1. 效能優化

#### 效能目標
```rust
// 目標效能指標
- 啟動時間: < 3秒 -> < 1秒
- CLI 回應時間: < 500ms -> < 200ms  
- 資料庫查詢: < 100ms -> < 50ms
- 記憶體使用: < 150MB -> < 100MB
```

#### 優化策略
```rust
// 1. 懶加載優化
static SERVICES: OnceCell<ServiceContainer> = OnceCell::const_new();

async fn get_service() -> &'static ServiceContainer {
    SERVICES.get_or_init(|| async {
        ServiceContainer::new().await
    }).await
}

// 2. 連線池優化
struct DatabaseConfig {
    max_connections: u32,     // 10 -> 5
    idle_timeout: Duration,   // 30s -> 10s
    connection_timeout: Duration, // 30s -> 5s
}

// 3. 緩存策略
struct PromptCache {
    lru: LruCache<i64, Prompt>,
    ttl: Duration,
}
```

### 2. 錯誤處理增強

#### 當前問題
```rust
// 不一致的錯誤處理
.map_err(|e| format!("錯誤: {}", e))?;          // 字串錯誤
.map_err(|e| e.to_string())?;                      // 不同轉換
Result<T, String>                                   // 非結構化錯誤
```

#### 改進目標
```rust
// 結構化錯誤處理
#[derive(Debug, Error)]
pub enum ClaudeNightPilotError {
    #[error("資料庫連線失敗: {source}")]
    DatabaseConnection {
        #[from]
        source: sqlx::Error,
        context: String,
    },
    
    #[error("Claude CLI 執行失敗: {message}")]
    ClaudeExecution {
        message: String,
        exit_code: Option<i32>,
        stderr: Option<String>,
    },
}

// 錯誤上下文增強
fn database_operation() -> Result<Data> {
    execute_query()
        .with_context(|| "執行資料庫查詢時發生錯誤")
        .map_err(Into::into)
}
```

## 📅 清理時間表

### Week 1: 立即處理項目
- [ ] **Day 1-2**: 廢棄模組刪除和影響評估
- [ ] **Day 3-4**: lib.rs 重構，移動命令到介面層
- [ ] **Day 5**: 基礎測試確保功能正常

### Week 2-3: 中等優先級項目
- [ ] **Week 2**: 模組依賴重組，建立分層架構
- [ ] **Week 3**: 測試結構重新組織，配置管理統一

### Week 4-6: 低優先級優化
- [ ] **Week 4**: 效能優化和基準測試
- [ ] **Week 5**: 錯誤處理增強和日誌系統
- [ ] **Week 6**: 文檔完善和最終驗證

## 📈 清理效益追蹤

### 代碼品質指標

#### 清理前 (Baseline)
```yaml
檔案數量: 26 個 Rust 檔案
lib.rs 行數: 506 行
重複模組: 3 個資料庫管理器
測試覆蓋: ~30% (主要為 E2E)
Cargo clippy 警告: 15+
```

#### 清理後 (Target)
```yaml
檔案數量: ~15 個模組 (減少 42%)
lib.rs 行數: < 50 行 (減少 90%)
重複模組: 0 個
測試覆蓋: > 80% (增加 167%)
Cargo clippy 警告: 0 個
```

### 維護性指標

#### 開發效率提升
```yaml
新功能開發: -40% 時間
Bug 修復: -60% 時間
代碼審查: -50% 時間
新人上手: -70% 時間
```

#### 系統穩定性提升
```yaml
編譯時間: -30%
啟動時間: -50%
記憶體使用: -33%
CLI 回應時間: -60%
```

## 🎯 清理驗收標準

### 必須達成 (Go/No-Go 標準)
- [ ] 所有廢棄模組已移除，編譯無警告
- [ ] lib.rs < 50 行，不包含業務邏輯
- [ ] 所有 Tauri 命令正常工作
- [ ] CLI 功能完整保留
- [ ] E2E 測試全部通過

### 高品質目標
- [ ] 單元測試覆蓋率 > 80%
- [ ] 模組依賴圖清晰無循環
- [ ] 所有公開 API 有文檔
- [ ] 性能基準測試通過

### 優秀目標
- [ ] 單元測試覆蓋率 > 90%
- [ ] 零 TODO/FIXME 註釋
- [ ] 零 Cargo clippy 警告
- [ ] 性能指標全部達成

## 🔄 持續改進策略

### 自動化品質门檢
```yaml
# .github/workflows/code-quality.yml
Pre-commit hooks:
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo fmt --check
  - cargo test
  - cargo audit

CI/CD Pipeline:
  - 測試覆蓋率檢查 (< 80% 失敗)
  - 性能基準測試
  - 安全掃描
  - 依賴分析
```

### 技術債務防範
- **定期審查**: 每月進行技術債務審查
- **代碼審查**: 每個 PR 必須經過 code review
- **重構警告**: 當檔案超過 200 行或模組超過 10 個函數時警告
- **依賴管理**: 禁止循環依賴，DI 容器管理生命周期