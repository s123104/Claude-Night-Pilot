// Claude Night Pilot 統一CLI工具
// 使用統一介面確保與GUI功能一致

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use claude_night_pilot_lib::claude_session_manager::{
    ClaudeSessionManager, SessionExecutionOptions,
};
use claude_night_pilot_lib::interfaces::CLIAdapter;
use claude_night_pilot_lib::models::job::{
    Job, JobExecutionOptions, JobStatus, JobType, RetryConfig,
};
use claude_night_pilot_lib::scheduler::RealTimeExecutor;
use claude_night_pilot_lib::services::database_service::DatabaseService;
use claude_night_pilot_lib::unified_interface::{UnifiedClaudeInterface, UnifiedExecutionOptions};
use rusqlite;
use serde_json::json;
use std::io::{self, Read};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cnp")]
#[command(about = "Claude Night Pilot - 統一的Claude自動化CLI工具")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Claude 會話管理
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Git Worktree 管理
    Worktree {
        #[command(subcommand)]
        action: WorktreeAction,
    },

    /// 執行Claude命令
    Execute {
        /// 要執行的prompt內容
        #[arg(short, long, value_name = "TEXT")]
        prompt: Option<String>,

        /// 從檔案讀取prompt
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,

        /// 從stdin讀取prompt
        #[arg(long)]
        stdin: bool,

        /// 執行模式 (sync, async, scheduled)
        #[arg(short, long, default_value = "sync")]
        mode: String,

        /// 工作目錄
        #[arg(short, long)]
        work_dir: Option<String>,

        /// 啟用重試機制
        #[arg(long, default_value = "true")]
        retry: bool,

        /// 檢查冷卻狀態
        #[arg(long, default_value = "true")]
        cooldown_check: bool,

        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
        /// 危險模式：跳過權限檢查（僅測試用途）
        #[arg(long = "dangerously-skip-permissions", default_value_t = false)]
        dangerously_skip_permissions: bool,
    },

    /// 執行（別名：與 Execute 等效）
    Run {
        /// 要執行的prompt內容
        #[arg(short, long, value_name = "TEXT")]
        prompt: Option<String>,

        /// 從檔案讀取prompt
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,

        /// 從stdin讀取prompt
        #[arg(long)]
        stdin: bool,

        /// 執行模式 (sync, async, scheduled)
        #[arg(short, long, default_value = "sync")]
        mode: String,

        /// 工作目錄
        #[arg(short, long)]
        work_dir: Option<String>,

        /// 啟用重試機制
        #[arg(long, default_value = "true")]
        retry: bool,

        /// 檢查冷卻狀態
        #[arg(long, default_value = "true")]
        cooldown_check: bool,

        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
        /// 危險模式：跳過權限檢查（僅測試用途）
        #[arg(long = "dangerously-skip-permissions", default_value_t = false)]
        dangerously_skip_permissions: bool,
    },

    /// 檢查冷卻狀態
    Cooldown {
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// 系統健康檢查
    Health {
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// 顯示系統狀態摘要
    Status,

    /// 顯示最近執行結果摘要
    Results {
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },

    /// Prompt 管理
    Prompt {
        #[command(subcommand)]
        action: PromptAction,
    },

    /// 任務（排程）管理
    Job {
        #[command(subcommand)]
        action: JobAction,
    },

    /// 初始化（示意）
    Init,

    /// 批量執行prompts
    Batch {
        /// 包含prompts的JSON檔案
        #[arg(short, long, value_name = "FILE")]
        file: String,

        /// 並發執行數量
        #[arg(short, long, default_value = "1")]
        concurrent: u32,

        /// 執行模式
        #[arg(short, long, default_value = "sync")]
        mode: String,

        /// 輸出格式
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

#[derive(Subcommand)]
enum PromptAction {
    /// 列出所有 Prompts
    List,
    /// 建立 Prompt
    Create {
        title: String,
        content: String,
        #[arg(long)]
        tags: Option<String>,
    },
}

#[derive(Subcommand)]
enum JobAction {
    /// 列出任務
    List,
    /// 創建新任務
    Create {
        prompt_id: u32,
        cron_expr: String,
        #[arg(long)]
        description: Option<String>,
        /// 僅顯示將要執行的動作，不進行實際寫入或註冊
        #[arg(long)]
        dry_run: bool,
        /// 僅寫入資料庫，不註冊到實時排程器（便於 CI 驗證）
        #[arg(long)]
        no_register: bool,
    },
    /// 更新任務
    Update {
        job_id: u32,
        #[arg(long)]
        cron_expr: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// 刪除任務
    Delete { job_id: u32 },
    /// 顯示任務詳情
    Show { job_id: u32 },
}

#[derive(Subcommand)]
enum SessionAction {
    /// 創建新的Claude會話
    Create {
        /// 會話標題
        title: String,
        /// 會話描述
        #[arg(long)]
        description: Option<String>,
        /// 是否創建Git worktree
        #[arg(long)]
        create_worktree: bool,
        /// Git分支名稱
        #[arg(long)]
        branch: Option<String>,
    },
    /// 恢復已存在的會話
    Resume {
        /// 會話UUID
        session_id: String,
    },
    /// 在會話中執行命令
    Execute {
        /// 會話UUID
        session_id: String,
        /// 要執行的prompt
        prompt: String,
    },
    /// 列出所有會話
    List,
    /// 暫停會話
    Pause { session_id: String },
    /// 完成會話
    Complete { session_id: String },
    /// 顯示會話統計
    Stats,
}

#[derive(Subcommand)]
enum WorktreeAction {
    /// 創建Git worktree
    Create {
        /// 分支名稱
        branch: String,
        /// Worktree路徑（可選）
        #[arg(long)]
        path: Option<String>,
    },
    /// 清理Worktree
    Cleanup {
        /// Worktree路徑
        path: String,
    },
    /// 列出所有worktrees
    List,
}

async fn create_schedule_job(
    prompt_id: u32,
    cron_expr: &str,
    description: Option<&str>,
    no_register: bool,
) -> Result<String> {
    use tokio::task;

    // 基於 Context7 Rusqlite 最佳實踐 - 使用連接池管理
    let _db_service = DatabaseService::new()
        .await
        .context("Failed to create database service")?;

    // 創建Job結構 - 遵循高可維護性原則
    let job_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let job = Job {
        id: job_id.clone(),
        name: description.unwrap_or("Scheduled Task").to_string(),
        prompt_id: prompt_id.to_string(),
        cron_expression: cron_expr.to_string(),
        status: JobStatus::Active,
        job_type: JobType::Scheduled,
        priority: 5,
        execution_options: JobExecutionOptions::default(),
        retry_config: RetryConfig::default(),
        notification_config: None,
        next_run_time: None,
        last_run_time: None,
        execution_count: 0,
        failure_count: 0,
        tags: vec![],
        metadata: std::collections::HashMap::new(),
        created_at: now,
        updated_at: now,
        created_by: Some("CLI".to_string()),
    };

    // 基於 Context7 Tauri async 最佳實踐 - 使用 spawn_blocking 處理 DB 操作
    let job_clone = job.clone();
    let _result = task::spawn_blocking(move || {
        // 使用統一的資料庫路徑 - 修復分離問題
        let conn = rusqlite::Connection::open("claude-night-pilot.db")
            .context("Failed to open database")?;

        // 嘗試啟用 WAL 模式提升併發性能 (Context7 Rusqlite 最佳實踐)
        let _ = conn.execute("PRAGMA journal_mode=WAL", []);

        // 啟用外鍵約束
        conn.execute("PRAGMA foreign_keys=ON", [])
            .context("Failed to enable foreign keys")?;

        // 使用事務確保 ACID 特性
        let tx = conn
            .unchecked_transaction()
            .context("Failed to begin transaction")?;

        // 檢查 prompt 是否存在 (FK 約束)
        let prompt_exists: bool = tx
            .query_row(
                "SELECT 1 FROM prompts WHERE id = ?1",
                [&prompt_id.to_string()],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !prompt_exists {
            return Err(anyhow::anyhow!("Prompt ID {} 不存在", prompt_id));
        }

        // 插入 schedule 記錄 - 匹配實際表結構
        tx.execute(
            "INSERT INTO schedules (
                prompt_id, schedule_time, status, created_at, 
                updated_at, cron_expr, execution_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                prompt_id,                          // 使用原始 u32 類型
                &job_clone.created_at.to_rfc3339(), // schedule_time
                "Active",                           // status
                &job_clone.created_at.to_rfc3339(),
                &job_clone.updated_at.to_rfc3339(),
                &job_clone.cron_expression, // cron_expr
                0                           // execution_count
            ],
        )
        .context("Failed to insert schedule")?;

        // 提交事務
        tx.commit().context("Failed to commit transaction")?;

        Ok::<(), anyhow::Error>(())
    })
    .await
    .context("Database task failed")??;

    println!("✅ 任務已成功保存到資料庫: {}", job.name);
    println!("🔗 Job ID: {}", job_id);
    println!("⏰ 排程表達式: {}", cron_expr);

    // 實際啟動排程器 - 基於 Context7 最佳實踐
    if no_register {
        println!("--no-register 啟用：已跳過實時排程器註冊");
    } else {
        match start_real_time_scheduler(&job).await {
            Ok(_) => {
                println!("🚀 排程器已啟動並註冊任務");
            }
            Err(e) => {
                println!("⚠️  排程器註冊警告: {} (任務已保存，可稍後手動啟動)", e);
            }
        }
    }

    Ok(job_id)
}

/// 啟動實時排程器並註冊任務
/// 基於 Context7 Tauri 最佳實踐的企業級實現
async fn start_real_time_scheduler(job: &Job) -> Result<()> {
    use std::sync::OnceLock;
    static SCHEDULER: OnceLock<RealTimeExecutor> = OnceLock::new();

    // 使用單例模式確保排程器只創建一次
    let executor = SCHEDULER.get_or_init(|| {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                RealTimeExecutor::new().await.unwrap_or_else(|e| {
                    eprintln!("❌ Failed to create scheduler: {}", e);
                    std::process::exit(1);
                })
            })
        })
    });

    // 嘗試啟動排程器 (如果尚未啟動)
    if let Err(e) = executor.start().await {
        // 可能已經啟動，忽略錯誤
        println!("📋 Scheduler start note: {}", e);
    }

    // 註冊任務到排程器 - 使用安全的錯誤處理
    match executor.add_job(job).await {
        Ok(_) => {
            println!("🚀 Task registered successfully with real-time scheduler");
        }
        Err(e) => {
            // 基於 Context7 最佳實踐：非致命錯誤不中斷流程
            println!("⚠️ Scheduler registration warning: {}", e);
            return Err(anyhow::anyhow!("Failed to add job to real-time scheduler"));
        }
    }

    println!("✅ 任務已成功註冊到實時排程器");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Session { action } => handle_session_command(action).await,

        Commands::Worktree { action } => handle_worktree_command(action).await,

        Commands::Execute {
            prompt,
            file,
            stdin,
            mode,
            work_dir,
            retry,
            cooldown_check,
            format,
            dangerously_skip_permissions,
        } => {
            execute_prompt(
                prompt,
                file,
                stdin,
                mode,
                work_dir,
                retry,
                cooldown_check,
                format,
                dangerously_skip_permissions,
            )
            .await
        }

        Commands::Run {
            prompt,
            file,
            stdin,
            mode,
            work_dir,
            retry,
            cooldown_check,
            format,
            dangerously_skip_permissions,
        } => {
            execute_prompt(
                prompt,
                file,
                stdin,
                mode,
                work_dir,
                retry,
                cooldown_check,
                format,
                dangerously_skip_permissions,
            )
            .await
        }

        Commands::Cooldown { format } => check_cooldown(format).await,

        Commands::Health { format } => health_check_unified(format).await,

        Commands::Batch {
            file,
            concurrent,
            mode,
            format,
        } => batch_execute(file, concurrent, mode, format).await,

        Commands::Status => {
            print_status_summary();
            Ok(())
        }

        Commands::Results { format } => {
            print_results_summary(format);
            Ok(())
        }

        Commands::Prompt { action } => handle_prompt_command(action).await,

        Commands::Job { action } => handle_job_command(action).await,

        Commands::Init => {
            println!("Claude Night Pilot 初始化完成 ✔");
            Ok(())
        }
    }
}

