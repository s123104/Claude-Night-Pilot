// Job 引擎 - 協調排程器和執行器的核心組件
// 參考 Context7 最佳實踐實現企業級任務管理

use crate::models::job::{Job, JobStatus, NotificationChannel};
use crate::services::job_executor::{JobExecutionResult, JobExecutor};
use crate::services::job_scheduler::{JobExecutionCallback, JobScheduler};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info};

/// Job 引擎 - 管理整個任務系統的核心組件
pub struct JobEngine {
    /// 任務排程器
    scheduler: Arc<JobScheduler>,
    /// 任務執行器
    executor: Arc<JobExecutor>,
    /// 任務儲存庫
    job_repository: Arc<dyn JobRepository + Send + Sync>,
    /// 通知服務 - 目前正在開發中，未來版本將啟用
    #[allow(dead_code)]
    notification_service: Arc<NotificationService>,
    /// 引擎狀態
    state: Arc<RwLock<JobEngineState>>,
    /// 任務監控器
    monitor: Arc<TaskMonitor>,
}

impl std::fmt::Debug for JobEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobEngine")
            .field("scheduler", &"[configured]")
            .field("executor", &"[configured]")
            .field("state", &"[runtime]")
            .finish()
    }
}

/// Job 引擎狀態
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobEngineState {
    /// 引擎是否運行中
    pub is_running: bool,
    /// 啟動時間
    pub started_at: Option<DateTime<Utc>>,
    /// 總任務數
    pub total_jobs: usize,
    /// 活躍任務數
    pub active_jobs: usize,
    /// 正在執行的任務數
    pub running_jobs: usize,
    /// 暫停任務數
    pub paused_jobs: usize,
    /// 失敗任務數
    pub failed_jobs: usize,
    /// 上次健康檢查
    pub last_health_check: Option<DateTime<Utc>>,
}

/// 任務儲存庫 trait
#[async_trait::async_trait]
pub trait JobRepository: std::fmt::Debug {
    /// 保存任務
    async fn save_job(&self, job: &Job) -> Result<()>;
    /// 獲取任務
    async fn get_job(&self, job_id: &str) -> Result<Option<Job>>;
    /// 獲取所有任務
    async fn get_all_jobs(&self) -> Result<Vec<Job>>;
    /// 更新任務
    async fn update_job(&self, job: &Job) -> Result<()>;
    /// 刪除任務
    async fn delete_job(&self, job_id: &str) -> Result<()>;
    /// 獲取待執行任務
    async fn get_pending_jobs(&self) -> Result<Vec<Job>>;
    /// 獲取按狀態篩選的任務
    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<Job>>;
}

/// 記憶體任務儲存庫 (用於測試和開發)
#[derive(Debug, Default)]
pub struct InMemoryJobRepository {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
}

#[async_trait::async_trait]
impl JobRepository for InMemoryJobRepository {
    async fn save_job(&self, job: &Job) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Result<Option<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(job_id).cloned())
    }

    async fn get_all_jobs(&self) -> Result<Vec<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    async fn update_job(&self, job: &Job) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn delete_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(job_id);
        Ok(())
    }

    async fn get_pending_jobs(&self) -> Result<Vec<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|job| job.status == JobStatus::Active && job.can_execute())
            .cloned()
            .collect())
    }

    async fn get_jobs_by_status(&self, status: JobStatus) -> Result<Vec<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|job| job.status == status)
            .cloned()
            .collect())
    }
}

/// 通知服務
#[derive(Debug)]
pub struct NotificationService {
    enabled: bool,
}

impl NotificationService {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 發送通知
    pub async fn send_notification(
        &self,
        job: &Job,
        event: NotificationEvent,
        message: Option<String>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(config) = &job.notification_config {
            if !config.enabled {
                return Ok(());
            }

            let should_notify = match event {
                NotificationEvent::JobStarted => config.notify_on_start,
                NotificationEvent::JobCompleted => config.notify_on_success,
                NotificationEvent::JobFailed => config.notify_on_failure,
            };

            if !should_notify {
                return Ok(());
            }

            for channel in &config.notification_channels {
                self.send_to_channel(channel, job, &event, message.as_deref())
                    .await?;
            }
        }

        Ok(())
    }

