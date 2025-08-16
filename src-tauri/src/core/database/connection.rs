// 数据库连接管理器
use crate::core::database::{DatabaseConfig, DatabaseError, DatabaseResult};
use rusqlite::{Connection, OpenFlags, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 连接管理器 - 负责数据库连接的创建、配置和维护
pub struct ConnectionManager {
    config: DatabaseConfig,
    connection: Arc<Mutex<Connection>>,
    last_health_check: Arc<Mutex<Instant>>,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let connection = Self::create_connection(&config)?;
        Self::configure_connection(&connection, &config)?;

        let manager = Self {
            config,
            connection: Arc::new(Mutex::new(connection)),
            last_health_check: Arc::new(Mutex::new(Instant::now())),
        };

        // 初始化数据库结构
        manager.initialize_database()?;

        Ok(manager)
    }

    /// 创建数据库连接
    fn create_connection(config: &DatabaseConfig) -> DatabaseResult<Connection> {
        // 确保数据库目录存在
        if let Some(parent) = Path::new(&config.path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::internal(format!("无法创建数据库目录: {}", e)))?;
        }

        // 设置连接标志
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX; // 单线程模式，由 Mutex 控制并发

        let connection =
            Connection::open_with_flags(&config.path, flags).map_err(DatabaseError::Connection)?;

        Ok(connection)
    }

    /// 配置数据库连接
    fn configure_connection(conn: &Connection, config: &DatabaseConfig) -> DatabaseResult<()> {
        // 启用外键约束（使用 pragma_update 以避免返回结果集導致的錯誤）
        if config.enable_foreign_keys {
            conn.pragma_update(None, "foreign_keys", true)
                .map_err(DatabaseError::Connection)?;
        }

        // 设置同步模式（"OFF"/"NORMAL"/"FULL"）
        conn.pragma_update(None, "synchronous", format!("{}", config.synchronous_mode))
            .map_err(DatabaseError::Connection)?;

        // 设置日志模式（journal_mode 會返回結果，使用 pragma_update）
        if config.enable_wal_mode {
            conn.pragma_update(None, "journal_mode", format!("{}", config.journal_mode))
                .map_err(DatabaseError::Connection)?;
        }

        // 设置缓存大小
        conn.pragma_update(None, "cache_size", config.cache_size)
            .map_err(DatabaseError::Connection)?;

        // 设置临时存储模式（DEFAULT/FILE/MEMORY）
        conn.pragma_update(None, "temp_store", format!("{}", config.temp_store))
            .map_err(DatabaseError::Connection)?;

        // 设置忙等超时
        conn.busy_timeout(Duration::from_millis(config.busy_timeout_ms as u64))
            .map_err(DatabaseError::Connection)?;

        Ok(())
    }

    /// 初始化数据库结构
    fn initialize_database(&self) -> DatabaseResult<()> {
        let conn = self
            .connection
            .lock()
            .map_err(|_| DatabaseError::internal("无法获取数据库连接锁"))?;

        // 创建核心表结构
        self.create_core_tables(&conn)?;

        // 设置数据库版本
        self.set_database_version(&conn, crate::core::database::DATABASE_VERSION)?;

        Ok(())
    }

    /// 创建核心表结构
    fn create_core_tables(&self, conn: &Connection) -> DatabaseResult<()> {
        // 创建 prompts 表
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Connection)?;

        // 创建 jobs 表 (统一的调度表)
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt_id INTEGER NOT NULL,
                name TEXT,
                schedule_type TEXT NOT NULL DEFAULT 'once',
                schedule_config TEXT NOT NULL DEFAULT '{}',
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 2,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                last_run_at TEXT,
                next_run_at TEXT,
                FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Connection)?;

        // 创建执行结果表
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS execution_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'success',
                content TEXT NOT NULL,
                error_message TEXT,
                input_tokens INTEGER,
                output_tokens INTEGER,
                total_tokens INTEGER,
                cost_usd REAL,
                execution_time_ms INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Connection)?;

        // 创建使用统计表
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS usage_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                period_start TEXT NOT NULL,
                period_end TEXT NOT NULL,
                total_executions INTEGER NOT NULL DEFAULT 0,
                successful_executions INTEGER NOT NULL DEFAULT 0,
                failed_executions INTEGER NOT NULL DEFAULT 0,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                total_cost_usd REAL,
                average_execution_time_ms REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Connection)?;

        // 创建系统元数据表
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS system_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Connection)?;

        // 创建索引
        self.create_indexes(conn)?;

        Ok(())
    }

    /// 创建数据库索引
    fn create_indexes(&self, conn: &Connection) -> DatabaseResult<()> {
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status)",
            "CREATE INDEX IF NOT EXISTS idx_jobs_next_run_at ON jobs(next_run_at)",
            "CREATE INDEX IF NOT EXISTS idx_jobs_prompt_id ON jobs(prompt_id)",
            "CREATE INDEX IF NOT EXISTS idx_execution_results_job_id ON execution_results(job_id)",
            "CREATE INDEX IF NOT EXISTS idx_execution_results_created_at ON execution_results(created_at)",
            "CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at)",
        ];

        for index_sql in &indexes {
            conn.execute(index_sql, [])
                .map_err(DatabaseError::Connection)?;
        }

        Ok(())
    }

    /// 设置数据库版本
    fn set_database_version(&self, conn: &Connection, version: u32) -> DatabaseResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO system_metadata (key, value, created_at, updated_at)
            VALUES ('db_version', ?, ?, ?)
            "#,
            [&version.to_string(), &now, &now],
        )
        .map_err(DatabaseError::Connection)?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO system_metadata (key, value, created_at, updated_at)
            VALUES ('schema_version', ?, ?, ?)
            "#,
            [crate::core::database::SCHEMA_VERSION, &now, &now],
        )
        .map_err(DatabaseError::Connection)?;

        Ok(())
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> DatabaseResult<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| DatabaseError::internal("无法获取数据库连接锁"))
    }

    /// 执行健康检查
    pub fn health_check(&self) -> DatabaseResult<HealthCheckResult> {
        let mut last_check = self
            .last_health_check
            .lock()
            .map_err(|_| DatabaseError::internal("无法获取健康检查锁"))?;

        let now = Instant::now();
        let check_result = {
            let conn = self.get_connection()?;

            // 执行简单查询测试连接
            let version: String = conn
                .query_row(
                    "SELECT value FROM system_metadata WHERE key = 'db_version'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "unknown".to_string());

            // 检查表完整性
            let table_count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('prompts', 'jobs', 'execution_results', 'usage_stats')",
                [],
                |row| row.get(0)
            ).map_err(DatabaseError::Connection)?;

            HealthCheckResult {
                is_healthy: table_count >= 4,
                database_version: version,
                table_count: table_count as u32,
                last_check_duration: now - *last_check,
                connection_path: self.config.path.clone(),
            }
        };

        *last_check = now;
        Ok(check_result)
    }

    /// 执行事务
    pub fn execute_transaction<F, R>(&self, f: F) -> DatabaseResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> SqliteResult<R>,
    {
        let conn = self.get_connection()?;
        let transaction = conn
            .unchecked_transaction()
            .map_err(DatabaseError::Connection)?;

        match f(&transaction) {
            Ok(result) => {
                transaction.commit().map_err(DatabaseError::Connection)?;
                Ok(result)
            }
            Err(e) => {
                if let Err(rollback_err) = transaction.rollback() {
                    log::error!("事务回滚失败: {}", rollback_err);
                }
                Err(DatabaseError::Connection(e))
            }
        }
    }

    /// 备份数据库
    pub fn backup_to_file<P: AsRef<Path>>(&self, backup_path: P) -> DatabaseResult<()> {
        let conn = self.get_connection()?;

        // 使用 SQLite VACUUM INTO 命令进行备份
        let backup_path_str = backup_path.as_ref().to_string_lossy();
        conn.execute(&format!("VACUUM INTO '{}'", backup_path_str), [])
            .map_err(DatabaseError::Connection)?;

        Ok(())
    }

    /// 获取数据库统计信息
    pub fn get_database_stats(&self) -> DatabaseResult<DatabaseStats> {
        let conn = self.get_connection()?;

        let prompt_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
            .map_err(DatabaseError::Connection)?;

        let job_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
            .map_err(DatabaseError::Connection)?;

        let result_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM execution_results", [], |row| {
                row.get(0)
            })
            .map_err(DatabaseError::Connection)?;

        // 获取数据库文件大小；若檔案大小為 0 或無法讀取，回退以 PRAGMA 計算
        let mut db_size = std::fs::metadata(&self.config.path)
            .map(|m| m.len())
            .unwrap_or(0);
        if db_size == 0 {
            let page_count: i64 = conn
                .query_row("PRAGMA page_count", [], |row| row.get(0))
                .unwrap_or(0);
            let page_size: i64 = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .unwrap_or(0);
            let calc = page_count.saturating_mul(page_size);
            if calc > 0 {
                db_size = calc as u64;
            }
        }

        Ok(DatabaseStats {
            prompt_count: prompt_count as u64,
            job_count: job_count as u64,
            result_count: result_count as u64,
            database_size_bytes: db_size,
            database_path: self.config.path.clone(),
        })
    }
}

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub is_healthy: bool,
    pub database_version: String,
    pub table_count: u32,
    pub last_check_duration: Duration,
    pub connection_path: String,
}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub prompt_count: u64,
    pub job_count: u64,
    pub result_count: u64,
    pub database_size_bytes: u64,
    pub database_path: String,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "数据库统计:\n  Prompts: {}\n  Jobs: {}\n  Results: {}\n  大小: {} KB\n  路径: {}",
            self.prompt_count,
            self.job_count,
            self.result_count,
            self.database_size_bytes / 1024,
            self.database_path
        )
    }
}

// 实现 Send 和 Sync
unsafe impl Send for ConnectionManager {}
unsafe impl Sync for ConnectionManager {}
