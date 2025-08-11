# API 參考文檔

本文檔提供 Claude Night Pilot 核心 API 的完整參考。

## Tauri Commands API

### Prompt 管理

#### `list_prompts()`
列出所有 Prompt 模板

**回傳**:
```typescript
Promise<Prompt[]>

interface Prompt {
  id: number;
  name: string;  
  content: string;
  tags?: string[];
  created_at: string;
  updated_at: string;
}
```

**範例**:
```javascript
const prompts = await invoke('list_prompts');
console.log(prompts);
```

#### `create_prompt(name: string, content: string, tags?: string[])`
建立新的 Prompt 模板

**參數**:
- `name`: Prompt 名稱
- `content`: Prompt 內容
- `tags`: 可選標籤陣列

**回傳**: `Promise<number>` - 新建立的 Prompt ID

**範例**:
```javascript
const promptId = await invoke('create_prompt', {
  name: '程式碼檢查',
  content: '請檢查這個檔案的程式碼品質並提供建議',
  tags: ['程式碼', '品質']
});
```

#### `update_prompt(id: number, name?: string, content?: string, tags?: string[])`
更新現有 Prompt

**參數**:
- `id`: Prompt ID
- `name`: 新名稱 (可選)
- `content`: 新內容 (可選)  
- `tags`: 新標籤 (可選)

**回傳**: `Promise<void>`

#### `delete_prompt(id: number)`
刪除 Prompt

**參數**:
- `id`: 要刪除的 Prompt ID

**回傳**: `Promise<void>`

### 任務執行

#### `execute_prompt(options: ExecutionOptions)`
執行 Prompt

**參數**:
```typescript
interface ExecutionOptions {
  prompt_id?: number;
  prompt_content?: string;
  working_directory?: string;
  timeout_seconds?: number;
  execution_mode?: 'standard' | 'safe' | 'debug';
  dry_run?: boolean;
  file_references?: string[]; // @ 符號檔案引用
}
```

**回傳**:
```typescript
Promise<ExecutionResult>

interface ExecutionResult {
  id: number;
  status: 'success' | 'error' | 'timeout';
  output: string;
  error_message?: string;
  execution_time: number;
  token_usage?: TokenUsage;
}

interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  estimated_cost: number;
}
```

**範例**:
```javascript
const result = await invoke('execute_prompt', {
  prompt_id: 1,
  working_directory: '/path/to/project',
  timeout_seconds: 300,
  execution_mode: 'standard'
});
```

### 排程管理

#### `create_scheduled_job(options: ScheduleOptions)`
建立排程任務

**參數**:
```typescript
interface ScheduleOptions {
  name: string;
  prompt_id: number;
  cron_expression: string;
  execution_options: ExecutionOptions;
  enabled?: boolean;
}
```

**回傳**: `Promise<number>` - 任務 ID

#### `list_scheduled_jobs()`
列出所有排程任務

**回傳**:
```typescript
Promise<ScheduledJob[]>

interface ScheduledJob {
  id: number;
  name: string;
  prompt_id: number;
  cron_expression: string;
  enabled: boolean;
  next_run?: string;
  last_run?: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
}
```

#### `toggle_scheduled_job(id: number, enabled: boolean)`
啟用/停用排程任務

**參數**:
- `id`: 任務 ID
- `enabled`: 是否啟用

**回傳**: `Promise<void>`

### 系統狀態

#### `get_system_status()`
取得系統狀態

**回傳**:
```typescript
Promise<SystemStatus>

interface SystemStatus {
  claude_code_available: boolean;
  database_healthy: boolean;
  scheduler_running: boolean;
  active_jobs: number;
  total_prompts: number;
  disk_usage: DiskUsage;
  performance_metrics: PerformanceMetrics;
}

interface DiskUsage {
  total_space: number;
  used_space: number;  
  available_space: number;
}

interface PerformanceMetrics {
  startup_time: number;
  memory_usage: number;
  cpu_usage: number;
}
```

#### `check_claude_cooldown()`
檢查 Claude API 冷卻狀態

**回傳**:
```typescript
Promise<CooldownStatus>

interface CooldownStatus {
  is_cooling_down: boolean;
  estimated_wait_time?: number;
  last_request_time?: string;
  requests_remaining?: number;
}
```

### 結果查詢

#### `list_execution_results(options?: ResultFilter)`
列出執行結果

**參數**:
```typescript
interface ResultFilter {
  limit?: number;
  offset?: number;
  status_filter?: 'success' | 'error' | 'timeout';
  date_from?: string;
  date_to?: string;
  prompt_id?: number;
}
```

**回傳**: `Promise<ExecutionResult[]>`

#### `get_execution_result(id: number)`
取得特定執行結果詳情

**參數**:
- `id`: 結果 ID

**回傳**: `Promise<ExecutionResult>`

## Rust 內部 API

### Database Module

#### `Database` Struct
主要資料庫介面

```rust
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self>;
    pub async fn migrate(&self) -> Result<()>;
    
    // Prompt 相關方法
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>>;
    pub async fn create_prompt(&self, prompt: &NewPrompt) -> Result<i64>;
    pub async fn update_prompt(&self, id: i64, prompt: &UpdatePrompt) -> Result<()>;
    pub async fn delete_prompt(&self, id: i64) -> Result<()>;
    
    // 任務相關方法
    pub async fn create_job(&self, job: &NewJob) -> Result<i64>;
    pub async fn list_jobs(&self) -> Result<Vec<Job>>;
    pub async fn update_job_status(&self, id: i64, status: JobStatus) -> Result<()>;
    
    // 結果相關方法
    pub async fn save_execution_result(&self, result: &ExecutionResult) -> Result<i64>;
    pub async fn list_execution_results(&self, filter: &ResultFilter) -> Result<Vec<ExecutionResult>>;
}
```

