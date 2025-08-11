// 数据库迁移管理器
use std::collections::HashMap;
use crate::core::database::{DatabaseError, DatabaseResult, ConnectionManager};

/// 数据库迁移信息
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Migration {
    pub fn new(
        version: u32, 
        name: String, 
        description: String, 
        up_sql: String, 
        down_sql: Option<String>
    ) -> Self {
        Self {
            version,
            name,
            description,
            up_sql,
            down_sql,
            created_at: chrono::Utc::now(),
        }
    }
}

/// 迁移管理器
pub struct MigrationManager {
    connection_manager: std::sync::Arc<ConnectionManager>,
    migrations: HashMap<u32, Migration>,
}

impl MigrationManager {
    pub fn new(connection_manager: std::sync::Arc<ConnectionManager>) -> Self {
        let mut manager = Self {
            connection_manager,
            migrations: HashMap::new(),
        };
        
        // 注册内置迁移
        manager.register_builtin_migrations();
        manager
    }
    
    /// 注册内置迁移
    fn register_builtin_migrations(&mut self) {
        // 初始架构迁移已经在 ConnectionManager 中处理
        // 这里可以添加后续的架构变更
        
        // 示例：添加索引优化迁移
        self.add_migration(Migration::new(
            2,
            "add_performance_indexes".to_string(),
            "添加性能优化索引".to_string(),
            r#"
            CREATE INDEX IF NOT EXISTS idx_jobs_status_priority ON jobs(status, priority);
            CREATE INDEX IF NOT EXISTS idx_execution_results_status ON execution_results(status);
            CREATE INDEX IF NOT EXISTS idx_prompts_title ON prompts(title);
            "#.to_string(),
            Some(r#"
            DROP INDEX IF EXISTS idx_jobs_status_priority;
            DROP INDEX IF EXISTS idx_execution_results_status;
            DROP INDEX IF EXISTS idx_prompts_title;
            "#.to_string()),
        ));
        
        // 示例：添加全文搜索支持
        self.add_migration(Migration::new(
            3,
            "add_fulltext_search".to_string(),
            "添加全文搜索支持".to_string(),
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS prompts_fts USING fts5(title, content, tags);
            INSERT OR IGNORE INTO prompts_fts SELECT title, content, tags FROM prompts;
            
            -- 创建触发器自动更新 FTS
            CREATE TRIGGER IF NOT EXISTS prompts_fts_insert AFTER INSERT ON prompts BEGIN
                INSERT INTO prompts_fts(title, content, tags) VALUES (NEW.title, NEW.content, NEW.tags);
            END;
            
            CREATE TRIGGER IF NOT EXISTS prompts_fts_update AFTER UPDATE ON prompts BEGIN
                UPDATE prompts_fts SET title = NEW.title, content = NEW.content, tags = NEW.tags 
                WHERE rowid = OLD.id;
            END;
            
            CREATE TRIGGER IF NOT EXISTS prompts_fts_delete AFTER DELETE ON prompts BEGIN
                DELETE FROM prompts_fts WHERE rowid = OLD.id;
            END;
            "#.to_string(),
            Some(r#"
            DROP TRIGGER IF EXISTS prompts_fts_delete;
            DROP TRIGGER IF EXISTS prompts_fts_update;
            DROP TRIGGER IF EXISTS prompts_fts_insert;
            DROP TABLE IF EXISTS prompts_fts;
            "#.to_string()),
        ));
    }
    
    /// 添加迁移
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.insert(migration.version, migration);
    }
    
    /// 获取当前数据库版本
    pub fn get_current_version(&self) -> DatabaseResult<u32> {
        let conn = self.connection_manager.get_connection()?;
        
        let version_str: String = conn.query_row(
            "SELECT value FROM system_metadata WHERE key = 'db_version'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "1".to_string());
        
        version_str.parse::<u32>()
            .map_err(|_| DatabaseError::internal("无效的数据库版本格式"))
    }
    
    /// 获取最新可用版本
    pub fn get_latest_version(&self) -> u32 {
        self.migrations.keys().max().copied().unwrap_or(1)
    }
    
    /// 检查是否需要迁移
    pub fn needs_migration(&self) -> DatabaseResult<bool> {
        let current = self.get_current_version()?;
        let latest = self.get_latest_version();
        Ok(current < latest)
    }
    
    /// 执行迁移到指定版本
    pub fn migrate_to(&self, target_version: u32) -> DatabaseResult<Vec<u32>> {
        let current_version = self.get_current_version()?;
        
        if target_version <= current_version {
            return Ok(Vec::new());
        }
        
        let mut applied_migrations = Vec::new();
        
        // 获取需要执行的迁移，按版本号排序
        let mut pending_migrations: Vec<&Migration> = self.migrations
            .values()
            .filter(|m| m.version > current_version && m.version <= target_version)
            .collect();
        pending_migrations.sort_by_key(|m| m.version);
        
        // 在事务中执行所有迁移
        self.connection_manager.execute_transaction(|tx| {
            for migration in &pending_migrations {
                log::info!("执行迁移 v{}: {}", migration.version, migration.name);
                
                // 执行迁移 SQL
                tx.execute_batch(&migration.up_sql)?;
                
                // 记录迁移历史
                let now = chrono::Utc::now().to_rfc3339();
                tx.execute(
                    r#"
                    INSERT OR REPLACE INTO system_metadata (key, value, created_at, updated_at)
                    VALUES (?, ?, ?, ?)
                    "#,
                    rusqlite::params![
                        format!("migration_{}", migration.version),
                        serde_json::json!({
                            "name": migration.name,
                            "description": migration.description,
                            "applied_at": now
                        }).to_string(),
                        now,
                        now
                    ],
                )?;
                
                applied_migrations.push(migration.version);
            }
            
            // 更新数据库版本
            let now = chrono::Utc::now().to_rfc3339();
            tx.execute(
                "UPDATE system_metadata SET value = ?, updated_at = ? WHERE key = 'db_version'",
                rusqlite::params![target_version.to_string(), now],
            )?;
            
            Ok(())
        })?;
        
        log::info!("迁移完成: v{} -> v{}", current_version, target_version);
        Ok(applied_migrations)
    }
    
    /// 迁移到最新版本
    pub fn migrate_to_latest(&self) -> DatabaseResult<Vec<u32>> {
        let latest_version = self.get_latest_version();
        self.migrate_to(latest_version)
    }
    
    /// 回滚到指定版本
    pub fn rollback_to(&self, target_version: u32) -> DatabaseResult<Vec<u32>> {
        let current_version = self.get_current_version()?;
        
        if target_version >= current_version {
            return Ok(Vec::new());
        }
        
        let mut rolled_back_migrations = Vec::new();
        
        // 获取需要回滚的迁移，按版本号倒序
        let mut rollback_migrations: Vec<&Migration> = self.migrations
            .values()
            .filter(|m| m.version > target_version && m.version <= current_version)
            .collect();
        rollback_migrations.sort_by_key(|m| std::cmp::Reverse(m.version));
        
        // 在事务中执行所有回滚
        self.connection_manager.execute_transaction(|tx| {
            for migration in &rollback_migrations {
                if let Some(down_sql) = &migration.down_sql {
                    log::info!("回滚迁移 v{}: {}", migration.version, migration.name);
                    
                    // 执行回滚 SQL
                    tx.execute_batch(down_sql)?;
                    
                    // 移除迁移历史记录
                    tx.execute(
                        "DELETE FROM system_metadata WHERE key = ?",
                        rusqlite::params![format!("migration_{}", migration.version)],
                    )?;
                    
                    rolled_back_migrations.push(migration.version);
                } else {
                    return Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                        Some(format!("迁移 v{} 不支持回滚", migration.version)),
                    ));
                }
            }
            
            // 更新数据库版本
            let now = chrono::Utc::now().to_rfc3339();
            tx.execute(
                "UPDATE system_metadata SET value = ?, updated_at = ? WHERE key = 'db_version'",
                rusqlite::params![target_version.to_string(), now],
            )?;
            
            Ok(())
        })?;
        
        log::info!("回滚完成: v{} -> v{}", current_version, target_version);
        Ok(rolled_back_migrations)
    }
    
