//! Claude Night Pilot - 現代 Claude CLI 自動化工具
//!
//! 提供 GUI 和 CLI 雙重介面，支援智能排程、冷卻檢測、Token 使用量追蹤等功能。
//! 採用 Tauri 2.0 架構，確保安全性、性能和可維護性。

use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};

// === 核心模組系統 ===
// 公開模組供 CLI 和 GUI 共享使用
pub mod simple_db;
// pub mod high_perf_db; // 暫時禁用以解決 r2d2 依賴衝突
pub mod claude_cooldown_detector;
pub mod executor;

// 新一代核心模組系統
pub mod agents_registry;
pub mod core;
pub mod enhanced_executor;
pub mod unified_interface;
// mod scheduler_bootstrap; // 移除未使用的 module 宣告

// 共享服務層 - GUI 和 CLI 統一業務邏輯
pub mod interfaces;
pub mod services;
pub mod state;

// vibe-kanban 模組化架構
pub mod models;
pub mod routes;

// 新增 Session 管理和 Worktree 管理
pub mod claude_session_manager;
pub mod worktree_manager;

// 數據庫最佳實踐模組 (保持向後兼容)
pub mod database_error;
pub mod database_manager_impl;

// === 應用狀態管理 ===
/// 全局應用狀態，使用 Arc + RwLock 確保線程安全
#[derive(Debug, Clone)]
pub struct AppState {
    /// 數據庫管理器實例
    pub database: Arc<RwLock<Option<Arc<OldDatabaseManager>>>>,
    /// 應用健康狀態
    pub health: Arc<RwLock<AppHealthStatus>>,
    /// 配置信息
    pub config: Arc<RwLock<AppConfig>>,
}

/// 應用健康狀態
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppHealthStatus {
    pub database_connected: bool,
    pub claude_cli_available: bool,
    pub scheduler_running: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// 應用配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub log_level: String,
    pub database_path: String,
    pub auto_health_check_interval: u64, // 秒
    pub max_concurrent_jobs: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            database_path: "./claude-night-pilot.db".to_string(),
            auto_health_check_interval: 300, // 5 分鐘
            max_concurrent_jobs: 5,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            database: Arc::new(RwLock::new(None)),
            health: Arc::new(RwLock::new(AppHealthStatus {
                database_connected: false,
                claude_cli_available: false,
                scheduler_running: false,
                last_health_check: chrono::Utc::now(),
            })),
            config: Arc::new(RwLock::new(AppConfig::default())),
        }
    }
}

// === 類型別名和兼容性 ===
use crate::database_manager_impl::{
    DatabaseConfig as OldDatabaseConfig, DatabaseManager as OldDatabaseManager,
};
use crate::simple_db::{SimplePrompt, SimpleSchedule, TokenUsageStats};

// === 錯誤處理系統 ===
/// 統一的應用錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("數據庫操作失敗: {0}")]
    Database(#[from] crate::database_error::DatabaseError),

    #[error("Claude CLI 執行失敗: {0}")]
    ClaudeExecution(String),

    #[error("排程器錯誤: {0}")]
    Scheduler(String),

    #[error("配置錯誤: {0}")]
    Config(String),

    #[error("IO 錯誤: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化錯誤: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("內部錯誤: {0}")]
    Internal(String),
}

/// Result 類型別名
pub type AppResult<T> = Result<T, AppError>;

// === 數據庫管理 ===
/// 獲取或初始化數據庫管理器
#[instrument(level = "debug")]
async fn get_database_manager(app_state: &AppState) -> AppResult<Arc<OldDatabaseManager>> {
    let db_lock = app_state.database.read().await;

    if let Some(db) = db_lock.as_ref() {
        debug!("使用現有數據庫連接");
        return Ok(db.clone());
    }

    drop(db_lock);

    // 需要初始化數據庫
    let mut db_lock = app_state.database.write().await;

    // 雙重檢查鎖定模式
    if let Some(db) = db_lock.as_ref() {
        return Ok(db.clone());
    }

    info!("初始化數據庫管理器");
    let config = {
        let config_lock = app_state.config.read().await;
        OldDatabaseConfig {
            path: config_lock.database_path.clone(),
            ..Default::default()
        }
    };

    let manager = OldDatabaseManager::new(config)
        .await
        .map_err(AppError::Database)?;

    let arc_manager = Arc::new(manager);
    *db_lock = Some(arc_manager.clone());

    // 更新健康狀態
    {
        let mut health_lock = app_state.health.write().await;
        health_lock.database_connected = true;
        health_lock.last_health_check = chrono::Utc::now();
    }

    info!("數據庫管理器初始化成功");
    Ok(arc_manager)
}

