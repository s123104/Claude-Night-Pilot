// 基於Rusqlite的最佳實踐數據庫管理器
// 結合Context7 SQLite建議和Vibe-Kanban的Active Record便利性

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, Result as SqliteResult, Row, Transaction};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;

/// 最佳實踐Rusqlite配置
#[derive(Debug, Clone)]
pub struct RusqliteBestPracticesConfig {
    pub database_path: PathBuf,
    pub connection_timeout: Duration,
    pub busy_timeout: Duration,
    pub enable_foreign_keys: bool,
    pub enable_wal_mode: bool,
    pub page_size: Option<u32>,
    pub cache_size: Option<i32>,
    pub synchronous_mode: SynchronousMode,
    pub journal_mode: JournalMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SynchronousMode {
    Off = 0,
    Normal = 1,
    Full = 2,
    Extra = 3,
}

#[derive(Debug, Clone)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl Default for RusqliteBestPracticesConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("claude-pilot-best-practices.db"),
            connection_timeout: Duration::from_secs(30),
            busy_timeout: Duration::from_secs(30),
            enable_foreign_keys: true,
            enable_wal_mode: true,
            page_size: Some(4096),
            cache_size: Some(-2000), // -2000 = 2MB cache
            synchronous_mode: SynchronousMode::Normal,
            journal_mode: JournalMode::Wal,
        }
    }
}

/// 企業級Rusqlite數據庫管理器
pub struct RusqliteBestPracticesManager {
    connection: Arc<RwLock<Connection>>,
    config: RusqliteBestPracticesConfig,
}

impl RusqliteBestPracticesManager {
    /// 使用最佳實踐創建數據庫管理器
    pub fn new(config: RusqliteBestPracticesConfig) -> Result<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;

        let connection = Connection::open_with_flags(&config.database_path, flags)?;
        
        // 配置數據庫最佳實踐設定
        Self::configure_connection(&connection, &config)?;
        