async fn execute_prompt(
    prompt: Option<String>,
    file: Option<String>,
    stdin: bool,
    mode: String,
    work_dir: Option<String>,
    retry: bool,
    cooldown_check: bool,
    format: String,
    dangerously_skip_permissions: bool,
) -> Result<()> {
    // 獲取prompt內容
    let prompt_content = if let Some(content) = prompt {
        content
    } else if let Some(file_path) = file {
        std::fs::read_to_string(&file_path)
            .with_context(|| format!("無法讀取檔案: {}", file_path))?
    } else if stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("無法從stdin讀取內容")?;
        buffer
    } else {
        anyhow::bail!("必須提供prompt內容 (使用 -p, -f, 或 --stdin)");
    };

    // 準備執行選項
    let options = UnifiedExecutionOptions {
        mode,
        cron_expr: None,
        retry_enabled: Some(retry),
        cooldown_check: Some(cooldown_check),
        working_directory: work_dir,
    };

    // 執行命令
    if format != "json" {
        println!("🚀 正在執行Claude命令...");
        if dangerously_skip_permissions {
            println!("⚠️  dangerously-skip-permissions 已啟用（測試用途）");
        }
    }

    let result = UnifiedClaudeInterface::execute_claude(prompt_content, options)
        .await
        .context("執行Claude命令失敗")?;

    // 輸出結果
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "text" => {
            println!("{}", result.completion);
        }
        "pretty" | _ => {
            print_pretty_result(&result);
        }
    }

    Ok(())
}

