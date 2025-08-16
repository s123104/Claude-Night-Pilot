// Services 模組 - 業務邏輯層
// 採用 vibe-kanban 架構模式

pub mod claude_service;
pub mod database_service;
pub mod monitoring_service;
pub mod scheduler_service;

// 新的任務管理系統模組 - Context7 最佳實踐
pub mod job_engine;
pub mod job_executor;
pub mod job_scheduler;

// 保留原有服務
pub mod health_service;
pub mod job_service;
pub mod prompt_service;
pub mod sync_service;

#[cfg(test)]
pub mod tests;

// 重新導出主要服務
pub use claude_service::*;
pub use database_service::*;
pub use monitoring_service::*;
pub use scheduler_service::*;

// 新的任務管理系統導出
pub use job_engine::{JobEngine, JobEngineState, TaskMetrics};
pub use job_executor::{JobExecutor, JobExecution, JobExecutionResult};
pub use job_scheduler::{JobScheduler, JobExecutionCallback};

// 保留原有服務導出
pub use health_service::HealthService;
pub use job_service::JobService;
pub use prompt_service::PromptService;
pub use sync_service::SyncService;

// Tauri命令包裝器 - 提供給GUI調用
pub use prompt_service::{
    prompt_service_create_prompt, prompt_service_delete_prompt, prompt_service_list_prompts,
};

pub use job_service::{job_service_create_job, job_service_delete_job, job_service_list_jobs};

pub use sync_service::{sync_service_get_status, sync_service_trigger_sync};

use anyhow::Result;
use std::sync::Arc;

/// 服務容器 - 依賴注入容器
///
/// 管理所有服務實例的生命週期和依賴關係
pub struct ServiceContainer {
    claude_service: Arc<ClaudeService>,
    database_service: Arc<DatabaseService>,
    scheduler_service: Arc<SchedulerService>,
    monitoring_service: Arc<MonitoringService>,
}

impl ServiceContainer {
    /// 創建新的服務容器
    pub async fn new() -> Result<Self> {
        // 初始化數據庫服務
        let database_service = Arc::new(DatabaseService::new().await?);

        // 初始化監控服務
        let monitoring_service = Arc::new(MonitoringService::new());

        // 初始化 Claude 服務
        let claude_service = Arc::new(ClaudeService::with_defaults());

        // 初始化調度服務
        let scheduler_service = Arc::new(
            SchedulerService::new(
                Arc::clone(&database_service),
                Arc::clone(&claude_service),
                Arc::clone(&monitoring_service),
            )
            .await?,
        );

        Ok(Self {
            claude_service,
            database_service,
            scheduler_service,
            monitoring_service,
        })
    }

    /// 獲取 Claude 服務
    pub fn claude_service(&self) -> Arc<ClaudeService> {
        Arc::clone(&self.claude_service)
    }

    /// 獲取數據庫服務
    pub fn database_service(&self) -> Arc<DatabaseService> {
        Arc::clone(&self.database_service)
    }

    /// 獲取調度服務
    pub fn scheduler_service(&self) -> Arc<SchedulerService> {
        Arc::clone(&self.scheduler_service)
    }

    /// 獲取監控服務
    pub fn monitoring_service(&self) -> Arc<MonitoringService> {
        Arc::clone(&self.monitoring_service)
    }

    /// 啟動所有服務
    pub async fn start(&self) -> Result<()> {
        // 啟動調度服務
        self.scheduler_service.start().await?;

        // 啟動監控服務
        self.monitoring_service.start().await?;

        tracing::info!("所有服務已啟動");
        Ok(())
    }

    /// 停止所有服務
    pub async fn stop(&self) -> Result<()> {
        // 停止調度服務
        self.scheduler_service.stop().await?;

        // 停止監控服務
        self.monitoring_service.stop().await?;

        tracing::info!("所有服務已停止");
        Ok(())
    }

    /// 健康檢查
    pub async fn health_check(&self) -> ServiceHealthStatus {
        let claude_healthy = self.claude_service.as_ref().health_check().await;
        let database_healthy = self.database_service.as_ref().health_check().await;
        let scheduler_healthy = self.scheduler_service.as_ref().health_check().await;
        let monitoring_healthy = self.monitoring_service.as_ref().health_check().await;

        ServiceHealthStatus {
            claude_service: claude_healthy,
            database_service: database_healthy,
            scheduler_service: scheduler_healthy,
            monitoring_service: monitoring_healthy,
            overall_healthy: claude_healthy
                && database_healthy
                && scheduler_healthy
                && monitoring_healthy,
        }
    }
}

/// 服務健康狀態
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceHealthStatus {
    pub claude_service: bool,
    pub database_service: bool,
    pub scheduler_service: bool,
    pub monitoring_service: bool,
    pub overall_healthy: bool,
}

/// 服務特性 - 所有服務必須實現的基本功能
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    /// 服務名稱
    fn name(&self) -> &'static str;

    /// 啟動服務
    async fn start(&self) -> Result<()>;

    /// 停止服務
    async fn stop(&self) -> Result<()>;

    /// 健康檢查
    async fn health_check(&self) -> bool;

    /// 獲取服務統計資訊
    async fn get_stats(&self) -> serde_json::Value;
}
