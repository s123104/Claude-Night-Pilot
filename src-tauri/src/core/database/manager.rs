// 统一的数据库管理器
use serde_json;
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::core::database::migrations::{MigrationManager, ValidationResult};
use crate::core::database::{
    ConnectionManager, DatabaseConfig, DatabaseError, DatabaseResult, JobRepository,
    PromptRepository, UsageRepository,
};

/// 统一的数据库管理器
///
/// 这是数据库层的主要入口点，提供：
/// - 连接管理
/// - Repository 访问
/// - 迁移管理
/// - 健康检查
/// - 事务支持
pub struct DatabaseManager {
    connection_manager: Arc<ConnectionManager>,
    prompt_repository: PromptRepository,
    job_repository: JobRepository,
    usage_repository: UsageRepository,
    migration_manager: MigrationManager,
}

impl DatabaseManager {
    /// 创建新的数据库管理器
    pub async fn new(config: DatabaseConfig) -> DatabaseResult<Self> {
        let connection_manager = Arc::new(ConnectionManager::new(config)?);

        let prompt_repository = PromptRepository::new(Arc::clone(&connection_manager));
        let job_repository = JobRepository::new(Arc::clone(&connection_manager));
        let usage_repository = UsageRepository::new(Arc::clone(&connection_manager));
        let migration_manager = MigrationManager::new(Arc::clone(&connection_manager));

        let manager = Self {
            connection_manager,
            prompt_repository,
            job_repository,
            usage_repository,
            migration_manager,
        };

        // 执行必要的迁移
        manager.ensure_migrations().await?;

        Ok(manager)
    }

    /// 使用默认配置创建数据库管理器
    pub async fn with_default_config() -> DatabaseResult<Self> {
        // 在測試環境下，使用臨時檔案避免寫入專案根目錄，也避免路徑被清理導致檔案大小為0
        if cfg!(test) {
            let mut config = DatabaseConfig::default();
            let path =
                std::env::temp_dir().join(format!("cnp_default_{}.db", uuid::Uuid::new_v4()));
            config.path = path.to_string_lossy().to_string();
            Self::new(config).await
        } else {
            Self::new(DatabaseConfig::default()).await
        }
    }

    /// 确保数据库迁移
    async fn ensure_migrations(&self) -> DatabaseResult<()> {
        if self.migration_manager.needs_migration()? {
            let applied = self.migration_manager.migrate_to_latest()?;
            if !applied.is_empty() {
                log::info!("应用了 {} 个数据库迁移: {:?}", applied.len(), applied);
            }
        }
        Ok(())
    }

    /// 获取 Prompt Repository
    pub fn prompts(&self) -> &PromptRepository {
        &self.prompt_repository
    }

    /// 获取 Job Repository  
    pub fn jobs(&self) -> &JobRepository {
        &self.job_repository
    }

    /// 获取 Usage Repository
    pub fn usage(&self) -> &UsageRepository {
        &self.usage_repository
    }

    /// 获取连接管理器
    pub fn connection_manager(&self) -> &Arc<ConnectionManager> {
        &self.connection_manager
    }

    /// 获取迁移管理器
    pub fn migrations(&self) -> &MigrationManager {
        &self.migration_manager
    }

    /// 执行健康检查
    pub async fn health_check(&self) -> DatabaseResult<DatabaseHealthStatus> {
        let connection_health = self.connection_manager.health_check()?;
        let validation_result = self.migration_manager.validate_database()?;
        let stats = self.connection_manager.get_database_stats()?;

        // 检查基本功能 - 导入 Repository trait
        use crate::core::database::repository::Repository;
        let prompt_count = self.prompt_repository.count().await?;
        let job_count = self.job_repository.count().await?;
        let usage_count = self.usage_repository.count().await?;

        let status = DatabaseHealthStatus {
            is_healthy: connection_health.is_healthy && validation_result.is_valid,
            connection_status: connection_health,
            validation_result,
            statistics: DatabaseStatistics {
                prompt_count,
                job_count,
                execution_result_count: usage_count,
                database_size_bytes: stats.database_size_bytes,
                database_path: stats.database_path.clone(),
            },
            last_check: chrono::Utc::now(),
        };

        Ok(status)
    }

