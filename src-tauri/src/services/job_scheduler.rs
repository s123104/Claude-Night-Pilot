// Job 排程器 - 基於 tokio-cron-scheduler 實現
// 參考 Context7 最佳實踐和 vibe-kanban 設計模式

use crate::models::job::{Job, JobStatus, JobType};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_cron_scheduler::{Job as CronJob, JobScheduler as TokioCronScheduler};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Job 排程器 - 管理所有排程任務的核心組件
pub struct JobScheduler {
    /// tokio-cron-scheduler 實例
    scheduler: Arc<TokioCronScheduler>,
    /// 活躍任務映射 (Job ID -> Cron Job ID)
    active_jobs: Arc<RwLock<HashMap<String, Uuid>>>,
    /// 任務執行狀態追蹤
    execution_tracker: Arc<Mutex<HashMap<String, JobExecutionState>>>,
    /// 排程器狀態
    state: Arc<RwLock<SchedulerState>>,
    /// 任務執行回調
    execution_callback: Arc<dyn JobExecutionCallback>,
}

/// 排程器狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerState {
    /// 是否運行中
    pub is_running: bool,
    /// 總任務數
    pub total_jobs: usize,
    /// 活躍任務數
    pub active_jobs: usize,
    /// 暫停任務數
    pub paused_jobs: usize,
    /// 上次健康檢查時間
    pub last_health_check: Option<DateTime<Utc>>,
    /// 啟動時間
    pub started_at: Option<DateTime<Utc>>,
}

/// 任務執行狀態
#[derive(Debug, Clone)]
pub struct JobExecutionState {
    /// 任務 ID
    pub job_id: String,
    /// 當前狀態
    pub status: JobStatus,
    /// 開始時間
    pub started_at: Option<DateTime<Utc>>,
    /// 執行計數
    pub execution_count: u64,
    /// 失敗計數
    pub failure_count: u64,
    /// 上次執行時間
    pub last_execution: Option<DateTime<Utc>>,
    /// 下次執行時間
    pub next_execution: Option<DateTime<Utc>>,
}

/// 任務執行回調 trait
#[async_trait::async_trait]
pub trait JobExecutionCallback: std::fmt::Debug + Send + Sync {
    /// 任務開始執行回調
    async fn on_job_start(&self, job_id: &str) -> Result<()>;
    
    /// 任務執行完成回調
    async fn on_job_complete(&self, job_id: &str, success: bool, output: Option<String>) -> Result<()>;
    
    /// 任務執行失敗回調
    async fn on_job_error(&self, job_id: &str, error: &str) -> Result<()>;
    
    /// 獲取任務詳情
    async fn get_job_details(&self, job_id: &str) -> Result<Option<Job>>;
    
    /// 執行任務
    async fn execute_job(&self, job: &Job) -> Result<String>;
}

/// 預設任務執行回調實現
#[derive(Debug)]
pub struct DefaultJobExecutionCallback {
    job_service: Arc<crate::services::job_service::JobService>,
}

impl DefaultJobExecutionCallback {
    pub async fn new() -> Result<Self> {
        let job_service = Arc::new(crate::services::job_service::JobService::new().await?);
        Ok(Self { job_service })
    }
}

#[async_trait::async_trait]
impl JobExecutionCallback for DefaultJobExecutionCallback {
    async fn on_job_start(&self, job_id: &str) -> Result<()> {
        info!("Job {} 開始執行", job_id);
        Ok(())
    }
    
    async fn on_job_complete(&self, job_id: &str, success: bool, output: Option<String>) -> Result<()> {
        if success {
            info!("Job {} 執行成功: {:?}", job_id, output);
        } else {
            warn!("Job {} 執行失敗: {:?}", job_id, output);
        }
        Ok(())
    }
    
    async fn on_job_error(&self, job_id: &str, error: &str) -> Result<()> {
        error!("Job {} 執行錯誤: {}", job_id, error);
        Ok(())
    }
    
    async fn get_job_details(&self, job_id: &str) -> Result<Option<Job>> {
        // TODO: 從資料庫獲取 Job 詳情
        // 這裡需要實現從 job_service 獲取 Job 的邏輯
        warn!("get_job_details 尚未完全實現: {}", job_id);
        Ok(None)
    }
    