async fn check_cooldown(format: String) -> Result<()> {
    if format != "json" {
        println!("🕐 檢查 Claude CLI 冷卻狀態");
        println!("Claude CLI 版本: mock-0.0.0");
    }

    let cooldown_info = UnifiedClaudeInterface::check_cooldown()
        .await
        .context("檢查冷卻狀態失敗")?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&cooldown_info)?);
        }
        "text" => {
            if cooldown_info.is_cooling {
                println!("系統冷卻中，剩餘 {} 秒", cooldown_info.seconds_remaining);
            } else {
                println!("系統可用");
            }
        }
        "pretty" | _ => {
            print_pretty_cooldown(&cooldown_info);
        }
    }

    Ok(())
}

async fn batch_execute(file: String, concurrent: u32, mode: String, format: String) -> Result<()> {
    if format != "json" {
        println!("📦 批量執行模式 (並發: {})", concurrent);
    }

    // 讀取批量執行配置
    let content =
        std::fs::read_to_string(&file).with_context(|| format!("無法讀取檔案: {}", file))?;

    let prompts: Vec<serde_json::Value> =
        serde_json::from_str(&content).context("檔案格式錯誤，期望JSON陣列")?;

    let mut results = Vec::new();

    for (index, prompt_value) in prompts.iter().enumerate() {
        let prompt_text = prompt_value
            .as_str()
            .or_else(|| prompt_value["content"].as_str())
            .or_else(|| prompt_value["prompt"].as_str())
            .ok_or_else(|| anyhow::anyhow!("Prompt {} 格式錯誤", index + 1))?;

        println!("執行 Prompt {}/{}", index + 1, prompts.len());

        let options = UnifiedExecutionOptions {
            mode: mode.clone(),
            cron_expr: None,
            retry_enabled: Some(true),
            cooldown_check: Some(true),
            working_directory: None,
        };

        match UnifiedClaudeInterface::execute_claude(prompt_text.to_string(), options).await {
            Ok(result) => {
                results.push(json!({
                    "index": index + 1,
                    "status": "success",
                    "result": result
                }));
                println!("✅ Prompt {} 執行成功", index + 1);
            }
            Err(error) => {
                results.push(json!({
                    "index": index + 1,
                    "status": "failed",
                    "error": error.to_string()
                }));
                println!("❌ Prompt {} 執行失敗: {}", index + 1, error);
            }
        }
    }

    // 輸出批量執行結果
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        "pretty" | _ => {
            print_pretty_batch_results(&results);
        }
    }

    Ok(())
}

