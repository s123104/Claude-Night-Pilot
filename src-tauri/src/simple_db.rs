// 簡化的資料庫模組，使用 rusqlite 避免依賴衝突

use rusqlite::{Connection, Result, params, OpenFlags};
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplePrompt {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleSchedule {
    pub id: i64,
    pub prompt_id: i64,
    pub schedule_time: String,
    pub status: String, // pending, running, completed, failed
    pub created_at: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
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

pub struct SimpleDatabase {
    conn: Connection,
}

impl SimpleDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        // 使用明確的標誌來確保資料庫是可讀寫的
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        )?;
        
        // 創建表格
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
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
                last_run TEXT,
                next_run TEXT,
                cron_expr TEXT,
                execution_count INTEGER DEFAULT 0,
                FOREIGN KEY(prompt_id) REFERENCES prompts(id)
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
                id INTEGER PRIMARY KEY AUTOINCREMENT,
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
        let now = Local::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO prompts (title, content, created_at) VALUES (?1, ?2, ?3)",
            [title, content, &now],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn create_schedule(&self, prompt_id: i64, schedule_time: &str, cron_expr: Option<&str>) -> Result<i64> {
        let now = Local::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO schedules (prompt_id, schedule_time, status, created_at, cron_expr) VALUES (?1, ?2, 'pending', ?3, ?4)",
            params![prompt_id, schedule_time, now, cron_expr],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn update_schedule(&self, id: i64, schedule_time: Option<&str>, status: Option<&str>, cron_expr: Option<&str>) -> Result<()> {
        let mut sql = "UPDATE schedules SET ".to_string();
        let mut params_vec = Vec::new();
        let mut updates = Vec::new();
        
        if let Some(time) = schedule_time {
            updates.push("schedule_time = ?");
            params_vec.push(time.to_string());
        }
        
        if let Some(stat) = status {
            updates.push("status = ?");
            params_vec.push(stat.to_string());
        }
        
        if let Some(cron) = cron_expr {
            updates.push("cron_expr = ?");
            params_vec.push(cron.to_string());
        }
        
        if updates.is_empty() {
            return Ok(());
        }
        
        sql.push_str(&updates.join(", "));
        sql.push_str(" WHERE id = ?");
        params_vec.push(id.to_string());
        
        // 使用動態參數構建
        match params_vec.len() {
            1 => self.conn.execute(&sql, [&params_vec[0]])?,
            2 => self.conn.execute(&sql, [&params_vec[0], &params_vec[1]])?,
            3 => self.conn.execute(&sql, [&params_vec[0], &params_vec[1], &params_vec[2]])?,
            _ => return Err(rusqlite::Error::InvalidParameterCount(params_vec.len(), params_vec.len())),
        };
        
        Ok(())
    }
    
    pub fn delete_schedule(&self, id: i64) -> Result<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM schedules WHERE id = ?1",
            params![id],
        )?;
        
        Ok(rows_affected > 0)
    }
    
    pub fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt_id, schedule_time, status, created_at, last_run, next_run, cron_expr, execution_count
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
                last_run: row.get(5)?,
                next_run: row.get(6)?,
                cron_expr: row.get(7)?,
                execution_count: row.get(8)?,
            })
        })?;
        
        let mut schedules = Vec::new();
        for schedule in schedule_iter {
            schedules.push(schedule?);
        }
        
        Ok(schedules)
    }
    
    pub fn record_execution_result(&self, schedule_id: i64, content: &str, status: &str, token_usage: Option<i64>, cost_usd: Option<f64>, execution_time_ms: i64) -> Result<i64> {
        let now = Local::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO execution_results (schedule_id, content, status, token_usage, cost_usd, execution_time_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![schedule_id, content, status, token_usage, cost_usd, execution_time_ms, now],
        )?;
        
        // 更新排程的執行次數
        self.conn.execute(
            "UPDATE schedules SET execution_count = execution_count + 1, last_run = ?1 WHERE id = ?2",
            params![now, schedule_id],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn update_token_usage_stats(&self, input_tokens: i64, output_tokens: i64, cost_usd: f64) -> Result<()> {
        let now = Local::now().to_rfc3339();
        
        // 檢查是否已有記錄
        let exists: bool = self.conn
            .prepare("SELECT COUNT(*) FROM token_usage_stats")?
            .query_row([], |row| row.get::<_, i64>(0))?
            > 0;
        
        if exists {
            self.conn.execute(
                "UPDATE token_usage_stats SET 
                 total_input_tokens = total_input_tokens + ?1,
                 total_output_tokens = total_output_tokens + ?2,
                 total_cost_usd = total_cost_usd + ?3,
                 session_count = session_count + 1,
                 last_updated = ?4",
                params![input_tokens, output_tokens, cost_usd, now],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO token_usage_stats (total_input_tokens, total_output_tokens, total_cost_usd, session_count, last_updated) VALUES (?1, ?2, ?3, 1, ?4)",
                params![input_tokens, output_tokens, cost_usd, now],
            )?;
        }
        
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
        self.conn.execute(
            "UPDATE schedules SET status = ?1 WHERE id = ?2",
            [status, &id.to_string()],
        )?;
        
        Ok(())
    }
    
    pub fn get_prompt(&self, id: i64) -> Result<Option<SimplePrompt>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, created_at FROM prompts WHERE id = ?1"
        )?;
        
        let mut prompt_iter = stmt.query_map([id], |row| {
            Ok(SimplePrompt {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        
        match prompt_iter.next() {
            Some(Ok(prompt)) => Ok(Some(prompt)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}