    async fn execute_job(&self, job: &Job) -> Result<String> {
        // TODO: 使用 ClaudeExecutor 執行任務
        // 這裡需要集成現有的 ClaudeExecutor
        info!("執行任務: {} (Prompt: {})", job.name, job.prompt_id);
        Ok(format!("Job {} 執行完成", job.id))
    }
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            is_running: false,
            total_jobs: 0,
            active_jobs: 0,
            paused_jobs: 0,
            last_health_check: None,
            started_at: None,
        }
    }
}

impl JobScheduler {
    /// 創建新的 Job 排程器
    pub async fn new() -> Result<Self> {
        let scheduler = TokioCronScheduler::new()
            .await
            .context("創建 tokio-cron-scheduler 失敗")?;
        
        let callback = Arc::new(
            DefaultJobExecutionCallback::new()
                .await
                .context("創建預設執行回調失敗")?
        );
        
        Ok(Self {
            scheduler: Arc::new(scheduler),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            execution_tracker: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(RwLock::new(SchedulerState::default())),
            execution_callback: callback,
        })
    }
    
    /// 使用自定義執行回調創建排程器
    pub async fn with_callback(
        callback: Arc<dyn JobExecutionCallback + Send + Sync>
    ) -> Result<Self> {
        let scheduler = TokioCronScheduler::new()
            .await
            .context("創建 tokio-cron-scheduler 失敗")?;
        
        Ok(Self {
            scheduler: Arc::new(scheduler),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            execution_tracker: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(RwLock::new(SchedulerState::default())),
            execution_callback: callback,
        })
    }
    
    /// 啟動排程器
    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await.context("啟動排程器失敗")?;
        
        let mut state = self.state.write().await;
        state.is_running = true;
        state.started_at = Some(Utc::now());
        
