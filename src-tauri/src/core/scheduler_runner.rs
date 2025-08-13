use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn, error, debug};
use std::sync::Arc;

use crate::database_manager_impl::DatabaseManager as OldDatabaseManager;
use crate::enhanced_executor::EnhancedClaudeExecutor;
use crate::unified_interface::UnifiedExecutionOptions;

/// 背景排程器最小原型：啟動後週期輪詢並派發到期任務
pub struct SchedulerRunnerConfig {
    pub enabled: bool,
    pub poll_interval_seconds: u64,
    pub max_batch: usize,
}

impl Default for SchedulerRunnerConfig {
    fn default() -> Self {
        Self { enabled: true, poll_interval_seconds: 15, max_batch: 3 }
    }
}

pub async fn scheduler_runner_loop(db: Arc<OldDatabaseManager>, cfg: SchedulerRunnerConfig) {
    if !cfg.enabled {
        info!("SchedulerRunner disabled");
        return;
    }

    info!("SchedulerRunner started: interval={}s batch={}", cfg.poll_interval_seconds, cfg.max_batch);

    loop {
        if let Err(e) = tick_once(db.as_ref(), cfg.max_batch).await {
            error!("SchedulerRunner tick error: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(cfg.poll_interval_seconds)).await;
    }
}

async fn tick_once(db: &OldDatabaseManager, max_batch: usize) -> Result<()> {
    // 取得待執行排程（向後相容 API）
    let mut pending = db.get_pending_schedules_async().await?;
    if pending.is_empty() {
        debug!("No pending schedules");
        return Ok(());
    }

    // 取前 N 筆避免一次性執行過多
    pending.truncate(max_batch);
    info!("Dispatching {} schedules", pending.len());

    for sch in pending {
        let schedule_id = sch.id;
        let prompt_id = sch.prompt_id;
        let scheduled_at = sch.schedule_time.clone();
        debug!("Executing schedule id={} prompt_id={} at {}", schedule_id, prompt_id, scheduled_at);

        // 讀取 prompt 內容
        let prompt = match db.get_prompt_async(prompt_id).await? {
            Some(p) => p.content,
            None => {
                warn!("Prompt {} not found for schedule {}", prompt_id, schedule_id);
                continue;
            }
        };

        // 使用增強執行器（內含 smart-wait 冷卻等待）
        let mut exec = EnhancedClaudeExecutor::with_smart_defaults()?;
        let options = UnifiedExecutionOptions {
            mode: "sync".to_string(),
            cron_expr: None,
            retry_enabled: Some(true),
            cooldown_check: Some(true),
            working_directory: None,
        };

        let started = std::time::Instant::now();
        match exec.execute_with_full_enhancement(&prompt, options.into()).await {
            Ok(resp) => {
                let elapsed_ms = started.elapsed().as_millis() as i64;
                // 記錄執行結果（最小化）
                let _ = db.record_execution_result_async(
                    schedule_id,
                    &resp.completion,
                    "success",
                    None,
                    None,
                    elapsed_ms,
                ).await;
                // 單次任務：標記完成；週期任務：此處先標記完成（之後擴充 cron 計算）
                let now_str = Utc::now().to_rfc3339();
                let _ = db.update_schedule_async(schedule_id, None, Some("completed"), None).await;
                debug!("Schedule {} completed at {}", schedule_id, now_str);
            }
            Err(e) => {
                let elapsed_ms = started.elapsed().as_millis() as i64;
                let _ = db.record_execution_result_async(
                    schedule_id,
                    &format!("error: {}", e),
                    "failed",
                    None,
                    None,
                    elapsed_ms,
                ).await;
                // 簡單標記為 pending（由 retry 策略與下一輪再嘗試，後續可回寫 next_run_at）
                let _ = db.update_schedule_async(schedule_id, None, Some("pending"), None).await;
                warn!("Schedule {} failed: {}", schedule_id, e);
            }
        }
    }

    Ok(())
}


