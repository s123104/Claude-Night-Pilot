use tauri_plugin_sql::{Migration, MigrationKind};
use std::sync::Arc;

// 宣告模組 - 公開讓 CLI 可以使用
pub mod db;
pub mod executor;
pub mod scheduler;
pub mod usage_tracker;
pub mod adaptive_monitor;
pub mod smart_scheduler;

// 新增核心模組系統
pub mod core;
pub mod enhanced_executor;
pub mod unified_interface;

// 取得資料庫遷移
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/0001_init.sql"),
            kind: MigrationKind::Up,
        }
    ]
}

// Tauri 命令定義 - 全部使用模擬資料
#[tauri::command]
async fn list_prompts(_app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let mock_prompts = vec![
        serde_json::json!({
            "id": 1,
            "title": "測試 Prompt",
            "content": "這是一個測試用的 prompt 內容",
            "tags": "test,demo",
            "created_at": "2025-07-22T21:41:13+08:00"
        }),
        serde_json::json!({
            "id": 2,
            "title": "Claude Code 範例", 
            "content": "@claude-code-zh-tw.md 請分析這個文檔",
            "tags": "claude,analysis",
            "created_at": "2025-07-22T20:41:13+08:00"
        }),
        serde_json::json!({
            "id": 3,
            "title": "CLI 整合測試",
            "content": "測試 CLI 和 GUI 的整合功能",
            "tags": "cli,integration,test",
            "created_at": "2025-07-22T19:41:13+08:00"
        })
    ];
    Ok(mock_prompts)
}

#[tauri::command]
async fn create_prompt(
    _app: tauri::AppHandle,
    title: String,
    content: String,
    _tags: Option<String>,
) -> Result<i64, String> {
    println!("建立 Prompt: {} - {}", title, content);
    Ok(999) // 模擬的 ID
}

#[tauri::command]
async fn delete_prompt(_app: tauri::AppHandle, id: i64) -> Result<bool, String> {
    println!("刪除 Prompt ID: {}", id);
    Ok(true)
}

#[tauri::command]
async fn run_prompt_sync(
    _app: tauri::AppHandle,
    prompt_id: i64,
    mode: String,
    cron_expr: Option<String>,
) -> Result<String, String> {
    println!("執行 Prompt ID: {}, 模式: {}", prompt_id, mode);
    
    if mode == "sync" {
        Ok("模擬的 Claude 回應：Hello from Claude! 這是一個測試回應。支援 CLI 整合功能已完成。".to_string())
    } else {
        Ok(format!("已建立排程任務，模式: {}, Cron: {:?}", mode, cron_expr))
    }
}

#[tauri::command]
async fn get_cooldown_status(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "is_cooling": false,
        "seconds_remaining": 0,
        "eta_human": "可立即執行",
        "last_check": "2025-07-22T21:41:13+08:00",
        "status_message": "Claude CLI 準備就緒",
        "cli_available": true,
        "last_checked": "2025-07-22T21:41:13+08:00"
    }))
}

#[tauri::command]
async fn create_scheduled_job(
    _app: tauri::AppHandle,
    prompt_id: i64,
    cron_expr: String,
    job_name: String,
) -> Result<i64, String> {
    println!("建立排程任務: {}, Prompt ID: {}, Cron: {}", job_name, prompt_id, cron_expr);
    Ok(456) // 模擬的 Job ID
}

#[tauri::command]
async fn list_jobs(_app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let mock_jobs = vec![
        serde_json::json!({
            "id": 1,
            "prompt_id": 1,
            "job_name": "每日自動分析",
            "cron_expr": "0 9 * * *",
            "status": "active",
            "last_run_at": "2025-07-22T09:00:00+08:00",
            "next_run_at": "2025-07-23T09:00:00+08:00",
            "created_at": "2025-07-22T21:41:13+08:00"
        }),
        serde_json::json!({
            "id": 2,
            "prompt_id": 2,
            "job_name": "週報生成",
            "cron_expr": "0 18 * * 5",
            "status": "pending",
            "last_run_at": null,
            "next_run_at": "2025-07-25T18:00:00+08:00",
            "created_at": "2025-07-22T20:41:13+08:00"
        })
    ];
    Ok(mock_jobs)
}

#[tauri::command]
async fn get_job_results(
    _app: tauri::AppHandle,
    job_id: i64,
    limit: Option<i64>,
) -> Result<Vec<serde_json::Value>, String> {
    let limit = limit.unwrap_or(10);
    println!("取得 Job {} 的結果，限制 {} 筆", job_id, limit);
    
    let mock_results = vec![
        serde_json::json!({
            "id": 1,
            "job_id": job_id,
            "content": "執行成功！分析結果：系統運行正常，性能指標在預期範圍內。",
            "status": "success",
            "execution_time": 1.25,
            "created_at": "2025-07-22T21:41:13+08:00"
        }),
        serde_json::json!({
            "id": 2,
            "job_id": job_id,
            "content": "執行失敗：Claude API 冷卻中，預計 15 分鐘後重試。",
            "status": "failed",
            "execution_time": 0.1,
            "created_at": "2025-07-22T20:41:13+08:00"
        })
    ];
    Ok(mock_results)
}

