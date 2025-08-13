// 高性能資料庫模組，使用直接 SQLite 連接（移除 r2d2 避免版本衝突）
// High-performance database module with direct SQLite connections (removed r2d2 to avoid version conflicts)

use rusqlite::{Connection, Result, params, OpenFlags};
use chrono::Utc;
use std::path::Path;
use std::sync::Arc;
use tokio::task;
use parking_lot::RwLock;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use anyhow::{Result as AnyhowResult, Context};
use tracing::{info, warn, error, debug};

// Re-export types from simple_db for compatibility
pub use crate::simple_db::{SimplePrompt, SimpleSchedule, ExecutionResult, TokenUsageStats};

// Performance tracking
static QUERY_METRICS: once_cell::sync::Lazy<Arc<DashMap<String, QueryMetric>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(DashMap::new()));

#[derive(Debug, Clone)]
struct QueryMetric {
    total_calls: u64,
    total_duration_ms: u64,
    avg_duration_ms: f64,
    last_updated: Instant,
}

// 簡化的數據庫結構，使用直接連接替代連接池
#[derive(Debug, Clone)]
pub struct HighPerfDatabase {
    db_path: Arc<String>,
    cache: Arc<DashMap<String, (String, Instant)>>, // Simple query result cache
    performance_mode: bool,
}