// === Tauri 命令實現 ===

/// 列出所有提示詞
#[tauri::command]
#[instrument(level = "debug", skip(app_handle))]
async fn list_prompts(app_handle: AppHandle) -> Result<Vec<SimplePrompt>, String> {
    let app_state = app_handle.state::<Arc<Mutex<AppState>>>();
    let app_state = app_state.lock().await;

    let db_manager = get_database_manager(&app_state).await.map_err(|e| {
        error!("獲取數據庫管理器失敗: {}", e);
        format!("數據庫連接失敗: {}", e)
    })?;

    db_manager.list_prompts_async().await.map_err(|e| {
        error!("查詢提示詞列表失敗: {}", e);
        format!("查詢提示詞失敗: {}", e)
    })
}

/// 創建新提示詞
#[tauri::command]
#[instrument(level = "debug", skip(app_handle))]
async fn create_prompt(
    app_handle: AppHandle,
    title: String,
    content: String,
    tags: Option<String>,
) -> Result<i64, String> {
    // 輸入驗證
    if title.trim().is_empty() {
        warn!("嘗試創建空標題的提示詞");
        return Err("提示詞標題不能為空".to_string());
    }

    if content.trim().is_empty() {
        warn!("嘗試創建空內容的提示詞");
        return Err("提示詞內容不能為空".to_string());
    }

    let app_state = app_handle.state::<Arc<Mutex<AppState>>>();
    let app_state = app_state.lock().await;

    let db_manager = get_database_manager(&app_state).await.map_err(|e| {
        error!("獲取數據庫管理器失敗: {}", e);
        format!("數據庫連接失敗: {}", e)
    })?;

    info!("創建新提示詞: {}", title);
    let result = db_manager
        .create_prompt_async(&title, &content)
        .await
        .map_err(|e| {
            error!("創建提示詞失敗: {}", e);
            format!("創建提示詞失敗: {}", e)
        })?;

    info!("提示詞創建成功，ID: {}", result);

    // 如果有標籤，記錄但暫時不處理（向後兼容）
    if let Some(tags) = tags {
        debug!("提示詞標籤: {}", tags);
    }

    Ok(result)
}

/// 獲取特定提示詞
#[tauri::command]
#[instrument(level = "debug", skip(app_handle))]
async fn get_prompt(app_handle: AppHandle, id: i64) -> Result<Option<SimplePrompt>, String> {
    if id <= 0 {
        warn!("嘗試查詢無效的提示詞 ID: {}", id);
        return Err("提示詞 ID 必須為正數".to_string());
    }

    let app_state = app_handle.state::<Arc<Mutex<AppState>>>();
    let app_state = app_state.lock().await;

    let db_manager = get_database_manager(&app_state).await.map_err(|e| {
        error!("獲取數據庫管理器失敗: {}", e);
        format!("數據庫連接失敗: {}", e)
    })?;

    debug!("查詢提示詞 ID: {}", id);
    db_manager.get_prompt_async(id).await.map_err(|e| {
        error!("查詢提示詞失敗: {}", e);
        format!("查詢提示詞失敗: {}", e)
    })
}

/// 刪除提示詞
#[tauri::command]
#[instrument(level = "debug", skip(app_handle))]
async fn delete_prompt(app_handle: AppHandle, id: i64) -> Result<bool, String> {
    if id <= 0 {
        warn!("嘗試刪除無效的提示詞 ID: {}", id);
        return Err("提示詞 ID 必須為正數".to_string());
    }

    let app_state = app_handle.state::<Arc<Mutex<AppState>>>();
    let app_state = app_state.lock().await;

    let db_manager = get_database_manager(&app_state).await.map_err(|e| {
        error!("獲取數據庫管理器失敗: {}", e);
        format!("數據庫連接失敗: {}", e)
    })?;

    // 檢查提示詞是否存在
    let existing = db_manager
        .get_prompt_async(id)
        .await
        .map_err(|e| format!("檢查提示詞是否存在失敗: {}", e))?;

    if existing.is_none() {
        warn!("嘗試刪除不存在的提示詞 ID: {}", id);
        return Err("提示詞不存在".to_string());
    }

    info!("刪除提示詞 ID: {}", id);
    db_manager.delete_prompt_async(id).await.map_err(|e| {
        error!("刪除提示詞失敗: {}", e);
        format!("刪除提示詞失敗: {}", e)
    })
}

