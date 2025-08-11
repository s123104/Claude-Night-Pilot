// Core module exports - 統一核心功能介面
pub mod scheduler;
pub mod cooldown;
pub mod retry;
pub mod process;
pub mod database;

// 測試模組 (暫時禁用，等待實現匹配)
// #[cfg(test)]
// pub mod tests;

// 排程相關導出
pub use scheduler::{
    Scheduler, SchedulerHandle, SchedulingConfig, SchedulerType,
    CronScheduler, AdaptiveScheduler, SessionScheduler,
    CronConfig, AdaptiveConfig, SessionConfig
};

// 冷卻檢測相關導出
pub use cooldown::{
    CooldownDetector, CooldownInfo, CooldownPattern
};

// 重試策略相關導出
pub use retry::{
    RetryStrategy, RetryConfig, RetryOrchestrator, 
    RetryAttempt, ErrorType, RetryStats
};

// 進程編排相關導出
pub use process::{
    ProcessOrchestrator, ProcessHandle, ProcessType, ProcessStatus,
    ProcessMetadata, ExecutionOptions, CleanupType, ProcessStats
};

// 數據庫相關導出
pub use database::{
    DatabaseManager, DatabaseConfig, DatabaseError, DatabaseResult,
    Repository, PromptRepository, JobRepository, UsageRepository,
    ConnectionManager, MigrationManager,
    Entity, Timestamped, EntityId,
    Prompt, Job, ExecutionResult, UsageStats,
    QueryOptions, PagedResult, JobStatus, ScheduleType, JobPriority,
    ResultStatus, TokenUsage,
    initialize_global_database_manager, get_global_database_manager,
    DatabaseHealthStatus, DatabaseStatistics, BackupResult, MaintenanceResult
};