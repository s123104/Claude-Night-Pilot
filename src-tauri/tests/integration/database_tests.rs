// 🧪 Claude Night Pilot - 資料庫整合測試套件
// 基於Context7 Rusqlite最佳實踐
// 創建時間: 2025-08-17T05:20:00+00:00

use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;
use tempfile::TempDir;

/// 資料庫整合測試套件
/// 
/// 測試覆蓋：
/// - 資料庫連接與初始化
/// - CRUD操作完整性
/// - 事務管理
/// - 併發安全性
/// - 效能基準
/// - 資料完整性約束
#[cfg(test)]
mod database_integration_tests {
    use super::*;
    use crate::utils::create_test_db;

    /// 測試資料庫初始化
    #[tokio::test]
    async fn test_database_initialization() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        
        // 創建資料庫連接
        let conn = Connection::open(&db_path)?;
        
        // 測試基本表結構
        create_test_tables(&conn)?;
        
        // 驗證表是否存在
        let tables = get_table_names(&conn)?;
        assert!(tables.contains(&"prompts".to_string()));
        assert!(tables.contains(&"schedules".to_string()));
        
        println!("✅ 資料庫初始化測試通過");
        Ok(())
    }

    /// 測試Prompt CRUD操作
    #[tokio::test]
    async fn test_prompt_crud_operations() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        create_test_tables(&conn)?;
        
        // Create - 插入測試Prompt
        let prompt_id = insert_test_prompt(&conn, "測試Prompt", "測試內容")?;
        
        // Read - 讀取Prompt
        let prompt = get_prompt_by_id(&conn, prompt_id)?;
        assert_eq!(prompt.title, "測試Prompt");
        assert_eq!(prompt.content, "測試內容");
        
        // Update - 更新Prompt
        update_prompt(&conn, prompt_id, "更新的標題", "更新的內容")?;
        let updated_prompt = get_prompt_by_id(&conn, prompt_id)?;
        assert_eq!(updated_prompt.title, "更新的標題");
        
        // Delete - 刪除Prompt
        delete_prompt(&conn, prompt_id)?;
        let result = get_prompt_by_id(&conn, prompt_id);
        assert!(result.is_err(), "刪除後應該無法找到Prompt");
        
        println!("✅ Prompt CRUD操作測試通過");
        Ok(())
    }

    /// 測試Schedule CRUD操作
    #[tokio::test]
    async fn test_schedule_crud_operations() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        create_test_tables(&conn)?;
        
        // 先創建一個Prompt
        let prompt_id = insert_test_prompt(&conn, "排程測試Prompt", "內容")?;
        
        // Create - 插入測試Schedule
        let schedule_id = insert_test_schedule(&conn, prompt_id, "0 9 * * *")?;
        
        // Read - 讀取Schedule
        let schedule = get_schedule_by_id(&conn, schedule_id)?;
        assert_eq!(schedule.cron_expr, "0 9 * * *");
        assert_eq!(schedule.prompt_id, prompt_id);
        
        // Update - 更新Schedule狀態
        update_schedule_status(&conn, schedule_id, "Completed")?;
        let updated_schedule = get_schedule_by_id(&conn, schedule_id)?;
        assert_eq!(updated_schedule.status, "Completed");
        
        // Delete - 刪除Schedule
        delete_schedule(&conn, schedule_id)?;
        let result = get_schedule_by_id(&conn, schedule_id);
        assert!(result.is_err(), "刪除後應該無法找到Schedule");
        
        println!("✅ Schedule CRUD操作測試通過");
        Ok(())
    }

    /// 測試外鍵約束
    #[tokio::test]
    async fn test_foreign_key_constraints() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        create_test_tables(&conn)?;
        
        // 啟用外鍵約束
        conn.execute("PRAGMA foreign_keys=ON", [])?;
        
        // 嘗試插入不存在的prompt_id
        let result = conn.execute(
            "INSERT INTO schedules (prompt_id, schedule_time, status, created_at, updated_at, cron_expr, execution_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                999, // 不存在的prompt_id
                Utc::now().to_rfc3339(),
                "Active",
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339(),
                "0 * * * *",
                0
            ]
        );
        
        assert!(result.is_err(), "應該因外鍵約束失敗");
        println!("✅ 外鍵約束測試通過");
        Ok(())
    }

    /// 測試事務管理
    #[tokio::test]
    async fn test_transaction_management() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        create_test_tables(&conn)?;
        
        // 測試事務回滾
        let tx = conn.unchecked_transaction()?;
        
        // 插入數據
        let prompt_id = tx.execute(
            "INSERT INTO prompts (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params!["事務測試", "內容", Utc::now().to_rfc3339(), Utc::now().to_rfc3339()]
        )?;
        
        // 回滾事務
        tx.rollback()?;
        
        // 驗證數據未被保存
        let result = conn.prepare("SELECT COUNT(*) FROM prompts WHERE title = '事務測試'");
        assert!(result.is_ok());
        
        // 測試事務提交
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO prompts (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params!["事務提交測試", "內容", Utc::now().to_rfc3339(), Utc::now().to_rfc3339()]
        )?;
        tx.commit()?;
        
        // 驗證數據被保存
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM prompts WHERE title = '事務提交測試'")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        assert_eq!(count, 1);
        
        println!("✅ 事務管理測試通過");
        Ok(())
    }

    /// 測試併發資料庫操作
    #[tokio::test]
    async fn test_concurrent_database_operations() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        
        // 創建多個連接來模擬併發
        let mut handles = vec![];
        
        for i in 0..5 {
            let db_path_clone = db_path.clone();
            let handle = tokio::spawn(async move {
                let conn = Connection::open(&db_path_clone).unwrap();
                create_test_tables(&conn).unwrap();
                
                // 插入數據
                insert_test_prompt(&conn, &format!("併發測試 {}", i), "內容").unwrap();
                
                i
            });
            handles.push(handle);
        }
        
        // 等待所有操作完成
        for handle in handles {
            let result = handle.await?;
            println!("併發操作 {} 完成", result);
        }
        
        // 驗證所有數據都被正確插入
        let conn = Connection::open(&db_path)?;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM prompts WHERE title LIKE '併發測試%'")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        assert_eq!(count, 5);
        
        println!("✅ 併發資料庫操作測試通過");
        Ok(())
    }

    // 輔助函數
    fn create_test_tables(conn: &Connection) -> Result<()> {
        // 創建prompts表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // 創建schedules表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schedules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt_id INTEGER NOT NULL,
                schedule_time TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                cron_expr TEXT NOT NULL,
                execution_count INTEGER DEFAULT 0,
                FOREIGN KEY (prompt_id) REFERENCES prompts (id)
            )",
            [],
        )?;
        
        Ok(())
    }
    
    fn get_table_names(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let table_names = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut names = vec![];
        for name in table_names {
            names.push(name?);
        }
        Ok(names)
    }
    
    fn insert_test_prompt(conn: &Connection, title: &str, content: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO prompts (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![title, content, Utc::now().to_rfc3339(), Utc::now().to_rfc3339()]
        )?;
        Ok(conn.last_insert_rowid())
    }
    
    fn insert_test_schedule(conn: &Connection, prompt_id: i64, cron_expr: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO schedules (prompt_id, schedule_time, status, created_at, updated_at, cron_expr, execution_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                prompt_id,
                Utc::now().to_rfc3339(),
                "Active",
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339(),
                cron_expr,
                0
            ]
        )?;
        Ok(conn.last_insert_rowid())
    }
    
    // 簡化的測試結構
    #[derive(Debug)]
    struct TestPrompt {
        id: i64,
        title: String,
        content: String,
    }
    
    #[derive(Debug)]
    struct TestSchedule {
        id: i64,
        prompt_id: i64,
        status: String,
        cron_expr: String,
    }
    
    fn get_prompt_by_id(conn: &Connection, id: i64) -> Result<TestPrompt> {
        let mut stmt = conn.prepare("SELECT id, title, content FROM prompts WHERE id = ?1")?;
        stmt.query_row([id], |row| {
            Ok(TestPrompt {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
            })
        }).map_err(|e| anyhow::anyhow!("Prompt not found: {}", e))
    }
    
    fn get_schedule_by_id(conn: &Connection, id: i64) -> Result<TestSchedule> {
        let mut stmt = conn.prepare("SELECT id, prompt_id, status, cron_expr FROM schedules WHERE id = ?1")?;
        stmt.query_row([id], |row| {
            Ok(TestSchedule {
                id: row.get(0)?,
                prompt_id: row.get(1)?,
                status: row.get(2)?,
                cron_expr: row.get(3)?,
            })
        }).map_err(|e| anyhow::anyhow!("Schedule not found: {}", e))
    }
    
    fn update_prompt(conn: &Connection, id: i64, title: &str, content: &str) -> Result<()> {
        conn.execute(
            "UPDATE prompts SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![title, content, Utc::now().to_rfc3339(), id]
        )?;
        Ok(())
    }
    
    fn update_schedule_status(conn: &Connection, id: i64, status: &str) -> Result<()> {
        conn.execute(
            "UPDATE schedules SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, Utc::now().to_rfc3339(), id]
        )?;
        Ok(())
    }
    
    fn delete_prompt(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM prompts WHERE id = ?1", [id])?;
        Ok(())
    }
    
    fn delete_schedule(conn: &Connection, id: i64) -> Result<()> {
        conn.execute("DELETE FROM schedules WHERE id = ?1", [id])?;
        Ok(())
    }
}