    /// 获取迁移历史
    pub fn get_migration_history(&self) -> DatabaseResult<Vec<MigrationRecord>> {
        let conn = self.connection_manager.get_connection()?;
        
        let mut stmt = conn.prepare(
            "SELECT key, value, created_at FROM system_metadata WHERE key LIKE 'migration_%' ORDER BY key"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value_str: String = row.get(1)?;
            let created_at_str: String = row.get(2)?;
            
            // 解析版本号
            let version_str = key.strip_prefix("migration_").unwrap_or("0");
            let version = version_str.parse::<u32>().unwrap_or(0);
            
            // 解析迁移信息
            let migration_info: serde_json::Value = serde_json::from_str(&value_str)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            let applied_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            
            Ok(MigrationRecord {
                version,
                name: migration_info["name"].as_str().unwrap_or("unknown").to_string(),
                description: migration_info["description"].as_str().unwrap_or("").to_string(),
                applied_at,
            })
        })?;
        
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        
        Ok(records)
    }
    
    /// 验证数据库完整性
    pub fn validate_database(&self) -> DatabaseResult<ValidationResult> {
        let conn = self.connection_manager.get_connection()?;
        let mut issues = Vec::new();
        
        // 检查必需的表是否存在
        let required_tables = ["prompts", "jobs", "execution_results", "usage_stats", "system_metadata"];
        for table in &required_tables {
            let count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                rusqlite::params![table],
                |row| row.get(0),
            )?;
            
            if count == 0 {
                issues.push(format!("缺少必需的表: {}", table));
            }
        }
        
        // 检查外键约束
        let foreign_key_enabled: bool = conn.query_row(
            "PRAGMA foreign_keys",
            [],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )?;
        
        if !foreign_key_enabled {
            issues.push("外键约束未启用".to_string());
        }
        
        // 检查数据完整性
        let integrity_check: String = conn.query_row(
            "PRAGMA integrity_check",
            [],
            |row| row.get(0),
        )?;
        
        if integrity_check != "ok" {
            issues.push(format!("数据完整性检查失败: {}", integrity_check));
        }
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            checked_at: chrono::Utc::now(),
        })
    }
}

/// 迁移记录
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub applied_at: chrono::DateTime<chrono::Utc>,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_valid {
            write!(f, "数据库验证通过 ({})", self.checked_at.format("%Y-%m-%d %H:%M:%S"))
        } else {
            write!(f, "数据库验证失败:\n{}", self.issues.join("\n"))
        }
    }
}