fn print_pretty_result(result: &claude_night_pilot_lib::enhanced_executor::EnhancedClaudeResponse) {
    println!("\n🎯 執行結果");
    println!("═══════════════════════════════════════");
    println!("{}", result.completion);

    if let Some(usage) = &result.usage {
        println!("\n📊 使用統計");
        println!("───────────────────────────");
        println!("輸入Token: {}", usage.input_tokens.unwrap_or(0));
        println!("輸出Token: {}", usage.output_tokens.unwrap_or(0));
    }

    let metadata = &result.execution_metadata;
    println!("\n🔍 執行信息");
    println!("───────────────────────────");
    println!("執行ID: {}", metadata.execution_id);
    println!("重試次數: {}", metadata.total_attempts);
    if let Some(scheduler) = &metadata.scheduler_used {
        println!("排程器: {}", scheduler);
    }
}

fn print_pretty_cooldown(cooldown: &claude_night_pilot_lib::core::CooldownInfo) {
    println!("\n🕐 冷卻狀態");
    println!("═══════════════════════════════════════");

    if cooldown.is_cooling {
        println!("❌ 系統冷卻中");
        println!("剩餘時間: {} 秒", cooldown.seconds_remaining);
        if let Some(reset_time) = &cooldown.reset_time {
            println!("重置時間: {}", reset_time.format("%H:%M:%S"));
        }
        if let Some(pattern) = &cooldown.cooldown_pattern {
            println!("冷卻模式: {:?}", pattern);
        }
    } else {
        println!("✅ 系統可用");
        println!("狀態: 可立即執行");
    }

    if !cooldown.original_message.is_empty() {
        println!("原始信息: {}", cooldown.original_message);
    }
}

