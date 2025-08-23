//! 資料庫適配器 - 統一 SimpleDatabase 與 Tauri 命令介面
//!
//! 此模組提供 SimpleDatabase 與現有 Tauri 命令系統的適配層，
//! 確保向後兼容性的同時消除技術債務。

use crate::simple_db::{SimpleDatabase, SimplePrompt, SimpleSchedule, TokenUsageStats};
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use serde_json;

/// 資料庫適配器 - 提供線程安全的資料庫訪問
#[derive(Debug)]
pub struct DatabaseAdapter {
    db: Arc<Mutex<SimpleDatabase>>,
}

impl DatabaseAdapter {
    /// 創建新的資料庫適配器
    pub fn new(db_path: &str) -> Result<Self> {
        let db = SimpleDatabase::new(db_path)
            .context("Failed to create database connection")?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// 列出所有提示詞
    pub fn list_prompts(&self) -> Result<Vec<SimplePrompt>> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.list_prompts().context("Failed to list prompts")
    }

    /// 創建新的提示詞
    pub fn create_prompt(&self, title: &str, content: &str, tags: Option<&str>) -> Result<i64> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.create_prompt_with_tags(title, content, tags)
            .context("Failed to create prompt")
    }

    /// 獲取指定 ID 的提示詞
    pub fn get_prompt(&self, id: i64) -> Result<Option<SimplePrompt>> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.get_prompt(id).context("Failed to get prompt")
    }

    /// 刪除提示詞
    pub fn delete_prompt(&self, id: i64) -> Result<bool> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.delete_prompt(id).context("Failed to delete prompt")
    }

    /// 創建排程任務
    pub fn create_schedule(&self, prompt_id: i64, schedule_time: &str, cron_expr: Option<&str>) -> Result<i64> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.create_schedule(prompt_id, schedule_time, cron_expr)
            .context("Failed to create schedule")
    }

    /// 列出所有排程
    pub fn list_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.list_schedules().context("Failed to list schedules")
    }

    /// 獲取待執行的排程
    pub fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.get_pending_schedules().context("Failed to get pending schedules")
    }

    /// 更新排程
    pub fn update_schedule(
        &self,
        id: i64,
        schedule_time: Option<&str>,
        status: Option<&str>,
        cron_expr: Option<&str>
    ) -> Result<()> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.update_schedule(id, schedule_time, status, cron_expr)
            .context("Failed to update schedule")
    }

    /// 刪除排程
    pub fn delete_schedule(&self, id: i64) -> Result<bool> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.delete_schedule(id).context("Failed to delete schedule")
    }

    /// 記錄執行結果
    pub fn record_execution_result(
        &self,
        schedule_id: i64,
        content: &str,
        status: &str,
        token_usage: Option<i64>,
        cost_usd: Option<f64>,
        execution_time_ms: i64,
    ) -> Result<i64> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.record_execution_result(schedule_id, content, status, token_usage, cost_usd, execution_time_ms)
            .context("Failed to record execution result")
    }

    /// 獲取 Token 使用統計
    pub fn get_token_usage_stats(&self) -> Result<Option<TokenUsageStats>> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.get_token_usage_stats().context("Failed to get token usage stats")
    }

    /// 更新 Token 使用統計
    pub fn update_token_usage_stats(&self, input_tokens: i64, output_tokens: i64, cost_usd: f64) -> Result<()> {
        let db = self.db.lock().map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
        db.update_token_usage_stats(input_tokens, output_tokens, cost_usd)
            .context("Failed to update token usage stats")
    }

    /// 將 SimpleSchedule 轉換為 JSON 格式（兼容現有 API）
    pub fn schedule_to_json(&self, schedule: &SimpleSchedule) -> serde_json::Value {
        serde_json::json!({
            "id": schedule.id,
            "prompt_id": schedule.prompt_id,
            "job_name": format!("Job #{}", schedule.id), // 生成預設名稱
            "cron_expr": schedule.cron_expr,
            "status": schedule.status,
            "last_run_at": schedule.last_run_at,
            "next_run_at": schedule.next_run_at,
            "created_at": schedule.created_at,
            "execution_count": schedule.execution_count
        })
    }

    /// 列出排程任務，返回 JSON 格式（兼容現有 list_jobs API）
    pub fn list_jobs_json(&self) -> Result<Vec<serde_json::Value>> {
        let schedules = self.list_schedules()?;
        Ok(schedules.iter().map(|s| self.schedule_to_json(s)).collect())
    }

    /// 獲取任務執行結果（真實資料庫查詢）
    pub fn get_job_results_json(&self, job_id: i64, limit: Option<i64>) -> Result<Vec<serde_json::Value>> {
        let limit = limit.unwrap_or(10);
        
        // 查詢真實的執行結果資料
        let db = self.db.lock()
            .map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
            
        // 首先獲取排程資訊
        let schedule = db.get_schedule(job_id)
            .context("Failed to get schedule")?;
            
        if let Some(schedule) = schedule {
            // 查詢真實的執行結果
            let results = db.get_execution_results(job_id)
                .map_err(|e| anyhow::anyhow!("Database query failed: {}", e))?;
            
            if results.is_empty() {
                // 如果沒有執行結果，返回基於排程狀態的資訊
                let result = serde_json::json!({
                    "id": job_id,
                    "job_id": job_id,
                    "content": format!("排程任務 (Prompt ID: {})", schedule.prompt_id),
                    "status": schedule.status,
                    "execution_count": schedule.execution_count,
                    "last_run_at": schedule.last_run_at,
                    "created_at": schedule.created_at
                });
                Ok(vec![result])
            } else {
                // 轉換真實執行結果為JSON
                let json_results: Vec<serde_json::Value> = results.into_iter()
                    .take(limit as usize)
                    .map(|result| serde_json::json!({
                        "id": result.id,
                        "job_id": result.schedule_id,
                        "content": result.content,
                        "status": result.status,
                        "token_usage": result.token_usage,
                        "cost_usd": result.cost_usd,
                        "execution_time_ms": result.execution_time_ms,
                        "created_at": result.created_at
                    }))
                    .collect();
                Ok(json_results)
            }
        } else {
            Ok(vec![])
        }
    }
}

/// 全域資料庫適配器實例（使用 lazy_static 或 std::sync::Once）
use std::sync::Once;
use std::sync::Mutex as StdMutex;

static INIT: Once = Once::new();
static mut GLOBAL_DB_ADAPTER: Option<DatabaseAdapter> = None;
static DB_INIT_LOCK: StdMutex<()> = StdMutex::new(());

/// 獲取全域資料庫適配器
pub fn get_database_adapter() -> Result<&'static DatabaseAdapter> {
    unsafe {
        INIT.call_once(|| {
            let _lock = DB_INIT_LOCK.lock().unwrap();
            let db_path = std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./claude-night-pilot.db".to_string());
            
            match DatabaseAdapter::new(&db_path) {
                Ok(adapter) => {
                    GLOBAL_DB_ADAPTER = Some(adapter);
                },
                Err(e) => {
                    eprintln!("Failed to initialize database adapter: {}", e);
                }
            }
        });
        
        GLOBAL_DB_ADAPTER.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Database adapter not initialized")
        })
    }
}

/// 為 Tauri 命令提供便捷的錯誤轉換
pub fn error_to_string(err: anyhow::Error) -> String {
    format!("{:#}", err)
}