        Ok(Self {
            connection: Arc::new(RwLock::new(connection)),
            config,
        })
    }

    /// 配置連接的最佳實踐設定
    fn configure_connection(conn: &Connection, config: &RusqliteBestPracticesConfig) -> SqliteResult<()> {
        // Context7推薦：啟用外鍵約束
        if config.enable_foreign_keys {
            conn.pragma_update(None, "foreign_keys", true)?;
        }

        // 設定繁忙超時
        conn.busy_timeout(config.busy_timeout)?;

        // Context7推薦：使用WAL模式提高並發性（journal_mode 會返回結果，使用 pragma_update 處理）
        if config.enable_wal_mode {
            let mode = match config.journal_mode {
                JournalMode::Delete => "DELETE",
                JournalMode::Truncate => "TRUNCATE",
                JournalMode::Persist => "PERSIST",
                JournalMode::Memory => "MEMORY",
                JournalMode::Wal => "WAL",
                JournalMode::Off => "OFF",
            };
            conn.pragma_update(None, "journal_mode", mode)?;
        }

        // 設定頁面大小
        if let Some(page_size) = config.page_size {
            conn.pragma_update(None, "page_size", page_size as i64)?;
        }

        // 設定緩存大小
        if let Some(cache_size) = config.cache_size {
            conn.pragma_update(None, "cache_size", cache_size as i64)?;
        }

        // 設定同步模式
        let sync_level = config.synchronous_mode as u8;
        conn.pragma_update(None, "synchronous", sync_level as i64)?;

        // SQLite最佳實踐：預編譯常用語句優化
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "mmap_size", 268435456i64)?; // 256MB memory map

        Ok(())
    }

    /// 執行健康檢查
    pub fn health_check(&self) -> Result<DatabaseHealthMetrics> {
        let start_time = std::time::Instant::now();
        let connection = self.connection.read();
        
        // 測試基本查詢
        let health_result: i32 = connection.query_row("SELECT 1", [], |row| {
            row.get(0)
        })?;
        
        let response_time = start_time.elapsed();
        
        // 獲取數據庫統計信息
        let page_count: i64 = connection.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = connection.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let database_size = page_count * page_size;

        Ok(DatabaseHealthMetrics {
            is_healthy: health_result == 1,
            response_time_ms: response_time.as_millis() as u32,
            database_size_bytes: database_size as u64,
            page_count: page_count as u32,
            checked_at: Utc::now(),
        })
    }

    /// 執行事務
    pub fn execute_transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Transaction) -> Result<T>,
    {
        let connection = self.connection.write();
        let tx = connection.unchecked_transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    /// 獲取數據庫連接的引用
    pub fn with_connection<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let connection = self.connection.read();
        f(&*connection)
    }

    /// 備份數據庫
    pub fn backup_database<P: AsRef<std::path::Path>>(&self, backup_path: P) -> Result<BackupMetrics> {
        let start_time = std::time::Instant::now();
        let connection = self.connection.read();
        
        // 使用SQLite的備份API
        let mut backup_conn = Connection::open(&backup_path)?;
        let backup = rusqlite::backup::Backup::new(&connection, &mut backup_conn)?;
        backup.run_to_completion(5, Duration::from_millis(250), None)?;
        
        let duration = start_time.elapsed();
        let file_size = std::fs::metadata(&backup_path)?.len();
        
        Ok(BackupMetrics {
            backup_path: backup_path.as_ref().to_path_buf(),
            file_size_bytes: file_size,
            duration,
            created_at: Utc::now(),
        })
    }

    /// 數據庫維護操作
    pub fn maintenance(&self) -> Result<MaintenanceResult> {
        let start_time = std::time::Instant::now();
        let connection = self.connection.write();
        let mut operations = Vec::new();

        // VACUUM優化
        connection.execute("VACUUM", [])?;
        operations.push("VACUUM優化完成".to_string());

        // 重新分析統計信息
        connection.execute("ANALYZE", [])?;
        operations.push("統計信息分析完成".to_string());

        // 完整性檢查
        let integrity_check: String = connection.query_row("PRAGMA integrity_check", [], |row| {
            row.get(0)
        })?;
        operations.push(format!("完整性檢查: {}", integrity_check));

        let duration = start_time.elapsed();

        Ok(MaintenanceResult {
            operations,
            duration,
            is_successful: integrity_check == "ok",
            performed_at: Utc::now(),
        })
    }
}

/// 最佳實踐Model基礎trait
pub trait RusqliteModel: Sized {
    type Id;
    
    fn find_by_id(conn: &Connection, id: Self::Id) -> Result<Option<Self>>;
    fn find_all(conn: &Connection, limit: Option<u32>) -> Result<Vec<Self>>;
    fn save(&self, conn: &Connection) -> Result<Self>;
    fn delete(conn: &Connection, id: Self::Id) -> Result<bool>;
}