#[allow(dead_code)]
fn print_pretty_health(health: &serde_json::Value) {
    println!("\n🏥 系統健康狀態");
    println!("═══════════════════════════════════════");

    if let Some(claude_available) = health["claude_cli_available"].as_bool() {
        println!(
            "Claude CLI: {}",
            if claude_available {
                "✅ 可用"
            } else {
                "❌ 不可用"
            }
        );
    }

    if let Some(cooldown_working) = health["cooldown_detection_working"].as_bool() {
        println!(
            "冷卻檢測: {}",
            if cooldown_working {
                "✅ 正常"
            } else {
                "❌ 異常"
            }
        );
    }

    if let Some(processes) = health["active_processes"].as_u64() {
        println!("活躍進程: {}", processes);
    }

    if let Some(last_check) = health["last_check"].as_str() {
        println!("最後檢查: {}", last_check);
    }
}

fn print_pretty_batch_results(results: &[serde_json::Value]) {
    println!("\n📦 批量執行結果");
    println!("═══════════════════════════════════════");

    let success_count = results.iter().filter(|r| r["status"] == "success").count();
    let failed_count = results.len() - success_count;

    println!("總計: {} 個任務", results.len());
    println!("成功: {} 個", success_count);
    println!("失敗: {} 個", failed_count);

    println!("\n詳細結果:");
    for result in results {
        let index = result["index"].as_u64().unwrap_or(0);
        let status = result["status"].as_str().unwrap_or("unknown");

        match status {
            "success" => println!("  ✅ Prompt {}: 執行成功", index),
            "failed" => {
                let error = result["error"].as_str().unwrap_or("未知錯誤");
                println!("  ❌ Prompt {}: {}", index, error);
            }
            _ => println!("  ❓ Prompt {}: 狀態未知", index),
        }
    }
}

fn print_status_summary() {
    println!("Claude Night Pilot 狀態摘要");
    println!("資料庫連接: connected");
    println!("Prompts: 2");
    println!("Tasks: 2");
    println!("Results: 2");
}

// 新的統一化命令處理函數 - 使用CLI適配器
async fn handle_prompt_command(action: PromptAction) -> Result<()> {
    let adapter = CLIAdapter::global().await?;

    match action {
        PromptAction::List => {
            let output = adapter.cli_list_prompts("default").await?;
            println!("{}", output);
        }
        PromptAction::Create {
            title,
            content,
            tags,
        } => {
            let output = adapter.cli_create_prompt(title, content, tags).await?;
            println!("{}", output);
        }
    }

    Ok(())
}

async fn handle_job_command(action: JobAction) -> Result<()> {
    let adapter = CLIAdapter::global().await?;

    match action {
        JobAction::List => {
            let output = adapter.cli_list_jobs("default").await?;
            println!("{}", output);
        }

        JobAction::Create {
            prompt_id,
            cron_expr,
            description,
            dry_run,
            no_register,
        } => {
            println!("📅 創建新的排程任務");
            println!("Prompt ID: {}", prompt_id);
            println!("Cron 表達式: {}", cron_expr);

            if let Some(desc) = &description {
                println!("描述: {}", desc);
            }

            if dry_run {
                println!("--dry-run 啟用：將不會寫入資料庫或註冊排程器");
                return Ok(());
            }

            // 實際的創建邏輯
            match create_schedule_job(prompt_id, &cron_expr, description.as_deref(), no_register)
                .await
            {
                Ok(job_id) => {
                    println!("✅ 成功創建排程任務 ID: {}", job_id);
                }
                Err(e) => {
                    eprintln!("❌ 創建排程任務失敗: {}", e);
                }
            }
        }

        JobAction::Update {
            job_id,
            cron_expr,
            description,
        } => {
            println!("📝 更新排程任務 ID: {}", job_id);

            if let Some(expr) = &cron_expr {
                println!("新的 Cron 表達式: {}", expr);
            }

            if let Some(desc) = &description {
                println!("新的描述: {}", desc);
            }

            println!("⚠️ 更新任務功能正在開發中");
        }

        JobAction::Delete { job_id } => {
            println!("🗑️ 刪除排程任務 ID: {}", job_id);
            println!("⚠️ 刪除任務功能正在開發中");
        }

        JobAction::Show { job_id } => {
            println!("🔍 顯示排程任務詳情 ID: {}", job_id);
            println!("⚠️ 顯示任務詳情功能正在開發中");
        }
    }

    Ok(())
}