impl HighPerfDatabase {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        Self::new_with_config(db_path, true)
    }
    
    pub fn new_with_config<P: AsRef<Path>>(db_path: P, performance_mode: bool) -> Result<Self> {
        let manager = SqliteConnectionManager::file(db_path.as_ref())
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE 
                | OpenFlags::SQLITE_OPEN_CREATE 
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX)  // Better for multithreaded apps
            .with_init(move |conn| {
                // High-performance SQLite configuration
                conn.pragma_update(None, "foreign_keys", true)?;
                conn.pragma_update(None, "journal_mode", "WAL")?;
                conn.pragma_update(None, "synchronous", if performance_mode { "NORMAL" } else { "FULL" })?;
                conn.pragma_update(None, "temp_store", "memory")?;
                conn.pragma_update(None, "cache_size", if performance_mode { -64000 } else { -32000 })?; // 64MB vs 32MB cache
                conn.pragma_update(None, "mmap_size", if performance_mode { 268435456 } else { 134217728 })?; // 256MB vs 128MB mmap
                conn.pragma_update(None, "page_size", 4096)?;
                conn.pragma_update(None, "wal_autocheckpoint", 1000)?;
                conn.pragma_update(None, "optimize", ())?; // Optimize database on open
                
                // Additional performance optimizations
                if performance_mode {
                    conn.pragma_update(None, "locking_mode", "NORMAL")?; // Allow concurrent reads
                    conn.pragma_update(None, "read_uncommitted", true)?; // Allow dirty reads for better performance
                    conn.pragma_update(None, "query_only", false)?;
                    conn.pragma_update(None, "defer_foreign_keys", true)?; // Defer FK checks for bulk operations
                }
                
                Ok(())
            });
            
        let pool = Pool::builder()
            .max_size(if performance_mode { 20 } else { 10 })
            .min_idle(Some(3))
            .connection_timeout(Duration::from_secs(30))
            .idle_timeout(Some(Duration::from_secs(600)))
            .max_lifetime(Some(Duration::from_secs(1800)))
            .test_on_check_out(true)
            .build(manager)
            .map_err(|e| rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN), 
                Some(format!("Failed to create connection pool: {}", e))
            ))?;
            
        // Initialize schema with a connection from the pool
        let conn = pool.get().map_err(|e| rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
            Some(format!("Failed to get connection from pool: {}", e))
        ))?;
        
        Self::init_schema(&*conn)?;
        
        Ok(Self { 
            pool: Arc::new(pool),
            cache: Arc::new(DashMap::new()),
            performance_mode,
        })
    }
    
    fn init_schema(conn: &Connection) -> Result<()> {
        // Create tables with optimized indexes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompts_tags ON prompts(tags) WHERE tags IS NOT NULL",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompts_title ON prompts(title COLLATE NOCASE)",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schedules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt_id INTEGER NOT NULL,
                schedule_time TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                last_run_at TEXT,
                next_run_at TEXT,
                updated_at TEXT,
                cron_expr TEXT,
                execution_count INTEGER DEFAULT 0,
                FOREIGN KEY(prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Optimize schedule queries with compound indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_schedules_status_time ON schedules(status, schedule_time)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_schedules_prompt_id ON schedules(prompt_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_schedules_next_run ON schedules(next_run_at) WHERE next_run_at IS NOT NULL",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                schedule_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL,
                token_usage INTEGER,
                cost_usd REAL,
                execution_time_ms INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(schedule_id) REFERENCES schedules(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_results_schedule_id ON execution_results(schedule_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_results_created_at ON execution_results(created_at DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_execution_results_status ON execution_results(status)",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS token_usage_stats (
                id INTEGER PRIMARY KEY DEFAULT 1,
                total_input_tokens INTEGER DEFAULT 0,
                total_output_tokens INTEGER DEFAULT 0,
                total_cost_usd REAL DEFAULT 0.0,
                session_count INTEGER DEFAULT 0,
                last_updated TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create views for common queries
        conn.execute(
            "CREATE VIEW IF NOT EXISTS v_schedule_summary AS
             SELECT s.*, p.title as prompt_title, p.tags as prompt_tags
             FROM schedules s 
             JOIN prompts p ON s.prompt_id = p.id",
            [],
        )?;
        
        info!("Database schema initialized with performance optimizations");
        Ok(())
    }
    
    // Get a connection from the pool
    fn get_conn(&self) -> Result<PooledConnection> {
        self.pool.get().map_err(|e| {
            error!("Failed to get database connection from pool: {}", e);
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                Some(format!("Connection pool exhausted: {}", e))
            )
        })
    }
    
    // Execute query with performance tracking
    async fn execute_with_metrics<F, T>(&self, operation: &str, f: F) -> Result<T> 
    where
        F: FnOnce(PooledConnection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let start = Instant::now();
        let pool = Arc::clone(&self.pool);
        let operation_name = operation.to_string();
        
        let result = task::spawn_blocking(move || {
            let conn = pool.get().map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_BUSY),
                    Some(format!("Connection pool error: {}", e))
                )
            })?;
            f(conn)
        }).await.map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISUSE),
                Some(format!("Task execution error: {}", e))
            )
        })??;
        
        let duration = start.elapsed();
        
        // Update metrics
        QUERY_METRICS.entry(operation_name.clone())
            .and_modify(|metric| {
                metric.total_calls += 1;
                metric.total_duration_ms += duration.as_millis() as u64;
                metric.avg_duration_ms = metric.total_duration_ms as f64 / metric.total_calls as f64;
                metric.last_updated = Instant::now();
            })
            .or_insert(QueryMetric {
                total_calls: 1,
                total_duration_ms: duration.as_millis() as u64,
                avg_duration_ms: duration.as_millis() as f64,
                last_updated: Instant::now(),
            });
            
        // Log slow queries in performance mode
        if self.performance_mode && duration.as_millis() > 100 {
            warn!("Slow database query '{}' took {}ms", operation_name, duration.as_millis());
        } else if duration.as_millis() > 10 {
            debug!("Database query '{}' took {}ms", operation_name, duration.as_millis());
        }
        
        Ok(result)
    }
    
    // Batch operations for better performance
    pub async fn batch_create_prompts(&self, prompts: Vec<(String, String, Option<String>)>) -> Result<Vec<i64>> {
        self.execute_with_metrics("batch_create_prompts", move |conn| {
            let now = Utc::now().to_rfc3339();
            let tx = conn.unchecked_transaction()?;
            
            let mut stmt = tx.prepare(
                "INSERT INTO prompts (title, content, tags, created_at) VALUES (?1, ?2, ?3, ?4)"
            )?;
            
            let mut ids = Vec::new();
            for (title, content, tags) in prompts {
                stmt.execute(params![title, content, tags, now])?;
                ids.push(tx.last_insert_rowid());
            }
            
            tx.commit()?;
            Ok(ids)
        }).await
    }
    
    pub fn get_performance_metrics(&self) -> Vec<(String, QueryMetric)> {
        QUERY_METRICS.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
    
    pub async fn optimize_database(&self) -> Result<()> {
        self.execute_with_metrics("optimize", |conn| {
            conn.pragma_update(None, "optimize", ())?;
            conn.execute("VACUUM", [])?;
            // PRAGMA wal_checkpoint 會返回結果列，需使用 query_row/查詢API 而非 execute
            let _checkpoint_result: (i64, i64, i64) = conn.query_row(
                "PRAGMA wal_checkpoint(TRUNCATE)",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;
            conn.execute("ANALYZE", [])?; // Update statistics
            info!("Database optimization completed");
            Ok(())
        }).await
    }
    
    pub async fn get_pool_status(&self) -> (u32, u32) {
        let state = self.pool.state();
        (state.connections, state.idle_connections)
    }
    
    // Async database operations
    pub async fn create_prompt(&self, title: &str, content: &str) -> Result<i64> {
        self.create_prompt_with_tags(title, content, None).await
    }
    
    pub async fn create_prompt_with_tags(&self, title: &str, content: &str, tags: Option<&str>) -> Result<i64> {
        let title = title.to_string();
        let content = content.to_string();
        let tags = tags.map(|t| t.to_string());
        
        self.execute_with_metrics("create_prompt", move |conn| {
            let now = Utc::now().to_rfc3339();
            
            conn.execute(
                "INSERT INTO prompts (title, content, tags, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![title, content, tags, now],
            )?;
            
            Ok(conn.last_insert_rowid())
        }).await
    }
    
    pub async fn list_prompts(&self) -> Result<Vec<SimplePrompt>> {
        // Check cache first
        let cache_key = "list_prompts";
        if let Some((cached_data, cached_time)) = self.cache.get(cache_key) {
            if cached_time.elapsed() < Duration::from_secs(30) { // 30 second cache
                if let Ok(prompts) = serde_json::from_str::<Vec<SimplePrompt>>(&cached_data) {
                    debug!("Returning cached prompts list ({} items)", prompts.len());
                    return Ok(prompts);
                }
            }
        }
        
        let result = self.execute_with_metrics("list_prompts", |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT id, title, content, tags, created_at FROM prompts ORDER BY created_at DESC LIMIT 1000"
            )?;
            
            let prompt_iter = stmt.query_map([], |row| {
                Ok(SimplePrompt {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    tags: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            
            let mut prompts = Vec::new();
            for prompt in prompt_iter {
                prompts.push(prompt?);
            }
            
            Ok(prompts)
        }).await?;
        
        // Cache the result
        if let Ok(cached_data) = serde_json::to_string(&result) {
            self.cache.insert(cache_key.to_string(), (cached_data, Instant::now()));
        }
        
        Ok(result)
    }
    
    pub async fn get_prompt(&self, id: i64) -> Result<Option<SimplePrompt>> {
        self.execute_with_metrics("get_prompt", move |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT id, title, content, tags, created_at FROM prompts WHERE id = ?1"
            )?;
            
            let mut prompt_iter = stmt.query_map([id], |row| {
                Ok(SimplePrompt {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    tags: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            
            match prompt_iter.next() {
                Some(Ok(prompt)) => Ok(Some(prompt)),
                Some(Err(e)) => Err(e),
                None => Ok(None),
            }
        }).await
    }
    
    pub async fn create_schedule(&self, prompt_id: i64, schedule_time: &str, cron_expr: Option<&str>) -> Result<i64> {
        let schedule_time = schedule_time.to_string();
        let cron_expr = cron_expr.map(|c| c.to_string());
        
        // Clear cache since we're modifying data
        self.cache.remove("get_pending_schedules");
        
        self.execute_with_metrics("create_schedule", move |conn| {
            let now = Utc::now().to_rfc3339();
            
            conn.execute(
                "INSERT INTO schedules (prompt_id, schedule_time, status, created_at, updated_at, cron_expr) VALUES (?1, ?2, 'pending', ?3, ?3, ?4)",
                params![prompt_id, schedule_time, now, cron_expr],
            )?;
            
            Ok(conn.last_insert_rowid())
        }).await
    }
    
    pub async fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let cache_key = "get_pending_schedules";
        if let Some((cached_data, cached_time)) = self.cache.get(cache_key) {
            if cached_time.elapsed() < Duration::from_secs(10) { // 10 second cache for pending schedules
                if let Ok(schedules) = serde_json::from_str::<Vec<SimpleSchedule>>(&cached_data) {
                    debug!("Returning cached pending schedules ({} items)", schedules.len());
                    return Ok(schedules);
                }
            }
        }
        
        let result = self.execute_with_metrics("get_pending_schedules", |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT id, prompt_id, schedule_time, status, created_at, last_run_at, next_run_at, updated_at, cron_expr, execution_count
                 FROM schedules 
                 WHERE status = 'pending' 
                 ORDER BY schedule_time
                 LIMIT 500"
            )?;
            
            let schedule_iter = stmt.query_map([], |row| {
                Ok(SimpleSchedule {
                    id: row.get(0)?,
                    prompt_id: row.get(1)?,
                    schedule_time: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    last_run_at: row.get(5)?,
                    next_run_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    cron_expr: row.get(8)?,
                    execution_count: row.get(9)?,
                })
            })?;
            
            let mut schedules = Vec::new();
            for schedule in schedule_iter {
                schedules.push(schedule?);
            }
            
            Ok(schedules)
        }).await?;
        
        // Cache the result
        if let Ok(cached_data) = serde_json::to_string(&result) {
            self.cache.insert(cache_key.to_string(), (cached_data, Instant::now()));
        }
        
        Ok(result)
    }
    
    pub async fn record_execution_result(&self, schedule_id: i64, content: &str, status: &str, token_usage: Option<i64>, cost_usd: Option<f64>, execution_time_ms: i64) -> Result<i64> {
        let content = content.to_string();
        let status = status.to_string();
        
        // Clear related caches
        self.cache.remove("get_pending_schedules");
        
        self.execute_with_metrics("record_execution_result", move |conn| {
            let now = Utc::now().to_rfc3339();
            
            // 使用事務確保數據一致性
            let tx = conn.unchecked_transaction()?;
            
            // 插入執行結果
            tx.execute(
                "INSERT INTO execution_results (schedule_id, content, status, token_usage, cost_usd, execution_time_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![schedule_id, content, status, token_usage, cost_usd, execution_time_ms, now],
            )?;
            
            let result_id = tx.last_insert_rowid();
            
            // 更新排程的執行次數和最後運行時間
            tx.execute(
                "UPDATE schedules SET execution_count = execution_count + 1, last_run_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, schedule_id],
            )?;
            
            tx.commit()?;
            Ok(result_id)
        }).await
    }
    
    pub async fn get_token_usage_stats(&self) -> Result<Option<TokenUsageStats>> {
        self.execute_with_metrics("get_token_usage_stats", |conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT total_input_tokens, total_output_tokens, total_cost_usd, session_count, last_updated FROM token_usage_stats LIMIT 1"
            )?;
            
            let result = stmt.query_row([], |row| {
                Ok(TokenUsageStats {
                    total_input_tokens: row.get(0)?,
                    total_output_tokens: row.get(1)?,
                    total_cost_usd: row.get(2)?,
                    session_count: row.get(3)?,
                    last_updated: row.get(4)?,
                })
            });
            
            match result {
                Ok(stats) => Ok(Some(stats)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        }).await
    }
    
    pub async fn update_token_usage_stats(&self, input_tokens: i64, output_tokens: i64, cost_usd: f64) -> Result<()> {
        self.execute_with_metrics("update_token_usage_stats", move |conn| {
            let now = Utc::now().to_rfc3339();
            
            // 使用 UPSERT 語法（SQLite 3.24+）來簡化邏輯
            conn.execute(
                "INSERT INTO token_usage_stats (id, total_input_tokens, total_output_tokens, total_cost_usd, session_count, last_updated) 
                 VALUES (1, ?1, ?2, ?3, 1, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                 total_input_tokens = total_input_tokens + excluded.total_input_tokens,
                 total_output_tokens = total_output_tokens + excluded.total_output_tokens,
                 total_cost_usd = total_cost_usd + excluded.total_cost_usd,
                 session_count = session_count + 1,
                 last_updated = excluded.last_updated",
                params![input_tokens, output_tokens, cost_usd, now],
            )?;
            
            Ok(())
        }).await
    }
    
    // Clear cache manually if needed
    pub fn clear_cache(&self) {
        self.cache.clear();
        info!("Database cache cleared");
    }
    
    // Health check
    pub async fn health_check(&self) -> Result<bool> {
        self.execute_with_metrics("health_check", |conn| {
            // SELECT 查詢需使用 query_row 以避免 ExecuteReturnedResults 錯誤
            let _: i32 = conn.query_row("SELECT 1", [], |row| row.get(0))?;
            Ok(true)
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_database() -> Result<HighPerfDatabase> {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_owned();
        drop(temp_file);
        
        // Use performance mode for tests
        HighPerfDatabase::new_with_config(&db_path, true)
    }

    #[tokio::test]
    async fn test_database_creation() {
        let db = create_test_database();
        assert!(db.is_ok());
        
        if let Ok(db) = db {
            let health = db.health_check().await;
            assert!(health.is_ok());
            assert!(health.unwrap());
        }
    }

    #[tokio::test]
    async fn test_create_and_retrieve_prompt() {
        let db = create_test_database().unwrap();
        
        // 创建提示词
        let prompt_id = db.create_prompt("Test Title", "Test Content").await.unwrap();
        assert!(prompt_id > 0);
        
        // 获取提示词
        let retrieved_prompt = db.get_prompt(prompt_id).await.unwrap();
        assert!(retrieved_prompt.is_some());
        
        let prompt = retrieved_prompt.unwrap();
        assert_eq!(prompt.id, prompt_id);
        assert_eq!(prompt.title, "Test Title");
        assert_eq!(prompt.content, "Test Content");
        assert!(!prompt.created_at.is_empty());
    }

    #[tokio::test]
    async fn test_batch_create_prompts() {
        let db = create_test_database().unwrap();
        
        let prompts = vec![
            ("Title 1".to_string(), "Content 1".to_string(), Some("tag1".to_string())),
            ("Title 2".to_string(), "Content 2".to_string(), None),
            ("Title 3".to_string(), "Content 3".to_string(), Some("tag2,tag3".to_string())),
        ];
        
        let ids = db.batch_create_prompts(prompts).await.unwrap();
        assert_eq!(ids.len(), 3);
        
        for id in ids {
            assert!(id > 0);
            let prompt = db.get_prompt(id).await.unwrap();
            assert!(prompt.is_some());
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        use tokio::task;
        
        let db = Arc::new(create_test_database().unwrap());
        let mut handles = vec![];
        
        // 创建多个任务并发访问数据库
        for i in 0..20 {
            let db_clone = Arc::clone(&db);
            let handle = task::spawn(async move {
                let prompt_id = db_clone.create_prompt(&format!("Title {}", i), &format!("Content {}", i)).await.unwrap();
                assert!(prompt_id > 0);
                
                // Also test read operations
                let prompt = db_clone.get_prompt(prompt_id).await.unwrap();
                assert!(prompt.is_some());
                
                prompt_id
            });
            handles.push(handle);
        }
        
        // 等待所有任务完成
        let mut prompt_ids = vec![];
        for handle in handles {
            prompt_ids.push(handle.await.unwrap());
        }
        
        // 验证所有数据都已创建
        let prompts = db.list_prompts().await.unwrap();
        assert_eq!(prompts.len(), 20);
        
        // 验证所有ID都是唯一的
        let mut unique_ids = std::collections::HashSet::new();
        for id in prompt_ids {
            assert!(unique_ids.insert(id), "Duplicate prompt ID found: {}", id);
        }
        
        // Test pool status
        let (total, idle) = db.get_pool_status().await;
        assert!(total > 0);
        println!("Pool status: {} total connections, {} idle", total, idle);
        
        // Test performance metrics
        let metrics = db.get_performance_metrics();
        assert!(!metrics.is_empty());
        for (operation, metric) in metrics {
            println!("Operation: {}, calls: {}, avg time: {:.2}ms", 
                    operation, metric.total_calls, metric.avg_duration_ms);
        }
    }

    #[tokio::test]
    async fn test_performance_optimization() {
        let db = create_test_database().unwrap();
        
        // Create some test data
        for i in 0..100 {
            db.create_prompt(&format!("Perf Test {}", i), &format!("Content {}", i)).await.unwrap();
        }
        
        let start = Instant::now();
        
        // Test multiple concurrent reads
        let mut handles = vec![];
        for _ in 0..10 {
            let db_clone = Arc::new(db.clone());
            let handle = tokio::spawn(async move {
                db_clone.list_prompts().await.unwrap()
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let prompts = handle.await.unwrap();
            assert!(prompts.len() >= 100);
        }
        
        let duration = start.elapsed();
        println!("10 concurrent list operations took: {:?}", duration);
        assert!(duration < Duration::from_millis(1000)); // Should complete within 1 second
        
        // Test database optimization
        db.optimize_database().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let db = create_test_database().unwrap();
        
        // Create some test data
        db.create_prompt("Cached Test", "Cached Content").await.unwrap();
        
        // First call should hit database
        let start1 = Instant::now();
        let prompts1 = db.list_prompts().await.unwrap();
        let duration1 = start1.elapsed();
        
        // Second call should hit cache (should be faster)
        let start2 = Instant::now();
        let prompts2 = db.list_prompts().await.unwrap();
        let duration2 = start2.elapsed();
        
        assert_eq!(prompts1.len(), prompts2.len());
        assert_eq!(prompts1[0].id, prompts2[0].id);
        
        println!("First call: {:?}, Second call (cached): {:?}", duration1, duration2);
        
        // Clear cache and test
        db.clear_cache();
        let prompts3 = db.list_prompts().await.unwrap();
        assert_eq!(prompts1.len(), prompts3.len());
    }
}