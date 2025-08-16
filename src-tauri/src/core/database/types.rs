// 统一的数据库类型定义
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 通用实体 ID 类型
pub type EntityId = i64;

/// 通用的数据库实体 trait
pub trait Entity {
    fn id(&self) -> Option<EntityId>;
    fn set_id(&mut self, id: EntityId);
    fn table_name() -> &'static str;
    fn validate(&self) -> Result<(), String>;
}

/// 时间戳 trait
pub trait Timestamped {
    fn created_at(&self) -> &DateTime<Utc>;
    fn updated_at(&self) -> Option<&DateTime<Utc>>;
    fn set_updated_at(&mut self, timestamp: DateTime<Utc>);
}

/// 统一的 Prompt 实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    pub id: Option<EntityId>,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Entity for Prompt {
    fn id(&self) -> Option<EntityId> {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = Some(id);
    }

    fn table_name() -> &'static str {
        "prompts"
    }

    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("标题不能为空".to_string());
        }
        if self.content.trim().is_empty() {
            return Err("内容不能为空".to_string());
        }
        if self.title.len() > 255 {
            return Err("标题长度不能超过255字符".to_string());
        }
        if self.content.len() > 1_000_000 {
            return Err("内容长度不能超过1MB".to_string());
        }
        Ok(())
    }
}

impl Timestamped for Prompt {
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }

    fn set_updated_at(&mut self, timestamp: DateTime<Utc>) {
        self.updated_at = Some(timestamp);
    }
}

/// 统一的 Job/Schedule 实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub id: Option<EntityId>,
    pub prompt_id: EntityId,
    pub name: Option<String>,
    pub schedule_type: ScheduleType,
    pub schedule_config: String, // JSON config for cron_expr or other schedule data
    pub status: JobStatus,
    pub priority: JobPriority,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

impl Entity for Job {
    fn id(&self) -> Option<EntityId> {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = Some(id);
    }

    fn table_name() -> &'static str {
        "jobs"
    }

    fn validate(&self) -> Result<(), String> {
        if self.max_retries > 10 {
            return Err("最大重试次数不能超过10".to_string());
        }
        if self.retry_count > self.max_retries {
            return Err("重试次数不能超过最大重试次数".to_string());
        }

        // 验证 schedule_config 是否为有效 JSON
        serde_json::from_str::<serde_json::Value>(&self.schedule_config)
            .map_err(|_| "调度配置不是有效的JSON格式".to_string())?;

        Ok(())
    }
}

impl Timestamped for Job {
    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }

    fn set_updated_at(&mut self, timestamp: DateTime<Utc>) {
        self.updated_at = Some(timestamp);
    }
}

/// 调度类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleType {
    /// 一次性执行
    Once,
    /// Cron 表达式调度
    Cron,
    /// 间隔调度
    Interval,
    /// 自适应调度
    Adaptive,
}

impl std::fmt::Display for ScheduleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Once => write!(f, "once"),
            Self::Cron => write!(f, "cron"),
            Self::Interval => write!(f, "interval"),
            Self::Adaptive => write!(f, "adaptive"),
        }
    }
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Suspended,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Suspended => write!(f, "suspended"),
        }
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// 统一的执行结果实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: Option<EntityId>,
    pub job_id: EntityId,
    pub status: ResultStatus,
    pub content: String,
    pub error_message: Option<String>,
    pub token_usage: Option<TokenUsage>,
    pub execution_time_ms: i64,
    pub created_at: DateTime<Utc>,
}

impl Entity for ExecutionResult {
    fn id(&self) -> Option<EntityId> {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = Some(id);
    }

    fn table_name() -> &'static str {
        "execution_results"
    }

    fn validate(&self) -> Result<(), String> {
        if self.execution_time_ms < 0 {
            return Err("执行时间不能为负数".to_string());
        }
        if self.content.len() > 10_000_000 {
            return Err("结果内容不能超过10MB".to_string());
        }
        Ok(())
    }
}

/// 执行结果状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResultStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
    Partial,
}

impl std::fmt::Display for ResultStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::Timeout => write!(f, "timeout"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Partial => write!(f, "partial"),
        }
    }
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
}

/// 使用统计实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub id: Option<EntityId>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_tokens: TokenUsage,
    pub average_execution_time_ms: f64,
    pub created_at: DateTime<Utc>,
}

impl Entity for UsageStats {
    fn id(&self) -> Option<EntityId> {
        self.id
    }

    fn set_id(&mut self, id: EntityId) {
        self.id = Some(id);
    }

    fn table_name() -> &'static str {
        "usage_stats"
    }

    fn validate(&self) -> Result<(), String> {
        if self.period_end <= self.period_start {
            return Err("结束时间必须晚于开始时间".to_string());
        }
        if self.successful_executions + self.failed_executions != self.total_executions {
            return Err("成功和失败执行次数之和必须等于总执行次数".to_string());
        }
        Ok(())
    }
}

/// 查询选项
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order_by: Option<String>,
    pub order_direction: OrderDirection,
    pub filters: std::collections::HashMap<String, QueryFilter>,
}

#[derive(Debug, Clone)]
pub enum QueryFilter {
    Equal(String),
    NotEqual(String),
    GreaterThan(String),
    LessThan(String),
    Like(String),
    In(Vec<String>),
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Default)]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

impl std::fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page_size: u32,
    pub current_page: u32,
    pub total_pages: u32,
}

impl<T> PagedResult<T> {
    pub fn new(items: Vec<T>, total_count: u64, page_size: u32, current_page: u32) -> Self {
        let total_pages = if total_count == 0 {
            1
        } else {
            ((total_count as f64) / (page_size as f64)).ceil() as u32
        };

        Self {
            items,
            total_count,
            page_size,
            current_page,
            total_pages,
        }
    }

    pub fn has_next_page(&self) -> bool {
        self.current_page < self.total_pages
    }

    pub fn has_previous_page(&self) -> bool {
        self.current_page > 1
    }
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub enable_foreign_keys: bool,
    pub enable_wal_mode: bool,
    pub synchronous_mode: SynchronousMode,
    pub cache_size: i32,
    pub temp_store: TempStore,
    pub journal_mode: JournalMode,
    pub connection_timeout_seconds: u64,
    pub busy_timeout_ms: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "claude-pilot.db".to_string(),
            enable_foreign_keys: true,
            enable_wal_mode: true,
            synchronous_mode: SynchronousMode::Normal,
            cache_size: 2000, // 2MB cache
            temp_store: TempStore::Memory,
            journal_mode: JournalMode::Wal,
            connection_timeout_seconds: 30,
            busy_timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SynchronousMode {
    Off = 0,
    Normal = 1,
    Full = 2,
}

impl std::fmt::Display for SynchronousMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "OFF"),
            Self::Normal => write!(f, "NORMAL"),
            Self::Full => write!(f, "FULL"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TempStore {
    Default = 0,
    File = 1,
    Memory = 2,
}

impl std::fmt::Display for TempStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "DEFAULT"),
            Self::File => write!(f, "FILE"),
            Self::Memory => write!(f, "MEMORY"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl std::fmt::Display for JournalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Delete => write!(f, "DELETE"),
            Self::Truncate => write!(f, "TRUNCATE"),
            Self::Persist => write!(f, "PERSIST"),
            Self::Memory => write!(f, "MEMORY"),
            Self::Wal => write!(f, "WAL"),
            Self::Off => write!(f, "OFF"),
        }
    }
}