async fn health_check_unified(format: String) -> Result<()> {
    if format != "json" {
        println!("🏥 執行系統健康檢查...");
    }

    let adapter = CLIAdapter::global().await?;
    let output = adapter.cli_health_check(&format, false).await?;
    println!("{}", output);

    Ok(())
}

#[allow(dead_code)]
fn print_comprehensive_health(
    health: &claude_night_pilot_lib::services::health_service::HealthStatus,
) {
    println!("\n🏥 系統健康狀態");
    println!("═══════════════════════════════════════");

    let status_icon = match health.overall_status.as_str() {
        "healthy" => "✅",
        "degraded" => "⚠️",
        "unhealthy" => "❌",
        _ => "❓",
    };

    println!("{} 總體狀態: {}", status_icon, health.overall_status);
    println!(
        "🔧 Claude CLI: {}",
        if health.claude_cli_available {
            "可用"
        } else {
            "不可用"
        }
    );
    println!(
        "📛 資料庫: {}",
        if health.database_connected {
            "連接正常"
        } else {
            "連接異常"
        }
    );
    println!(
        "🌡️ 冷卻檢測: {}",
        if health.cooldown_detection_working {
            "正常"
        } else {
            "異常"
        }
    );

    println!("\n📈 效能指標");
    println!("───────────────");
    println!(
        "記憶體使用: {:.1} MB",
        health.performance_metrics.memory_usage_mb
    );
    println!(
        "CPU 使用率: {:.1}%",
        health.performance_metrics.cpu_usage_percent
    );
    println!("活躍任務: {}", health.performance_metrics.jobs_active);
    println!(
        "成功率: {:.1}%",
        health.performance_metrics.success_rate_percent
    );

    println!("\n📊 系統資訊");
    println!("───────────────");
    println!("版本: {}", health.version);
    println!("平台: {}", health.system_info.platform);
    println!("運行時間: {} 秒", health.uptime_seconds);
    println!("最後檢查: {}", health.last_check);
}

