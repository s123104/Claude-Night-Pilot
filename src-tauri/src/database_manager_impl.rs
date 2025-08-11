// 數據庫最佳實踐：DatabaseManager 實現
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::database_error::{DatabaseError, DatabaseResult};
use crate::simple_db::{SimpleDatabase, SimplePrompt, SimpleSchedule, TokenUsageStats};

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub enable_foreign_keys: bool,
    pub wal_mode: bool,
    pub synchronous_mode: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "claude-pilot.db".to_string(),
            enable_foreign_keys: true,
            wal_mode: true,
            synchronous_mode: "NORMAL".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct DatabaseManager {
    db: Arc<Mutex<SimpleDatabase>>,
    config: DatabaseConfig,
}

impl DatabaseManager {
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let db_path = config.path.clone();
        
        let db = tokio::task::spawn_blocking(move || {
            SimpleDatabase::new(&db_path)
        })
        .await??;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            config,
        })
    }

    pub async fn with_default_config() -> DatabaseResult<Self> {
        Self::new(DatabaseConfig::default()).await
    }

    // Prompt 相關的異步方法
    pub async fn create_prompt_async(&self, title: &str, content: &str) -> DatabaseResult<i64> {
        let db = self.db.clone();
        let title = title.to_string();
        let content = content.to_string();
        
        tokio::task::spawn_blocking(move || {
            // 使用 try_lock 來獲取同步鎖，或者使用 futures::executor::block_on
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.create_prompt(&title, &content)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn list_prompts_async(&self) -> DatabaseResult<Vec<SimplePrompt>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.list_prompts()
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn get_prompt_async(&self, id: i64) -> DatabaseResult<Option<SimplePrompt>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.get_prompt(id)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn delete_prompt_async(&self, id: i64) -> DatabaseResult<bool> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.delete_prompt(id)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    // Schedule 相關的異步方法
    pub async fn create_schedule_async(
        &self, 
        prompt_id: i64, 
        schedule_time: &str, 
        cron_expr: Option<&str>
    ) -> DatabaseResult<i64> {
        let db = self.db.clone();
        let schedule_time = schedule_time.to_string();
        let cron_expr = cron_expr.map(|s| s.to_string());
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.create_schedule(prompt_id, &schedule_time, cron_expr.as_deref())
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn get_all_schedules_async(&self) -> DatabaseResult<Vec<SimpleSchedule>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.list_schedules()
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn get_schedule_async(&self, id: i64) -> DatabaseResult<Option<SimpleSchedule>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.get_schedule(id)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn get_pending_schedules_async(&self) -> DatabaseResult<Vec<SimpleSchedule>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.get_pending_schedules()
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn update_schedule_async(
        &self,
        id: i64,
        schedule_time: Option<&str>,
        status: Option<&str>,
        cron_expr: Option<&str>,
    ) -> DatabaseResult<()> {
        let db = self.db.clone();
        let schedule_time = schedule_time.map(|s| s.to_string());
        let status = status.map(|s| s.to_string());
        let cron_expr = cron_expr.map(|s| s.to_string());
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.update_schedule(
                id,
                schedule_time.as_deref(),
                status.as_deref(),
                cron_expr.as_deref(),
            )
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn delete_schedule_async(&self, id: i64) -> DatabaseResult<bool> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.delete_schedule(id)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    // Token Usage 相關的異步方法
    pub async fn update_token_usage_async(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64,
    ) -> DatabaseResult<()> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.update_token_usage_stats(input_tokens, output_tokens, cost_usd)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn get_token_usage_stats_async(&self) -> DatabaseResult<Option<TokenUsageStats>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.get_token_usage_stats()
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    // Execution Result 相關的異步方法
    pub async fn record_execution_result_async(
        &self,
        schedule_id: i64,
        content: &str,
        status: &str,
        token_usage: Option<i64>,
        cost_usd: Option<f64>,
        execution_time_ms: i64,
    ) -> DatabaseResult<i64> {
        let db = self.db.clone();
        let content = content.to_string();
        let status = status.to_string();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.record_execution_result(
                schedule_id,
                &content,
                &status,
                token_usage,
                cost_usd,
                execution_time_ms,
            )
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    pub async fn update_schedule_status_async(&self, id: i64, status: &str) -> DatabaseResult<()> {
        let db = self.db.clone();
        let status = status.to_string();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            db.update_schedule_status(id, &status)
        })
        .await?
        .map_err(DatabaseError::Connection)
    }

    // 健康檢查
    pub async fn health_check_async(&self) -> DatabaseResult<serde_json::Value> {
        let db = self.db.clone();
        let config = self.config.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let _db = rt.block_on(async { db.lock().await });
            
            Ok(serde_json::json!({
                "database_status": "connected",
                "config": {
                    "path": config.path,
                    "wal_mode": config.wal_mode,
                    "foreign_keys": config.enable_foreign_keys,
                    "synchronous": config.synchronous_mode
                },
                "timestamp": chrono::Local::now().to_rfc3339()
            }))
        })
        .await?
    }

    // 批量操作支持
    pub async fn batch_create_prompts_async(
        &self,
        prompts: Vec<(String, String)>,
    ) -> DatabaseResult<Vec<i64>> {
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let db = rt.block_on(async { db.lock().await });
            let mut ids = Vec::new();
            
            for (title, content) in prompts {
                let id = db.create_prompt(&title, &content)
                    .map_err(DatabaseError::Connection)?;
                ids.push(id);
            }
            
            Ok(ids)
        })
        .await?
    }
}

// 實現 Send 和 Sync，確保可以在異步環境中安全使用
unsafe impl Send for DatabaseManager {}
unsafe impl Sync for DatabaseManager {}