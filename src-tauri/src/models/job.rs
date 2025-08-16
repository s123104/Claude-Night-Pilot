// 工作任務模型 - 參考 vibe-kanban 設計

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// 排程工作任務
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Job {
    /// 工作 ID
    pub id: String,

    /// 工作名稱
    pub name: String,

    /// 關聯的提示 ID
    pub prompt_id: String,

    /// Cron 表達式
    pub cron_expression: String,

    /// 工作狀態
    pub status: JobStatus,

    /// 工作類型
    pub job_type: JobType,

    /// 優先級 (1-10, 10 最高)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// 執行選項
    #[serde(default)]
    pub execution_options: JobExecutionOptions,

    /// 重試配置
    #[serde(default)]
    pub retry_config: RetryConfig,

    /// 通知配置
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub notification_config: Option<NotificationConfig>,

    /// 下次執行時間
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "string | null")]
    pub next_run_time: Option<DateTime<Utc>>,

    /// 上次執行時間
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "string | null")]
    pub last_run_time: Option<DateTime<Utc>>,

    /// 執行次數
    #[serde(default)]
    pub execution_count: u64,

    /// 失敗次數
    #[serde(default)]
    pub failure_count: u64,

    /// 標籤
    #[serde(default)]
    pub tags: Vec<String>,

    /// 元數據
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// 創建時間
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    /// 更新時間
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,

    /// 創建者
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub created_by: Option<String>,
}

/// 工作狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum JobStatus {
    /// 已啟用，等待執行
    Active,
    /// 已暫停
    Paused,
    /// 執行中
    Running,
    /// 已完成 (一次性任務)
    Completed,
    /// 已取消
    Cancelled,
    /// 失敗
    Failed,
    /// 冷卻中 (因為 API 限制)
    Cooldown,
}

/// 工作類型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum JobType {
    /// 定時執行
    Scheduled,
    /// 一次性執行
    OneTime,
    /// 間隔執行
    Interval,
    /// 觸發執行
    Triggered,
}

/// 工作執行選項
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobExecutionOptions {
    /// 超時時間 (秒)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// 並行執行限制
    #[serde(default = "default_max_parallel")]
    pub max_parallel_executions: u32,

    /// 跳過如果前一個還在執行
    #[serde(default)]
    pub skip_if_running: bool,

    /// 工作目錄
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_directory: Option<String>,

    /// 環境變數
    #[serde(default)]
    pub environment_variables: HashMap<String, String>,

    /// 資源限制
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub resource_limits: Option<ResourceLimits>,
}

/// 重試配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RetryConfig {
    /// 最大重試次數
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// 重試間隔 (秒)
    #[serde(default = "default_retry_interval")]
    pub retry_interval_seconds: u64,

    /// 重試策略
    #[serde(default)]
    pub retry_strategy: RetryStrategy,

    /// 退避倍數 (指數退避使用)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// 最大退避時間 (秒)
    #[serde(default = "default_max_backoff")]
    pub max_backoff_seconds: u64,
}

