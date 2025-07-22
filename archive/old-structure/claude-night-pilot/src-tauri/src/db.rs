use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prompt {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Job {
    pub id: Option<i64>,
    pub prompt_id: i64,
    pub cron_expr: String,
    pub mode: String,   // 'sync' | 'async'
    pub status: String, // 'pending' | 'running' | 'done' | 'error'
    pub eta_unix: Option<i64>,
    pub last_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JobResult {
    pub id: Option<i64>,
    pub job_id: i64,
    pub content: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPromptRequest {
    pub prompt_id: i64,
    pub mode: String, // 'sync' | 'async'
    pub cron_expr: Option<String>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // 執行初始化 SQL（手動建立表格）
        Self::create_tables(&pool).await?;

        Ok(Database { pool })
    }

    async fn create_tables(pool: &SqlitePool) -> Result<()> {
        // 創建 prompts 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 創建 jobs 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prompt_id INTEGER NOT NULL,
                cron_expr TEXT NOT NULL,
                mode TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                eta_unix INTEGER,
                last_run_at DATETIME,
                FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 創建 results 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // Prompt CRUD 操作
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>> {
        let prompts = sqlx::query_as::<_, Prompt>(
            "SELECT id, title, content, tags, created_at FROM prompts ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(prompts)
    }

    pub async fn create_prompt(&self, req: CreatePromptRequest) -> Result<i64> {
        let result = sqlx::query!(
            "INSERT INTO prompts (title, content, tags) VALUES (?, ?, ?)",
            req.title,
            req.content,
            req.tags
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_prompt(&self, id: i64) -> Result<Option<Prompt>> {
        let prompt = sqlx::query_as::<_, Prompt>(
            "SELECT id, title, content, tags, created_at FROM prompts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(prompt)
    }

    pub async fn delete_prompt(&self, id: i64) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM prompts WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Job 操作
    pub async fn create_job(&self, req: RunPromptRequest) -> Result<i64> {
        let cron_expr = req.cron_expr.unwrap_or_else(|| "*".to_string());
        let result = sqlx::query!(
            "INSERT INTO jobs (prompt_id, cron_expr, mode, status) VALUES (?, ?, ?, 'pending')",
            req.prompt_id,
            cron_expr,
            req.mode
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn list_jobs(&self) -> Result<Vec<Job>> {
        let jobs = sqlx::query_as::<_, Job>(
            "SELECT id, prompt_id, cron_expr, mode, status, eta_unix, last_run_at FROM jobs ORDER BY id DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    pub async fn update_job_status(
        &self,
        job_id: i64,
        status: &str,
        eta_unix: Option<i64>,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE jobs SET status = ?, eta_unix = ?, last_run_at = CURRENT_TIMESTAMP WHERE id = ?",
            status,
            eta_unix,
            job_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_jobs(&self) -> Result<Vec<Job>> {
        let jobs = sqlx::query_as::<_, Job>(
            "SELECT id, prompt_id, cron_expr, mode, status, eta_unix, last_run_at FROM jobs WHERE status = 'pending'"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    // Results 操作
    pub async fn create_result(&self, job_id: i64, content: &str) -> Result<i64> {
        let result = sqlx::query!(
            "INSERT INTO results (job_id, content) VALUES (?, ?)",
            job_id,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn list_results(&self, job_id: Option<i64>) -> Result<Vec<JobResult>> {
        let results = if let Some(job_id) = job_id {
            sqlx::query_as::<_, JobResult>(
                "SELECT id, job_id, content, created_at FROM results WHERE job_id = ? ORDER BY created_at DESC"
            )
            .bind(job_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, JobResult>(
                "SELECT id, job_id, content, created_at FROM results ORDER BY created_at DESC LIMIT 50"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(results)
    }
}
