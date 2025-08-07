use crate::simple_db::{SimpleDatabase, SimpleSchedule, TokenUsageStats};
use std::sync::Arc;
use thiserror::Error;

/// 簡化的資料庫錯誤類型
#[derive(Debug, Error)]
pub enum SimpleDatabaseError {
    #[error("連接失敗: {0}")]
    ConnectionFailed(#[from] rusqlite::Error),
    
    #[error("數據不存在: {id}")]
    NotFound { id: i64 },
    
    #[error("驗證失敗: {message}")]
    ValidationError { message: String },
}

/// 簡化的資料庫管理器 - 使用 std::sync::Mutex
#[derive(Clone)]
pub struct SimpleDatabaseManager {
    db: Arc<std::sync::Mutex<SimpleDatabase>>,
    db_path: String,
}

impl SimpleDatabaseManager {
    /// 創建新的簡化資料庫管理器
    pub fn new(db_path: &str) -> Result<Self, SimpleDatabaseError> {
        let db = SimpleDatabase::new(db_path)?;
        Ok(Self {
            db: Arc::new(std::sync::Mutex::new(db)),
            db_path: db_path.to_string(),
        })
    }
    
    /// 執行資料庫操作的通用方法（同步）
    pub fn with_db<F, R>(&self, f: F) -> Result<R, SimpleDatabaseError>
    where
        F: FnOnce(&SimpleDatabase) -> Result<R, rusqlite::Error>,
    {
        let db = self.db.lock().map_err(|_| SimpleDatabaseError::ValidationError {
            message: "無法獲取資料庫鎖".to_string(),
        })?;
        f(&*db).map_err(SimpleDatabaseError::ConnectionFailed)
    }
    
    /// 創建新的 prompt
    pub fn create_prompt(&self, title: &str, content: &str) -> Result<i64, SimpleDatabaseError> {
        self.with_db(|db| db.create_prompt(title, content))
    }
    
    /// 創建新的排程
    pub fn create_schedule(
        &self, 
        prompt_id: i64, 
        schedule_time: &str, 
        cron_expr: Option<&str>
    ) -> Result<i64, SimpleDatabaseError> {
        self.with_db(|db| db.create_schedule(prompt_id, schedule_time, cron_expr))
    }
    
    /// 獲取待執行的排程
    pub fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>, SimpleDatabaseError> {
        self.with_db(|db| db.get_pending_schedules())
    }
    
    /// 更新排程
    pub fn update_schedule(
        &self,
        id: i64,
        schedule_time: Option<&str>,
        status: Option<&str>,
        cron_expr: Option<&str>
    ) -> Result<(), SimpleDatabaseError> {
        self.with_db(|db| db.update_schedule(id, schedule_time, status, cron_expr))
    }
    
    /// 刪除排程
    pub fn delete_schedule(&self, id: i64) -> Result<bool, SimpleDatabaseError> {
        self.with_db(|db| db.delete_schedule(id))
    }
    
    /// 更新 token 使用統計
    pub fn update_token_usage_stats(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64
    ) -> Result<(), SimpleDatabaseError> {
        self.with_db(|db| db.update_token_usage_stats(input_tokens, output_tokens, cost_usd))
    }
    
    /// 獲取 token 使用統計
    pub fn get_token_usage_stats(&self) -> Result<Option<TokenUsageStats>, SimpleDatabaseError> {
        self.with_db(|db| db.get_token_usage_stats())
    }
    
    /// 更新排程狀態
    pub fn update_schedule_status(&self, id: i64, status: &str) -> Result<(), SimpleDatabaseError> {
        self.with_db(|db| db.update_schedule_status(id, status))
    }
    
    /// 健康檢查
    pub fn health_check(&self) -> Result<serde_json::Value, SimpleDatabaseError> {
        self.with_db(|_db| {
            // 簡單的健康檢查，如果能獲取到資料庫連接就認為健康
            Ok(serde_json::json!({
                "database_status": "connected",
                "path": self.db_path,
                "timestamp": chrono::Local::now().to_rfc3339()
            }))
        })
    }
    
    /// 獲取資料庫路徑
    pub fn get_path(&self) -> &str {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> SimpleDatabaseManager {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("simple_test.db");
        
        // 檢查權限
        let metadata = std::fs::metadata(temp_dir.path()).unwrap();
        println!("臨時目錄權限: {:?}", metadata.permissions());
        println!("資料庫路徑: {:?}", db_path);
        
        // 確保目錄可寫
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(temp_dir.path(), perms).unwrap();
        }
        
        SimpleDatabaseManager::new(db_path.to_str().unwrap()).unwrap()
    }

    #[test]
    fn test_simple_database_manager_creation() {
        let db = setup_test_db();
        assert!(db.health_check().is_ok());
    }

    #[test]
    fn test_simple_create_prompt() {
        let db = setup_test_db();
        let result = db.create_prompt("測試標題", "測試內容");
        assert!(result.is_ok(), "創建 prompt 失敗: {:?}", result.err());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_simple_concurrent_access() {
        use std::thread;
        use std::sync::Arc;
        
        let db = Arc::new(setup_test_db());
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let db = db.clone();
            let handle = thread::spawn(move || {
                db.create_prompt(&format!("標題 {}", i), &format!("內容 {}", i))
            });
            handles.push(handle);
        }

        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }
    }
}