/// 重試策略
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub enum RetryStrategy {
    /// 固定間隔
    #[default]
    Fixed,
    /// 指數退避
    ExponentialBackoff,
    /// 線性增長
    Linear,
    /// 自定義間隔
    Custom(Vec<u64>),
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotificationConfig {
    /// 啟用通知
    #[serde(default)]
    pub enabled: bool,

    /// 成功時通知
    #[serde(default)]
    pub notify_on_success: bool,

    /// 失敗時通知
    #[serde(default = "default_true")]
    pub notify_on_failure: bool,

    /// 開始時通知
    #[serde(default)]
    pub notify_on_start: bool,

    /// 通知方式
    #[serde(default)]
    pub notification_channels: Vec<NotificationChannel>,

    /// 自定義訊息模板
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub custom_message_template: Option<String>,
}

/// 通知管道
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum NotificationChannel {
    /// 系統通知
    System,
    /// 郵件
    Email(String),
    /// Webhook
    Webhook(String),
    /// 日誌
    Log,
}

/// 資源限制
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ResourceLimits {
    /// 最大記憶體 (MB)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_memory_mb: Option<u64>,

    /// 最大 CPU 使用率 (%)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_cpu_percent: Option<f64>,

    /// 最大磁碟使用 (MB)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_disk_mb: Option<u64>,
}

impl Job {
    /// 創建新工作
    pub fn new(
        name: impl Into<String>,
        prompt_id: impl Into<String>,
        cron_expression: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            prompt_id: prompt_id.into(),
            cron_expression: cron_expression.into(),
            status: JobStatus::Active,
            job_type: JobType::Scheduled,
            priority: default_priority(),
            execution_options: JobExecutionOptions::default(),
            retry_config: RetryConfig::default(),
            notification_config: None,
            next_run_time: None,
            last_run_time: None,
            execution_count: 0,
            failure_count: 0,
            tags: vec![],
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by: None,
        }
    }

    /// 創建一次性工作
    pub fn one_time(
        name: impl Into<String>,
        prompt_id: impl Into<String>,
        run_time: DateTime<Utc>,
    ) -> Self {
        let mut job = Self::new(name, prompt_id, "");
        job.job_type = JobType::OneTime;
        job.next_run_time = Some(run_time);
        job
    }

    /// 設置優先級
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self.updated_at = Utc::now();
        self
    }

    /// 設置執行選項
    pub fn with_execution_options(mut self, options: JobExecutionOptions) -> Self {
        self.execution_options = options;
        self.updated_at = Utc::now();
        self
    }

    /// 設置重試配置
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self.updated_at = Utc::now();
        self
    }

    /// 設置通知配置
    pub fn with_notification_config(mut self, config: NotificationConfig) -> Self {
        self.notification_config = Some(config);
        self.updated_at = Utc::now();
        self
    }

    /// 添加標籤
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 添加元數據
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }

    /// 更新狀態
    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// 記錄執行開始
    pub fn start_execution(&mut self) {
        self.status = JobStatus::Running;
        self.last_run_time = Some(Utc::now());
        self.execution_count += 1;
        self.updated_at = Utc::now();
    }

    /// 記錄執行完成
    pub fn complete_execution(&mut self, success: bool) {
        if success {
            match self.job_type {
                JobType::OneTime => self.status = JobStatus::Completed,
                _ => self.status = JobStatus::Active,
            }
        } else {
            self.failure_count += 1;
            if self.failure_count >= self.retry_config.max_retries as u64 {
                self.status = JobStatus::Failed;
            } else {
                self.status = JobStatus::Active;
            }
        }
        self.updated_at = Utc::now();
    }

    /// 暫停工作
    pub fn pause(&mut self) {
        if self.status == JobStatus::Active {
            self.status = JobStatus::Paused;
            self.updated_at = Utc::now();
        }
    }

    /// 恢復工作
    pub fn resume(&mut self) {
        if self.status == JobStatus::Paused {
            self.status = JobStatus::Active;
            self.updated_at = Utc::now();
        }
    }

    /// 取消工作
    pub fn cancel(&mut self) {
        if !matches!(self.status, JobStatus::Completed | JobStatus::Cancelled) {
            self.status = JobStatus::Cancelled;
            self.updated_at = Utc::now();
        }
    }

    /// 檢查是否可以執行
    pub fn can_execute(&self) -> bool {
        matches!(self.status, JobStatus::Active)
            && self.next_run_time.is_some_and(|time| time <= Utc::now())
    }

    /// 檢查是否正在執行
    pub fn is_running(&self) -> bool {
        self.status == JobStatus::Running
    }

    /// 計算成功率
    pub fn success_rate(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            (self.execution_count - self.failure_count) as f64 / self.execution_count as f64
        }
    }

    /// 獲取下次重試時間
    pub fn next_retry_time(&self) -> Option<DateTime<Utc>> {
        if self.failure_count == 0 || self.failure_count >= self.retry_config.max_retries as u64 {
            return None;
        }

        let base_interval = self.retry_config.retry_interval_seconds;
        let wait_seconds = match self.retry_config.retry_strategy {
            RetryStrategy::Fixed => base_interval,
            RetryStrategy::ExponentialBackoff => {
                let backoff = (base_interval as f64
                    * self
                        .retry_config
                        .backoff_multiplier
                        .powi(self.failure_count as i32)) as u64;
                backoff.min(self.retry_config.max_backoff_seconds)
            }
            RetryStrategy::Linear => base_interval * (self.failure_count + 1),
            RetryStrategy::Custom(ref intervals) => intervals
                .get(self.failure_count as usize - 1)
                .copied()
                .unwrap_or(base_interval),
        };

        Some(Utc::now() + chrono::Duration::seconds(wait_seconds as i64))
    }
}

