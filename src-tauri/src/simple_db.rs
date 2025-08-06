// 簡化的資料庫模組，使用 rusqlite 避免依賴衝突

use rusqlite::{Connection, Result};
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
}

pub struct SimpleDatabase {
    conn: Connection,
}

impl SimpleDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
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
                FOREIGN KEY(prompt_id) REFERENCES prompts(id)
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
    
    pub fn create_schedule(&self, prompt_id: i64, schedule_time: &str) -> Result<i64> {
        let now = Local::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO schedules (prompt_id, schedule_time, status, created_at) VALUES (?1, ?2, 'pending', ?3)",
            [&prompt_id.to_string(), schedule_time, &now],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn get_pending_schedules(&self) -> Result<Vec<SimpleSchedule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, prompt_id, schedule_time, status, created_at 
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
            })
        })?;
        
        let mut schedules = Vec::new();
        for schedule in schedule_iter {
            schedules.push(schedule?);
        }
        
        Ok(schedules)
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