### Executor Module

#### `ClaudeExecutor` Struct
Claude Code 執行介面

```rust
pub struct ClaudeExecutor {
    config: ExecutorConfig,
}

impl ClaudeExecutor {
    pub fn new(config: ExecutorConfig) -> Self;
    
    pub async fn execute(&self, options: ExecutionOptions) -> Result<ExecutionResult>;
    pub async fn check_availability(&self) -> Result<bool>;
    pub async fn get_cooldown_status(&self) -> Result<CooldownStatus>;
    
    // @ 符號檔案引用處理
    pub fn resolve_file_references(&self, content: &str, working_dir: &Path) -> Result<String>;
    pub fn validate_file_access(&self, paths: &[PathBuf]) -> Result<()>;
}
```

#### `ExecutorConfig` Struct
執行器配置

```rust
pub struct ExecutorConfig {
    pub claude_binary_path: Option<PathBuf>,
    pub default_timeout: Duration,
    pub max_retries: u32,
    pub working_directory: Option<PathBuf>,
    pub security_level: SecurityLevel,
}

pub enum SecurityLevel {
    Strict,    // 嚴格模式：需要所有確認
    Standard,  // 標準模式：重要操作需確認  
    Permissive // 寬鬆模式：跳過大部分確認
}
```

### Scheduler Module

#### `TaskScheduler` Struct
任務排程器

```rust
pub struct TaskScheduler {
    jobs: Arc<Mutex<HashMap<i64, ScheduledJob>>>,
    executor: Arc<ClaudeExecutor>,
    database: Arc<Database>,
}

impl TaskScheduler {
    pub fn new(executor: Arc<ClaudeExecutor>, database: Arc<Database>) -> Self;
    
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    pub async fn add_job(&self, job: ScheduledJob) -> Result<()>;
    pub async fn remove_job(&self, job_id: i64) -> Result<()>;
    pub async fn pause_job(&self, job_id: i64) -> Result<()>;
    pub async fn resume_job(&self, job_id: i64) -> Result<()>;
    
    // Cron 表達式處理
    pub fn validate_cron_expression(&self, cron: &str) -> Result<()>;
    pub fn get_next_execution_time(&self, cron: &str) -> Result<DateTime<Utc>>;
}
```

## 錯誤處理

### 錯誤類型

```rust
#[derive(Debug, thiserror::Error)]
pub enum CNPError {
    #[error("資料庫錯誤: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Claude Code 執行錯誤: {0}")]  
    Execution(String),
    
    #[error("排程錯誤: {0}")]
    Scheduler(String),
    
    #[error("設定錯誤: {0}")]
    Config(String),
    
    #[error("檔案系統錯誤: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("JSON 解析錯誤: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CNPError>;
```

### 錯誤處理模式

```rust
// Tauri 命令錯誤處理
#[tauri::command]
async fn some_command() -> Result<String, String> {
    match internal_function().await {
        Ok(result) => Ok(result),
        Err(err) => {
            log::error!("命令執行失敗: {}", err);
            Err(err.to_string())
        }
    }
}
```

## 事件系統

### 事件類型

```rust
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    ExecutionStarted { job_id: i64 },
    ExecutionCompleted { job_id: i64, status: String },
    ScheduleUpdated { job_id: i64 },
    SystemStatusChanged { status: SystemStatus },
    CooldownDetected { wait_time: u64 },
}
```

### 事件發送

```javascript
// 監聽事件
import { listen } from '@tauri-apps/api/event';

listen('execution-completed', (event) => {
  console.log('任務執行完成:', event.payload);
});

// 發送自訂事件  
import { emit } from '@tauri-apps/api/event';

emit('user-action', { action: 'prompt-created', prompt_id: 123 });
```

## 配置管理

### 配置結構

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub executor: ExecutorConfig, 
    pub scheduler: SchedulerConfig,
    pub logging: LoggingConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub migration_on_startup: bool,
}
```

### 配置載入

```rust
impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}
```

## 測試工具

### Mock 模式

```rust
#[cfg(test)]
pub struct MockClaudeExecutor {
    responses: HashMap<String, String>,
}

impl MockClaudeExecutor {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    pub fn add_response(&mut self, prompt: String, response: String) {
        self.responses.insert(prompt, response);
    }
}
```

### 整合測試

```javascript  
// E2E 測試範例
import { test, expect } from '@playwright/test';

test('建立並執行 Prompt', async ({ page }) => {
  await page.goto('/');
  
  // 建立 Prompt
  await page.click('text=新增 Prompt');
  await page.fill('[data-testid=prompt-name]', '測試 Prompt');
  await page.fill('[data-testid=prompt-content]', '請說 Hello World');
  await page.click('text=儲存');
  
  // 執行 Prompt
  await page.click('[data-testid=execute-prompt]');
  await expect(page.locator('text=執行中')).toBeVisible();
  await expect(page.locator('text=執行完成')).toBeVisible({ timeout: 30000 });
});
```

---

更多詳細範例請參閱 [範例目錄](../examples/) 或 [開發者指南](../CLAUDE.md)。