impl Default for JobExecutionOptions {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            max_parallel_executions: default_max_parallel(),
            skip_if_running: false,
            working_directory: None,
            environment_variables: HashMap::new(),
            resource_limits: None,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            retry_interval_seconds: default_retry_interval(),
            retry_strategy: RetryStrategy::default(),
            backoff_multiplier: default_backoff_multiplier(),
            max_backoff_seconds: default_max_backoff(),
        }
    }
}

// 默認值函數
fn default_priority() -> u8 {
    5
}
fn default_timeout() -> u64 {
    300
} // 5分鐘
fn default_max_parallel() -> u32 {
    1
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_interval() -> u64 {
    60
} // 1分鐘
fn default_backoff_multiplier() -> f64 {
    2.0
}
fn default_max_backoff() -> u64 {
    3600
} // 1小時
fn default_true() -> bool {
    true
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Active => write!(f, "啟用"),
            JobStatus::Paused => write!(f, "暫停"),
            JobStatus::Running => write!(f, "執行中"),
            JobStatus::Completed => write!(f, "已完成"),
            JobStatus::Cancelled => write!(f, "已取消"),
            JobStatus::Failed => write!(f, "失敗"),
            JobStatus::Cooldown => write!(f, "冷卻中"),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::Scheduled => write!(f, "定時"),
            JobType::OneTime => write!(f, "一次性"),
            JobType::Interval => write!(f, "間隔"),
            JobType::Triggered => write!(f, "觸發"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new("測試工作", "prompt123", "0 0 * * *");

        assert!(!job.id.is_empty());
        assert_eq!(job.name, "測試工作");
        assert_eq!(job.prompt_id, "prompt123");
        assert_eq!(job.cron_expression, "0 0 * * *");
        assert_eq!(job.status, JobStatus::Active);
    }

    #[test]
    fn test_one_time_job() {
        let run_time = Utc::now() + chrono::Duration::hours(1);
        let job = Job::one_time("一次性任務", "prompt456", run_time);

        assert!(matches!(job.job_type, JobType::OneTime));
        assert_eq!(job.next_run_time, Some(run_time));
    }

    #[test]
    fn test_job_status_transitions() {
        let mut job = Job::new("狀態測試", "prompt789", "0 * * * *");

        // 開始執行
        job.start_execution();
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.execution_count, 1);

        // 完成執行
        job.complete_execution(true);
        assert_eq!(job.status, JobStatus::Active);

        // 失敗執行
        job.start_execution();
        job.complete_execution(false);
        assert_eq!(job.failure_count, 1);
        assert_eq!(job.status, JobStatus::Active); // 還可以重試
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut job = Job::new("成功率測試", "prompt101", "0 * * * *");

        // 執行 3 次，2 次成功
        job.start_execution();
        job.complete_execution(true);

        job.start_execution();
        job.complete_execution(false);

        job.start_execution();
        job.complete_execution(true);

        assert_eq!(job.execution_count, 3);
        assert_eq!(job.failure_count, 1);
        assert!((job.success_rate() - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_time_calculation() {
        let mut job = Job::new("重試測試", "prompt202", "0 * * * *");
        job.retry_config.retry_strategy = RetryStrategy::ExponentialBackoff;
        job.retry_config.retry_interval_seconds = 10;
        job.retry_config.backoff_multiplier = 2.0;

        // 第一次失敗
        job.failure_count = 1;
        let retry_time = job.next_retry_time();
        assert!(retry_time.is_some());

        // 超過最大重試次數
        job.failure_count = 5;
        job.retry_config.max_retries = 3;
        assert!(job.next_retry_time().is_none());
    }
}
