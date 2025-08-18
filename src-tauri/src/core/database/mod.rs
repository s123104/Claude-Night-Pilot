// 数据库核心模块
// 统一的数据库管理架构，基于 Repository 模式

pub mod connection;
pub mod errors;
pub mod manager;
pub mod migrations;
pub mod repository;
pub mod types;

// 最佳实践模块（Rusqlite 实现為主，SQLx 版本暫不編譯以避免依賴衝突）
// pub mod best_practices_manager; // 需 sqlx 依賴，暫時停用
pub mod best_practices_rusqlite; // Rusqlite实现，避免 sqlx 冲突
                                 // pub mod models { pub mod prompt_best_practices; }

#[cfg(test)]
pub mod tests {
    pub mod integration_tests;
    pub mod performance_tests;
    pub mod unit_tests;
}

// 公开核心类型和接口
pub use connection::ConnectionManager;
pub use errors::{DatabaseError, DatabaseResult};
pub use manager::{
    get_global_database_manager, initialize_global_database_manager, BackupResult,
    DatabaseHealthStatus, DatabaseManager, DatabaseStatistics, MaintenanceResult,
};
pub use migrations::{MigrationManager, ValidationResult};
pub use repository::{JobRepository, PromptRepository, Repository, UsageRepository};
pub use types::*;

// 最佳实践模块导出 - SQLx版本（停用）
// pub use best_practices_manager::{
//     BestPracticesDbManager, BestPracticesDbConfig, DatabaseHealthMetrics, BackupMetrics,
//     BestPracticesModel, QueryBuilder
// };

// Rusqlite版本导出（主要使用）
pub use best_practices_rusqlite::{
    BackupMetrics as RusqliteBackupMetrics, DatabaseHealthMetrics as RusqliteDatabaseHealthMetrics,
    MaintenanceResult as RusqliteMaintenanceResult, PromptStatistics as RusqlitePromptStatistics,
    RusqliteBestPracticesConfig, RusqliteBestPracticesManager, RusqliteModel, RusqlitePrompt,
};

// pub use models::prompt_best_practices::{
//     PromptBestPractices, CreatePromptInput, UpdatePromptInput, PromptFilter, PromptStatistics
// };

// 版本信息
pub const DATABASE_VERSION: u32 = 1;
pub const SCHEMA_VERSION: &str = "1.0.0";

// 默认配置
pub const DEFAULT_DATABASE_PATH: &str = "claude-night-pilot.db";
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30; // seconds
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;
