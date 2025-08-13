// 数据库核心模块
// 统一的数据库管理架构，基于 Repository 模式

pub mod manager;
pub mod repository;
pub mod connection;
pub mod types;
pub mod errors;
pub mod migrations;

// 最佳实践模块（Rusqlite 实现為主，SQLx 版本暫不編譯以避免依賴衝突）
// pub mod best_practices_manager; // 需 sqlx 依賴，暫時停用
pub mod best_practices_rusqlite; // Rusqlite实现，避免 sqlx 冲突
// pub mod models { pub mod prompt_best_practices; }

#[cfg(test)]
pub mod tests {
    pub mod integration_tests;
    pub mod unit_tests;
    pub mod performance_tests;
}

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

// 最佳实践模块导出 - SQLx版本（停用）
// pub use best_practices_manager::{
//     BestPracticesDbManager, BestPracticesDbConfig, DatabaseHealthMetrics, BackupMetrics,
//     BestPracticesModel, QueryBuilder
// };

// Rusqlite版本导出（主要使用）
pub use best_practices_rusqlite::{
    RusqliteBestPracticesManager, RusqliteBestPracticesConfig, RusqliteModel,
    RusqlitePrompt, PromptStatistics as RusqlitePromptStatistics, 
    DatabaseHealthMetrics as RusqliteDatabaseHealthMetrics,
    BackupMetrics as RusqliteBackupMetrics, MaintenanceResult as RusqliteMaintenanceResult
};

// pub use models::prompt_best_practices::{
//     PromptBestPractices, CreatePromptInput, UpdatePromptInput, PromptFilter, PromptStatistics
// };

// 版本信息
pub const DATABASE_VERSION: u32 = 1;
pub const SCHEMA_VERSION: &str = "1.0.0";

// 默认配置
pub const DEFAULT_DATABASE_PATH: &str = "claude-pilot.db";
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30; // seconds
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;