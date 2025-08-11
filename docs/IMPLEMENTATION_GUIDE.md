# 🚀 實施指導文檔

## 📋 第一階段：核心重構 (Week 1-2)

### 步驟 1: 建立新模組結構

#### 1.1 創建核心目錄結構
```bash
mkdir -p src-tauri/src/{core,services,interfaces,infrastructure,utils}
mkdir -p src-tauri/src/core/{database,executor,scheduler,types}
mkdir -p src-tauri/src/core/database/repositories
mkdir -p src-tauri/src/services/{prompt_service,job_service,usage_service,health_service}
mkdir -p src-tauri/src/interfaces/{tauri_commands,cli_interface,shared}
mkdir -p src-tauri/src/infrastructure/{config,logging,error}
```

#### 1.2 定義核心類型系統
```rust
// src-tauri/src/core/types/mod.rs
pub mod prompt;
pub mod job;
pub mod execution;
pub mod error;

pub use prompt::*;
pub use job::*;
pub use execution::*;
pub use error::*;
```

```rust
// src-tauri/src/core/types/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClaudeNightPilotError {
    #[error("資料庫錯誤: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("執行錯誤: {message}")]
    Execution { message: String },
    
    #[error("配置錯誤: {message}")]
    Config { message: String },
    
    #[error("冷卻錯誤: {message}")]
    Cooldown { message: String },
    
    #[error("序列化錯誤: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ClaudeNightPilotError>;
```

### 步驟 2: 資料庫層重構

#### 2.1 統一資料庫管理器
```rust
// src-tauri/src/core/database/mod.rs
use sqlx::{Pool, Sqlite};
use crate::core::types::Result;

pub struct DatabaseManager {
    pool: Pool<Sqlite>,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(ClaudeNightPilotError::Database)?;
            
        Ok(Self { pool })
    }
    
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
```

#### 2.2 Repository 模式實現
```rust
// src-tauri/src/core/database/repositories/prompt_repository.rs
use async_trait::async_trait;
use crate::core::types::{Prompt, CreatePromptRequest, Result};

#[async_trait]
pub trait PromptRepository: Send + Sync {
    async fn create(&self, request: &CreatePromptRequest) -> Result<Prompt>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Prompt>>;
    async fn list(&self, limit: Option<i64>) -> Result<Vec<Prompt>>;
    async fn update(&self, id: i64, request: &UpdatePromptRequest) -> Result<Prompt>;
    async fn delete(&self, id: i64) -> Result<bool>;
}

struct SqlitePromptRepository {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

#[async_trait]
impl PromptRepository for SqlitePromptRepository {
    async fn create(&self, request: &CreatePromptRequest) -> Result<Prompt> {
        let prompt = sqlx::query_as::<_, Prompt>(
            "INSERT INTO prompts (title, content, tags, created_at) 
             VALUES (?, ?, ?, CURRENT_TIMESTAMP) 
             RETURNING *"
        )
        .bind(&request.title)
        .bind(&request.content)
        .bind(&request.tags)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(prompt)
    }
    
    // ... 其他方法實現
}
```

### 步驟 3: 服務層實現

#### 3.1 Prompt 服務
```rust
// src-tauri/src/services/prompt_service/mod.rs
use std::sync::Arc;
use crate::core::{
    database::repositories::PromptRepository,
    types::{Prompt, CreatePromptRequest, Result},
};

pub struct PromptService {
    repository: Arc<dyn PromptRepository>,
}

impl PromptService {
    pub fn new(repository: Arc<dyn PromptRepository>) -> Self {
        Self { repository }
    }
    
    pub async fn create_prompt(&self, request: CreatePromptRequest) -> Result<Prompt> {
        // 驗證邏輯
        if request.title.trim().is_empty() {
            return Err(ClaudeNightPilotError::Config {
                message: "Prompt title cannot be empty".to_string(),
            });
        }
        
        // 調用 repository
        self.repository.create(&request).await
    }
    
    pub async fn list_prompts(&self, limit: Option<i64>) -> Result<Vec<Prompt>> {
        self.repository.list(limit).await
    }
    
    // ... 其他業務方法
}
```

### 步驟 4: 清理技術債務

#### 4.1 移除廢棄模組
```bash
# 刪除廢棄檔案
rm src-tauri/src/database_manager.rs
rm src-tauri/src/simple_database_manager.rs
rm src-tauri/src/simple_db.rs
```

#### 4.2 重構 lib.rs
```rust
// src-tauri/src/lib.rs - 精簡到 < 50 行
pub mod core;
pub mod services;
pub mod interfaces;
pub mod infrastructure;
pub mod utils;

// 重新導出核心類型
pub use core::types::*;

// Tauri 應用程式入口
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_cli::init())
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            interfaces::tauri_commands::list_prompts,
            interfaces::tauri_commands::create_prompt,
            // ... 其他命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 應用程式初始化邏輯
    Ok(())
}
```

## 📋 第二階段：服務層完善 (Week 3-4)

### 步驟 1: 依賴注入系統