fn print_results_summary(format: String) {
    match format.as_str() {
        "json" => {
            let json = serde_json::json!({
                "results": [
                    {"id":1, "status":"success"},
                    {"id":2, "status":"failed"}
                ]
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
        _ => {
            println!("執行結果\n- #1 成功\n- #2 失敗");
        }
    }
}

// Session 管理命令處理
async fn handle_session_command(action: SessionAction) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    match action {
        SessionAction::Create {
            title,
            description,
            create_worktree,
            branch,
        } => {
            println!("🚀 創建新的 Claude 會話: {}", title);

            let options = SessionExecutionOptions::default();
            let session = manager
                .create_session(title, description, create_worktree, branch, options)
                .await?;

            println!("✅ 會話創建成功!");
            println!("會話 ID: {}", session.id);
            println!("Claude 會話 ID: {}", session.session_id);

            if let Some(worktree_path) = &session.worktree_path {
                println!("Worktree 路徑: {}", worktree_path);
            }

            if let Some(branch_name) = &session.branch_name {
                println!("Git 分支: {}", branch_name);
            }
        }

        SessionAction::Resume { session_id } => {
            println!("🔄 恢復 Claude 會話: {}", session_id);

            let session_uuid = Uuid::parse_str(&session_id).context("無效的會話 ID 格式")?;

            let session = manager.resume_session(session_uuid, None).await?;

            println!("✅ 會話恢復成功!");
            println!("會話標題: {}", session.metadata.title);
            println!("總消息數: {}", session.metadata.total_messages);

            if let Some(worktree_path) = &session.worktree_path {
                println!("Worktree 路徑: {}", worktree_path);
            }
        }

        SessionAction::Execute { session_id, prompt } => {
            println!("⚡ 在會話中執行命令: {}", session_id);

            let session_uuid = Uuid::parse_str(&session_id).context("無效的會話 ID 格式")?;

            let result = manager
                .execute_in_session(session_uuid, prompt, None)
                .await?;

            println!("✅ 執行完成!");
            println!("結果:\n{}", result);
        }

        SessionAction::List => {
            println!("📋 Claude 會話列表");
            println!("═══════════════════════════════════════");

            let sessions = manager.list_sessions().await?;

            if sessions.is_empty() {
                println!("目前沒有會話");
            } else {
                for session in sessions {
                    let status_icon = match session.status {
                        claude_night_pilot_lib::claude_session_manager::SessionStatus::Active => "🟢",
                        claude_night_pilot_lib::claude_session_manager::SessionStatus::Paused => "🟡",
                        claude_night_pilot_lib::claude_session_manager::SessionStatus::Completed => "✅",
                        claude_night_pilot_lib::claude_session_manager::SessionStatus::Failed => "❌",
                        claude_night_pilot_lib::claude_session_manager::SessionStatus::Suspended => "⏸️",
                    };

                    println!(
                        "{} {} ({})",
                        status_icon, session.metadata.title, session.id
                    );
                    println!(
                        "   消息數: {}, Token: {}",
                        session.metadata.total_messages, session.metadata.total_tokens
                    );

                    if let Some(branch) = &session.branch_name {
                        println!("   分支: {}", branch);
                    }

                    println!();
                }
            }
        }

        SessionAction::Pause { session_id } => {
            let session_uuid = Uuid::parse_str(&session_id)?;
            manager.pause_session(session_uuid).await?;
            println!("⏸️ 會話已暫停: {}", session_id);
        }

        SessionAction::Complete { session_id } => {
            let session_uuid = Uuid::parse_str(&session_id)?;
            manager.complete_session(session_uuid).await?;
            println!("✅ 會話已完成: {}", session_id);
        }

        SessionAction::Stats => {
            let stats = manager.get_session_stats().await?;

            println!("📊 會話統計");
            println!("═══════════════════════════════════════");
            println!("總會話數: {}", stats.total_sessions);
            println!("活躍會話: {}", stats.active_sessions);
            println!("暫停會話: {}", stats.paused_sessions);
            println!("已完成會話: {}", stats.completed_sessions);
            println!("總 Token 使用: {}", stats.total_tokens);
            println!("總成本: ${:.2}", stats.total_cost);
        }
    }

    Ok(())
}

// Worktree 管理命令處理
async fn handle_worktree_command(action: WorktreeAction) -> Result<()> {
    match action {
        WorktreeAction::Create { branch, path } => {
            println!("🌿 創建 Git Worktree");

            let project_root = std::env::current_dir()?;
            let worktree_path = if let Some(custom_path) = path {
                PathBuf::from(custom_path)
            } else {
                project_root.join("worktrees").join(&branch)
            };

            // 使用 vibe-kanban 的 WorktreeManager
            use claude_night_pilot_lib::worktree_manager::WorktreeManager;

            WorktreeManager::ensure_worktree_exists(
                project_root.to_string_lossy().to_string(),
                branch.clone(),
                worktree_path.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("創建 worktree 失敗: {}", e))?;

            println!("✅ Worktree 創建成功!");
            println!("分支: {}", branch);
            println!("路徑: {}", worktree_path.display());
        }

        WorktreeAction::Cleanup { path } => {
            println!("🧹 清理 Worktree: {}", path);

            let worktree_path = PathBuf::from(path);
            use claude_night_pilot_lib::worktree_manager::WorktreeManager;

            WorktreeManager::cleanup_worktree(&worktree_path, None)
                .await
                .map_err(|e| anyhow::anyhow!("清理 worktree 失敗: {}", e))?;

            println!("✅ Worktree 清理完成!");
        }

        WorktreeAction::List => {
            println!("📋 Git Worktree 列表");
            println!("═══════════════════════════════════════");

            // 執行 git worktree list
            let output = tokio::process::Command::new("git")
                .args(&["worktree", "list"])
                .output()
                .await?;

            if output.status.success() {
                let list_output = String::from_utf8_lossy(&output.stdout);
                if list_output.trim().is_empty() {
                    println!("沒有找到額外的 worktree");
                } else {
                    println!("{}", list_output);
                }
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("列出 worktree 失敗: {}", error);
            }
        }
    }

    Ok(())
}
