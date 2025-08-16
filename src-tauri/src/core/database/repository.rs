// Repository 模式基类和具体实现
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::sync::Arc;

use crate::core::database::{
    ConnectionManager, DatabaseError, DatabaseResult, Entity, EntityId, ExecutionResult, Job,
    JobPriority, JobStatus, PagedResult, Prompt, QueryOptions, ResultStatus, ScheduleType,
    Timestamped, TokenUsage,
};

/// 通用 Repository trait
#[async_trait]
pub trait Repository<T: Entity + Send + Sync>: Send + Sync {
    /// 创建新实体
    async fn create(&self, entity: &mut T) -> DatabaseResult<EntityId>;

    /// 根据 ID 查找实体
    async fn find_by_id(&self, id: EntityId) -> DatabaseResult<Option<T>>;

    /// 更新实体
    async fn update(&self, entity: &mut T) -> DatabaseResult<bool>;

    /// 删除实体
    async fn delete(&self, id: EntityId) -> DatabaseResult<bool>;

    /// 列出所有实体
    async fn list(&self, options: Option<QueryOptions>) -> DatabaseResult<PagedResult<T>>;

    /// 计数
    async fn count(&self) -> DatabaseResult<u64>;
}

/// Repository 基类实现
pub struct BaseRepository {
    connection_manager: Arc<ConnectionManager>,
}

impl BaseRepository {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// 获取数据库连接
    pub fn get_connection(
        &self,
    ) -> DatabaseResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
        self.connection_manager.get_connection()
    }

    /// 执行事务
    pub fn execute_transaction<F, R>(&self, f: F) -> DatabaseResult<R>
    where
        F: FnOnce(&rusqlite::Transaction) -> rusqlite::Result<R>,
    {
        self.connection_manager.execute_transaction(f)
    }
}

/// Prompt Repository 实现
pub struct PromptRepository {
    base: BaseRepository,
}

