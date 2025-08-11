# Claude Night Pilot 統一架構設計

## 📁 推薦架構結構

```rust
src-tauri/src/
├── core/                           // 🧠 核心業務邏輯層
│   ├── database/                  // 統一資料庫抽象
│   │   ├── mod.rs                // 資料庫介面定義
│   │   ├── connection.rs         // 連線管理
│   │   ├── migrations.rs         // 資料庫遷移
│   │   └── repositories/         // Repository 模式
│   │       ├── prompt_repository.rs
│   │       ├── job_repository.rs
│   │       └── usage_repository.rs
│   ├── executor/                  // Claude 執行器核心
│   │   ├── mod.rs                // 執行器介面
│   │   ├── claude_executor.rs    // Claude CLI 整合
│   │   ├── stream_processor.rs   // 串流處理
│   │   └── cooldown_manager.rs   // 冷卻管理
│   ├── scheduler/                 // 排程服務核心
│   │   ├── mod.rs
│   │   ├── cron_scheduler.rs
│   │   ├── adaptive_scheduler.rs
│   │   └── session_scheduler.rs
│   └── types/                     // 共享型別定義
│       ├── mod.rs
│       ├── prompt.rs
│       ├── job.rs
│       ├── execution.rs
│       └── error.rs
├── services/                       // 🚀 業務服務層
│   ├── prompt_service/
│   │   ├── mod.rs
│   │   ├── prompt_manager.rs
│   │   └── template_engine.rs
│   ├── job_service/
│   │   ├── mod.rs
│   │   ├── job_manager.rs
│   │   └── execution_tracker.rs
│   ├── usage_service/
│   │   ├── mod.rs
│   │   ├── token_tracker.rs
│   │   └── cost_calculator.rs
│   └── health_service/
│       ├── mod.rs
│       ├── system_monitor.rs
│       └── diagnostic_collector.rs
├── interfaces/                     // 🌐 對外介面層
│   ├── tauri_commands/            // GUI 命令層
│   │   ├── mod.rs
│   │   ├── prompt_commands.rs
│   │   ├── job_commands.rs
│   │   ├── usage_commands.rs
│   │   └── system_commands.rs
│   ├── cli_interface/             // CLI 介面層
│   │   ├── mod.rs
│   │   ├── cli_handlers.rs
│   │   └── command_parser.rs
│   └── shared/                    // 共享邏輯
│       ├── mod.rs
│       ├── validation.rs
│       └── serialization.rs
├── infrastructure/                 // ⚙️ 基礎設施層
│   ├── config/
│   │   ├── mod.rs
│   │   ├── app_config.rs
│   │   └── environment.rs
│   ├── logging/
│   │   ├── mod.rs
│   │   └── structured_logger.rs
│   └── error/
│       ├── mod.rs
│       ├── error_types.rs
│       └── error_handler.rs
├── utils/                          // 🔧 工具函數層
│   ├── mod.rs
│   ├── file_utils.rs
│   ├── time_utils.rs
│   └── crypto_utils.rs
├── bin/                            // 🎯 二進位目標
│   ├── cnp.rs                    // CLI 主程式
│   └── cnp-daemon.rs             // 後台服務
├── lib.rs                          // 📚 函式庫根目錄
└── main.rs                         // 🚪 GUI 主程式
```

## 🏛️ 設計原則

### 1. **分層架構 (Layered Architecture)**
- **介面層**: 處理外部請求和回應
- **服務層**: 實現業務邏輯
- **核心層**: 定義領域模型和規則
- **基礎設施層**: 提供技術支援

### 2. **依賴注入 (Dependency Injection)**
```rust
// 服務層依賴核心層，而非具體實現
pub struct PromptService {
    repository: Arc<dyn PromptRepository>,
    executor: Arc<dyn ClaudeExecutor>,
}
```

### 3. **Repository 模式**
```rust
#[async_trait]
pub trait PromptRepository {
    async fn create(&self, prompt: &CreatePromptRequest) -> Result<Prompt>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Prompt>>;
    async fn list(&self, filter: &PromptFilter) -> Result<Vec<Prompt>>;
}
```

### 4. **錯誤處理統一**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ClaudeNightPilotError {
    #[error("資料庫錯誤: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("執行錯誤: {0}")]
    Execution(String),
    
    #[error("配置錯誤: {0}")]
    Config(String),
}
```

## 🚀 實施優先級建議

### **第一階段 (Week 1-2): 核心重構**
1. **建立新的模組結構**
   - 創建 `core/`, `services/`, `interfaces/`, `infrastructure/` 目錄
   - 定義統一的型別系統

2. **資料庫層重構**
   - 統一為單一 `DatabaseManager`
   - 實施 Repository 模式
   - 移除廢棄的資料庫管理器

3. **錯誤處理統一**
   - 定義統一的錯誤型別
   - 實施錯誤處理中介軟體

### **第二階段 (Week 3-4): 服務層實現**
1. **業務服務重構**
   - 將 `lib.rs` 中的邏輯遷移到服務層
   - 實施服務介面和實現分離

2. **執行器重構**
   - 統一 Claude 執行器介面
   - 實施串流處理和冷卻管理

### **第三階段 (Week 5-6): 介面層優化**
1. **GUI-CLI 統一**
   - 創建共享的命令處理邏輯
   - 實施統一的驗證和序列化

2. **測試框架建立**
   - 單元測試覆蓋核心邏輯
   - 整合測試覆蓋服務層

## 🧹 技術債務清理計劃

### **立即清理 (高優先級)**
```rust
// 移除這些廢棄模組
#[deprecated] pub mod database_manager;     // ❌ 立即移除
pub mod simple_database_manager;           // ❌ 合併到統一管理器
pub mod simple_db;                         // ❌ 重構為 repository
```

### **重構目標 (中優先級)**
```rust
// lib.rs 從 500 行縮減到 < 50 行
// 只保留模組宣告和主要配置
pub mod core;
pub mod services;
pub mod interfaces;
pub mod infrastructure;
pub mod utils;
```

### **測試覆蓋 (持續改進)**
- 核心邏輯：90% 覆蓋率
- 服務層：80% 覆蓋率
- 介面層：70% 覆蓋率

## 🔄 GUI-CLI 同步整合策略

### **共享服務層**
```rust
// GUI 和 CLI 都使用相同的服務層
struct UnifiedServiceLayer {
    prompt_service: Arc<PromptService>,
    job_service: Arc<JobService>,
    usage_service: Arc<UsageService>,
}
```

### **介面適配器模式**
```rust
// GUI 適配器
struct TauriCommandAdapter {
    services: Arc<UnifiedServiceLayer>,
}

// CLI 適配器
struct CliCommandAdapter {
    services: Arc<UnifiedServiceLayer>,
}
```

### **配置共享**
```rust
// 統一的配置管理
struct AppConfig {
    database_url: String,
    claude_cli_path: String,
    log_level: LogLevel,
    // GUI 和 CLI 共享相同配置
}
```

## 📈 預期效益

### **維護性提升**
- 模組職責清晰，修改影響範圍可控
- 統一的錯誤處理和日誌記錄
- 更容易進行單元測試

### **技術債務減少**
- 消除重複代碼和廢棄模組
- 統一的資料存取模式
- 清晰的依賴關係

### **可擴展性增強**
- 易於添加新的執行器類型
- 易於擴展排程策略
- 支援插件式架構

### **開發效率提升**
- GUI 和 CLI 開發可並行進行
- 新功能開發時間減少 40%
- bug 修復時間減少 60%