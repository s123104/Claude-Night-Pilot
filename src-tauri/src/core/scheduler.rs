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
    _id: Uuid,
    _config: CronConfig,
    job_id: Option<uuid::Uuid>,
    status: JobStatus,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are used in future implementations
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
            _id: job_id,
            _config: config,
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
    _config: AdaptiveConfig,
}

struct AdaptiveTimer {
    _id: Uuid,
    handle: Option<tokio::task::JoinHandle<()>>,
    status: JobStatus,
}

impl AdaptiveScheduler {
    pub fn new(config: AdaptiveConfig) -> Self {
        Self {
            timers: HashMap::new(),
            _config: config,
        }
    }
    
    async fn _get_adaptive_interval(&self, remaining_minutes: u64) -> std::time::Duration {
        for (threshold, interval_seconds) in &self._config.intervals {
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
            _id: timer_id,
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
    _id: Uuid,
    _config: SessionConfig,
    _next_execution: chrono::DateTime<chrono::Local>,
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
            _id: session_id,
            _config: config,
            _next_execution: next_execution,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_cron_scheduler_creation() {
        let result = CronScheduler::new().await;
        assert!(result.is_ok());
        let scheduler = result.unwrap();
        assert_eq!(scheduler.jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_cron_scheduler_lifecycle() {
        let mut scheduler = CronScheduler::new().await.unwrap();
        
        // 启动调度器
        scheduler.start().await.unwrap();
        
        // 关闭调度器
        scheduler.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_cron_scheduler_schedule_job() {
        let mut scheduler = CronScheduler::new().await.unwrap();
        scheduler.start().await.unwrap();
        
        let config = CronConfig {
            expression: "*/5 * * * * *".to_string(), // 每5秒执行
            timezone: "Asia/Shanghai".to_string(),
            max_concurrent: 1,
            timeout_seconds: 30,
        };
        
        let handle = scheduler.schedule(config).await.unwrap();
        assert!(!scheduler.is_running(&handle)); // 刚调度时不是运行状态
        
        let active_jobs = scheduler.list_active().await;
        assert_eq!(active_jobs.len(), 1);
        assert_eq!(active_jobs[0], handle);
        
        // 取消任务
        scheduler.cancel(handle).await.unwrap();
        let active_jobs_after_cancel = scheduler.list_active().await;
        assert_eq!(active_jobs_after_cancel.len(), 0);
        
        scheduler.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_cron_scheduler_reschedule() {
        let mut scheduler = CronScheduler::new().await.unwrap();
        scheduler.start().await.unwrap();
        
        let original_config = CronConfig {
            expression: "*/10 * * * * *".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            max_concurrent: 1,
            timeout_seconds: 30,
        };
        
        let original_handle = scheduler.schedule(original_config).await.unwrap();
        
        let new_config = CronConfig {
            expression: "*/5 * * * * *".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            max_concurrent: 2,
            timeout_seconds: 60,
        };
        
        let new_handle = scheduler.reschedule(original_handle, new_config).await.unwrap();
        assert_ne!(original_handle, new_handle);
        
        let active_jobs = scheduler.list_active().await;
        assert_eq!(active_jobs.len(), 1);
        assert_eq!(active_jobs[0], new_handle);
        
        scheduler.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_adaptive_scheduler_creation() {
        let config = AdaptiveConfig {
            intervals: vec![(60, 30), (30, 15), (10, 5)],
            ccusage_integration: false,
            fallback_to_time_based: true,
        };
        
        let scheduler = AdaptiveScheduler::new(config);
        assert_eq!(scheduler.timers.len(), 0);
    }

    #[tokio::test]
    async fn test_adaptive_scheduler_schedule() {
        let config = AdaptiveConfig {
            intervals: vec![(60, 30), (30, 15), (10, 5)],
            ccusage_integration: false,
            fallback_to_time_based: true,
        };
        
        let mut scheduler = AdaptiveScheduler::new(config.clone());
        let handle = scheduler.schedule(config).await.unwrap();
        
        assert!(!scheduler.is_running(&handle));
        
        let active_timers = scheduler.list_active().await;
        assert_eq!(active_timers.len(), 1);
        assert_eq!(active_timers[0], handle);
        
        // 取消任务
        scheduler.cancel(handle).await.unwrap();
        let active_after_cancel = scheduler.list_active().await;
        assert_eq!(active_after_cancel.len(), 0);
    }

    #[test]
    fn test_adaptive_scheduler_interval_calculation() {
        let intervals = vec![(60, 30), (30, 15), (10, 5)];
        
        // 测试不同剩余时间的间隔计算
        assert_eq!(AdaptiveScheduler::calculate_adaptive_interval(&intervals, 70), Duration::from_secs(30));
        assert_eq!(AdaptiveScheduler::calculate_adaptive_interval(&intervals, 45), Duration::from_secs(15));
        assert_eq!(AdaptiveScheduler::calculate_adaptive_interval(&intervals, 15), Duration::from_secs(5));
        assert_eq!(AdaptiveScheduler::calculate_adaptive_interval(&intervals, 5), Duration::from_secs(30)); // 默认值
    }

    #[tokio::test]
    async fn test_session_scheduler_creation() {
        let scheduler = SessionScheduler::new();
        assert_eq!(scheduler.sessions.len(), 0);
    }

    #[test]
    fn test_session_scheduler_time_parsing() {
        let scheduler = SessionScheduler::new();
        
        // 测试有效时间格式
        assert_eq!(scheduler.parse_time("09:30").unwrap(), (9, 30));
        assert_eq!(scheduler.parse_time("23:59").unwrap(), (23, 59));
        assert_eq!(scheduler.parse_time("00:00").unwrap(), (0, 0));
        
        // 测试无效时间格式
        assert!(scheduler.parse_time("9:30").is_ok()); // 单位数小时有效
        assert!(scheduler.parse_time("25:00").is_err()); // 小时超出范围
        assert!(scheduler.parse_time("12:60").is_err()); // 分钟超出范围
        assert!(scheduler.parse_time("12-30").is_err()); // 错误分隔符
        assert!(scheduler.parse_time("12:30:45").is_err()); // 过多组件
    }

    #[test]
    fn test_session_scheduler_next_execution_calculation() {
        let scheduler = SessionScheduler::new();
        let now = chrono::Local::now();
        
        // 测试未来时间（今天）
        let future_hour = if now.hour() < 23 { now.hour() + 1 } else { 1 };
        let next_execution = scheduler.calculate_next_execution(future_hour, 0);
        
        if now.hour() < 23 {
            // 应该是今天的未来时间
            assert_eq!(next_execution.date_naive(), now.date_naive());
        } else {
            // 如果现在是23点，下一个执行时间应该是明天
            assert_eq!(next_execution.date_naive(), (now + chrono::Duration::days(1)).date_naive());
        }
        
        // 测试过去时间（应该安排到明天）
        let past_hour = if now.hour() > 0 { now.hour() - 1 } else { 23 };
        let past_execution = scheduler.calculate_next_execution(past_hour, 0);
        
        if now.hour() > 0 {
            // 应该是明天的同一时间
            assert_eq!(past_execution.date_naive(), (now + chrono::Duration::days(1)).date_naive());
        }
        // 如果现在是0点，过去时间（23点）应该是今天
    }

    #[tokio::test]
    async fn test_session_scheduler_schedule() {
        let mut scheduler = SessionScheduler::new();
        
        // 计算一个未来时间（5秒后）
        let now = chrono::Local::now();
        let future_time = now + chrono::Duration::seconds(5);
        let time_str = format!("{}:{:02}", future_time.hour(), future_time.minute());
        
        let config = SessionConfig {
            scheduled_time: time_str,
            auto_reschedule: false,
            daily_repeat: false,
        };
        
        let handle = scheduler.schedule(config).await.unwrap();
        assert!(!scheduler.is_running(&handle));
        
        let active_sessions = scheduler.list_active().await;
        assert_eq!(active_sessions.len(), 1);
        
        // 取消会话
        scheduler.cancel(handle).await.unwrap();
        let active_after_cancel = scheduler.list_active().await;
        assert_eq!(active_after_cancel.len(), 0);
    }

    #[tokio::test]
    async fn test_session_scheduler_invalid_time() {
        let mut scheduler = SessionScheduler::new();
        
        let config = SessionConfig {
            scheduled_time: "25:00".to_string(), // 无效小时
            auto_reschedule: false,
            daily_repeat: false,
        };
        
        let result = scheduler.schedule(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_schedulers_integration() {
        // 测试多种调度器可以同时工作
        
        // Cron 调度器
        let mut cron_scheduler = CronScheduler::new().await.unwrap();
        cron_scheduler.start().await.unwrap();
        
        let cron_config = CronConfig {
            expression: "*/30 * * * * *".to_string(),
            timezone: "UTC".to_string(),
            max_concurrent: 1,
            timeout_seconds: 30,
        };
        
        let cron_handle = cron_scheduler.schedule(cron_config).await.unwrap();
        
        // 自适应调度器
        let adaptive_config = AdaptiveConfig {
            intervals: vec![(60, 30)],
            ccusage_integration: false,
            fallback_to_time_based: true,
        };
        
        let mut adaptive_scheduler = AdaptiveScheduler::new(adaptive_config.clone());
        let adaptive_handle = adaptive_scheduler.schedule(adaptive_config).await.unwrap();
        
        // 会话调度器
        let mut session_scheduler = SessionScheduler::new();
        let future_time = chrono::Local::now() + chrono::Duration::seconds(10);
        let session_config = SessionConfig {
            scheduled_time: format!("{}:{:02}", future_time.hour(), future_time.minute()),
            auto_reschedule: false,
            daily_repeat: false,
        };
        
        let session_handle = session_scheduler.schedule(session_config).await.unwrap();
        
        // 验证所有调度器都有活动任务
        assert_eq!(cron_scheduler.list_active().await.len(), 1);
        assert_eq!(adaptive_scheduler.list_active().await.len(), 1);
        assert_eq!(session_scheduler.list_active().await.len(), 1);
        
        // 清理
        cron_scheduler.cancel(cron_handle).await.unwrap();
        adaptive_scheduler.cancel(adaptive_handle).await.unwrap();
        session_scheduler.cancel(session_handle).await.unwrap();
        
        cron_scheduler.shutdown().await.unwrap();
        
        // 验证所有任务已取消
        assert_eq!(cron_scheduler.list_active().await.len(), 0);
        assert_eq!(adaptive_scheduler.list_active().await.len(), 0);
        assert_eq!(session_scheduler.list_active().await.len(), 0);
    }

    #[test]
    fn test_scheduling_config_serialization() {
        let config = SchedulingConfig {
            scheduler_type: SchedulerType::Cron,
            cron: Some(CronConfig {
                expression: "0 9 * * *".to_string(),
                timezone: "Asia/Shanghai".to_string(),
                max_concurrent: 3,
                timeout_seconds: 300,
            }),
            adaptive: None,
            session: None,
        };
        
        // 测试序列化和反序列化
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SchedulingConfig = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized.scheduler_type, SchedulerType::Cron));
        assert!(deserialized.cron.is_some());
        assert!(deserialized.adaptive.is_none());
        assert!(deserialized.session.is_none());
        
        let cron_config = deserialized.cron.unwrap();
        assert_eq!(cron_config.expression, "0 9 * * *");
        assert_eq!(cron_config.timezone, "Asia/Shanghai");
        assert_eq!(cron_config.max_concurrent, 3);
        assert_eq!(cron_config.timeout_seconds, 300);
    }

    #[test]
    fn test_job_status_variants() {
        // 测试所有 JobStatus 变体
        let scheduled = JobStatus::Scheduled;
        let running = JobStatus::Running;
        let completed = JobStatus::Completed;
        let failed = JobStatus::Failed("Test error".to_string());
        let cancelled = JobStatus::Cancelled;
        
        // 测试模式匹配
        assert!(matches!(scheduled, JobStatus::Scheduled));
        assert!(matches!(running, JobStatus::Running));
        assert!(matches!(completed, JobStatus::Completed));
        assert!(matches!(failed, JobStatus::Failed(_)));
        assert!(matches!(cancelled, JobStatus::Cancelled));
        
        // 测试失败状态的错误消息
        if let JobStatus::Failed(msg) = &failed {
            assert_eq!(msg, "Test error");
        }
    }
}