impl PromptRepository {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            base: BaseRepository::new(connection_manager),
        }
    }

    /// 根据标题搜索
    pub async fn find_by_title_like(&self, pattern: &str) -> DatabaseResult<Vec<Prompt>> {
        let conn = self.base.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, tags, created_at, updated_at 
             FROM prompts 
             WHERE title LIKE ? 
             ORDER BY created_at DESC",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![format!("%{}%", pattern)], |row| {
                self.map_row_to_prompt(row)
            })
            .map_err(DatabaseError::Connection)?;

        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row.map_err(DatabaseError::Connection)?);
        }

        Ok(prompts)
    }

    /// 根据标签搜索
    pub async fn find_by_tags(&self, tags: &[&str]) -> DatabaseResult<Vec<Prompt>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.base.get_connection()?;
        let placeholders = tags
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(" OR tags LIKE ");
        let sql = format!(
            "SELECT id, title, content, tags, created_at, updated_at 
             FROM prompts 
             WHERE tags LIKE {} 
             ORDER BY created_at DESC",
            placeholders
        );

        let mut stmt = conn.prepare(&sql).map_err(DatabaseError::Connection)?;
        let tag_params: Vec<String> = tags.iter().map(|tag| format!("%{}%", tag)).collect();
        let params: Vec<&dyn rusqlite::ToSql> = tag_params
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let rows = stmt
            .query_map(params.as_slice(), |row| self.map_row_to_prompt(row))
            .map_err(DatabaseError::Connection)?;

        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row.map_err(DatabaseError::Connection)?);
        }

        Ok(prompts)
    }

    fn map_row_to_prompt(&self, row: &Row) -> rusqlite::Result<Prompt> {
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: Option<String> = row.get("updated_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "created_at".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?
            .with_timezone(&Utc);

        let updated_at = updated_at_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        Ok(Prompt {
            id: row.get("id")?,
            title: row.get("title")?,
            content: row.get("content")?,
            tags: row.get("tags")?,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl Repository<Prompt> for PromptRepository {
    async fn create(&self, prompt: &mut Prompt) -> DatabaseResult<EntityId> {
        prompt
            .validate()
            .map_err(DatabaseError::validation)?;

        let now = Utc::now();
        prompt.created_at = now;

        let conn = self.base.get_connection()?;
        let id = conn.query_row(
            "INSERT INTO prompts (title, content, tags, created_at) VALUES (?, ?, ?, ?) RETURNING id",
            params![
                prompt.title,
                prompt.content,
                prompt.tags,
                prompt.created_at.to_rfc3339()
            ],
            |row| row.get::<_, i64>(0),
        ).map_err(DatabaseError::Connection)?;

        prompt.set_id(id);
        Ok(id)
    }

    async fn find_by_id(&self, id: EntityId) -> DatabaseResult<Option<Prompt>> {
        let conn = self.base.get_connection()?;
        let result = conn.query_row(
            "SELECT id, title, content, tags, created_at, updated_at FROM prompts WHERE id = ?",
            params![id],
            |row| self.map_row_to_prompt(row),
        );

        match result {
            Ok(prompt) => Ok(Some(prompt)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::Connection(e)),
        }
    }

    async fn update(&self, prompt: &mut Prompt) -> DatabaseResult<bool> {
        prompt
            .validate()
            .map_err(DatabaseError::validation)?;

        let id = prompt
            .id()
            .ok_or_else(|| DatabaseError::validation("无法更新没有ID的实体"))?;

        let now = Utc::now();
        prompt.set_updated_at(now);

        let conn = self.base.get_connection()?;
        let rows_affected = conn
            .execute(
                "UPDATE prompts SET title = ?, content = ?, tags = ?, updated_at = ? WHERE id = ?",
                params![
                    prompt.title,
                    prompt.content,
                    prompt.tags,
                    prompt.updated_at.map(|dt| dt.to_rfc3339()),
                    id
                ],
            )
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    async fn delete(&self, id: EntityId) -> DatabaseResult<bool> {
        let conn = self.base.get_connection()?;
        let rows_affected = conn
            .execute("DELETE FROM prompts WHERE id = ?", params![id])
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    async fn list(&self, options: Option<QueryOptions>) -> DatabaseResult<PagedResult<Prompt>> {
        let opts = options.unwrap_or_default();
        let limit = opts.limit.unwrap_or(50);
        let offset = opts.offset.unwrap_or(0);

        let conn = self.base.get_connection()?;

        // 获取总数
        let total_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM prompts", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(DatabaseError::Connection)? as u64;

        // 获取数据
        let mut stmt = conn
            .prepare(
                "SELECT id, title, content, tags, created_at, updated_at 
             FROM prompts 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![limit, offset], |row| {
                self.map_row_to_prompt(row)
            })
            .map_err(DatabaseError::Connection)?;

        let mut prompts = Vec::new();
        for row in rows {
            prompts.push(row.map_err(DatabaseError::Connection)?);
        }

        let current_page = (offset / limit) + 1;
        Ok(PagedResult::new(prompts, total_count, limit, current_page))
    }

    async fn count(&self) -> DatabaseResult<u64> {
        let conn = self.base.get_connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
            .map_err(DatabaseError::Connection)?;

        Ok(count as u64)
    }
}

/// Job Repository 实现
pub struct JobRepository {
    base: BaseRepository,
}

impl JobRepository {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            base: BaseRepository::new(connection_manager),
        }
    }

    /// 获取待执行的任务
    pub async fn find_pending_jobs(&self) -> DatabaseResult<Vec<Job>> {
        let conn = self.base.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, prompt_id, name, schedule_type, schedule_config, status, priority, 
                    retry_count, max_retries, created_at, updated_at, last_run_at, next_run_at 
             FROM jobs 
             WHERE status = 'pending' AND (next_run_at IS NULL OR next_run_at <= datetime('now'))
             ORDER BY priority DESC, created_at ASC",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map([], |row| self.map_row_to_job(row))
            .map_err(DatabaseError::Connection)?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(DatabaseError::Connection)?);
        }

        Ok(jobs)
    }

    /// 根据状态查找任务
    pub async fn find_by_status(&self, status: JobStatus) -> DatabaseResult<Vec<Job>> {
        let conn = self.base.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, prompt_id, name, schedule_type, schedule_config, status, priority, 
                    retry_count, max_retries, created_at, updated_at, last_run_at, next_run_at 
             FROM jobs 
             WHERE status = ? 
             ORDER BY created_at DESC",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![status.to_string()], |row| {
                self.map_row_to_job(row)
            })
            .map_err(DatabaseError::Connection)?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(DatabaseError::Connection)?);
        }

        Ok(jobs)
    }

    /// 更新任务状态
    pub async fn update_status(
        &self,
        id: EntityId,
        status: JobStatus,
        next_run_at: Option<DateTime<Utc>>,
    ) -> DatabaseResult<bool> {
        let conn = self.base.get_connection()?;
        let now = Utc::now().to_rfc3339();

        let rows_affected = conn
            .execute(
                "UPDATE jobs SET status = ?, updated_at = ?, next_run_at = ?, 
             last_run_at = CASE WHEN ? = 'running' THEN ? ELSE last_run_at END 
             WHERE id = ?",
                params![
                    status.to_string(),
                    now,
                    next_run_at.map(|dt| dt.to_rfc3339()),
                    status.to_string(),
                    if status == JobStatus::Running {
                        Some(&now)
                    } else {
                        None
                    },
                    id
                ],
            )
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    fn map_row_to_job(&self, row: &Row) -> rusqlite::Result<Job> {
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: Option<String> = row.get("updated_at")?;
        let last_run_at_str: Option<String> = row.get("last_run_at")?;
        let next_run_at_str: Option<String> = row.get("next_run_at")?;

        let parse_datetime = |s: &str| -> Result<DateTime<Utc>, rusqlite::Error> {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        s.to_string(),
                        rusqlite::types::Type::Text,
                    )
                })
        };

        let created_at = parse_datetime(&created_at_str)?;
        let updated_at = updated_at_str
            .as_ref()
            .map(|s| parse_datetime(s))
            .transpose()?;
        let last_run_at = last_run_at_str
            .as_ref()
            .map(|s| parse_datetime(s))
            .transpose()?;
        let next_run_at = next_run_at_str
            .as_ref()
            .map(|s| parse_datetime(s))
            .transpose()?;

        // 解析枚举值
        let schedule_type_str: String = row.get("schedule_type")?;
        let schedule_type = match schedule_type_str.as_str() {
            "once" => ScheduleType::Once,
            "cron" => ScheduleType::Cron,
            "interval" => ScheduleType::Interval,
            "adaptive" => ScheduleType::Adaptive,
            _ => ScheduleType::Once,
        };

        let status_str: String = row.get("status")?;
        let status = match status_str.as_str() {
            "pending" => JobStatus::Pending,
            "running" => JobStatus::Running,
            "completed" => JobStatus::Completed,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            "suspended" => JobStatus::Suspended,
            _ => JobStatus::Pending,
        };

        let priority_int: i32 = row.get("priority")?;
        let priority = match priority_int {
            1 => JobPriority::Low,
            2 => JobPriority::Normal,
            3 => JobPriority::High,
            4 => JobPriority::Critical,
            _ => JobPriority::Normal,
        };

        Ok(Job {
            id: row.get("id")?,
            prompt_id: row.get("prompt_id")?,
            name: row.get("name")?,
            schedule_type,
            schedule_config: row.get("schedule_config")?,
            status,
            priority,
            retry_count: row.get("retry_count")?,
            max_retries: row.get("max_retries")?,
            created_at,
            updated_at,
            last_run_at,
            next_run_at,
        })
    }
}

