// 統一排程介面與實現
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use chrono::Timelike;

#[async_trait]
pub trait Scheduler: Send + Sync {
    type Config;
    type Handle;
    
    async fn schedule(&mut self, config: Self::Config) -> Result<Self::Handle>;
    async fn cancel(&mut self, handle: Self::Handle) -> Result<()>;
    async fn reschedule(&mut self, handle: Self::Handle, config: Self::Config) -> Result<Self::Handle>;
    fn is_running(&self, handle: &Self::Handle) -> bool;
    async fn list_active(&self) -> Vec<Self::Handle>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    pub scheduler_type: SchedulerType,
    pub cron: Option<CronConfig>,
    pub adaptive: Option<AdaptiveConfig>,
    pub session: Option<SessionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerType {
    Cron,
    Adaptive,
    Session,
    Immediate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronConfig {
    pub expression: String,
    pub timezone: String,
    pub max_concurrent: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub intervals: Vec<(u64, u64)>, // (threshold_minutes, check_interval_seconds)
    pub ccusage_integration: bool,
    pub fallback_to_time_based: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub scheduled_time: String, // HH:MM format
    pub auto_reschedule: bool,
    pub daily_repeat: bool,
}

pub type SchedulerHandle = Uuid;

// Cron 排程器實現
pub struct CronScheduler {
    jobs: HashMap<Uuid, CronJob>,
    scheduler: tokio_cron_scheduler::JobScheduler,
}

struct CronJob {
    id: Uuid,
    config: CronConfig,
    job_id: Option<uuid::Uuid>,
    status: JobStatus,
}

#[derive(Debug, Clone)]
enum JobStatus {
    Scheduled,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

impl CronScheduler {
    pub async fn new() -> Result<Self> {
        let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;
        Ok(Self {
            jobs: HashMap::new(),
            scheduler,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;
        Ok(())
    }
    
    pub async fn shutdown(&mut self) -> Result<()> {
        self.scheduler.shutdown().await?;
        Ok(())
    }
}

#[async_trait]
impl Scheduler for CronScheduler {
    type Config = CronConfig;
    type Handle = Uuid;
    
    async fn schedule(&mut self, config: Self::Config) -> Result<Self::Handle> {
        let job_id = Uuid::new_v4();
        
        // 創建 cron job
        let job = tokio_cron_scheduler::Job::new_async(&config.expression, |_uuid, _l| {
            Box::pin(async move {
                // 執行任務邏輯
                tracing::info!("執行排程任務: {}", _uuid);
                // TODO: 調用實際的任務執行邏輯
            })
        })?;
        
        let scheduler_job_id = self.scheduler.add(job).await?;
        
        let cron_job = CronJob {
            id: job_id,
            config,
            job_id: Some(scheduler_job_id),
            status: JobStatus::Scheduled,
        };
        
        self.jobs.insert(job_id, cron_job);
        Ok(job_id)
    }
    
    async fn cancel(&mut self, handle: Self::Handle) -> Result<()> {
        if let Some(job) = self.jobs.get_mut(&handle) {
            if let Some(job_id) = &job.job_id {
                self.scheduler.remove(job_id).await?;
                job.status = JobStatus::Cancelled;
            }
        }
        Ok(())
    }
    
    async fn reschedule(&mut self, handle: Self::Handle, config: Self::Config) -> Result<Self::Handle> {
        self.cancel(handle).await?;
        self.schedule(config).await
    }
    
    fn is_running(&self, handle: &Self::Handle) -> bool {
        self.jobs.get(handle)
            .map(|job| matches!(job.status, JobStatus::Running))
            .unwrap_or(false)
    }
    
    async fn list_active(&self) -> Vec<Self::Handle> {
        self.jobs.iter()
            .filter(|(_, job)| matches!(job.status, JobStatus::Scheduled | JobStatus::Running))
            .map(|(id, _)| *id)
            .collect()
    }
}

// 適應性排程器實現
pub struct AdaptiveScheduler {
    timers: HashMap<Uuid, AdaptiveTimer>,
    config: AdaptiveConfig,
}

struct AdaptiveTimer {
    id: Uuid,
    handle: Option<tokio::task::JoinHandle<()>>,
    status: JobStatus,
}

impl AdaptiveScheduler {
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            timers: HashMap::new(),
            config,
        }
    }
    
    async fn get_adaptive_interval(&self, remaining_minutes: u64) -> std::time::Duration {
        for (threshold, interval_seconds) in &self.config.intervals {
            if remaining_minutes > *threshold {
                return std::time::Duration::from_secs(*interval_seconds);
            }
        }
        // 預設間隔
        std::time::Duration::from_secs(30)
    }
}

#[async_trait]
impl Scheduler for AdaptiveScheduler {
    type Config = AdaptiveConfig;
    type Handle = Uuid;
    
    async fn schedule(&mut self, config: Self::Config) -> Result<Self::Handle> {
        let timer_id = Uuid::new_v4();
        
        let handle = tokio::spawn(async move {
            loop {
                // 獲取剩餘時間
                let remaining_minutes = if config.ccusage_integration {
                    // TODO: 整合 ccusage 邏輯
                    60 // 暫時預設值
                } else {
                    // 基於時間的計算
                    60
                };
                
                // 計算下次檢查間隔
                let interval = Self::calculate_adaptive_interval(&config.intervals, remaining_minutes);
                
                if remaining_minutes <= 2 {
                    // 執行任務
                    tracing::info!("適應性排程觸發執行");
                    break;
                }
                
                tokio::time::sleep(interval).await;
            }
        });
        
        let timer = AdaptiveTimer {
            id: timer_id,
            handle: Some(handle),
            status: JobStatus::Scheduled,
        };
        
        self.timers.insert(timer_id, timer);
        Ok(timer_id)
    }
    
    async fn cancel(&mut self, handle: Self::Handle) -> Result<()> {
        if let Some(timer) = self.timers.get_mut(&handle) {
            if let Some(task_handle) = timer.handle.take() {
                task_handle.abort();
                timer.status = JobStatus::Cancelled;
            }
        }
        Ok(())
    }
    
    async fn reschedule(&mut self, handle: Self::Handle, config: Self::Config) -> Result<Self::Handle> {
        self.cancel(handle).await?;
        self.schedule(config).await
    }
    
    fn is_running(&self, handle: &Self::Handle) -> bool {
        self.timers.get(handle)
            .map(|timer| matches!(timer.status, JobStatus::Running))
            .unwrap_or(false)
    }
    
    async fn list_active(&self) -> Vec<Self::Handle> {
        self.timers.iter()
            .filter(|(_, timer)| matches!(timer.status, JobStatus::Scheduled | JobStatus::Running))
            .map(|(id, _)| *id)
            .collect()
    }
}

impl AdaptiveScheduler {
    fn calculate_adaptive_interval(intervals: &[(u64, u64)], remaining_minutes: u64) -> std::time::Duration {
        for (threshold, interval_seconds) in intervals {
            if remaining_minutes > *threshold {
                return std::time::Duration::from_secs(*interval_seconds);
            }
        }
        std::time::Duration::from_secs(30) // 預設
    }
}

// 會話排程器實現
pub struct SessionScheduler {
    sessions: HashMap<Uuid, ScheduledSession>,
}

struct ScheduledSession {
    id: Uuid,
    config: SessionConfig,
    next_execution: chrono::DateTime<chrono::Local>,
    handle: Option<tokio::task::JoinHandle<()>>,
    status: JobStatus,
}

impl SessionScheduler {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    
    fn parse_time(&self, time_str: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("無效的時間格式: {}", time_str));
        }
        
        let hours: u32 = parts[0].parse()?;
        let minutes: u32 = parts[1].parse()?;
        
        if hours >= 24 || minutes >= 60 {
            return Err(anyhow::anyhow!("無效的時間值: {}:{}", hours, minutes));
        }
        
        Ok((hours, minutes))
    }
    
    fn calculate_next_execution(&self, hours: u32, minutes: u32) -> chrono::DateTime<chrono::Local> {
        let now = chrono::Local::now();
        let mut target = now
            .with_hour(hours)
            .and_then(|t| t.with_minute(minutes))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap();
        
        // 如果已過期，安排到明天
        if target <= now {
            target = target + chrono::Duration::days(1);
        }
        
        target
    }
}

#[async_trait]
impl Scheduler for SessionScheduler {
    type Config = SessionConfig;
    type Handle = Uuid;
    