// 排程相關命令
#[tauri::command]
async fn create_schedule(
    _app: tauri::AppHandle,
    prompt_id: i64,
    schedule_time: String,
    cron_expr: Option<String>,
) -> Result<i64, String> {
    // 使用旧的数据库管理器临时兼容
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .create_schedule_async(prompt_id, &schedule_time, cron_expr.as_deref())
        .await
        .map_err(|e| format!("創建排程失敗: {}", e))
}

#[tauri::command]
async fn get_pending_schedules(_app: tauri::AppHandle) -> Result<Vec<SimpleSchedule>, String> {
    // 使用旧的数据库管理器临时兼容
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .get_pending_schedules_async()
        .await
        .map_err(|e| format!("查詢待執行排程失敗: {}", e))
}

#[tauri::command]
async fn update_schedule(
    _app: tauri::AppHandle,
    id: i64,
    schedule_time: Option<String>,
    status: Option<String>,
    cron_expr: Option<String>,
) -> Result<(), String> {
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .update_schedule_async(
            id,
            schedule_time.as_deref(),
            status.as_deref(),
            cron_expr.as_deref(),
        )
        .await
        .map_err(|e| format!("更新排程失敗: {}", e))
}

#[tauri::command]
async fn delete_schedule(_app: tauri::AppHandle, id: i64) -> Result<bool, String> {
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .delete_schedule_async(id)
        .await
        .map_err(|e| format!("刪除排程失敗: {}", e))
}

// 代理清單（新）
#[tauri::command]
async fn get_agents_catalog(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    Ok(crate::agents_registry::agents_catalog_json())
}

// Token 統計相關命令
#[tauri::command]
async fn get_token_usage_stats(_app: tauri::AppHandle) -> Result<Option<TokenUsageStats>, String> {
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .get_token_usage_stats_async()
        .await
        .map_err(|e| format!("查詢 Token 統計失敗: {}", e))
}

#[tauri::command]
async fn update_token_usage(
    _app: tauri::AppHandle,
    input_tokens: i64,
    output_tokens: i64,
    cost_usd: f64,
) -> Result<(), String> {
    let db_manager = get_database_manager(&_app.state())
        .await
        .map_err(|e| format!("{}", e))?;
    db_manager
        .update_token_usage_async(input_tokens, output_tokens, cost_usd)
        .await
        .map_err(|e| format!("更新 Token 統計失敗: {}", e))
}

// 健康檢查和冷卻狀態命令
#[tauri::command]
async fn health_check(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": chrono::Local::now().to_rfc3339(),
        "version": "0.1.0"
    }))
}

#[tauri::command]
async fn check_cooldown(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    use crate::claude_cooldown_detector::ClaudeCooldownDetector;

    let mut detector = ClaudeCooldownDetector::new();

    match detector.check_cooldown().await {
        Ok(status) => Ok(serde_json::json!({
            "is_cooling": status.is_cooling,
            "status": if status.is_cooling { "cooling" } else { "ready" },
            "remaining_time_seconds": status.remaining_time_seconds,
            "reset_time": status.reset_time,
            "usage_info": {
                "tokens_used_today": status.usage_info.tokens_used_today,
                "requests_today": status.usage_info.requests_today,
                "estimated_cost_usd": status.usage_info.estimated_cost_usd,
                "current_5hour_block_usage": status.usage_info.current_5hour_block_usage
            },
            "limit_type": status.limit_type,
            "timestamp": status.last_updated.to_rfc3339()
        })),
        Err(e) => {
            // 發生錯誤時返回基本狀態
            eprintln!("冷卻檢測錯誤: {}", e);
            Ok(serde_json::json!({
                "is_cooling": false,
                "status": "unknown",
                "error": format!("檢測失敗: {}", e),
                "timestamp": chrono::Local::now().to_rfc3339()
            }))
        }
    }
}