#[tauri::command]
async fn get_system_info(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "app_name": "Claude Night Pilot",
        "app_version": "0.1.0",
        "tauri_version": "2.0",
        "database_status": "connected",
        "claude_cli_status": "available",
        "last_updated": "2025-07-22T21:41:13+08:00",
        "features": {
            "scheduler": true,
            "notifications": true,
            "cli_integration": true,
            "auto_updates": false
        },
        "cli_integrated": true
    }))
}

// 新增的 CLI 命令執行功能
#[tauri::command]
async fn run_cli_command(command: String, args: Vec<String>) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("cnp")
        .arg(&command)
        .args(&args)
        .output()
        .map_err(|e| format!("執行 CLI 命令失敗: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// 統一的Claude執行命令 (替代run_prompt_sync)
#[tauri::command]
async fn execute_unified_claude(
    prompt: String,
    options: unified_interface::UnifiedExecutionOptions,
) -> Result<enhanced_executor::EnhancedClaudeResponse, String> {
    unified_interface::UnifiedClaudeInterface::execute_claude(prompt, options)
        .await
        .map_err(|e| e.to_string())
}

// 統一的冷卻狀態檢查 (替代get_cooldown_status)
#[tauri::command]
async fn get_unified_cooldown_status() -> Result<core::CooldownInfo, String> {
    unified_interface::UnifiedClaudeInterface::check_cooldown()
        .await
        .map_err(|e| e.to_string())
}

// 統一的系統健康檢查 (增強版get_system_info)
#[tauri::command]
async fn get_unified_system_health() -> Result<serde_json::Value, String> {
    unified_interface::UnifiedClaudeInterface::health_check()
        .await
        .map_err(|e| e.to_string())
}

// 初始化排程器
async fn initialize_scheduler() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::db::Database;
    use crate::scheduler::TaskScheduler;
    
    println!("🚀 正在初始化 TaskScheduler...");
    
    // 創建數據庫連接
    let db = Arc::new(Database::new("sqlite:claude-pilot.db").await?);
    
    // 創建並啟動排程器
    let scheduler = TaskScheduler::new(db.clone()).await?;
    scheduler.start().await?;
    
    // 載入所有待執行的 cron 任務
    load_pending_cron_jobs(&scheduler, db).await?;
    
    println!("✅ TaskScheduler 初始化完成");
    
    // 防止排程器被回收 - 保持運行
    tokio::spawn(async move {
        let _scheduler = scheduler; // 保持 scheduler 存活
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // 每小時檢查一次
        }
    });
    
    Ok(())
}

// 載入待執行的 cron 任務
async fn load_pending_cron_jobs(
    scheduler: &crate::scheduler::TaskScheduler, 
    db: Arc<crate::db::Database>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📋 載入待執行的 cron 任務...");
    
    // 獲取所有任務
    let all_jobs = db.list_jobs().await?;
    let mut cron_jobs_count = 0;
    
    for job in all_jobs {
        // 只處理 async 模式、pending 狀態且有 cron 表達式的任務
        if job.mode == "async" && job.status == "pending" && job.cron_expr != "*" {
            // 獲取關聯的 prompt
            if let Some(prompt) = db.get_prompt(job.prompt_id).await? {
                match scheduler.register_cron_job(&job, &prompt.content).await {
                    Ok(_) => {
                        cron_jobs_count += 1;
                        println!("✅ 註冊 Cron 任務: ID {} ({})", job.id.unwrap_or(0), job.cron_expr);
                    }
                    Err(e) => {
                        eprintln!("❌ 註冊 Cron 任務失敗: {}", e);
                    }
                }
            }
        }
    }
    
    println!("📊 已載入 {} 個 Cron 任務", cron_jobs_count);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:claude-pilot.db", get_migrations())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            println!("Claude Night Pilot 啟動中...");
            println!("當前時間：2025-07-22T21:55:57+08:00");
            println!("CLI 整合狀態：已啟用");
            
            // 初始化並啟動排程器
            let _app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                match initialize_scheduler().await {
                    Ok(_) => println!("✅ 排程器啟動成功"),
                    Err(e) => eprintln!("❌ 排程器啟動失敗: {}", e),
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 基礎資料管理命令 (保留用於向後兼容)
            list_prompts,
            create_prompt, 
            delete_prompt,
            create_scheduled_job,
            list_jobs,
            get_job_results,
            get_system_info,
            run_cli_command,
            // 統一介面命令 (推薦使用)
            execute_unified_claude,
            get_unified_cooldown_status,
            get_unified_system_health,
            // 增強執行器命令 (低層級存取)
            enhanced_executor::execute_enhanced_claude,
            enhanced_executor::check_enhanced_cooldown,
            enhanced_executor::health_check_enhanced
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
} 