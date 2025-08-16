// Core module exports - 統一核心功能介面
pub mod cooldown;
pub mod database;
pub mod process;
pub mod retry;
pub mod scheduler;
pub mod scheduler_runner;

// 測試模組 (暫時禁用，等待實現匹配)
// #[cfg(test)]
// pub mod tests;

// 排程相關導出
pub use scheduler::{
    AdaptiveConfig, AdaptiveScheduler, CronConfig, CronScheduler, Scheduler, SchedulerHandle,
    SchedulerType, SchedulingConfig, SessionConfig, SessionScheduler,
};

// 冷卻檢測相關導出
pub use cooldown::{CooldownDetector, CooldownInfo, CooldownPattern};

// 重試策略相關導出
pub use retry::{
    ErrorType, RetryAttempt, RetryConfig, RetryOrchestrator, RetryStats, RetryStrategy,
};

// 進程編排相關導出
pub use process::{
    CleanupType, ExecutionOptions, ProcessHandle, ProcessMetadata, ProcessOrchestrator,
    ProcessStats, ProcessStatus, ProcessType,
};

// 數據庫相關導出
pub use database::{
    get_global_database_manager, initialize_global_database_manager, BackupResult,
    ConnectionManager, DatabaseConfig, DatabaseError, DatabaseHealthStatus, DatabaseManager,
    DatabaseResult, DatabaseStatistics, Entity, EntityId, ExecutionResult, Job, JobPriority,
    JobRepository, JobStatus, MaintenanceResult, MigrationManager, PagedResult, Prompt,
    PromptRepository, QueryOptions, Repository, ResultStatus, ScheduleType, Timestamped,
    TokenUsage, UsageRepository, UsageStats,
};