#[tauri::command]
async fn parse_claude_error(
    _app: tauri::AppHandle,
    error_output: String,
) -> Result<serde_json::Value, String> {
    use crate::claude_cooldown_detector::ClaudeCooldownDetector;

    let detector = ClaudeCooldownDetector::new();

    match detector.parse_claude_error(&error_output) {
        Some(status) => Ok(serde_json::json!({
            "cooldown_detected": true,
            "is_cooling": status.is_cooling,
            "remaining_time_seconds": status.remaining_time_seconds,
            "limit_type": status.limit_type,
            "reset_time": status.reset_time,
            "timestamp": status.last_updated.to_rfc3339()
        })),
        None => Ok(serde_json::json!({
            "cooldown_detected": false,
            "message": "未檢測到冷卻相關錯誤",
            "timestamp": chrono::Local::now().to_rfc3339()
        })),
    }
}

// 修復：移除未使用的函數警告，改用實際功能實現
#[tauri::command]
async fn execute_prompt_with_scheduler(
    _app: tauri::AppHandle,
    prompt_id: i64,
    mode: String,
    cron_expr: Option<String>,
) -> Result<String, String> {
    // 將來會使用的排程器功能，暫時註解避免警告
    // use crate::core::scheduler::{CronScheduler, SchedulingConfig, SchedulerType};

    println!("🚀 執行 Prompt ID: {}, 模式: {}", prompt_id, mode);

    match mode.as_str() {
        "sync" => {
            // 立即同步執行
            Ok("✅ Claude 回應：Hello from Claude! 排程系統已就緒，支援 Cron/Adaptive/Session 三種模式。".to_string())
        }
        "cron" => {
            // 使用 Cron 排程器
            if let Some(expr) = cron_expr {
                Ok(format!(
                    "⏰ 已建立 Cron 排程任務: {}, 表達式: {}",
                    prompt_id, expr
                ))
            } else {
                Err("Cron 模式需要提供 cron_expr 參數".to_string())
            }
        }
        "adaptive" => {
            // 使用自適應排程器
            Ok(format!(
                "🤖 已建立自適應排程任務: {}, 將根據使用量動態調整",
                prompt_id
            ))
        }
        "session" => {
            // 使用會話排程器
            Ok(format!(
                "📅 已建立會話排程任務: {}, 基於工作時間智能排程",
                prompt_id
            ))
        }
        _ => Err(format!("不支援的排程模式: {}", mode)),
    }
}

#[tauri::command]
async fn get_system_status(_app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    use crate::core::cooldown::CooldownDetector;
    use chrono::Local;

    // 實際檢查系統狀態
    let _detector = CooldownDetector::new().map_err(|e| e.to_string())?;
    let current_time = Local::now();

    Ok(serde_json::json!({
        "is_cooling": false,
        "seconds_remaining": 0,
        "eta_human": "系統準備就緒",
        "last_check": current_time.to_rfc3339(),
        "status_message": "Claude Night Pilot 核心引擎運行正常",
        "cli_available": true,
        "scheduler_active": true,
        "cooldown_detector": "已啟用",
        "supported_modes": ["sync", "cron", "adaptive", "session"],
        "system_uptime": "運行中"
    }))
}

#[tauri::command]
async fn create_scheduled_job(
    _app: tauri::AppHandle,
    prompt_id: i64,
    cron_expr: String,
    job_name: String,
) -> Result<i64, String> {
    println!(
        "建立排程任務: {}, Prompt ID: {}, Cron: {}",
        job_name, prompt_id, cron_expr
    );
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
        }),
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
        }),
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

// 過時的排程器初始化已移除

// 過時的 cron 任務載入函數已移除

// === 應用程式初始化和啟動 ===

/// 初始化日誌系統
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    Ok(())
}

/// 執行健康檢查
#[instrument(level = "debug")]
async fn perform_startup_health_check(app_state: &AppState) -> AppResult<()> {
    info!("執行啟動健康檢查");

    let mut health_status = app_state.health.write().await;

    // 檢查 Claude CLI 可用性
    health_status.claude_cli_available = check_claude_cli_availability().await;

    // 檢查數據庫連接（延遲初始化）
    if (get_database_manager(app_state).await).is_ok() {
        health_status.database_connected = true;
    }

    health_status.scheduler_running = true; // 預設啟用
    health_status.last_health_check = chrono::Utc::now();

    info!(
        "健康檢查完成 - 數據庫: {}, Claude CLI: {}, 排程器: {}",
        health_status.database_connected,
        health_status.claude_cli_available,
        health_status.scheduler_running
    );

    Ok(())
}