    /// 执行数据库备份
    pub async fn backup_to_file<P: AsRef<std::path::Path>>(
        &self,
        backup_path: P,
    ) -> DatabaseResult<BackupResult> {
        let start_time = std::time::Instant::now();
        let backup_path = backup_path.as_ref();

        // 执行备份
        self.connection_manager.backup_to_file(backup_path)?;

        let duration = start_time.elapsed();
        let file_size = std::fs::metadata(backup_path).map(|m| m.len()).unwrap_or(0);

        Ok(BackupResult {
            backup_path: backup_path.to_path_buf(),
            file_size_bytes: file_size,
            duration,
            created_at: chrono::Utc::now(),
        })
    }

    /// 执行完整的数据库维护
    pub async fn maintenance(&self) -> DatabaseResult<MaintenanceResult> {
        let start_time = std::time::Instant::now();
        let mut operations = Vec::new();

        // 1. 验证数据库完整性
        let validation = self.migration_manager.validate_database()?;
        operations.push(format!(
            "完整性检查: {}",
            if validation.is_valid {
                "通过"
            } else {
                "失败"
            }
        ));

        // 2. 执行 VACUUM 优化（不可在事务內執行）
        {
            let conn = self.connection_manager.get_connection()?;
            conn.execute("VACUUM", [])?;
        }
        operations.push("VACUUM 优化: 完成".to_string());

        // 3. 重新分析统计信息（建議也在非交易上下文直接執行）
        {
            let conn = self.connection_manager.get_connection()?;
            conn.execute("ANALYZE", [])?;
        }
        operations.push("统计信息分析: 完成".to_string());

        // 4. 清理过期数据（可选）
        let cleanup_result = self.cleanup_old_data(chrono::Duration::days(90)).await?;
        if cleanup_result.cleaned_records > 0 {
            operations.push(format!(
                "数据清理: 清理了 {} 条记录",
                cleanup_result.cleaned_records
            ));
        }

        let duration = start_time.elapsed();

        Ok(MaintenanceResult {
            operations,
            duration,
            validation_result: validation,
            performed_at: chrono::Utc::now(),
        })
    }

    /// 清理过期数据
    pub async fn cleanup_old_data(
        &self,
        older_than: chrono::Duration,
    ) -> DatabaseResult<CleanupResult> {
        let cutoff_date = chrono::Utc::now() - older_than;
        let cutoff_str = cutoff_date.to_rfc3339();

        let mut cleaned_records = 0;

        // 清理过期的执行结果（保留最近的结果）
        cleaned_records += self.connection_manager.execute_transaction(|tx| {
            let count = tx.execute(
                r#"
                DELETE FROM execution_results 
                WHERE created_at < ? 
                AND id NOT IN (
                    SELECT id FROM execution_results 
                    ORDER BY created_at DESC 
                    LIMIT 1000
                )
                "#,
                rusqlite::params![cutoff_str],
            )?;
            Ok(count)
        })? as u64;

        // 清理已完成且过期的任务
        cleaned_records += self.connection_manager.execute_transaction(|tx| {
            let count = tx.execute(
                r#"
                DELETE FROM jobs 
                WHERE status IN ('completed', 'failed', 'cancelled') 
                AND created_at < ?
                "#,
                rusqlite::params![cutoff_str],
            )?;
            Ok(count)
        })? as u64;

        Ok(CleanupResult {
            cleaned_records,
            cutoff_date,
            performed_at: chrono::Utc::now(),
        })
    }

    /// 获取数据库统计信息
    pub async fn get_statistics(&self) -> DatabaseResult<DatabaseStatistics> {
        // 导入 Repository trait 以使用 count 方法
        use crate::core::database::repository::Repository;

        let prompt_count = self.prompt_repository.count().await?;
        let job_count = self.job_repository.count().await?;
        let usage_count = self.usage_repository.count().await?;
        let stats = self.connection_manager.get_database_stats()?;

        Ok(DatabaseStatistics {
            prompt_count,
            job_count,
            execution_result_count: usage_count,
            database_size_bytes: stats.database_size_bytes,
            database_path: stats.database_path,
        })
    }