#[async_trait]
impl Repository<Job> for JobRepository {
    async fn create(&self, job: &mut Job) -> DatabaseResult<EntityId> {
        job.validate()
            .map_err(DatabaseError::validation)?;

        let now = Utc::now();
        job.created_at = now;

        let conn = self.base.get_connection()?;
        let id = conn.query_row(
            "INSERT INTO jobs (prompt_id, name, schedule_type, schedule_config, status, priority, 
                               retry_count, max_retries, created_at, next_run_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
            params![
                job.prompt_id,
                job.name,
                job.schedule_type.to_string(),
                job.schedule_config,
                job.status.to_string(),
                job.priority as i32,
                job.retry_count,
                job.max_retries,
                job.created_at.to_rfc3339(),
                job.next_run_at.map(|dt| dt.to_rfc3339())
            ],
            |row| row.get::<_, i64>(0),
        ).map_err(DatabaseError::Connection)?;

        job.set_id(id);
        Ok(id)
    }

    async fn find_by_id(&self, id: EntityId) -> DatabaseResult<Option<Job>> {
        let conn = self.base.get_connection()?;
        let result = conn.query_row(
            "SELECT id, prompt_id, name, schedule_type, schedule_config, status, priority, 
                    retry_count, max_retries, created_at, updated_at, last_run_at, next_run_at 
             FROM jobs WHERE id = ?",
            params![id],
            |row| self.map_row_to_job(row),
        );

        match result {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::Connection(e)),
        }
    }

    async fn update(&self, job: &mut Job) -> DatabaseResult<bool> {
        job.validate()
            .map_err(DatabaseError::validation)?;

        let id = job
            .id()
            .ok_or_else(|| DatabaseError::validation("无法更新没有ID的实体"))?;

        let now = Utc::now();
        job.set_updated_at(now);

        let conn = self.base.get_connection()?;
        let rows_affected = conn
            .execute(
                "UPDATE jobs SET prompt_id = ?, name = ?, schedule_type = ?, schedule_config = ?, 
                            status = ?, priority = ?, retry_count = ?, max_retries = ?, 
                            updated_at = ?, last_run_at = ?, next_run_at = ? 
             WHERE id = ?",
                params![
                    job.prompt_id,
                    job.name,
                    job.schedule_type.to_string(),
                    job.schedule_config,
                    job.status.to_string(),
                    job.priority as i32,
                    job.retry_count,
                    job.max_retries,
                    job.updated_at.map(|dt| dt.to_rfc3339()),
                    job.last_run_at.map(|dt| dt.to_rfc3339()),
                    job.next_run_at.map(|dt| dt.to_rfc3339()),
                    id
                ],
            )
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    async fn delete(&self, id: EntityId) -> DatabaseResult<bool> {
        let conn = self.base.get_connection()?;
        let rows_affected = conn
            .execute("DELETE FROM jobs WHERE id = ?", params![id])
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    async fn list(&self, options: Option<QueryOptions>) -> DatabaseResult<PagedResult<Job>> {
        let opts = options.unwrap_or_default();
        let limit = opts.limit.unwrap_or(50);
        let offset = opts.offset.unwrap_or(0);

        let conn = self.base.get_connection()?;

        // 获取总数
        let total_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get::<_, i64>(0))
            .map_err(DatabaseError::Connection)? as u64;

        // 获取数据
        let mut stmt = conn
            .prepare(
                "SELECT id, prompt_id, name, schedule_type, schedule_config, status, priority, 
                    retry_count, max_retries, created_at, updated_at, last_run_at, next_run_at 
             FROM jobs 
             ORDER BY priority DESC, created_at DESC 
             LIMIT ? OFFSET ?",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![limit, offset], |row| self.map_row_to_job(row))
            .map_err(DatabaseError::Connection)?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row.map_err(DatabaseError::Connection)?);
        }

        let current_page = (offset / limit) + 1;
        Ok(PagedResult::new(jobs, total_count, limit, current_page))
    }

    async fn count(&self) -> DatabaseResult<u64> {
        let conn = self.base.get_connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
            .map_err(DatabaseError::Connection)?;

        Ok(count as u64)
    }
}

