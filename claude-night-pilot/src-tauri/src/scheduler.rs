use crate::db::{Database, Job};
use crate::executor::ClaudeExecutor;
use anyhow::Result;
use std::sync::Arc;
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

pub struct TaskScheduler {
    scheduler: JobScheduler,
    db: Arc<Database>,
}

impl TaskScheduler {
    pub async fn new(db: Arc<Database>) -> Result<Self> {
        let scheduler = JobScheduler::new().await?;

        Ok(TaskScheduler { scheduler, db })
    }

    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;
        println!("Task scheduler started");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.scheduler.shutdown().await?;
        println!("Task scheduler stopped");
        Ok(())
    }

    /// 註冊新的 Cron 任務
    pub async fn register_cron_job(&self, job: &Job, prompt_content: &str) -> Result<()> {
        let job_id = job.id.unwrap_or(0);
        let cron_expr = job.cron_expr.clone();
        let prompt = prompt_content.to_string();
        let db = Arc::clone(&self.db);

        // 如果是手動執行 (cron_expr = "*")，直接返回
        if cron_expr == "*" {
            return Ok(());
        }

        let cron_job = CronJob::new_async(&cron_expr, move |_uuid, _l| {
            let prompt = prompt.clone();
            let db = Arc::clone(&db);

            Box::pin(async move {
                println!("🚀 執行排程任務 ID: {}", job_id);

                // 更新任務狀態為執行中
                if let Err(e) = db.update_job_status(job_id, "running", None).await {
                    eprintln!("❌ 更新任務狀態失敗: {}", e);
                    return;
                }

                // 執行 Claude CLI
                match ClaudeExecutor::run_sync(&prompt).await {
                    Ok(result) => {
                        println!("✅ 任務 {} 執行成功", job_id);

                        // 保存結果
                        if let Err(e) = db.create_result(job_id, &result).await {
                            eprintln!("❌ 保存結果失敗: {}", e);
                        }

                        // 更新任務狀態為完成
                        if let Err(e) = db.update_job_status(job_id, "done", None).await {
                            eprintln!("❌ 更新任務狀態失敗: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ 任務 {} 執行失敗: {}", job_id, e);

                        // 檢查是否為冷卻錯誤
                        let error_msg = e.to_string();
                        if let Some(eta) = ClaudeExecutor::parse_cooldown_from_error(&error_msg) {
                            // 更新 ETA 並保持 pending 狀態
                            if let Err(e) = db
                                .update_job_status(job_id, "pending", Some(eta as i64))
                                .await
                            {
                                eprintln!("❌ 更新冷卻時間失敗: {}", e);
                            }
                        } else {
                            // 其他錯誤，標記為失敗
                            if let Err(e) = db.update_job_status(job_id, "error", None).await {
                                eprintln!("❌ 更新任務狀態失敗: {}", e);
                            }

                            // 保存錯誤訊息
                            if let Err(e) = db
                                .create_result(job_id, &format!("錯誤: {}", error_msg))
                                .await
                            {
                                eprintln!("❌ 保存錯誤結果失敗: {}", e);
                            }
                        }
                    }
                }
            })
        })?;

        self.scheduler.add(cron_job).await?;
        println!("📅 已註冊 Cron 任務: {} (ID: {})", cron_expr, job_id);

        Ok(())
    }

    /// 載入所有待執行的排程任務
    pub async fn load_pending_jobs(&self) -> Result<()> {
        let jobs = self.db.get_pending_jobs().await?;

        for job in jobs {
            if let Some(job_id) = job.id {
                // 獲取 prompt 內容
                if let Ok(Some(prompt)) = self.db.get_prompt(job.prompt_id).await {
                    if let Err(e) = self.register_cron_job(&job, &prompt.content).await {
                        eprintln!("❌ 註冊任務 {} 失敗: {}", job_id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 手動執行任務
    pub async fn execute_manual_job(&self, job_id: i64, prompt_content: &str) -> Result<String> {
        println!("🎯 手動執行任務 ID: {}", job_id);

        // 更新任務狀態為執行中
        self.db.update_job_status(job_id, "running", None).await?;

        // 執行 Claude CLI
        match ClaudeExecutor::run_sync(prompt_content).await {
            Ok(result) => {
                println!("✅ 手動任務 {} 執行成功", job_id);

                // 保存結果
                self.db.create_result(job_id, &result).await?;

                // 更新任務狀態為完成
                self.db.update_job_status(job_id, "done", None).await?;

                Ok(result)
            }
            Err(e) => {
                eprintln!("❌ 手動任務 {} 執行失敗: {}", job_id, e);

                let error_msg = e.to_string();

                // 檢查是否為冷卻錯誤
                if let Some(eta) = ClaudeExecutor::parse_cooldown_from_error(&error_msg) {
                    // 更新 ETA 並保持 pending 狀態
                    self.db
                        .update_job_status(job_id, "pending", Some(eta as i64))
                        .await?;
                    Err(anyhow::anyhow!("Claude API 冷卻中，預計 {} 秒後可用", eta))
                } else {
                    // 其他錯誤，標記為失敗
                    self.db.update_job_status(job_id, "error", None).await?;

                    // 保存錯誤訊息
                    self.db
                        .create_result(job_id, &format!("錯誤: {}", error_msg))
                        .await?;

                    Err(e)
                }
            }
        }
    }

    /// 獲取任務統計
    pub async fn get_job_stats(&self) -> Result<(usize, usize)> {
        // 返回 (活躍任務數, 總任務數)
        let jobs = self.db.list_jobs().await?;
        let active_jobs = jobs
            .iter()
            .filter(|j| j.status == "running" || j.status == "pending")
            .count();
        Ok((active_jobs, jobs.len()))
    }
}