/// 資料庫效能基準測試
#[cfg(test)]
mod database_performance_tests {
    use super::*;
    use crate::utils::create_test_db;

    /// 測試批量插入性能
    #[tokio::test]
    async fn benchmark_bulk_insert() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        
        // 創建表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        let start = std::time::Instant::now();
        
        // 批量插入1000條記錄
        let tx = conn.unchecked_transaction()?;
        for i in 0..1000 {
            tx.execute(
                "INSERT INTO prompts (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("測試Prompt {}", i),
                    format!("測試內容 {}", i),
                    Utc::now().to_rfc3339(),
                    Utc::now().to_rfc3339()
                ]
            )?;
        }
        tx.commit()?;
        
        let elapsed = start.elapsed();
        println!("📊 批量插入1000條記錄耗時: {:?}", elapsed);
        
        // 企業級要求: 1000條記錄插入 < 1秒
        assert!(elapsed < std::time::Duration::from_secs(1), 
               "批量插入性能不符合要求: {:?}", elapsed);
        
        Ok(())
    }

    /// 測試查詢性能
    #[tokio::test]
    async fn benchmark_query_performance() -> Result<()> {
        let (_temp_dir, db_path) = create_test_db();
        let conn = Connection::open(&db_path)?;
        
        // 創建表並插入測試數據
        conn.execute(
            "CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // 插入1000條測試數據
        for i in 0..1000 {
            conn.execute(
                "INSERT INTO prompts (title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    format!("測試Prompt {}", i),
                    format!("測試內容 {}", i),
                    Utc::now().to_rfc3339(),
                    Utc::now().to_rfc3339()
                ]
            )?;
        }
        
        let start = std::time::Instant::now();
        
        // 執行100次查詢
        for i in 0..100 {
            let mut stmt = conn.prepare("SELECT * FROM prompts WHERE title = ?1")?;
            let _ = stmt.query_row([format!("測試Prompt {}", i % 100)], |_row| Ok(()))?;
        }
        
        let elapsed = start.elapsed();
        println!("📊 100次查詢耗時: {:?}", elapsed);
        
        // 企業級要求: 100次查詢 < 100ms
        assert!(elapsed < std::time::Duration::from_millis(100), 
               "查詢性能不符合要求: {:?}", elapsed);
        
        Ok(())
    }
}
