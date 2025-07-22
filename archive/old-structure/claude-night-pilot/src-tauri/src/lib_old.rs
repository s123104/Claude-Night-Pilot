use std::sync::Arc;
use tauri::{Manager, State};
use tauri_plugin_sql::{Migration, MigrationKind};

mod db;
mod executor;
mod scheduler;

use db::{CreatePromptRequest, Database, RunPromptRequest};
use executor::{ClaudeExecutor, CooldownInfo};
use scheduler::TaskScheduler;

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

// 應用程式狀態 - 簡化版本
pub struct AppState {
    // 使用 Tauri SQL plugin，不需要自己管理連接
}

// Tauri 命令定義

#[tauri::command]
async fn list_prompts(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    // 在開發模式下返回模擬資料
    if cfg!(debug_assertions) {
        let mock_prompts = vec![
            serde_json::json!({
                "id": 1,
                "title": "測試 Prompt",
                "content": "這是一個測試用的 prompt 內容",
                "tags": "test,demo",
                "created_at": "2025-01-22T04:00:00Z"
            }),
            serde_json::json!({
                "id": 2,
                "title": "Claude Code 範例",
                "content": "@claude-code-zh-tw.md 請分析這個文檔",
                "tags": "claude,analysis",
                "created_at": "2025-01-22T03:00:00Z"
            })
        ];
        return Ok(mock_prompts);
    }
    
    // 生產模式的實際資料庫查詢會在後面實作
    Ok(vec![])
}

#[tauri::command]
async fn create_prompt(
    app: tauri::AppHandle,
    title: String,
    content: String,
    tags: Option<String>,
) -> Result<i64, String> {
    // 在開發模式下返回模擬 ID
    if cfg!(debug_assertions) {
        println!("建立 Prompt: {} - {}", title, content);
        return Ok(999); // 模擬的 ID
    }
    
    // 生產模式的實際資料庫操作會在後面實作
    Ok(1)
}

#[tauri::command]
async fn delete_prompt(app: tauri::AppHandle, id: i64) -> Result<bool, String> {
    // 在開發模式下總是返回成功
    if cfg!(debug_assertions) {
        println!("刪除 Prompt ID: {}", id);
        return Ok(true);
    }
    
    // 生產模式的實際資料庫操作會在後面實作
    Ok(true)
}

#[tauri::command]
async fn run_prompt_sync(state: State<'_, AppState>, prompt_id: i64) -> Result<String, String> {
    // 獲取 prompt 內容
    let prompt = state
        .db
        .get_prompt(prompt_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Prompt 不存在")?;

    // 建立 job 記錄
    let job_req = RunPromptRequest {
        prompt_id,
        mode: "sync".to_string(),
        cron_expr: None,
    };
    let job_id = state
        .db
        .create_job(job_req)
        .await
        .map_err(|e| e.to_string())?;

    // 更新 job 狀態為 running
    state
        .db
        .update_job_status(job_id, "running", None)
        .await
        .map_err(|e| e.to_string())?;

    // 執行 Claude CLI（開發階段使用 mock）
    let result = if cfg!(debug_assertions) {
        ClaudeExecutor::run_mock(&prompt.content).await
    } else {
        ClaudeExecutor::run_sync(&prompt.content).await
    };

    match result {
        Ok(response) => {
            // 儲存結果
            state
                .db
                .create_result(job_id, &response)
                .await
                .map_err(|e| e.to_string())?;

            // 更新 job 狀態為 done
            state
                .db
                .update_job_status(job_id, "done", None)
                .await
                .map_err(|e| e.to_string())?;

            Ok(response)
        }
        Err(e) => {
            // 更新 job 狀態為 error
            state
                .db
                .update_job_status(job_id, "error", None)
                .await
                .map_err(|e| e.to_string())?;

            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_cooldown_status() -> Result<CooldownInfo, String> {
    if cfg!(debug_assertions) {
        // 開發階段返回模擬狀態
        Ok(CooldownInfo {
            is_cooling: false,
            seconds_remaining: 0,
            next_available_time: None,
        })
    } else {
        ClaudeExecutor::check_cooldown()
            .await
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn list_jobs(state: State<'_, AppState>) -> Result<Vec<db::Job>, String> {
    state.db.list_jobs().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_manual_job(state: State<'_, AppState>, job_id: i64) -> Result<String, String> {
    // 先獲取任務和對應的 prompt
    if let Ok(jobs) = state.db.list_jobs().await {
        if let Some(job) = jobs.iter().find(|j| j.id == Some(job_id)) {
            if let Ok(Some(prompt)) = state.db.get_prompt(job.prompt_id).await {
                return state
                    .scheduler
                    .execute_manual_job(job_id, &prompt.content)
                    .await
                    .map_err(|e| e.to_string());
            }
        }
    }
    Err("找不到指定的任務或 Prompt".to_string())
}

#[tauri::command]
async fn get_job_results(
    state: State<'_, AppState>,
    job_id: Option<i64>,
) -> Result<Vec<db::JobResult>, String> {
    state
        .db
        .list_results(job_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_job_stats(state: State<'_, AppState>) -> Result<(usize, usize), String> {
    state
        .scheduler
        .get_job_stats()
        .await
        .map_err(|e| e.to_string())
}


#[tauri::command]
async fn verify_claude_cli() -> Result<bool, String> {
    if cfg!(debug_assertions) {
        // 開發階段總是返回 true
        Ok(true)
    } else {
        ClaudeExecutor::verify_claude_cli()
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:claude-pilot.db", get_migrations())
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
                .setup(|app| {
            // 資料庫會通過 plugin migration 自動初始化
            println!("Claude Night Pilot 應用程式啟動成功！");
            
            // 初始化應用狀態 - 使用簡化的狀態管理
            let state = AppState {
                // 移除複雜的資料庫狀態，直接使用 Tauri SQL plugin
            };
            app.manage(state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_prompts,
            create_prompt,
            delete_prompt,
            run_prompt_sync,
            get_cooldown_status,
            list_jobs,
            execute_manual_job,
            get_job_results,
            get_job_stats,
            verify_claude_cli
        ])
        .run(tauri::generate_context!())
        .expect("啟動 Tauri 應用程式時發生錯誤");
}