        info!("Job 排程器已啟動");
        Ok(())
    }
    
    /// 停止排程器
    pub async fn shutdown(&self) -> Result<()> {
        self.scheduler.shutdown().await.context("停止排程器失敗")?;
        
        let mut state = self.state.write().await;
        state.is_running = false;
        
        info!("Job 排程器已停止");
        Ok(())
    }
    
    /// 添加或更新排程任務
    pub async fn schedule_job(&self, job: Job) -> Result<()> {
        match job.job_type {
            JobType::Scheduled => self.schedule_cron_job(job).await,
            JobType::OneTime => self.schedule_one_time_job(job).await,
            JobType::Interval => self.schedule_interval_job(job).await,
            JobType::Triggered => {
                // 觸發式任務不需要排程，只需要記錄
                self.register_triggered_job(job).await
            }
        }
    }
    
    /// 排程 Cron 任務
    async fn schedule_cron_job(&self, job: Job) -> Result<()> {
        let job_id = job.id.clone();
        let job_name = job.name.clone();
        let cron_expr = job.cron_expression.clone();
        
        // 如果任務已存在，先移除
        self.unschedule_job(&job_id).await?;
        
        // 創建執行回調的 Arc 引用
        let callback = Arc::clone(&self.execution_callback);
        let execution_tracker = Arc::clone(&self.execution_tracker);
        let active_jobs = Arc::clone(&self.active_jobs);
        
        // 創建 Cron 任務
        let cron_job = CronJob::new_async(&cron_expr, move |_uuid, _lock| {
            let job_id = job_id.clone();
            let job_name = job_name.clone();
            let callback = Arc::clone(&callback);
            let execution_tracker = Arc::clone(&execution_tracker);
            
            Box::pin(async move {
                debug!("執行 Cron 任務: {} ({})", job_name, job_id);
                
                // 更新執行狀態
                {
                    let mut tracker = execution_tracker.lock().await;
                    if let Some(state) = tracker.get_mut(&job_id) {
                        state.status = JobStatus::Running;
                        state.started_at = Some(Utc::now());
                        state.execution_count += 1;
                        state.last_execution = Some(Utc::now());
                    }
                }
                
                // 執行任務回調
                let result = match callback.on_job_start(&job_id).await {
                    Ok(_) => {
                        // 獲取任務詳情並執行
                        match callback.get_job_details(&job_id).await {
                            Ok(Some(job_details)) => {
                                match callback.execute_job(&job_details).await {
                                    Ok(output) => {
                                        callback.on_job_complete(&job_id, true, Some(output)).await
                                    }
                                    Err(e) => {
                                        let error_msg = e.to_string();
                                        callback.on_job_error(&job_id, &error_msg).await?;
                                        callback.on_job_complete(&job_id, false, Some(error_msg)).await
                                    }
                                }
                            }
                            Ok(None) => {
                                let error_msg = format!("找不到任務詳情: {}", job_id);
                                callback.on_job_error(&job_id, &error_msg).await?;
                                callback.on_job_complete(&job_id, false, Some(error_msg)).await
                            }
                            Err(e) => {
                                let error_msg = format!("獲取任務詳情失敗: {}", e);
                                callback.on_job_error(&job_id, &error_msg).await?;
                                callback.on_job_complete(&job_id, false, Some(error_msg)).await
                            }
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("任務啟動失敗: {}", e);
                        callback.on_job_error(&job_id, &error_msg).await?;
                        callback.on_job_complete(&job_id, false, Some(error_msg)).await
                    }
                };
                
                // 更新完成狀態
                {
                    let mut tracker = execution_tracker.lock().await;
                    if let Some(state) = tracker.get_mut(&job_id) {
                        state.status = JobStatus::Active;
                        state.started_at = None;
                        if result.is_err() {
                            state.failure_count += 1;
                        }
                    }
                }
                
                if let Err(e) = result {
                    error!("任務執行回調失敗: {}", e);
                }
            })
        })
        .context("創建 Cron 任務失敗")?;
        
        // 添加到排程器
        let cron_job_id = self.scheduler.add(cron_job).await.context("添加 Cron 任務失敗")?;
        
        // 記錄任務映射
        {
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.insert(job.id.clone(), cron_job_id);
        }
        
        // 初始化執行狀態
        {
            let mut tracker = self.execution_tracker.lock().await;
            tracker.insert(job.id.clone(), JobExecutionState {
                job_id: job.id.clone(),
                status: JobStatus::Active,
                started_at: None,
                execution_count: job.execution_count,
                failure_count: job.failure_count,
                last_execution: job.last_run_time,
                next_execution: job.next_run_time,
            });
        }
        
        info!("Cron 任務已排程: {} ({})", job.name, job.id);
        Ok(())
    }
    
    /// 排程一次性任務
    async fn schedule_one_time_job(&self, job: Job) -> Result<()> {
        // TODO: 實現一次性任務排程
        // tokio-cron-scheduler 支援一次性任務，需要計算延遲時間
        warn!("一次性任務排程尚未實現: {}", job.name);
        Ok(())
    }
    
    /// 排程間隔任務
    async fn schedule_interval_job(&self, job: Job) -> Result<()> {
        // TODO: 實現間隔任務排程
        // 可以轉換為 Cron 表達式或使用 tokio::time::interval
        warn!("間隔任務排程尚未實現: {}", job.name);
        Ok(())
    }
    
    /// 註冊觸發式任務
    async fn register_triggered_job(&self, job: Job) -> Result<()> {
        // 觸發式任務不需要排程，只需要記錄狀態
        let mut tracker = self.execution_tracker.lock().await;
        tracker.insert(job.id.clone(), JobExecutionState {
            job_id: job.id.clone(),
            status: job.status,
            started_at: None,
            execution_count: job.execution_count,
            failure_count: job.failure_count,
            last_execution: job.last_run_time,
            next_execution: job.next_run_time,
        });
        
        info!("觸發式任務已註冊: {} ({})", job.name, job.id);
        Ok(())
    }
    
    /// 取消排程任務
    pub async fn unschedule_job(&self, job_id: &str) -> Result<()> {
        // 從活躍任務中移除
        let cron_job_id = {
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.remove(job_id)
        };
        
        // 如果存在 Cron 任務，從排程器移除
        if let Some(cron_job_id) = cron_job_id {
            self.scheduler.remove(&cron_job_id).await.context("移除 Cron 任務失敗")?;
        }
        
        // 從執行追蹤中移除
        {
            let mut tracker = self.execution_tracker.lock().await;
            tracker.remove(job_id);
        }
        
        debug!("任務已取消排程: {}", job_id);
        Ok(())
    }
    
    /// 暫停任務
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        // 從排程器移除但保留狀態
        if let Some(cron_job_id) = {
            let active_jobs = self.active_jobs.read().await;
            active_jobs.get(job_id).copied()
        } {
            self.scheduler.remove(&cron_job_id).await.context("移除任務失敗")?;
            
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.remove(job_id);
        }
        
        // 更新狀態
        {
            let mut tracker = self.execution_tracker.lock().await;
            if let Some(state) = tracker.get_mut(job_id) {
                state.status = JobStatus::Paused;
            }
        }
        
        info!("任務已暫停: {}", job_id);
        Ok(())
    }
    
    /// 恢復任務
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        // TODO: 從資料庫重新載入任務並重新排程
        warn!("任務恢復功能尚未實現: {}", job_id);
        Ok(())
    }
    
    /// 手動觸發任務執行
    pub async fn trigger_job(&self, job_id: &str) -> Result<()> {
        // TODO: 立即執行指定任務
        warn!("手動觸發功能尚未實現: {}", job_id);
        Ok(())
    }
    
    /// 獲取任務執行狀態
    pub async fn get_job_state(&self, job_id: &str) -> Option<JobExecutionState> {
        let tracker = self.execution_tracker.lock().await;
        tracker.get(job_id).cloned()
    }
    
    /// 獲取所有任務狀態
    pub async fn get_all_job_states(&self) -> HashMap<String, JobExecutionState> {
        let tracker = self.execution_tracker.lock().await;
        tracker.clone()
    }
    
    /// 獲取排程器狀態
    pub async fn get_scheduler_state(&self) -> SchedulerState {
        let mut state = self.state.write().await;
        
        // 更新統計信息
        let tracker = self.execution_tracker.lock().await;
        state.total_jobs = tracker.len();
        state.active_jobs = tracker.values().filter(|s| s.status == JobStatus::Active).count();
        state.paused_jobs = tracker.values().filter(|s| s.status == JobStatus::Paused).count();
        state.last_health_check = Some(Utc::now());
        
        state.clone()
    }
    
    /// 健康檢查
    pub async fn health_check(&self) -> Result<bool> {
        // 檢查排程器是否運行
        let state = self.state.read().await;
        Ok(state.is_running)
    }
    
    /// 清理已完成的任務狀態
    pub async fn cleanup_completed_jobs(&self) -> Result<usize> {
        let mut tracker = self.execution_tracker.lock().await;
        let initial_count = tracker.len();
        
        // 移除已完成、已取消或失敗的任務狀態
        tracker.retain(|_, state| {
            !matches!(
                state.status,
                JobStatus::Completed | JobStatus::Cancelled | JobStatus::Failed
            )
        });
        
        let removed_count = initial_count - tracker.len();
        debug!("清理了 {} 個已完成的任務狀態", removed_count);
        Ok(removed_count)
    }
}

