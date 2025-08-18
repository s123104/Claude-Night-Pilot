// 🕒 Claude Night Pilot - 排程模塊
// 企業級排程系統核心模塊
// 創建時間: 2025-08-17T04:05:00+00:00

pub mod real_time_executor;
pub mod unified_scheduler;

pub use real_time_executor::{ExecutionStats, RealTimeExecutor};
pub use unified_scheduler::UnifiedScheduler;

/// 排程器介面定義
/// 基於 Context7 Tauri 最佳實踐的統一介面
pub trait SchedulerExecutor: Send + Sync {
    /// 啟動排程器
    async fn start(&self) -> anyhow::Result<()>;

    /// 停止排程器
    async fn stop(&self) -> anyhow::Result<()>;

    /// 添加排程任務
    async fn add_job(&self, job: &crate::models::job::Job) -> anyhow::Result<String>;

    /// 移除排程任務
    async fn remove_job(&self, job_id: &str) -> anyhow::Result<bool>;

    /// 獲取任務狀態
    async fn get_job_status(
        &self,
        job_id: &str,
    ) -> anyhow::Result<Option<crate::models::job::JobStatus>>;

    /// 獲取活躍任務列表
    async fn get_active_jobs(&self) -> anyhow::Result<Vec<String>>;
}

impl SchedulerExecutor for RealTimeExecutor {
    async fn start(&self) -> anyhow::Result<()> {
        self.start().await
    }

    async fn stop(&self) -> anyhow::Result<()> {
        self.stop().await
    }

    async fn add_job(&self, job: &crate::models::job::Job) -> anyhow::Result<String> {
        self.add_job(job).await
    }

    async fn remove_job(&self, job_id: &str) -> anyhow::Result<bool> {
        self.remove_job(job_id).await
    }

    async fn get_job_status(
        &self,
        job_id: &str,
    ) -> anyhow::Result<Option<crate::models::job::JobStatus>> {
        self.get_job_status(job_id).await
    }

    async fn get_active_jobs(&self) -> anyhow::Result<Vec<String>> {
        self.get_active_jobs().await
    }
}