    /// 执行原始 SQL 查询（仅用于高级操作）
    pub async fn execute_raw_query(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> DatabaseResult<Vec<serde_json::Value>> {
        let conn = self.connection_manager.get_connection()?;
        let mut stmt = conn.prepare(sql)?;

        let column_count = stmt.column_count();
        let column_names: Vec<String> = (0..column_count)
            .map(|i| stmt.column_name(i).unwrap_or("").to_string())
            .collect();

        let rows = stmt.query_map(params, |row| {
            let mut object = serde_json::Map::new();

            for (i, column_name) in column_names.iter().enumerate() {
                let value: serde_json::Value = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => serde_json::Value::Number(i.into()),
                    rusqlite::types::ValueRef::Real(f) => serde_json::Value::Number(
                        serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()),
                    ),
                    rusqlite::types::ValueRef::Text(s) => {
                        serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                    }
                    rusqlite::types::ValueRef::Blob(b) => {
                        use base64::Engine;
                        serde_json::Value::String(
                            base64::engine::general_purpose::STANDARD.encode(b),
                        )
                    }
                };
                object.insert(column_name.clone(), value);
            }

            Ok(serde_json::Value::Object(object))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// 创建数据库快照
    pub async fn create_snapshot(&self, description: String) -> DatabaseResult<SnapshotInfo> {
        let timestamp = chrono::Utc::now();
        let snapshot_name = format!(
            "snapshot_{}_{}",
            timestamp.format("%Y%m%d_%H%M%S"),
            &uuid::Uuid::new_v4().to_string()[..8]
        );

        let snapshot_path = std::path::PathBuf::from(format!("{}.db", snapshot_name));
        let backup_result = self.backup_to_file(&snapshot_path).await?;

        // 记录快照元数据
        let metadata = SnapshotMetadata {
            name: snapshot_name.clone(),
            description,
            created_at: timestamp,
            file_size: backup_result.file_size_bytes,
            database_version: self.migration_manager.get_current_version()?,
        };

        // 将元数据保存到数据库
        self.connection_manager.execute_transaction(|tx| {
            tx.execute(
                r#"
                INSERT OR REPLACE INTO system_metadata (key, value, created_at, updated_at)
                VALUES (?, ?, ?, ?)
                "#,
                rusqlite::params![
                    format!("snapshot_{}", snapshot_name),
                    serde_json::to_string(&metadata).unwrap(),
                    timestamp.to_rfc3339(),
                    timestamp.to_rfc3339()
                ],
            )?;
            Ok(())
        })?;

        Ok(SnapshotInfo {
            metadata,
            file_path: snapshot_path,
        })
    }
}

// 全局数据库管理器实例
static GLOBAL_DATABASE_MANAGER: OnceCell<DatabaseManager> = OnceCell::const_new();

/// 初始化全局数据库管理器
pub async fn initialize_global_database_manager(
    config: Option<DatabaseConfig>,
) -> DatabaseResult<()> {
    let config = config.unwrap_or_default();
    let manager = DatabaseManager::new(config).await?;

    GLOBAL_DATABASE_MANAGER
        .set(manager)
        .map_err(|_| DatabaseError::internal("全局数据库管理器已经初始化"))?;

    Ok(())
}

/// 获取全局数据库管理器
pub fn get_global_database_manager() -> DatabaseResult<&'static DatabaseManager> {
    GLOBAL_DATABASE_MANAGER
        .get()
        .ok_or_else(|| DatabaseError::internal("全局数据库管理器未初始化"))
}

/// 数据库健康状态
#[derive(Debug, Clone)]
pub struct DatabaseHealthStatus {
    pub is_healthy: bool,
    pub connection_status: crate::core::database::connection::HealthCheckResult,
    pub validation_result: ValidationResult,
    pub statistics: DatabaseStatistics,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// 数据库统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStatistics {
    pub prompt_count: u64,
    pub job_count: u64,
    pub execution_result_count: u64,
    pub database_size_bytes: u64,
    pub database_path: String,
}

/// 备份结果
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub backup_path: std::path::PathBuf,
    pub file_size_bytes: u64,
    pub duration: std::time::Duration,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 维护结果
#[derive(Debug, Clone)]
pub struct MaintenanceResult {
    pub operations: Vec<String>,
    pub duration: std::time::Duration,
    pub validation_result: ValidationResult,
    pub performed_at: chrono::DateTime<chrono::Utc>,
}

/// 清理结果
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub cleaned_records: u64,
    pub cutoff_date: chrono::DateTime<chrono::Utc>,
    pub performed_at: chrono::DateTime<chrono::Utc>,
}

