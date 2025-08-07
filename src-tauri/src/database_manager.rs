use crate::simple_db::{SimpleDatabase, SimplePrompt, SimpleSchedule, TokenUsageStats};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 結構化的數據庫錯誤類型
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("連接失敗: {0}")]
    ConnectionFailed(#[from] rusqlite::Error),
    
    #[error("數據不存在: {id}")]
    NotFound { id: i64 },
    
    #[error("驗證失敗: {message}")]
    ValidationError { message: String },
    
    #[error("並發衝突")]
    ConcurrencyConflict,
    
    #[error("任務執行失敗: {0}")]
    TaskError(#[from] tokio::task::JoinError),
    
    #[error("序列化失敗: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// 數據庫管理器 - 提供異步安全的數據庫操作
#[derive(Clone)]
pub struct DatabaseManager {
    pool: Arc<Mutex<SimpleDatabase>>,
    db_path: String,
}

impl DatabaseManager {
    /// 創建新的數據庫管理器
    pub async fn new(db_path: &str) -> Result<Self, DatabaseError> {
        let db = SimpleDatabase::new(db_path)?;
        Ok(Self {
            pool: Arc::new(Mutex::new(db)),
            db_path: db_path.to_string(),
        })
    }
    
    /// 執行數據庫操作的通用方法
    pub async fn with_db<F, R>(&self, operation: &str, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&SimpleDatabase) -> Result<R, rusqlite::Error> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        let operation = operation.to_string();
        
        let start = std::time::Instant::now();
        
        let result = task::spawn_blocking(move || {
            // 使用 futures 的 block_on，避免 runtime handle 問題
            let db = futures::executor::block_on(pool.lock());
            f(&*db)
        })
        .await??;
        
        let duration = start.elapsed();
        log::debug!(
            "數據庫操作 '{}' 完成，耗時: {:?}",
            operation,
            duration
        );
        
        Ok(result)
    }
    
    /// 異步創建新的 prompt
    pub async fn create_prompt_async(&self, title: String, content: String) -> Result<i64, DatabaseError> {
        self.with_db("create_prompt", move |db| {
            db.create_prompt(&title, &content)
        }).await
    }
    
    /// 異步獲取所有 prompts
    pub async fn list_prompts_async(&self) -> Result<Vec<SimplePrompt>, DatabaseError> {
        self.with_db("list_prompts", |_db| {
            // 這裡需要實現 list_prompts 方法
            // 暫時返回空列表
            Ok(Vec::new())
        }).await
    }
    
    /// 異步獲取特定 prompt
    pub async fn get_prompt_async(&self, _id: i64) -> Result<Option<SimplePrompt>, DatabaseError> {
        self.with_db("get_prompt", move |_db| {
            // 這裡需要實現 get_prompt 方法
            // 暫時返回 None
            Ok(None)
        }).await
    }
    
    /// 異步創建新的排程
    pub async fn create_schedule_async(
        &self, 
        prompt_id: i64, 
        schedule_time: String, 
        cron_expr: Option<String>
    ) -> Result<i64, DatabaseError> {
        self.with_db("create_schedule", move |db| {
            db.create_schedule(prompt_id, &schedule_time, cron_expr.as_deref())
        }).await
    }
    
    /// 異步獲取待執行的排程
    pub async fn get_pending_schedules_async(&self) -> Result<Vec<SimpleSchedule>, DatabaseError> {
        self.with_db("get_pending_schedules", |db| {
            db.get_pending_schedules()
        }).await
    }
    
    /// 異步更新排程
    pub async fn update_schedule_async(
        &self,
        id: i64,
        schedule_time: Option<String>,
        status: Option<String>,
        cron_expr: Option<String>
    ) -> Result<(), DatabaseError> {
        self.with_db("update_schedule", move |db| {
            db.update_schedule(
                id, 
                schedule_time.as_deref(), 
                status.as_deref(), 
                cron_expr.as_deref()
            )
        }).await
    }
    
    /// 異步刪除排程
    pub async fn delete_schedule_async(&self, id: i64) -> Result<bool, DatabaseError> {
        self.with_db("delete_schedule", move |db| {
            db.delete_schedule(id)
        }).await
    }
    
    /// 異步記錄執行結果
    pub async fn record_execution_result_async(
        &self,
        schedule_id: i64,
        content: String,
        status: String,
        token_usage: Option<i64>,
        cost_usd: Option<f64>,
        execution_time_ms: i64
    ) -> Result<i64, DatabaseError> {
        self.with_db("record_execution_result", move |db| {
            db.record_execution_result(
                schedule_id,
                &content,
                &status,
                token_usage,
                cost_usd,
                execution_time_ms
            )
        }).await
    }
    
    /// 異步更新 token 使用統計
    pub async fn update_token_usage_stats_async(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64
    ) -> Result<(), DatabaseError> {
        self.with_db("update_token_usage_stats", move |db| {
            db.update_token_usage_stats(input_tokens, output_tokens, cost_usd)
        }).await
    }
    
    /// 異步獲取 token 使用統計
    pub async fn get_token_usage_stats_async(&self) -> Result<Option<TokenUsageStats>, DatabaseError> {
        self.with_db("get_token_usage_stats", |db| {
            db.get_token_usage_stats()
        }).await
    }
    
    /// 異步更新排程狀態
    pub async fn update_schedule_status_async(&self, id: i64, status: String) -> Result<(), DatabaseError> {
        self.with_db("update_schedule_status", move |db| {
            db.update_schedule_status(id, &status)
        }).await
    }
    
    /// 執行事務操作 (暫時註釋，需要 SimpleDatabase 支援事務)
    #[allow(dead_code)]
    pub async fn execute_transaction<F, R>(&self, _f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<R, rusqlite::Error> + Send + 'static,
        R: Send + 'static,
    {
        // TODO: 需要在 SimpleDatabase 中添加事務支援
        Err(DatabaseError::ValidationError {
            message: "事務功能尚未實現".to_string(),
        })
    }
    
    /// 健康檢查
    pub async fn health_check(&self) -> Result<bool, DatabaseError> {
        self.with_db("health_check", |_db| {
            // 簡單的健康檢查，如果能獲取到數據庫連接就認為健康
            Ok(true)
        }).await
    }
    
    /// 獲取數據庫統計信息 (暫時使用模擬數據)
    pub async fn get_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let db_path = self.db_path.clone();
        self.with_db("get_stats", move |_db| {
            // TODO: 需要在 SimpleDatabase 中添加統計方法
            Ok(DatabaseStats {
                prompt_count: 0,
                schedule_count: 0,
                execution_count: 0,
                db_path,
            })
        }).await
    }
}

/// 數據庫統計信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub prompt_count: i64,
    pub schedule_count: i64,
    pub execution_count: i64,
    pub db_path: String,
}