    async fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        job: &Job,
        event: &NotificationEvent,
        message: Option<&str>,
    ) -> Result<()> {
        match channel {
            NotificationChannel::System => {
                info!("系統通知: 任務 {} - {:?}", job.name, event);
            }
            NotificationChannel::Email(email) => {
                info!("郵件通知 ({}): 任務 {} - {:?}", email, job.name, event);
                // TODO: 實際發送郵件
            }
            NotificationChannel::Webhook(url) => {
                info!("Webhook 通知 ({}): 任務 {} - {:?}", url, job.name, event);
                // TODO: 實際發送 HTTP 請求
            }
            NotificationChannel::Log => {
                info!(
                    "日誌通知: 任務 {} - {:?} - {}",
                    job.name,
                    event,
                    message.unwrap_or("")
                );
            }
        }

        Ok(())
    }
}

/// 通知事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    JobStarted,
    JobCompleted,
    JobFailed,
}

/// 任務監控器
#[derive(Debug)]
pub struct TaskMonitor {
    execution_history: Arc<Mutex<Vec<JobExecutionResult>>>,
    metrics: Arc<Mutex<TaskMetrics>>,
}

/// 任務指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// 總執行次數
    pub total_executions: u64,
    /// 成功執行次數
    pub successful_executions: u64,
    /// 失敗執行次數
    pub failed_executions: u64,
    /// 平均執行時間 (毫秒)
    pub average_execution_time_ms: f64,
    /// 成功率
    pub success_rate: f64,
    /// 上次更新時間
    pub last_updated: DateTime<Utc>,
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time_ms: 0.0,
            success_rate: 0.0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for TaskMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskMonitor {
    pub fn new() -> Self {
        Self {
            execution_history: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(TaskMetrics::default())),
        }
    }

    /// 記錄執行結果
    pub async fn record_execution(&self, result: JobExecutionResult) {
        let success = result.error.is_none();
        let duration_ms = result.total_duration_ms as f64;

        // 添加到歷史記錄
        {
            let mut history = self.execution_history.lock().await;
            history.push(result);

            // 保持最近 1000 條記錄
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // 更新指標
        {
            let mut metrics = self.metrics.lock().await;
            metrics.total_executions += 1;

            if success {
                metrics.successful_executions += 1;
            } else {
                metrics.failed_executions += 1;
            }

            // 更新平均執行時間
            let total_time = metrics.average_execution_time_ms
                * (metrics.total_executions - 1) as f64
                + duration_ms;
            metrics.average_execution_time_ms = total_time / metrics.total_executions as f64;

            // 更新成功率
            metrics.success_rate =
                metrics.successful_executions as f64 / metrics.total_executions as f64;
            metrics.last_updated = Utc::now();
        }
    }

    /// 獲取指標
    pub async fn get_metrics(&self) -> TaskMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    /// 獲取執行歷史
    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<JobExecutionResult> {
        let history = self.execution_history.lock().await;
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.clone()
        }
    }
}

/// Job 執行回調實現
#[derive(Debug)]
pub struct JobEngineExecutionCallback {
    executor: Arc<JobExecutor>,
    repository: Arc<dyn JobRepository + Send + Sync>,
    notification_service: Arc<NotificationService>,
    monitor: Arc<TaskMonitor>,
}

impl JobEngineExecutionCallback {
    pub fn new(
        executor: Arc<JobExecutor>,
        repository: Arc<dyn JobRepository + Send + Sync>,
        notification_service: Arc<NotificationService>,
        monitor: Arc<TaskMonitor>,
    ) -> Self {
        Self {
            executor,
            repository,
            notification_service,
            monitor,
        }
    }
}

#[async_trait::async_trait]
impl JobExecutionCallback for JobEngineExecutionCallback {
    async fn on_job_start(&self, job_id: &str) -> Result<()> {
        if let Some(job) = self.repository.get_job(job_id).await? {
            self.notification_service
                .send_notification(&job, NotificationEvent::JobStarted, None)
                .await?;
        }
        Ok(())
    }

    async fn on_job_complete(
        &self,
        job_id: &str,
        success: bool,
        output: Option<String>,
    ) -> Result<()> {
        if let Some(mut job) = self.repository.get_job(job_id).await? {
            job.complete_execution(success);
            self.repository.update_job(&job).await?;

            let event = if success {
                NotificationEvent::JobCompleted
            } else {
                NotificationEvent::JobFailed
            };

            self.notification_service
                .send_notification(&job, event, output)
                .await?;
        }
        Ok(())
    }