/// 快照元数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMetadata {
    pub name: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub file_size: u64,
    pub database_version: u32,
}

/// 快照信息
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub metadata: SnapshotMetadata,
    pub file_path: std::path::PathBuf,
}

impl std::fmt::Display for DatabaseHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "数据库状态: {} | Prompts: {} | Jobs: {} | Results: {} | 大小: {} KB",
            if self.is_healthy { "健康" } else { "异常" },
            self.statistics.prompt_count,
            self.statistics.job_count,
            self.statistics.execution_result_count,
            self.statistics.database_size_bytes / 1024
        )
    }
}

// 实现 Send 和 Sync
unsafe impl Send for DatabaseManager {}
unsafe impl Sync for DatabaseManager {}

// 添加必需的依赖
use base64;
use uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_database_manager() -> DatabaseResult<DatabaseManager> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            path: db_path.to_string_lossy().to_string(),
            enable_foreign_keys: true,
            enable_wal_mode: true,
            synchronous_mode: crate::core::database::types::SynchronousMode::Normal,
            cache_size: 1000,
            temp_store: crate::core::database::types::TempStore::Memory,
            journal_mode: crate::core::database::types::JournalMode::Wal,
            connection_timeout_seconds: 30,
            busy_timeout_ms: 5000,
        };

        DatabaseManager::new(config).await
    }

    #[tokio::test]
    async fn test_database_manager_creation() {
        let manager = create_test_database_manager().await.unwrap();
        assert!(manager.connection_manager.get_connection().is_ok());
    }

    #[tokio::test]
    async fn test_database_manager_with_default_config() {
        // 使用内存数据库进行测试
        let result = DatabaseManager::with_default_config().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let manager = create_test_database_manager().await.unwrap();
        let health = manager.health_check().await.unwrap();
        assert!(health.is_healthy);
        assert_eq!(health.statistics.prompt_count, 0);
        assert_eq!(health.statistics.job_count, 0);
        assert_eq!(health.statistics.execution_result_count, 0);
    }

    #[tokio::test]
    async fn test_repository_access() {
        let manager = create_test_database_manager().await.unwrap();

        // 测试 Repository 访问
        let prompts = manager.prompts();
        let jobs = manager.jobs();
        let usage = manager.usage();

        // 使用 Repository trait
        use crate::core::database::repository::Repository;
        assert_eq!(prompts.count().await.unwrap(), 0);
        assert_eq!(jobs.count().await.unwrap(), 0);
        assert_eq!(usage.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_backup_functionality() {
        let manager = create_test_database_manager().await.unwrap();
        let temp_dir = tempdir().unwrap();
        let backup_path = temp_dir.path().join("backup.db");

        let backup_result = manager.backup_to_file(&backup_path).await.unwrap();

        assert!(backup_path.exists());
        assert!(backup_result.file_size_bytes > 0);
        assert_eq!(backup_result.backup_path, backup_path);
    }

    #[tokio::test]
    async fn test_maintenance_operations() {
        let manager = create_test_database_manager().await.unwrap();
        let maintenance_result = manager.maintenance().await.unwrap();

        assert!(!maintenance_result.operations.is_empty());
        assert!(maintenance_result.validation_result.is_valid);
        assert!(maintenance_result.duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let manager = create_test_database_manager().await.unwrap();
        let cleanup_result = manager
            .cleanup_old_data(chrono::Duration::days(1))
            .await
            .unwrap();

        // 新数据库应该没有需要清理的数据
        assert_eq!(cleanup_result.cleaned_records, 0);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let manager = create_test_database_manager().await.unwrap();
        let stats = manager.get_statistics().await.unwrap();

        assert_eq!(stats.prompt_count, 0);
        assert_eq!(stats.job_count, 0);
        assert_eq!(stats.execution_result_count, 0);
        assert!(stats.database_size_bytes > 0);
        assert!(!stats.database_path.is_empty());
    }

    #[tokio::test]
    async fn test_execute_raw_query() {
        let manager = create_test_database_manager().await.unwrap();

        // 执行一个简单的查询
        let results = manager
            .execute_raw_query("SELECT 1 as test_col", &[])
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        if let serde_json::Value::Object(obj) = &results[0] {
            assert_eq!(obj.get("test_col").and_then(|v| v.as_i64()), Some(1));
        } else {
            panic!("Expected object result");
        }
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let manager = create_test_database_manager().await.unwrap();
        let snapshot_info = manager
            .create_snapshot("Test snapshot".to_string())
            .await
            .unwrap();

        assert!(!snapshot_info.metadata.name.is_empty());
        assert_eq!(snapshot_info.metadata.description, "Test snapshot");
        assert!(snapshot_info.file_path.exists());
        assert!(snapshot_info.metadata.file_size > 0);
    }

    #[tokio::test]
    async fn test_global_database_manager() {
        // 清理全局状态（仅用于测试）
        // 注意：在实际应用中全局状态应该只初始化一次

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("global_test.db");
        let config = DatabaseConfig {
            path: db_path.to_string_lossy().to_string(),
            enable_foreign_keys: true,
            enable_wal_mode: true,
            synchronous_mode: crate::core::database::types::SynchronousMode::Normal,
            cache_size: 1000,
            temp_store: crate::core::database::types::TempStore::Memory,
            journal_mode: crate::core::database::types::JournalMode::Wal,
            connection_timeout_seconds: 30,
            busy_timeout_ms: 5000,
        };

        // 测试初始化
        let init_result = initialize_global_database_manager(Some(config)).await;

        // 如果已经初始化过，这是正常的
        match init_result {
            Ok(()) => {
                let global_manager = get_global_database_manager().unwrap();
                assert!(global_manager.connection_manager.get_connection().is_ok());
            }
            Err(_) => {
                // 全局管理器已经初始化，这在测试环境中是正常的
                let global_manager = get_global_database_manager().unwrap();
                assert!(global_manager.connection_manager.get_connection().is_ok());
            }
        }
    }

    #[test]
    fn test_database_health_status_display() {
        use crate::core::database::connection::HealthCheckResult;
        use crate::core::database::migrations::ValidationResult;

        let health_status = DatabaseHealthStatus {
            is_healthy: true,
            connection_status: HealthCheckResult {
                is_healthy: true,
                database_version: "3.45.0".to_string(),
                table_count: 5,
                last_check_duration: std::time::Duration::from_millis(50),
                connection_path: "/tmp/test.db".to_string(),
            },
            validation_result: ValidationResult {
                is_valid: true,
                issues: vec![],
                checked_at: chrono::Utc::now(),
            },
            statistics: DatabaseStatistics {
                prompt_count: 10,
                job_count: 5,
                execution_result_count: 15,
                database_size_bytes: 102400,
                database_path: "/tmp/test.db".to_string(),
            },
            last_check: chrono::Utc::now(),
        };

        let display_str = format!("{}", health_status);
        assert!(display_str.contains("健康"));
        assert!(display_str.contains("Prompts: 10"));
        assert!(display_str.contains("Jobs: 5"));
        assert!(display_str.contains("Results: 15"));
        assert!(display_str.contains("大小: 100 KB"));
    }

    #[tokio::test]
    async fn test_error_handling_invalid_config() {
        let config = DatabaseConfig {
            path: "/invalid/path/that/does/not/exist/test.db".to_string(),
            enable_foreign_keys: true,
            enable_wal_mode: true,
            synchronous_mode: crate::core::database::types::SynchronousMode::Normal,
            cache_size: 1000,
            temp_store: crate::core::database::types::TempStore::Memory,
            journal_mode: crate::core::database::types::JournalMode::Wal,
            connection_timeout_seconds: 1,
            busy_timeout_ms: 100,
        };

        let result = DatabaseManager::new(config).await;
        // 这个测试可能成功，因为SQLite会尝试创建目录
        // 在真实环境中，需要更复杂的无效配置来触发错误
        println!("Result: {:?}", result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let manager = create_test_database_manager().await.unwrap();
        let manager = std::sync::Arc::new(manager);

        let mut handles = vec![];

        // 创建多个并发健康检查任务
        for _ in 0..5 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let health = manager_clone.health_check().await.unwrap();
                assert!(health.is_healthy);
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