/// 最佳實踐Prompt模型（基於Rusqlite）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RusqlitePrompt {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RusqlitePrompt {
    /// 從Row創建Prompt
    pub fn from_row(row: &Row) -> SqliteResult<Self> {
        let id_str: String = row.get("id")?;
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;

        Ok(Self {
            id: Uuid::parse_str(&id_str).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "id".to_string(),
                    rusqlite::types::Type::Text
                )
            })?,
            title: row.get("title")?,
            content: row.get("content")?,
            tags: row.get("tags")?,
            is_favorite: row.get("is_favorite")?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "created_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(0, "updated_at".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc),
        })
    }

    /// 創建新Prompt
    pub fn create(conn: &Connection, title: String, content: String, tags: Option<String>, is_favorite: bool) -> Result<Self> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        conn.execute(
            r#"
            INSERT INTO prompts (id, title, content, tags, is_favorite, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            rusqlite::params![
                id.to_string(),
                title,
                content,
                tags,
                is_favorite,
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        Ok(Self {
            id,
            title,
            content,
            tags,
            is_favorite,
            created_at: now,
            updated_at: now,
        })
    }

    /// 搜索Prompts
    pub fn search(
        conn: &Connection,
        tag_filter: Option<&str>,
        is_favorite_filter: Option<bool>,
        search_term: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Self>> {
        let mut sql = String::from(
            "SELECT id, title, content, tags, is_favorite, created_at, updated_at FROM prompts WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_index = 1;

        if let Some(tag) = tag_filter {
            sql.push_str(&format!(" AND tags LIKE ?{}", param_index));
            params.push(Box::new(format!("%{}%", tag)));
            param_index += 1;
        }

        if let Some(is_fav) = is_favorite_filter {
            sql.push_str(&format!(" AND is_favorite = ?{}", param_index));
            params.push(Box::new(is_fav));
            param_index += 1;
        }

        if let Some(term) = search_term {
            sql.push_str(&format!(" AND (title LIKE ?{} OR content LIKE ?{})", param_index, param_index + 1));
            let search_pattern = format!("%{}%", term);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
            param_index += 2;
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(limit_val) = limit {
            sql.push_str(&format!(" LIMIT ?{}", param_index));
            params.push(Box::new(limit_val));
            param_index += 1;

            if let Some(offset_val) = offset {
                sql.push_str(&format!(" OFFSET ?{}", param_index));
                params.push(Box::new(offset_val));
            }
        }

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let rows = stmt.query_map(&param_refs[..], Self::from_row)?;
        let mut results = Vec::new();
        
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }

    /// 獲取統計信息
    pub fn get_statistics(conn: &Connection) -> Result<PromptStatistics> {
        let stats = conn.query_row(
            r#"
            SELECT 
                COUNT(*) as total_count,
                COUNT(CASE WHEN is_favorite = 1 THEN 1 END) as favorite_count,
                COUNT(CASE WHEN tags IS NOT NULL AND tags != '' THEN 1 END) as tagged_count,
                AVG(LENGTH(content)) as avg_content_length
            FROM prompts
            "#,
            [],
            |row| {
                Ok(PromptStatistics {
                    total_count: row.get::<_, i64>("total_count")? as u64,
                    favorite_count: row.get::<_, i64>("favorite_count")? as u64,
                    tagged_count: row.get::<_, i64>("tagged_count")? as u64,
                    average_content_length: row.get::<_, Option<f64>>("avg_content_length")?.unwrap_or(0.0),
                })
            }
        )?;

        Ok(stats)
    }

    /// 批量更新收藏狀態
    pub fn bulk_update_favorite(conn: &Connection, ids: &[Uuid], is_favorite: bool) -> Result<u64> {
        let tx = conn.unchecked_transaction()?;
        let mut affected_rows = 0;

        {
            let mut stmt = tx.prepare("UPDATE prompts SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3")?;
            
            for id in ids {
                let rows_changed = stmt.execute(rusqlite::params![
                    is_favorite,
                    Utc::now().to_rfc3339(),
                    id.to_string()
                ])?;
                affected_rows += rows_changed;
            }
        } // stmt is dropped here
        
        tx.commit()?;
        Ok(affected_rows as u64)
    }
}

impl RusqliteModel for RusqlitePrompt {
    type Id = Uuid;

    fn find_by_id(conn: &Connection, id: Self::Id) -> Result<Option<Self>> {
        let result = conn.query_row(
            "SELECT id, title, content, tags, is_favorite, created_at, updated_at FROM prompts WHERE id = ?1",
            [id.to_string()],
            Self::from_row,
        );

        match result {
            Ok(prompt) => Ok(Some(prompt)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_all(conn: &Connection, limit: Option<u32>) -> Result<Vec<Self>> {
        let sql = match limit {
            Some(limit_val) => format!(
                "SELECT id, title, content, tags, is_favorite, created_at, updated_at FROM prompts ORDER BY updated_at DESC LIMIT {}",
                limit_val
            ),
            None => "SELECT id, title, content, tags, is_favorite, created_at, updated_at FROM prompts ORDER BY updated_at DESC".to_string(),
        };

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Self::from_row)?;
        let mut results = Vec::new();
        
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }

    fn save(&self, conn: &Connection) -> Result<Self> {
        let now = Utc::now();
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO prompts (id, title, content, tags, is_favorite, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            rusqlite::params![
                self.id.to_string(),
                self.title,
                self.content,
                self.tags,
                self.is_favorite,
                self.created_at.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        let mut updated = self.clone();
        updated.updated_at = now;
        Ok(updated)
    }

    fn delete(conn: &Connection, id: Self::Id) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM prompts WHERE id = ?1",
            [id.to_string()],
        )?;
        
        Ok(rows_affected > 0)
    }
}

/// 健康檢查指標
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealthMetrics {
    pub is_healthy: bool,
    pub response_time_ms: u32,
    pub database_size_bytes: u64,
    pub page_count: u32,
    pub checked_at: DateTime<Utc>,
}

/// 備份指標
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupMetrics {
    pub backup_path: PathBuf,
    pub file_size_bytes: u64,
    pub duration: Duration,
    pub created_at: DateTime<Utc>,
}

/// 維護結果
#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceResult {
    pub operations: Vec<String>,
    pub duration: Duration,
    pub is_successful: bool,
    pub performed_at: DateTime<Utc>,
}

/// Prompt統計資料
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptStatistics {
    pub total_count: u64,
    pub favorite_count: u64,
    pub tagged_count: u64,
    pub average_content_length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_manager() -> RusqliteBestPracticesManager {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = RusqliteBestPracticesConfig {
            database_path: db_path,
            ..Default::default()
        };
        
        let manager = RusqliteBestPracticesManager::new(config).unwrap();
        
        // 創建測試表
        manager.with_connection(|conn| {
            conn.execute(
                r#"
                CREATE TABLE IF NOT EXISTS prompts (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    content TEXT NOT NULL,
                    tags TEXT,
                    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )
                "#,
                [],
            )?;
            Ok(())
        }).unwrap();
        
        manager
    }

    #[test]
    fn test_manager_creation_and_health() {
        let manager = create_test_manager();
        let health = manager.health_check().unwrap();
        
        assert!(health.is_healthy);
        assert!(health.response_time_ms < 1000);
        assert!(health.database_size_bytes > 0);
        
        println!("✅ 管理器創建和健康檢查測試通過");
    }

    #[test]
    fn test_prompt_crud_operations() {
        let manager = create_test_manager();
        
        manager.with_connection(|conn| {
            // 創建測試
            let prompt = RusqlitePrompt::create(
                conn,
                "測試標題".to_string(),
                "測試內容".to_string(),
                Some("測試,標籤".to_string()),
                true,
            )?;
            
            assert_eq!(prompt.title, "測試標題");
            assert!(prompt.is_favorite);
            
            // 查找測試
            let found = RusqlitePrompt::find_by_id(conn, prompt.id)?;
            assert!(found.is_some());
            assert_eq!(found.unwrap().title, "測試標題");
            
            // 查找全部測試
            let all_prompts = RusqlitePrompt::find_all(conn, Some(10))?;
            assert_eq!(all_prompts.len(), 1);
            
            // 刪除測試
            let deleted = RusqlitePrompt::delete(conn, prompt.id)?;
            assert!(deleted);
            
            let not_found = RusqlitePrompt::find_by_id(conn, prompt.id)?;
            assert!(not_found.is_none());
            
            Ok::<(), anyhow::Error>(())
        }).unwrap();
        
        println!("✅ Prompt CRUD操作測試通過");
    }

    #[test]
    fn test_search_functionality() {
        let manager = create_test_manager();
        
        manager.with_connection(|conn| {
            // 創建測試數據
            RusqlitePrompt::create(conn, "React組件".to_string(), "React組件開發".to_string(), Some("react,前端".to_string()), true)?;
            RusqlitePrompt::create(conn, "Rust編程".to_string(), "Rust系統編程".to_string(), Some("rust,後端".to_string()), false)?;
            RusqlitePrompt::create(conn, "無標籤".to_string(), "沒有標籤的內容".to_string(), None, true)?;
            
            // 測試標籤搜索
            let react_results = RusqlitePrompt::search(conn, Some("react"), None, None, None, None)?;
            assert_eq!(react_results.len(), 1);
            assert_eq!(react_results[0].title, "React組件");
            
            // 測試收藏搜索
            let favorite_results = RusqlitePrompt::search(conn, None, Some(true), None, None, None)?;
            assert_eq!(favorite_results.len(), 2);
            
            // 測試內容搜索
            let content_results = RusqlitePrompt::search(conn, None, None, Some("系統"), None, None)?;
            assert_eq!(content_results.len(), 1);
            assert_eq!(content_results[0].title, "Rust編程");
            
            // 測試分頁
            let paginated = RusqlitePrompt::search(conn, None, None, None, Some(2), Some(1))?;
            assert_eq!(paginated.len(), 2);
            
            Ok::<(), anyhow::Error>(())
        }).unwrap();
        
        println!("✅ 搜索功能測試通過");
    }

    #[test]
    fn test_statistics_and_bulk_operations() {
        let manager = create_test_manager();
        
        manager.with_connection(|conn| {
            // 創建測試數據
            let mut ids = Vec::new();
            for i in 0..5 {
                let prompt = RusqlitePrompt::create(
                    conn,
                    format!("測試 {}", i),
                    "a".repeat((i + 1) * 10), // 不同長度的內容
                    if i % 2 == 0 { Some("標籤".to_string()) } else { None },
                    i < 2,
                )?;
                ids.push(prompt.id);
            }
            
            // 測試統計功能
            let stats = RusqlitePrompt::get_statistics(conn)?;
            assert_eq!(stats.total_count, 5);
            assert_eq!(stats.favorite_count, 2);
            assert_eq!(stats.tagged_count, 3);
            assert!(stats.average_content_length > 0.0);
            
            // 測試批量操作
            let updated_count = RusqlitePrompt::bulk_update_favorite(conn, &ids[2..], true)?;
            assert_eq!(updated_count, 3);
            
            // 驗證批量更新結果
            let new_stats = RusqlitePrompt::get_statistics(conn)?;
            assert_eq!(new_stats.favorite_count, 5); // 所有都應該是收藏了
            
            Ok::<(), anyhow::Error>(())
        }).unwrap();
        
        println!("✅ 統計和批量操作測試通過");
    }

    #[test]
    fn test_transaction_handling() {
        let manager = create_test_manager();
        
        // 測試成功的事務
        let result = manager.execute_transaction(|tx| {
            for i in 0..3 {
                tx.execute(
                    r#"
                    INSERT INTO prompts (id, title, content, tags, is_favorite, created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                    "#,
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        format!("事務測試 {}", i),
                        format!("事務內容 {}", i),
                        "事務,測試",
                        false,
                        Utc::now().to_rfc3339(),
                        Utc::now().to_rfc3339()
                    ],
                )?;
            }
            Ok(3)
        }).unwrap();
        
        assert_eq!(result, 3);
        
        // 驗證事務提交成功
        manager.with_connection(|conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM prompts WHERE title LIKE '事務測試%'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 3);
            Ok::<(), anyhow::Error>(())
        }).unwrap();
        
        println!("✅ 事務處理測試通過");
    }

    #[test]
    fn test_backup_and_maintenance() {
        let manager = create_test_manager();
        
        // 創建一些測試數據
        manager.with_connection(|conn| {
            for i in 0..3 {
                RusqlitePrompt::create(
                    conn,
                    format!("備份測試 {}", i),
                    format!("備份內容 {}", i),
                    Some("備份,測試".to_string()),
                    i % 2 == 0,
                )?;
            }
            Ok::<(), anyhow::Error>(())
        }).unwrap();
        
        // 測試備份功能
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");
        let backup_metrics = manager.backup_database(&backup_path).unwrap();
        
        assert!(backup_path.exists());
        assert!(backup_metrics.file_size_bytes > 0);
        assert!(backup_metrics.duration.as_millis() < 5000);
        
        // 測試維護功能
        let maintenance_result = manager.maintenance().unwrap();
        assert!(maintenance_result.is_successful);
        assert!(!maintenance_result.operations.is_empty());
        assert!(maintenance_result.duration.as_millis() < 5000);
        
        println!("✅ 備份和維護功能測試通過");
    }
}