    async fn on_job_error(&self, job_id: &str, error: &str) -> Result<()> {
        if let Some(job) = self.repository.get_job(job_id).await? {
            self.notification_service
                .send_notification(&job, NotificationEvent::JobFailed, Some(error.to_string()))
                .await?;
        }
        Ok(())
    }

    async fn get_job_details(&self, job_id: &str) -> Result<Option<Job>> {
        self.repository.get_job(job_id).await
    }

    async fn execute_job(&self, job: &Job) -> Result<String> {
        let result = self.executor.execute_job(job).await?;

        // 記錄執行結果
        self.monitor.record_execution(result.clone()).await;

        if let Some(error) = &result.error {
            Err(anyhow::anyhow!("任務執行失敗: {}", error))
        } else {
            Ok(result.output.unwrap_or_else(|| "任務執行成功".to_string()))
        }
    }
}

// 移除手動實現的 Default，已使用 #[derive(Default)] 自動派生

impl JobEngine {
    /// 創建新的 Job 引擎
    pub async fn new() -> Result<Self> {
        let repository = Arc::new(InMemoryJobRepository::default());
        let notification_service = Arc::new(NotificationService::new(true));
        let monitor = Arc::new(TaskMonitor::new());
        let executor = Arc::new(JobExecutor::new().await?);

        // 創建執行回調
        let callback = Arc::new(JobEngineExecutionCallback::new(
            Arc::clone(&executor),
            Arc::clone(&repository) as Arc<dyn JobRepository + Send + Sync>,
            Arc::clone(&notification_service),
            Arc::clone(&monitor),
        ));

        let scheduler = Arc::new(JobScheduler::with_callback(callback).await?);

        Ok(Self {
            scheduler,
            executor,
            job_repository: repository,
            notification_service,
            state: Arc::new(RwLock::new(JobEngineState::default())),
            monitor,
        })
    }

    /// 使用自定義儲存庫創建引擎
    pub async fn with_repository(repository: Arc<dyn JobRepository + Send + Sync>) -> Result<Self> {
        let notification_service = Arc::new(NotificationService::new(true));
        let monitor = Arc::new(TaskMonitor::new());
        let executor = Arc::new(JobExecutor::new().await?);

        let callback = Arc::new(JobEngineExecutionCallback::new(
            Arc::clone(&executor),
            Arc::clone(&repository),
            Arc::clone(&notification_service),
            Arc::clone(&monitor),
        ));

        let scheduler = Arc::new(JobScheduler::with_callback(callback).await?);

        Ok(Self {
            scheduler,
            executor,
            job_repository: repository,
            notification_service,
            state: Arc::new(RwLock::new(JobEngineState::default())),
            monitor,
        })
    }

    /// 啟動引擎
    pub async fn start(&self) -> Result<()> {
        info!("啟動 Job 引擎");

        // 啟動排程器
        self.scheduler.start().await?;

        // 載入並排程所有活躍任務
        let jobs = self
            .job_repository
            .get_jobs_by_status(JobStatus::Active)
            .await?;
        for job in jobs {
            if let Err(e) = self.scheduler.schedule_job(job.clone()).await {
                error!("排程任務失敗: {} - {}", job.name, e);
            }
        }

        // 更新狀態
        {
            let mut state = self.state.write().await;
            state.is_running = true;
            state.started_at = Some(Utc::now());
        }

        info!("Job 引擎已啟動");
        Ok(())
    }

    /// 停止引擎
    pub async fn shutdown(&self) -> Result<()> {
        info!("停止 Job 引擎");

        // 停止排程器
        self.scheduler.shutdown().await?;

        // 更新狀態
        {
            let mut state = self.state.write().await;
            state.is_running = false;
        }

        info!("Job 引擎已停止");
        Ok(())
    }

    /// 創建任務
    pub async fn create_job(&self, job: Job) -> Result<String> {
        info!("創建任務: {}", job.name);

        // 保存到儲存庫
        self.job_repository.save_job(&job).await?;

        // 如果任務是活躍的，添加到排程器
        if job.status == JobStatus::Active {
            self.scheduler.schedule_job(job.clone()).await?;
        }

        Ok(job.id)
    }

    /// 更新任務
    pub async fn update_job(&self, job: Job) -> Result<()> {
        info!("更新任務: {}", job.name);

        // 從排程器移除舊的排程
        self.scheduler.unschedule_job(&job.id).await?;

        // 更新儲存庫
        self.job_repository.update_job(&job).await?;

        // 如果任務是活躍的，重新排程
        if job.status == JobStatus::Active {
            self.scheduler.schedule_job(job).await?;
        }

        Ok(())
    }