```rust
// src-tauri/src/infrastructure/container.rs
use std::sync::Arc;
use crate::{
    core::database::{DatabaseManager, repositories::*},
    services::{PromptService, JobService, UsageService},
};

pub struct ServiceContainer {
    pub database_manager: Arc<DatabaseManager>,
    pub prompt_service: Arc<PromptService>,
    pub job_service: Arc<JobService>,
    pub usage_service: Arc<UsageService>,
}

impl ServiceContainer {
    pub async fn new(database_url: &str) -> crate::core::types::Result<Self> {
        // 初始化資料庫
        let database_manager = Arc::new(DatabaseManager::new(database_url).await?);
        
        // 初始化 Repositories
        let prompt_repo = Arc::new(SqlitePromptRepository::new(database_manager.pool().clone()));
        let job_repo = Arc::new(SqliteJobRepository::new(database_manager.pool().clone()));
        let usage_repo = Arc::new(SqliteUsageRepository::new(database_manager.pool().clone()));
        
        // 初始化 Services
        let prompt_service = Arc::new(PromptService::new(prompt_repo));
        let job_service = Arc::new(JobService::new(job_repo));
        let usage_service = Arc::new(UsageService::new(usage_repo));
        
        Ok(Self {
            database_manager,
            prompt_service,
            job_service,
            usage_service,
        })
    }
}
```

### 步驟 2: 執行器統一

```rust
// src-tauri/src/core/executor/mod.rs
use async_trait::async_trait;
use crate::core::types::{ExecutionRequest, ExecutionResult, Result};

#[async_trait]
pub trait ClaudeExecutor: Send + Sync {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult>;
    async fn check_cooldown(&self) -> Result<CooldownStatus>;
    async fn health_check(&self) -> Result<HealthStatus>;
}

pub struct UnifiedClaudeExecutor {
    cli_path: String,
    cooldown_manager: Arc<CooldownManager>,
}

#[async_trait]
impl ClaudeExecutor for UnifiedClaudeExecutor {
    async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        // 檢查冷卻狀態
        let cooldown_status = self.cooldown_manager.check().await?;
        if cooldown_status.is_cooling {
            return Err(ClaudeNightPilotError::Cooldown {
                message: format!("Claude CLI is cooling down for {} seconds", 
                               cooldown_status.remaining_seconds),
            });
        }
        
        // 執行 Claude CLI
        let output = self.execute_cli_command(&request).await?;
        
        // 解析回應
        let result = self.parse_response(&output).await?;
        
        Ok(result)
    }
    
    // ... 其他方法實現
}
```

## 📋 第三階段：介面層統一 (Week 5-6)

### 步驟 1: GUI 命令重構

```rust
// src-tauri/src/interfaces/tauri_commands/prompt_commands.rs
use crate::{
    core::types::{CreatePromptRequest, Prompt, Result},
    infrastructure::container::ServiceContainer,
};
use tauri::State;

#[tauri::command]
pub async fn list_prompts(
    container: State<'_, ServiceContainer>,
) -> Result<Vec<Prompt>, String> {
    container.prompt_service
        .list_prompts(None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_prompt(
    container: State<'_, ServiceContainer>,
    title: String,
    content: String,
    tags: Option<String>,
) -> Result<Prompt, String> {
    let request = CreatePromptRequest {
        title,
        content,
        tags,
    };
    
    container.prompt_service
        .create_prompt(request)
        .await
        .map_err(|e| e.to_string())
}
```

### 步驟 2: CLI 介面統一

```rust
// src-tauri/src/interfaces/cli_interface/mod.rs
use clap::{Parser, Subcommand};
use crate::infrastructure::container::ServiceContainer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 管理 Prompts
    Prompt {
        #[command(subcommand)]
        action: PromptAction,
    },
    /// 管理 Jobs
    Job {
        #[command(subcommand)]
        action: JobAction,
    },
    /// 系統狀態
    Status,
}

pub async fn handle_cli_command(cli: Cli, container: &ServiceContainer) -> crate::core::types::Result<()> {
    match cli.command {
        Commands::Prompt { action } => {
            handle_prompt_command(action, container).await
        },
        Commands::Job { action } => {
            handle_job_command(action, container).await
        },
        Commands::Status => {
            handle_status_command(container).await
        },
    }
}
```

## 🧪 測試框架建立

### 單元測試
```rust
// src-tauri/src/services/prompt_service/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::repositories::MockPromptRepository;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_create_prompt_success() {
        let mut mock_repo = MockPromptRepository::new();
        mock_repo
            .expect_create()
            .returning(|_| Ok(Prompt::default()));
            
        let service = PromptService::new(Arc::new(mock_repo));
        let request = CreatePromptRequest {
            title: "Test Prompt".to_string(),
            content: "Test content".to_string(),
            tags: None,
        };
        
        let result = service.create_prompt(request).await;
        assert!(result.is_ok());
    }
}
```

## 📊 進度追蹤清單

### ✅ 第一階段完成標準
- [ ] 新模組結構建立完成
- [ ] 核心類型系統定義完成
- [ ] 資料庫層 Repository 模式實現
- [ ] 廢棄模組清理完成
- [ ] lib.rs 精簡到 < 50 行
- [ ] 基礎錯誤處理系統實現

### ✅ 第二階段完成標準
- [ ] 服務層依賴注入系統完成
- [ ] Claude 執行器統一實現
- [ ] 排程器核心邏輯重構
- [ ] 使用量追蹤服務完善

### ✅ 第三階段完成標準
- [ ] GUI 命令層重構完成
- [ ] CLI 介面統一實現
- [ ] 共享驗證和序列化邏輯
- [ ] 單元測試覆蓋率達到 80%
- [ ] 整合測試框架建立

## 🎯 成功指標

### 程式碼品質指標
- **模組數量**: 從 26 個檔案重組為 ~15 個核心模組
- **lib.rs 行數**: 從 500+ 行減少到 < 50 行
- **重複程式碼**: 消除 3 個重複的資料庫管理器
- **測試覆蓋率**: 核心邏輯達到 90%

### 維護性指標
- **新功能開發時間**: 減少 40%
- **Bug 修復時間**: 減少 60%
- **程式碼審查時間**: 減少 50%
- **技術文檔完整性**: 100% API 文檔覆蓋