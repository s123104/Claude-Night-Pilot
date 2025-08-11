// 数据库核心模块
// 统一的数据库管理架构，基于 Repository 模式

pub mod manager;
pub mod repository;
pub mod connection;
pub mod types;
pub mod errors;
pub mod migrations;

#[cfg(test)]
pub mod tests;

// 公开核心类型和接口
pub use manager::{
    DatabaseManager, initialize_global_database_manager, get_global_database_manager,
    DatabaseHealthStatus, DatabaseStatistics, BackupResult, MaintenanceResult
};
pub use repository::{Repository, PromptRepository, JobRepository, UsageRepository};
pub use connection::ConnectionManager;
pub use migrations::{MigrationManager, ValidationResult};
pub use types::*;
pub use errors::{DatabaseError, DatabaseResult};

// 版本信息
pub const DATABASE_VERSION: u32 = 1;
pub const SCHEMA_VERSION: &str = "1.0.0";

// 默认配置
pub const DEFAULT_DATABASE_PATH: &str = "claude-pilot.db";
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30; // seconds
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;