    /// 刪除任務
    pub async fn delete_job(&self, job_id: &str) -> Result<()> {
        info!("刪除任務: {}", job_id);

        // 從排程器移除
        self.scheduler.unschedule_job(job_id).await?;

        // 從儲存庫刪除
        self.job_repository.delete_job(job_id).await?;

        Ok(())
    }

    /// 暫停任務
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        if let Some(mut job) = self.job_repository.get_job(job_id).await? {
            job.pause();
            self.job_repository.update_job(&job).await?;
            self.scheduler.pause_job(job_id).await?;
            info!("任務已暫停: {}", job.name);
        }
        Ok(())
    }

    /// 恢復任務
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        if let Some(mut job) = self.job_repository.get_job(job_id).await? {
            job.resume();
            self.job_repository.update_job(&job).await?;
            self.scheduler.schedule_job(job.clone()).await?;
            info!("任務已恢復: {}", job.name);
        }
        Ok(())
    }

    /// 手動執行任務
    pub async fn trigger_job(&self, job_id: &str) -> Result<JobExecutionResult> {
        if let Some(job) = self.job_repository.get_job(job_id).await? {
            info!("手動觸發任務: {}", job.name);
            let result = self.executor.execute_job(&job).await?;
            self.monitor.record_execution(result.clone()).await;
            Ok(result)
        } else {
            Err(anyhow::anyhow!("找不到任務: {}", job_id))
        }
    }

    /// 獲取任務
    pub async fn get_job(&self, job_id: &str) -> Result<Option<Job>> {
        self.job_repository.get_job(job_id).await
    }

    /// 獲取所有任務
    pub async fn get_all_jobs(&self) -> Result<Vec<Job>> {
        self.job_repository.get_all_jobs().await
    }

    /// 獲取引擎狀態
    pub async fn get_engine_state(&self) -> JobEngineState {
        let mut state = self.state.write().await;

        // 更新統計信息
        if let Ok(all_jobs) = self.job_repository.get_all_jobs().await {
            state.total_jobs = all_jobs.len();
            state.active_jobs = all_jobs
                .iter()
                .filter(|j| j.status == JobStatus::Active)
                .count();
            state.running_jobs = all_jobs
                .iter()
                .filter(|j| j.status == JobStatus::Running)
                .count();
            state.paused_jobs = all_jobs
                .iter()
                .filter(|j| j.status == JobStatus::Paused)
                .count();
            state.failed_jobs = all_jobs
                .iter()
                .filter(|j| j.status == JobStatus::Failed)
                .count();
        }

        state.last_health_check = Some(Utc::now());
        state.clone()
    }

    /// 獲取任務指標
    pub async fn get_task_metrics(&self) -> TaskMetrics {
        self.monitor.get_metrics().await
    }

    /// 健康檢查
    pub async fn health_check(&self) -> Result<bool> {
        let scheduler_ok = self.scheduler.health_check().await.unwrap_or(false);
        let executor_ok = self.executor.health_check().await.unwrap_or(false);

        Ok(scheduler_ok && executor_ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::job::JobType;

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = JobEngine::new().await.unwrap();
        assert!(!engine.get_engine_state().await.is_running);
    }

    #[tokio::test]
    async fn test_engine_start_stop() {
        let engine = JobEngine::new().await.unwrap();

        engine.start().await.unwrap();
        assert!(engine.get_engine_state().await.is_running);

        engine.shutdown().await.unwrap();
        assert!(!engine.get_engine_state().await.is_running);
    }

    #[tokio::test]
    async fn test_job_lifecycle() {
        let engine = JobEngine::new().await.unwrap();

        let job = Job::new("測試任務", "test_prompt", "0 * * * * *");
        let job_id = job.id.clone();

        // 創建任務
        engine.create_job(job).await.unwrap();

        // 獲取任務
        let retrieved_job = engine.get_job(&job_id).await.unwrap();
        assert!(retrieved_job.is_some());

        // 暫停任務
        engine.pause_job(&job_id).await.unwrap();

        // 恢復任務
        engine.resume_job(&job_id).await.unwrap();

        // 刪除任務
        engine.delete_job(&job_id).await.unwrap();

        // 驗證任務已刪除
        let deleted_job = engine.get_job(&job_id).await.unwrap();
        assert!(deleted_job.is_none());
    }
}