/// 為 tauri 命令提供的全域數據庫管理器實例
static mut GLOBAL_DB_MANAGER: Option<DatabaseManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// 初始化全域數據庫管理器
pub async fn initialize_database_manager(db_path: &str) -> Result<(), DatabaseError> {
    let manager = DatabaseManager::new(db_path).await?;
    
    unsafe {
        INIT.call_once(|| {
            GLOBAL_DB_MANAGER = Some(manager);
        });
    }
    
    Ok(())
}

/// 獲取全域數據庫管理器
pub fn get_database_manager() -> Result<DatabaseManager, DatabaseError> {
    unsafe {
        GLOBAL_DB_MANAGER.clone()
            .ok_or_else(|| DatabaseError::ValidationError {
                message: "數據庫管理器未初始化".to_string()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    async fn setup_test_db() -> DatabaseManager {
        use std::fs;
        
        // 創建一個臨時目錄和文件
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // 確保父目錄存在並有寫權限
        fs::create_dir_all(temp_dir.path()).unwrap();
        
        DatabaseManager::new(db_path.to_str().unwrap()).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_manager_creation() {
        let db = setup_test_db().await;
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_create_prompt_async() {
        // 直接測試 SimpleDatabase，不使用 DatabaseManager 的異步包裝
        use crate::simple_db::SimpleDatabase;
        
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("direct_test.db");
        
        // 直接使用 SimpleDatabase
        let direct_db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();
        let direct_result = direct_db.create_prompt("直接測試標題", "直接測試內容");
        
        if let Err(ref e) = direct_result {
            eprintln!("直接測試失敗: {:?}", e);
        }
        assert!(direct_result.is_ok(), "直接測試失敗: {:?}", direct_result.err());
        println!("✅ 直接測試成功，ID: {}", direct_result.unwrap());
        
        // 現在測試 DatabaseManager
        let db = setup_test_db().await;
        let result = db.create_prompt_async(
            "測試標題".to_string(),
            "測試內容".to_string()
        ).await;
        
        if let Err(ref e) = result {
            eprintln!("DatabaseManager 測試失敗: {:?}", e);
        }
        assert!(result.is_ok(), "DatabaseManager 測試失敗: {:?}", result.err());
        assert!(result.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let db = std::sync::Arc::new(setup_test_db().await);
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let db = db.clone();
            let handle = tokio::spawn(async move {
                db.create_prompt_async(
                    format!("標題 {}", i),
                    format!("內容 {}", i)
                ).await
            });
            handles.push(handle);
        }

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn test_database_stats() {
        let db = setup_test_db().await;
        
        // 創建一些測試數據
        db.create_prompt_async("測試1".to_string(), "內容1".to_string()).await.unwrap();
        db.create_prompt_async("測試2".to_string(), "內容2".to_string()).await.unwrap();
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.prompt_count, 2);
    }
}