// 實現 Debug trait
impl std::fmt::Debug for JobScheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobScheduler")
            .field("active_jobs_count", &"[computed]")
            .field("execution_tracker_count", &"[computed]")
            .finish()
    }
}

// 實現 Drop trait 確保資源清理
impl Drop for JobScheduler {
    fn drop(&mut self) {
        debug!("JobScheduler 正在清理資源");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = JobScheduler::new().await.unwrap();
        assert!(!scheduler.get_scheduler_state().await.is_running);
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let scheduler = JobScheduler::new().await.unwrap();
        
        scheduler.start().await.unwrap();
        assert!(scheduler.get_scheduler_state().await.is_running);
        
        scheduler.shutdown().await.unwrap();
        assert!(!scheduler.get_scheduler_state().await.is_running);
    }

    #[tokio::test]
    async fn test_job_scheduling() {
        let scheduler = JobScheduler::new().await.unwrap();
        scheduler.start().await.unwrap();

        let job = Job::new(
            "測試任務",
            "test_prompt_123",
            "0 * * * * *" // 每分鐘執行
        );

        scheduler.schedule_job(job.clone()).await.unwrap();
        
        // 檢查任務是否被記錄
        let state = scheduler.get_job_state(&job.id).await;
        assert!(state.is_some());
        assert_eq!(state.unwrap().status, JobStatus::Active);

        scheduler.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_job_unscheduling() {
        let scheduler = JobScheduler::new().await.unwrap();
        scheduler.start().await.unwrap();

        let job = Job::new("測試任務", "test_prompt", "0 * * * * *");
        let job_id = job.id.clone();

        scheduler.schedule_job(job).await.unwrap();
        scheduler.unschedule_job(&job_id).await.unwrap();

        // 檢查任務是否被移除
        let state = scheduler.get_job_state(&job_id).await;
        assert!(state.is_none());

        scheduler.shutdown().await.unwrap();
    }
}