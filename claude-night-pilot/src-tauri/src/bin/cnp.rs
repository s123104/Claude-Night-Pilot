use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{sqlite::SqlitePool, Row};
use std::path::Path;
use tokio;

// CLI 主結構
#[derive(Parser)]
#[command(name = "cnp")]
#[command(about = "Claude Night Pilot - CLI 工具", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// 子命令定義
#[derive(Subcommand)]
enum Commands {
    /// 初始化資料庫
    Init,
    /// Prompt 管理
    Prompt {
        #[command(subcommand)]
        action: PromptAction,
    },
    /// 任務管理
    Job {
        #[command(subcommand)]
        action: JobAction,
    },
    /// 執行 Claude CLI 命令
    Run {
        /// Prompt ID 或內容
        #[arg(short, long)]
        prompt: String,
        /// 執行模式 (sync/async)
        #[arg(short, long, default_value = "sync")]
        mode: String,
        /// Cron 表達式 (僅用於 async 模式)
        #[arg(short, long)]
        cron: Option<String>,
    },
    /// 系統狀態檢查
    Status,
    /// 檢查 Claude CLI 冷卻狀態
    Cooldown,
    /// 列出執行結果
    Results {
        /// 任務 ID (可選)
        #[arg(short, long)]
        job_id: Option<i64>,
        /// 限制結果數量
        #[arg(short, long, default_value = "10")]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum PromptAction {
    /// 列出所有 Prompts
    List {
        /// 標籤篩選
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// 建立新 Prompt
    Create {
        /// Prompt 標題
        #[arg(short, long)]
        title: String,
        /// Prompt 內容
        #[arg(short, long)]
        content: String,
        /// 標籤 (逗號分隔)
        #[arg(short = 'g', long)]
        tags: Option<String>,
    },
    /// 顯示 Prompt 詳情
    Show {
        /// Prompt ID
        id: i64,
    },
    /// 編輯 Prompt
    Edit {
        /// Prompt ID
        id: i64,
        /// 新標題
        #[arg(short, long)]
        title: Option<String>,
        /// 新內容
        #[arg(short, long)]
        content: Option<String>,
        /// 新標籤
        #[arg(short = 'g', long)]
        tags: Option<String>,
    },
    /// 刪除 Prompt
    Delete {
        /// Prompt ID
        id: i64,
    },
}

#[derive(Subcommand)]
enum JobAction {
    /// 列出所有任務
    List {
        /// 狀態篩選
        #[arg(short, long)]
        status: Option<String>,
    },
    /// 顯示任務詳情
    Show {
        /// 任務 ID
        id: i64,
    },
    /// 取消任務
    Cancel {
        /// 任務 ID
        id: i64,
    },
}

// 簡化的資料結構
#[derive(Debug, Serialize, Deserialize)]
struct Prompt {
    id: Option<i64>,
    title: String,
    content: String,
    tags: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    id: Option<i64>,
    prompt_id: i64,
    cron_expr: String,
    mode: String,
    status: String,
    eta_unix: Option<i64>,
    last_run_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobResult {
    id: Option<i64>,
    job_id: i64,
    content: String,
    created_at: Option<String>,
}

// 簡化的執行器
struct SimpleClaudeExecutor;

impl SimpleClaudeExecutor {
    async fn run_sync(prompt: &str) -> Result<String> {
        use tokio::process::Command;
        
        let output = Command::new("claude")
            .arg("-p")
            .arg(prompt)
            .arg("--output-format")
            .arg("json")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Claude CLI 執行失敗: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    async fn verify_claude_cli() -> Result<bool> {
        use tokio::process::Command;
        
        match Command::new("claude").arg("--version").output().await {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn check_cooldown() -> Result<bool> {
        // 簡化版本，實際實現中需要解析 claude doctor 輸出
        Self::verify_claude_cli().await
    }
}

// 資料庫管理器
struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    async fn new() -> Result<Self> {
        let db_path = "claude-pilot.db";
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
        
        // 建立表格
        let init_sql = r#"
-- 初始化資料庫 schema
CREATE TABLE IF NOT EXISTS prompts (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  title     TEXT NOT NULL,
  content   TEXT NOT NULL,
  tags      TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS jobs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  prompt_id   INTEGER NOT NULL,
  cron_expr   TEXT NOT NULL,
  mode        TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'pending',
  eta_unix    INTEGER,
  last_run_at DATETIME,
  FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS results (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id    INTEGER NOT NULL,
  content   TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);
        "#;
        
        sqlx::query(init_sql).execute(&pool).await?;
        
        Ok(DatabaseManager { pool })
    }

    async fn list_prompts(&self, tag_filter: Option<String>) -> Result<Vec<Prompt>> {
        let query = if let Some(tag) = tag_filter {
            sqlx::query("SELECT * FROM prompts WHERE tags LIKE ? ORDER BY created_at DESC")
                .bind(format!("%{}%", tag))
        } else {
            sqlx::query("SELECT * FROM prompts ORDER BY created_at DESC")
        };

        let rows = query.fetch_all(&self.pool).await?;
        let mut prompts = Vec::new();

        for row in rows {
            prompts.push(Prompt {
                id: Some(row.get::<i64, _>("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
                created_at: row.get("created_at"),
            });
        }

        Ok(prompts)
    }

    async fn create_prompt(&self, title: &str, content: &str, tags: Option<&str>) -> Result<i64> {
        let result = sqlx::query("INSERT INTO prompts (title, content, tags) VALUES (?, ?, ?)")
            .bind(title)
            .bind(content)
            .bind(tags)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    async fn get_prompt(&self, id: i64) -> Result<Option<Prompt>> {
        let row = sqlx::query("SELECT * FROM prompts WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Prompt {
                id: Some(row.get::<i64, _>("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_prompt(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM prompts WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn create_job(&self, prompt_id: i64, mode: &str, cron_expr: Option<&str>) -> Result<i64> {
        let cron = cron_expr.unwrap_or("*");
        let result = sqlx::query("INSERT INTO jobs (prompt_id, cron_expr, mode, status) VALUES (?, ?, ?, 'pending')")
            .bind(prompt_id)
            .bind(cron)
            .bind(mode)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    async fn list_jobs(&self, status_filter: Option<String>) -> Result<Vec<Job>> {
        let query = if let Some(status) = status_filter {
            sqlx::query("SELECT * FROM jobs WHERE status = ? ORDER BY id DESC")
                .bind(status)
        } else {
            sqlx::query("SELECT * FROM jobs ORDER BY id DESC")
        };

        let rows = query.fetch_all(&self.pool).await?;
        let mut jobs = Vec::new();

        for row in rows {
            jobs.push(Job {
                id: Some(row.get::<i64, _>("id")),
                prompt_id: row.get("prompt_id"),
                cron_expr: row.get("cron_expr"),
                mode: row.get("mode"),
                status: row.get("status"),
                eta_unix: row.get("eta_unix"),
                last_run_at: row.get("last_run_at"),
            });
        }

        Ok(jobs)
    }

    async fn create_result(&self, job_id: i64, content: &str) -> Result<i64> {
        let result = sqlx::query("INSERT INTO results (job_id, content) VALUES (?, ?)")
            .bind(job_id)
            .bind(content)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    async fn list_results(&self, job_id: Option<i64>, limit: i64) -> Result<Vec<JobResult>> {
        let query = if let Some(job_id) = job_id {
            sqlx::query("SELECT * FROM results WHERE job_id = ? ORDER BY created_at DESC LIMIT ?")
                .bind(job_id)
                .bind(limit)
        } else {
            sqlx::query("SELECT * FROM results ORDER BY created_at DESC LIMIT ?")
                .bind(limit)
        };

        let rows = query.fetch_all(&self.pool).await?;
        let mut results = Vec::new();

        for row in rows {
            results.push(JobResult {
                id: Some(row.get::<i64, _>("id")),
                job_id: row.get("job_id"),
                content: row.get("content"),
                created_at: row.get("created_at"),
            });
        }

        Ok(results)
    }
}

// 工具函數
fn print_success(msg: &str) {
    println!("{} {}", "✅".green(), msg.green());
}

fn print_error(msg: &str) {
    eprintln!("{} {}", "❌".red(), msg.red());
}

fn print_info(msg: &str) {
    println!("{} {}", "ℹ️".blue(), msg.blue());
}

fn print_warning(msg: &str) {
    println!("{} {}", "⚠️".yellow(), msg.yellow());
}

fn format_datetime(dt_str: Option<&String>) -> String {
    match dt_str {
        Some(dt) => {
            if let Ok(parsed) = dt.parse::<DateTime<Utc>>() {
                parsed.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                dt.clone()
            }
        }
        None => "N/A".to_string(),
    }
}

// 主要處理函數
async fn handle_init() -> Result<()> {
    print_info("初始化 Claude Night Pilot 資料庫...");
    
    let _db = DatabaseManager::new().await?;
    print_success("資料庫初始化完成！");
    
    // 檢查 Claude CLI
    if SimpleClaudeExecutor::verify_claude_cli().await.unwrap_or(false) {
        print_success("Claude CLI 已安裝並可用");
    } else {
        print_warning("Claude CLI 未找到或未正確配置");
        print_info("請確保已安裝 Claude CLI: https://docs.anthropic.com/claude/docs/claude-cli");
    }
    
    Ok(())
}

async fn handle_prompt_list(tag: Option<String>) -> Result<()> {
    let db = DatabaseManager::new().await?;
    let prompts = db.list_prompts(tag.clone()).await?;
    
    if prompts.is_empty() {
        if let Some(tag) = tag {
            print_info(&format!("沒有找到標籤為 '{}' 的 Prompts", tag));
        } else {
            print_info("沒有找到任何 Prompts");
        }
        return Ok(());
    }
    
    println!("{}", "Prompt 列表:".bold().blue());
    println!("{}", "─".repeat(80));
    
    for prompt in prompts {
        println!("{} {} [ID: {}]", 
            "📝".cyan(), 
            prompt.title.bold(), 
            prompt.id.unwrap_or(0).to_string().yellow()
        );
        
        if let Some(tags) = &prompt.tags {
            println!("   標籤: {}", tags.green());
        }
        
        // 顯示內容預覽 (前 100 字元)
        let preview = if prompt.content.len() > 100 {
            format!("{}...", &prompt.content[..100])
        } else {
            prompt.content.clone()
        };
        println!("   內容: {}", preview.dimmed());
        
        println!("   建立時間: {}", format_datetime(prompt.created_at.as_ref()).dimmed());
        println!();
    }
    
    Ok(())
}

async fn handle_prompt_create(title: String, content: String, tags: Option<String>) -> Result<()> {
    let db = DatabaseManager::new().await?;
    let id = db.create_prompt(&title, &content, tags.as_deref()).await?;
    
    print_success(&format!("Prompt 建立成功！ID: {}", id));
    
    // 顯示建立的 Prompt 詳情
    if let Some(prompt) = db.get_prompt(id).await? {
        println!("\n{}", "建立的 Prompt:".bold());
        println!("標題: {}", prompt.title.cyan());
        if let Some(tags) = prompt.tags {
            println!("標籤: {}", tags.green());
        }
        println!("內容: {}", prompt.content);
    }
    
    Ok(())
}

async fn handle_prompt_show(id: i64) -> Result<()> {
    let db = DatabaseManager::new().await?;
    
    if let Some(prompt) = db.get_prompt(id).await? {
        println!("{}", format!("Prompt 詳情 [ID: {}]", id).bold().blue());
        println!("{}", "─".repeat(50));
        println!("標題: {}", prompt.title.cyan());
        if let Some(tags) = prompt.tags {
            println!("標籤: {}", tags.green());
        }
        println!("建立時間: {}", format_datetime(prompt.created_at.as_ref()));
        println!("\n內容:");
        println!("{}", prompt.content);
    } else {
        print_error(&format!("找不到 ID 為 {} 的 Prompt", id));
    }
    
    Ok(())
}

async fn handle_prompt_delete(id: i64) -> Result<()> {
    let db = DatabaseManager::new().await?;
    
    // 檢查 Prompt 是否存在
    if let Some(prompt) = db.get_prompt(id).await? {
        println!("即將刪除 Prompt:");
        println!("ID: {}", id);
        println!("標題: {}", prompt.title.red());
        
        let deleted = db.delete_prompt(id).await?;
        
        if deleted {
            print_success(&format!("Prompt ID {} 已刪除", id));
        } else {
            print_error("刪除失敗");
        }
    } else {
        print_error(&format!("找不到 ID 為 {} 的 Prompt", id));
    }
    
    Ok(())
}

async fn handle_run(prompt: String, mode: String, cron: Option<String>) -> Result<()> {
    let db = DatabaseManager::new().await?;
    
    // 判斷 prompt 是 ID 還是內容
    let (prompt_id, prompt_content) = if let Ok(id) = prompt.parse::<i64>() {
        // 是 ID，從資料庫獲取內容
        if let Some(p) = db.get_prompt(id).await? {
            (Some(id), p.content)
        } else {
            print_error(&format!("找不到 ID 為 {} 的 Prompt", id));
            return Ok(());
        }
    } else {
        // 是內容
        (None, prompt)
    };
    
    // 建立任務記錄
    let job_id = if let Some(pid) = prompt_id {
        db.create_job(pid, &mode, cron.as_deref()).await?
    } else {
        // 為直接內容建立臨時 Prompt
        let temp_id = db.create_prompt("臨時 Prompt", &prompt_content, Some("temp,cli")).await?;
        db.create_job(temp_id, &mode, cron.as_deref()).await?
    };
    
    print_info(&format!("建立任務 ID: {}", job_id));
    
    if mode == "sync" {
        // 同步執行
        print_info("開始執行 Claude CLI...");
        
        match SimpleClaudeExecutor::run_sync(&prompt_content).await {
            Ok(response) => {
                print_success("執行成功！");
                
                // 保存結果
                db.create_result(job_id, &response).await?;
                
                println!("\n{}", "Claude 回應:".bold().green());
                println!("{}", "─".repeat(50));
                println!("{}", response);
            }
            Err(e) => {
                print_error(&format!("執行失敗: {}", e));
                db.create_result(job_id, &format!("錯誤: {}", e.to_string())).await?;
            }
        }
    } else {
        // 非同步執行
        print_info(&format!("任務已排程，模式: {}", mode));
        if let Some(cron_expr) = cron {
            print_info(&format!("Cron 表達式: {}", cron_expr));
        }
        print_info("使用 'cnp job list' 查看任務狀態");
    }
    
    Ok(())
}

async fn handle_status() -> Result<()> {
    println!("{}", "Claude Night Pilot 系統狀態".bold().blue());
    println!("{}", "─".repeat(40));
    
    // 檢查資料庫
    match DatabaseManager::new().await {
        Ok(db) => {
            print_success("資料庫連接正常");
            
            // 統計資訊
            let prompts = db.list_prompts(None).await?;
            let jobs = db.list_jobs(None).await?;
            let results = db.list_results(None, 1000).await?;
            
            println!("  Prompts: {}", prompts.len().to_string().cyan());
            println!("  任務: {}", jobs.len().to_string().cyan());
            println!("  結果: {}", results.len().to_string().cyan());
        }
        Err(e) => {
            print_error(&format!("資料庫連接失敗: {}", e));
        }
    }
    
    // 檢查 Claude CLI
    println!();
    match SimpleClaudeExecutor::verify_claude_cli().await {
        Ok(true) => print_success("Claude CLI 可用"),
        Ok(false) => print_warning("Claude CLI 不可用"),
        Err(e) => print_error(&format!("Claude CLI 檢查失敗: {}", e)),
    }
    
    Ok(())
}

async fn handle_cooldown() -> Result<()> {
    print_info("檢查 Claude CLI 冷卻狀態...");
    
    match SimpleClaudeExecutor::check_cooldown().await {
        Ok(true) => print_success("✨ Claude API 可用"),
        Ok(false) => print_warning("Claude CLI 不可用"),
        Err(e) => print_error(&format!("無法檢查冷卻狀態: {}", e)),
    }
    
    Ok(())
}

async fn handle_job_list(status_filter: Option<String>) -> Result<()> {
    let db = DatabaseManager::new().await?;
    let jobs = db.list_jobs(status_filter.clone()).await?;
    
    if jobs.is_empty() {
        if let Some(status) = status_filter {
            print_info(&format!("沒有找到狀態為 '{}' 的任務", status));
        } else {
            print_info("沒有找到任何任務");
        }
        return Ok(());
    }
    
    println!("{}", "任務列表:".bold().blue());
    println!("{}", "─".repeat(80));
    
    for job in jobs {
        let status_color = match job.status.as_str() {
            "done" => "green",
            "running" => "yellow", 
            "pending" => "blue",
            "error" => "red",
            _ => "white",
        };
        
        println!("{} 任務 ID: {} | 狀態: {} | 模式: {}", 
            "🔧".cyan(),
            job.id.unwrap_or(0).to_string().yellow(),
            job.status.color(status_color),
            job.mode.cyan()
        );
        
        println!("   Prompt ID: {}", job.prompt_id);
        
        if job.cron_expr != "*" {
            println!("   Cron: {}", job.cron_expr.green());
        }
        
        if let Some(last_run) = &job.last_run_at {
            println!("   最後執行: {}", format_datetime(Some(last_run)).dimmed());
        }
        
        println!();
    }
    
    Ok(())
}

async fn handle_results(job_id: Option<i64>, limit: i64) -> Result<()> {
    let db = DatabaseManager::new().await?;
    let results = db.list_results(job_id, limit).await?;
    
    if results.is_empty() {
        if let Some(jid) = job_id {
            print_info(&format!("任務 {} 沒有執行結果", jid));
        } else {
            print_info("沒有找到任何執行結果");
        }
        return Ok(());
    }
    
    println!("{}", "執行結果:".bold().blue());
    println!("{}", "─".repeat(80));
    
    for (i, result) in results.iter().enumerate() {
        println!("{} 結果 ID: {} | 任務 ID: {} | 時間: {}", 
            "📄".cyan(),
            result.id.unwrap_or(0).to_string().yellow(),
            result.job_id.to_string().cyan(),
            format_datetime(result.created_at.as_ref()).dimmed()
        );
        
        println!("內容:");
        // 如果內容很長，只顯示前面部分
        if result.content.len() > 500 {
            println!("{}", &result.content[..500]);
            println!("{}", "... (內容已截斷)".dimmed());
        } else {
            println!("{}", result.content);
        }
        
        if i < results.len() - 1 {
            println!("{}", "─".repeat(40).dimmed());
        }
        println!();
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init => handle_init().await?,
        Commands::Prompt { action } => {
            match action {
                PromptAction::List { tag } => handle_prompt_list(tag).await?,
                PromptAction::Create { title, content, tags } => {
                    handle_prompt_create(title, content, tags).await?
                }
                PromptAction::Show { id } => handle_prompt_show(id).await?,
                PromptAction::Edit { id, title, content, tags } => {
                    print_info(&format!("編輯功能開發中... (Prompt ID: {})", id));
                }
                PromptAction::Delete { id } => handle_prompt_delete(id).await?,
            }
        }
        Commands::Job { action } => {
            match action {
                JobAction::List { status } => handle_job_list(status).await?,
                JobAction::Show { id } => {
                    print_info(&format!("顯示任務詳情功能開發中... (任務 ID: {})", id));
                }
                JobAction::Cancel { id } => {
                    print_info(&format!("取消任務功能開發中... (任務 ID: {})", id));
                }
            }
        }
        Commands::Run { prompt, mode, cron } => handle_run(prompt, mode, cron).await?,
        Commands::Status => handle_status().await?,
        Commands::Cooldown => handle_cooldown().await?,
        Commands::Results { job_id, limit } => handle_results(job_id, limit).await?,
    }
    
    Ok(())
} 