    async fn schedule(&mut self, config: Self::Config) -> Result<Self::Handle> {
        let session_id = Uuid::new_v4();
        let (hours, minutes) = self.parse_time(&config.scheduled_time)?;
        let next_execution = self.calculate_next_execution(hours, minutes);
        
        let duration_until = next_execution.signed_duration_since(chrono::Local::now());
        let tokio_duration = duration_until.to_std()
            .map_err(|_| anyhow::anyhow!("無法計算等待時間"))?;
        
        let auto_reschedule = config.auto_reschedule;
        let daily_repeat = config.daily_repeat;
        let scheduled_time = config.scheduled_time.clone();
        
        let handle = tokio::spawn(async move {
            tokio::time::sleep(tokio_duration).await;
            
            // 執行任務
            tracing::info!("會話排程觸發執行: {}", scheduled_time);
            
            // 如果需要自動重新排程
            if auto_reschedule && daily_repeat {
                // TODO: 重新排程到明天同一時間
                tracing::info!("自動重新排程到明天同一時間");
            }
        });
        
        let session = ScheduledSession {
            id: session_id,
            config,
            next_execution,
            handle: Some(handle),
            status: JobStatus::Scheduled,
        };
        
        self.sessions.insert(session_id, session);
        Ok(session_id)
    }
    
    async fn cancel(&mut self, handle: Self::Handle) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(&handle) {
            if let Some(task_handle) = session.handle.take() {
                task_handle.abort();
                session.status = JobStatus::Cancelled;
            }
        }
        Ok(())
    }
    
    async fn reschedule(&mut self, handle: Self::Handle, config: Self::Config) -> Result<Self::Handle> {
        self.cancel(handle).await?;
        self.schedule(config).await
    }
    
    fn is_running(&self, handle: &Self::Handle) -> bool {
        self.sessions.get(handle)
            .map(|session| matches!(session.status, JobStatus::Running))
            .unwrap_or(false)
    }
    
    async fn list_active(&self) -> Vec<Self::Handle> {
        self.sessions.iter()
            .filter(|(_, session)| matches!(session.status, JobStatus::Scheduled | JobStatus::Running))
            .map(|(id, _)| *id)
            .collect()
    }
}