/// Usage Repository 实现（简化版，用于向后兼容）
pub struct UsageRepository {
    base: BaseRepository,
}

impl UsageRepository {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            base: BaseRepository::new(connection_manager),
        }
    }

    /// 记录执行结果
    pub async fn record_execution(
        &self,
        _job_id: EntityId,
        result: &ExecutionResult,
    ) -> DatabaseResult<EntityId> {
        let mut exec_result = result.clone();
        self.create(&mut exec_result).await
    }

    /// 获取任务的执行结果
    pub async fn get_job_results(
        &self,
        job_id: EntityId,
        limit: Option<u32>,
    ) -> DatabaseResult<Vec<ExecutionResult>> {
        let limit = limit.unwrap_or(50);

        let conn = self.base.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, status, content, error_message, input_tokens, output_tokens, 
                    total_tokens, cost_usd, execution_time_ms, created_at 
             FROM execution_results 
             WHERE job_id = ? 
             ORDER BY created_at DESC 
             LIMIT ?",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![job_id, limit], |row| {
                self.map_row_to_execution_result(row)
            })
            .map_err(DatabaseError::Connection)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(DatabaseError::Connection)?);
        }

        Ok(results)
    }

    fn map_row_to_execution_result(&self, row: &Row) -> rusqlite::Result<ExecutionResult> {
        let created_at_str: String = row.get("created_at")?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| {
                rusqlite::Error::InvalidColumnType(
                    10,
                    "created_at".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?
            .with_timezone(&Utc);

        let status_str: String = row.get("status")?;
        let status = match status_str.as_str() {
            "success" => ResultStatus::Success,
            "failed" => ResultStatus::Failed,
            "timeout" => ResultStatus::Timeout,
            "cancelled" => ResultStatus::Cancelled,
            "partial" => ResultStatus::Partial,
            _ => ResultStatus::Success,
        };

        let token_usage = if row.get::<_, Option<u64>>("input_tokens")?.is_some() {
            Some(TokenUsage {
                input_tokens: row.get::<_, Option<u64>>("input_tokens")?.unwrap_or(0),
                output_tokens: row.get::<_, Option<u64>>("output_tokens")?.unwrap_or(0),
                total_tokens: row.get::<_, Option<u64>>("total_tokens")?.unwrap_or(0),
                cost_usd: row.get::<_, Option<f64>>("cost_usd")?,
            })
        } else {
            None
        };

        Ok(ExecutionResult {
            id: row.get("id")?,
            job_id: row.get("job_id")?,
            status,
            content: row.get("content")?,
            error_message: row.get("error_message")?,
            token_usage,
            execution_time_ms: row.get("execution_time_ms")?,
            created_at,
        })
    }
}

#[async_trait]
impl Repository<ExecutionResult> for UsageRepository {
    async fn create(&self, result: &mut ExecutionResult) -> DatabaseResult<EntityId> {
        result
            .validate()
            .map_err(DatabaseError::validation)?;

        let now = Utc::now();
        result.created_at = now;

        let conn = self.base.get_connection()?;
        let id = conn.query_row(
            "INSERT INTO execution_results (job_id, status, content, error_message, input_tokens, 
                                          output_tokens, total_tokens, cost_usd, execution_time_ms, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
            params![
                result.job_id,
                result.status.to_string(),
                result.content,
                result.error_message,
                result.token_usage.as_ref().map(|t| t.input_tokens),
                result.token_usage.as_ref().map(|t| t.output_tokens),
                result.token_usage.as_ref().map(|t| t.total_tokens),
                result.token_usage.as_ref().and_then(|t| t.cost_usd),
                result.execution_time_ms,
                result.created_at.to_rfc3339()
            ],
            |row| row.get::<_, i64>(0),
        ).map_err(DatabaseError::Connection)?;

        result.set_id(id);
        Ok(id)
    }

    async fn find_by_id(&self, id: EntityId) -> DatabaseResult<Option<ExecutionResult>> {
        let conn = self.base.get_connection()?;
        let result = conn.query_row(
            "SELECT id, job_id, status, content, error_message, input_tokens, output_tokens, 
                    total_tokens, cost_usd, execution_time_ms, created_at 
             FROM execution_results WHERE id = ?",
            params![id],
            |row| self.map_row_to_execution_result(row),
        );

        match result {
            Ok(exec_result) => Ok(Some(exec_result)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DatabaseError::Connection(e)),
        }
    }

    async fn update(&self, _result: &mut ExecutionResult) -> DatabaseResult<bool> {
        // 执行结果通常不允许更新
        Err(DatabaseError::validation("执行结果不支持更新操作"))
    }

    async fn delete(&self, id: EntityId) -> DatabaseResult<bool> {
        let conn = self.base.get_connection()?;
        let rows_affected = conn
            .execute("DELETE FROM execution_results WHERE id = ?", params![id])
            .map_err(DatabaseError::Connection)?;

        Ok(rows_affected > 0)
    }

    async fn list(
        &self,
        options: Option<QueryOptions>,
    ) -> DatabaseResult<PagedResult<ExecutionResult>> {
        let opts = options.unwrap_or_default();
        let limit = opts.limit.unwrap_or(50);
        let offset = opts.offset.unwrap_or(0);

        let conn = self.base.get_connection()?;

        // 获取总数
        let total_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM execution_results", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(DatabaseError::Connection)? as u64;

        // 获取数据
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, status, content, error_message, input_tokens, output_tokens, 
                    total_tokens, cost_usd, execution_time_ms, created_at 
             FROM execution_results 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?",
            )
            .map_err(DatabaseError::Connection)?;

        let rows = stmt
            .query_map(params![limit, offset], |row| {
                self.map_row_to_execution_result(row)
            })
            .map_err(DatabaseError::Connection)?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(DatabaseError::Connection)?);
        }

        let current_page = (offset / limit) + 1;
        Ok(PagedResult::new(results, total_count, limit, current_page))
    }

    async fn count(&self) -> DatabaseResult<u64> {
        let conn = self.base.get_connection()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM execution_results", [], |row| {
                row.get(0)
            })
            .map_err(DatabaseError::Connection)?;

        Ok(count as u64)
    }
}
