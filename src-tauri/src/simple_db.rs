// 簡化的資料庫模組，使用 rusqlite 避免依賴衝突

use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplePrompt {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleSchedule {
    pub id: i64,
    pub prompt_id: i64,
    pub schedule_time: String,
    pub status: String, // pending, running, completed, failed
    pub created_at: String,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub updated_at: Option<String>,
    pub cron_expr: Option<String>,
    pub execution_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: i64,
    pub schedule_id: i64,
    pub content: String,
    pub status: String, // success, failed, timeout
    pub token_usage: Option<i64>,
    pub cost_usd: Option<f64>,
    pub execution_time_ms: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsageStats {
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub session_count: i64,
    pub last_updated: String,
}

#[derive(Debug)]
pub struct SimpleDatabase {
    conn: Connection,
}

impl SimpleDatabase {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        // 使用明確的標誌來確保資料庫是可讀寫的，包含URI支持
        let conn = Connection::open_with_flags(
            db_path.as_ref(),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )?;

        // 啟用外鍵約束和WAL模式以提高並發性
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "temp_store", "memory")?;
        conn.pragma_update(None, "cache_size", 10000)?;

        // 創建表格
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

        Ok(Self { conn })
    }

    pub fn create_prompt(&self, title: &str, content: &str) -> Result<i64> {
        self.create_prompt_with_tags(title, content, None)
    }

    pub fn create_prompt_with_tags(
        &self,
        title: &str,
        content: &str,
        tags: Option<&str>,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO prompts (title, content, tags, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![title, content, tags, now],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_schedule(
        &self,
        prompt_id: i64,
        schedule_time: &str,
        cron_expr: Option<&str>,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO schedules (prompt_id, schedule_time, status, created_at, updated_at, cron_expr) VALUES (?1, ?2, 'pending', ?3, ?3, ?4)",
            params![prompt_id, schedule_time, now, cron_expr],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_schedule(
        &self,
        id: i64,
        schedule_time: Option<&str>,
        status: Option<&str>,
        cron_expr: Option<&str>,
    ) -> Result<()> {
        let mut sql = "UPDATE schedules SET updated_at = ?".to_string();
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(Utc::now().to_rfc3339())];

        if let Some(time) = schedule_time {
            sql.push_str(", schedule_time = ?");
            param_values.push(Box::new(time.to_string()));
        }

        if let Some(stat) = status {
            sql.push_str(", status = ?");
            param_values.push(Box::new(stat.to_string()));
        }

        if let Some(cron) = cron_expr {
            sql.push_str(", cron_expr = ?");
            param_values.push(Box::new(cron.to_string()));
        }

        sql.push_str(" WHERE id = ?");
        param_values.push(Box::new(id));

        let params: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        self.conn.execute(&sql, params.as_slice())?;

        Ok(())
    }

    pub fn delete_schedule(&self, id: i64) -> Result<bool> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM schedules WHERE id = ?1", params![id])?;

        Ok(rows_affected > 0)
    }

    pub fn list_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt_id, schedule_time, status, created_at, last_run_at, next_run_at, updated_at, cron_expr, execution_count
             FROM schedules 
             ORDER BY created_at DESC"
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
    }

    pub fn get_schedule(&self, id: i64) -> Result<Option<SimpleSchedule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt_id, schedule_time, status, created_at, last_run_at, next_run_at, updated_at, cron_expr, execution_count
             FROM schedules 
             WHERE id = ?1"
        )?;

        let mut schedule_iter = stmt.query_map([id], |row| {
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

        match schedule_iter.next() {
            Some(Ok(schedule)) => Ok(Some(schedule)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt_id, schedule_time, status, created_at, last_run_at, next_run_at, updated_at, cron_expr, execution_count
             FROM schedules 
             WHERE status = 'pending' 
             ORDER BY schedule_time"
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
    }

    pub fn record_execution_result(
        &self,
        schedule_id: i64,
        content: &str,
        status: &str,
        token_usage: Option<i64>,
        cost_usd: Option<f64>,
        execution_time_ms: i64,
    ) -> Result<i64> {
        let now = Utc::now().to_rfc3339();

        // 使用事務確保數據一致性
        let tx = self.conn.unchecked_transaction()?;

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
    }

    // 查詢特定排程的執行結果
    pub fn get_execution_results(&self, schedule_id: i64) -> Result<Vec<ExecutionResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, schedule_id, content, status, token_usage, cost_usd, execution_time_ms, created_at 
             FROM execution_results 
             WHERE schedule_id = ?1 
             ORDER BY created_at DESC"
        )?;

        let result_iter = stmt.query_map(params![schedule_id], |row| {
            Ok(ExecutionResult {
                id: row.get(0)?,
                schedule_id: row.get(1)?,
                content: row.get(2)?,
                status: row.get(3)?,
                token_usage: row.get(4)?,
                cost_usd: row.get(5)?,
                execution_time_ms: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }

        Ok(results)
    }

    // 查詢所有執行結果（用於總覽）
    pub fn list_all_execution_results(&self, limit: Option<i64>) -> Result<Vec<ExecutionResult>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let sql = format!(
            "SELECT id, schedule_id, content, status, token_usage, cost_usd, execution_time_ms, created_at 
             FROM execution_results 
             ORDER BY created_at DESC{}",
            limit_clause
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let result_iter = stmt.query_map([], |row| {
            Ok(ExecutionResult {
                id: row.get(0)?,
                schedule_id: row.get(1)?,
                content: row.get(2)?,
                status: row.get(3)?,
                token_usage: row.get(4)?,
                cost_usd: row.get(5)?,
                execution_time_ms: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for result in result_iter {
            results.push(result?);
        }

        Ok(results)
    }

    pub fn update_token_usage_stats(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        cost_usd: f64,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // 使用 UPSERT 語法（SQLite 3.24+）來簡化邏輯
        self.conn.execute(
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
    }

    pub fn get_token_usage_stats(&self) -> Result<Option<TokenUsageStats>> {
        let mut stmt = self.conn.prepare(
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
    }

    pub fn update_schedule_status(&self, id: i64, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE schedules SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;

        Ok(())
    }

    pub fn list_prompts(&self) -> Result<Vec<SimplePrompt>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, created_at FROM prompts ORDER BY created_at DESC",
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
    }

    pub fn get_prompt(&self, id: i64) -> Result<Option<SimplePrompt>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, content, tags, created_at FROM prompts WHERE id = ?1")?;

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
    }

    pub fn delete_prompt(&self, id: i64) -> Result<bool> {
        let mut stmt = self.conn.prepare("DELETE FROM prompts WHERE id = ?1")?;

        let rows_affected = stmt.execute([id])?;
        Ok(rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_database() -> Result<SimpleDatabase> {
        // 創建命名臨時文件而不是目錄中的文件，確保文件可寫
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_owned();
        // 關閉文件句柄，讓SQLite可以打開它
        drop(temp_file);

        SimpleDatabase::new(&db_path)
    }

    #[allow(dead_code)]
    fn create_in_memory_database() -> Result<SimpleDatabase> {
        SimpleDatabase::new(":memory:")
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_database();
        match &db {
            Ok(_) => {}
            Err(e) => println!("Database creation failed: {:?}", e),
        }
        assert!(db.is_ok());
    }

    #[test]
    fn test_create_and_retrieve_prompt() {
        let db = create_test_database().unwrap();

        // 创建提示词
        let prompt_id = db.create_prompt("Test Title", "Test Content").unwrap();
        assert!(prompt_id > 0);

        // 获取提示词
        let retrieved_prompt = db.get_prompt(prompt_id).unwrap();
        assert!(retrieved_prompt.is_some());

        let prompt = retrieved_prompt.unwrap();
        assert_eq!(prompt.id, prompt_id);
        assert_eq!(prompt.title, "Test Title");
        assert_eq!(prompt.content, "Test Content");
        assert!(!prompt.created_at.is_empty());
    }

    #[test]
    fn test_list_prompts() {
        let db = create_test_database().unwrap();

        // 创建多个提示词
        let id1 = db.create_prompt("Title 1", "Content 1").unwrap();
        let id2 = db.create_prompt("Title 2", "Content 2").unwrap();

        // 列出所有提示词
        let prompts = db.list_prompts().unwrap();
        assert_eq!(prompts.len(), 2);

        // 验证排序（应该按创建时间倒序）
        assert_eq!(prompts[0].id, id2); // 最新的在前
        assert_eq!(prompts[1].id, id1);
    }

    #[test]
    fn test_create_and_retrieve_schedule() {
        let db = create_test_database().unwrap();

        // 首先创建一个提示词
        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();

        // 创建排程
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", Some("0 0 9 * * *")) // 6欄位格式：每日上夈9點
            .unwrap();
        assert!(schedule_id > 0);

        // 获取排程
        let retrieved_schedule = db.get_schedule(schedule_id).unwrap();
        assert!(retrieved_schedule.is_some());

        let schedule = retrieved_schedule.unwrap();
        assert_eq!(schedule.id, schedule_id);
        assert_eq!(schedule.prompt_id, prompt_id);
        assert_eq!(schedule.schedule_time, "2025-01-01T09:00:00Z");
        assert_eq!(schedule.status, "pending");
        assert_eq!(schedule.cron_expr, Some("0 0 9 * * *".to_string())); // 6欄位格式
        assert_eq!(schedule.execution_count, 0);
    }

    #[test]
    fn test_update_schedule() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 更新排程
        db.update_schedule(
            schedule_id,
            Some("2025-01-02T10:00:00Z"),
            Some("running"),
            Some("0 10 * * *"),
        )
        .unwrap();

        // 验证更新
        let updated_schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(updated_schedule.schedule_time, "2025-01-02T10:00:00Z");
        assert_eq!(updated_schedule.status, "running");
        assert_eq!(updated_schedule.cron_expr, Some("0 10 * * *".to_string()));
    }

    #[test]
    fn test_update_schedule_partial() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", Some("original"))
            .unwrap();

        // 只更新状态
        db.update_schedule(schedule_id, None, Some("completed"), None)
            .unwrap();

        let updated_schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(updated_schedule.schedule_time, "2025-01-01T09:00:00Z"); // 未变
        assert_eq!(updated_schedule.status, "completed"); // 已更新
        assert_eq!(updated_schedule.cron_expr, Some("original".to_string())); // 未变
    }

    #[test]
    fn test_update_schedule_no_changes() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 不更新任何内容
        let result = db.update_schedule(schedule_id, None, None, None);
        assert!(result.is_ok()); // 应该成功但不做任何更改
    }

    #[test]
    fn test_delete_schedule() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 删除排程
        let deleted = db.delete_schedule(schedule_id).unwrap();
        assert!(deleted);

        // 验证已删除
        let retrieved_schedule = db.get_schedule(schedule_id).unwrap();
        assert!(retrieved_schedule.is_none());

        // 删除不存在的排程
        let not_deleted = db.delete_schedule(9999).unwrap();
        assert!(!not_deleted);
    }

    #[test]
    fn test_list_schedules() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let id1 = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();
        let id2 = db
            .create_schedule(prompt_id, "2025-01-02T10:00:00Z", None)
            .unwrap();

        let schedules = db.list_schedules().unwrap();
        assert_eq!(schedules.len(), 2);

        // 验证排序（按创建时间倒序）
        assert_eq!(schedules[0].id, id2);
        assert_eq!(schedules[1].id, id1);
    }

    #[test]
    fn test_get_pending_schedules() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let pending_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();
        let completed_id = db
            .create_schedule(prompt_id, "2025-01-02T10:00:00Z", None)
            .unwrap();

        // 更新一个为completed状态
        db.update_schedule_status(completed_id, "completed")
            .unwrap();

        // 获取待处理排程
        let pending_schedules = db.get_pending_schedules().unwrap();
        assert_eq!(pending_schedules.len(), 1);
        assert_eq!(pending_schedules[0].id, pending_id);
        assert_eq!(pending_schedules[0].status, "pending");
    }

    #[test]
    fn test_record_execution_result() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 记录执行结果
        let result_id = db
            .record_execution_result(
                schedule_id,
                "Execution successful",
                "success",
                Some(1000),
                Some(0.01),
                2500,
            )
            .unwrap();

        assert!(result_id > 0);

        // 验证排程的执行次数已更新
        let updated_schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(updated_schedule.execution_count, 1);
        assert!(updated_schedule.last_run_at.is_some());
    }

    #[test]
    fn test_token_usage_stats_creation() {
        let db = create_test_database().unwrap();

        // 初始时应该没有统计数据
        let initial_stats = db.get_token_usage_stats().unwrap();
        assert!(initial_stats.is_none());

        // 更新统计数据
        db.update_token_usage_stats(1000, 500, 0.05).unwrap();

        // 获取统计数据
        let stats = db.get_token_usage_stats().unwrap();
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_input_tokens, 1000);
        assert_eq!(stats.total_output_tokens, 500);
        assert_eq!(stats.total_cost_usd, 0.05);
        assert_eq!(stats.session_count, 1);
        assert!(!stats.last_updated.is_empty());
    }

    #[test]
    fn test_token_usage_stats_accumulation() {
        let db = create_test_database().unwrap();

        // 第一次更新
        db.update_token_usage_stats(1000, 500, 0.05).unwrap();

        // 第二次更新
        db.update_token_usage_stats(2000, 800, 0.10).unwrap();

        // 验证累积结果
        let stats = db.get_token_usage_stats().unwrap().unwrap();
        assert_eq!(stats.total_input_tokens, 3000);
        assert_eq!(stats.total_output_tokens, 1300);
        assert!((stats.total_cost_usd - 0.15).abs() < 0.001); // 使用浮點數精度比較
        assert_eq!(stats.session_count, 2);
    }

    #[test]
    fn test_update_schedule_status() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 更新状态
        db.update_schedule_status(schedule_id, "running").unwrap();

        // 验证状态更新
        let updated_schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(updated_schedule.status, "running");
    }

    #[test]
    fn test_data_structures_serialization() {
        // 测试 SimplePrompt 序列化
        let prompt = SimplePrompt {
            id: 1,
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            tags: Some("tag1,tag2".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&prompt).unwrap();
        let deserialized: SimplePrompt = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.title, "Test Title");
        assert_eq!(deserialized.content, "Test Content");
        assert_eq!(deserialized.tags, Some("tag1,tag2".to_string()));

        // 测试 SimpleSchedule 序列化
        let schedule = SimpleSchedule {
            id: 1,
            prompt_id: 1,
            schedule_time: "2025-01-01T09:00:00Z".to_string(),
            status: "pending".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_run_at: None,
            next_run_at: Some("2025-01-02T09:00:00Z".to_string()),
            updated_at: None,
            cron_expr: Some("0 0 9 * * *".to_string()), // 6欄位格式
            execution_count: 0,
        };

        let serialized = serde_json::to_string(&schedule).unwrap();
        let deserialized: SimpleSchedule = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.status, "pending");
    }

    #[test]
    fn test_database_transaction_consistency() {
        let db = create_test_database().unwrap();

        let prompt_id = db.create_prompt("Test Prompt", "Test Content").unwrap();
        let schedule_id = db
            .create_schedule(prompt_id, "2025-01-01T09:00:00Z", None)
            .unwrap();

        // 记录执行结果应该自动更新排程统计
        db.record_execution_result(
            schedule_id,
            "Success",
            "success",
            Some(1000),
            Some(0.01),
            1500,
        )
        .unwrap();

        let schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(schedule.execution_count, 1);
        assert!(schedule.last_run_at.is_some());

        // 再次执行
        db.record_execution_result(
            schedule_id,
            "Success again",
            "success",
            Some(1200),
            Some(0.012),
            1800,
        )
        .unwrap();

        let updated_schedule = db.get_schedule(schedule_id).unwrap().unwrap();
        assert_eq!(updated_schedule.execution_count, 2);
    }

    #[test]
    fn test_error_handling() {
        let db = create_test_database().unwrap();

        // 尝试获取不存在的提示词
        let non_existent_prompt = db.get_prompt(9999).unwrap();
        assert!(non_existent_prompt.is_none());

        // 尝试获取不存在的排程
        let non_existent_schedule = db.get_schedule(9999).unwrap();
        assert!(non_existent_schedule.is_none());

        // 尝试删除不存在的排程
        let delete_result = db.delete_schedule(9999).unwrap();
        assert!(!delete_result);
    }

    #[test]
    fn test_foreign_key_constraint() {
        let db = create_test_database().unwrap();

        // 尝试创建引用不存在prompt的排程
        // 注意: 我们已經啟用了外鍵約束，所以這應該失敗
        let result = db.create_schedule(9999, "2025-01-01T09:00:00Z", None);
        assert!(result.is_err()); // 應該因為外鍵約束失敗
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let db = Arc::new(Mutex::new(create_test_database().unwrap()));
        let mut handles = vec![];

        // 创建多个线程并发访问数据库
        for i in 0..5 {
            let db_clone = Arc::clone(&db);
            let handle = thread::spawn(move || {
                let db = db_clone.lock().unwrap();
                let prompt_id = db
                    .create_prompt(&format!("Title {}", i), &format!("Content {}", i))
                    .unwrap();
                assert!(prompt_id > 0);
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有数据都已创建
        let db = db.lock().unwrap();
        let prompts = db.list_prompts().unwrap();
        assert_eq!(prompts.len(), 5);
    }
}