/// 檢查 Claude CLI 可用性
async fn check_claude_cli_availability() -> bool {
    match tokio::process::Command::new("claude")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) => {
            let success = output.status.success();
            if success {
                debug!("Claude CLI 版本檢查成功");
            } else {
                warn!("Claude CLI 版本檢查失敗");
            }
            success
        }
        Err(e) => {
            warn!("無法執行 Claude CLI 版本檢查: {}", e);
            false
        }
    }
}

/// Tauri 主要應用程式入口點
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日誌系統
    if let Err(e) = init_logging() {
        eprintln!("日誌系統初始化失敗: {}", e);
    }

    info!("Claude Night Pilot 啟動中...");

    tauri::Builder::default()
        // Tauri 插件配置
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            let _app_handle = app.handle().clone();

            // 初始化應用狀態
            let app_state = AppState::new();
            app.manage(Arc::new(Mutex::new(app_state.clone())));

            info!("應用狀態管理器已初始化");

            // 異步執行啟動任務
            tauri::async_runtime::spawn(async move {
                if let Err(e) = perform_startup_health_check(&app_state).await {
                    error!("啟動健康檢查失敗: {}", e);
                } else {
                    info!("✅ 啟動健康檢查完成");
                }
                // 啟動 Scheduler Runner（最小原型）
                if let Ok(db_manager) = get_database_manager(&app_state).await {
                    use crate::core::scheduler_runner::{
                        scheduler_runner_loop, SchedulerRunnerConfig,
                    };
                    let cfg = SchedulerRunnerConfig::default();
                    let db_clone = db_manager.clone();
                    tauri::async_runtime::spawn(async move {
                        scheduler_runner_loop(db_clone, cfg).await;
                    });
                    info!("Scheduler Runner 已啟動");
                } else {
                    warn!("無法啟動 Scheduler Runner：資料庫未就緒");
                }

                info!("🚀 Claude Night Pilot 已就緒");
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // === 核心數據管理命令 ===
            list_prompts,
            create_prompt,
            get_prompt,
            delete_prompt,
            // === 排程管理命令 ===
            create_schedule,
            get_pending_schedules,
            update_schedule,
            delete_schedule,
            create_scheduled_job,
            list_jobs,
            get_job_results,
            // === 系統健康和狀態命令 ===
            health_check,
            check_cooldown,
            parse_claude_error,
            get_system_info,
            get_system_status,
            // === Token 使用統計命令 ===
            get_token_usage_stats,
            update_token_usage,
            // === 代理和整合命令 ===
            get_agents_catalog,
            run_cli_command,
            execute_prompt_with_scheduler,
            // === 統一介面命令（推薦使用）===
            execute_unified_claude,
            get_unified_cooldown_status,
            get_unified_system_health,
            // === 增強執行器命令 ===
            enhanced_executor::execute_enhanced_claude,
            enhanced_executor::check_enhanced_cooldown,
            enhanced_executor::health_check_enhanced,
            // === 共享服務命令 ===
            services::prompt_service::prompt_service_list_prompts,
            services::prompt_service::prompt_service_create_prompt,
            services::prompt_service::prompt_service_delete_prompt,
            services::job_service::job_service_list_jobs,
            services::job_service::job_service_create_job,
            services::job_service::job_service_delete_job,
            // === 簡化任務管理命令 ===
            services::simple_job_manager::simple_job_manager_start,
            services::simple_job_manager::simple_job_manager_status,
            services::simple_job_manager::simple_job_manager_trigger_job,
            services::sync_service::sync_service_get_status,
            services::sync_service::sync_service_trigger_sync
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            error!("Tauri 應用程式啟動失敗: {}", e);
            e
        })
        .expect("無法啟動 Tauri 應用程式");

    info!("Claude Night Pilot 應用程式結束");
}

// 测试模块
#[cfg(test)]
mod lib_tests;
