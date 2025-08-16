// Job服務 - GUI和CLI共享排程管理邏輯
use crate::database_manager_impl::DatabaseManager as OldDatabaseManager;
use crate::simple_db::SimpleSchedule;
use crate::state::AppStateManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateJobRequest {
    pub prompt_id: i64,
    pub name: String,
    pub cron_expression: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateJobRequest {
    pub id: i64,
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobServiceResponse {
    pub id: i64,
    pub prompt_id: i64,
    pub name: String,
    pub cron_expression: String,
    pub status: String,
    pub description: Option<String>,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<SimpleSchedule> for JobServiceResponse {
    fn from(schedule: SimpleSchedule) -> Self {
        Self {
            id: schedule.id,
            prompt_id: schedule.prompt_id,
            name: schedule.schedule_time.clone(), // 使用schedule_time作為name
            cron_expression: schedule.cron_expr.unwrap_or_default(),
            status: schedule.status,
            description: None,
            last_run_at: schedule.last_run_at,
            next_run_at: schedule.next_run_at,
            created_at: schedule.created_at,
            updated_at: schedule.updated_at,
        }
    }
}

pub struct JobService {
    db_manager: Arc<OldDatabaseManager>,
    state_manager: Arc<AppStateManager>,
}

impl JobService {
    pub async fn new() -> Result<Self> {
        let config = crate::database_manager_impl::DatabaseConfig::default();
        let db_manager = Arc::new(OldDatabaseManager::new(config).await?);
        let state_manager = Arc::new(AppStateManager::new());

        Ok(Self {
            db_manager,
            state_manager,
        })
    }

    /// 列出所有排程任務
    pub async fn list_jobs(&self) -> Result<Vec<JobServiceResponse>> {
        let schedules = self.db_manager.get_all_schedules_async().await?;
        let responses: Vec<JobServiceResponse> = schedules
            .into_iter()
            .map(JobServiceResponse::from)
            .collect();

        // 觸發狀態同步
        self.state_manager.notify_jobs_changed(&responses).await?;

        Ok(responses)
    }

    /// 創建新排程任務
    pub async fn create_job(&self, request: CreateJobRequest) -> Result<i64> {
        let job_id = self
            .db_manager
            .create_schedule_async(
                request.prompt_id,
                &request.name,
                Some(&request.cron_expression),
            )
            .await?;

        // 觸發狀態同步
        self.state_manager
            .notify_job_created(job_id, &request)
            .await?;

        Ok(job_id)
    }

    /// 獲取單個排程任務
    pub async fn get_job(&self, id: i64) -> Result<Option<JobServiceResponse>> {
        let schedule = self.db_manager.get_schedule_async(id).await?;
        Ok(schedule.map(JobServiceResponse::from))
    }

    /// 獲取待執行的任務
    pub async fn get_pending_jobs(&self) -> Result<Vec<JobServiceResponse>> {
        let schedules = self.db_manager.get_pending_schedules_async().await?;
        let responses: Vec<JobServiceResponse> = schedules
            .into_iter()
            .map(JobServiceResponse::from)
            .collect();

        Ok(responses)
    }

    /// 更新排程任務
    pub async fn update_job(&self, request: UpdateJobRequest) -> Result<()> {
        // 檢查任務是否存在
        let existing = self.get_job(request.id).await?;
        if existing.is_none() {
            return Err(anyhow::anyhow!("排程任務 {} 不存在", request.id));
        }

        // 使用DatabaseManager的更新方法
        self.db_manager
            .update_schedule_async(
                request.id,
                request.cron_expression.as_deref(),
                request.status.as_deref(),
                None, // schedule_time不變
            )
            .await?;

        // 觸發狀態同步
        self.state_manager.notify_job_updated(&request).await?;

        Ok(())
    }

    /// 刪除排程任務
    pub async fn delete_job(&self, id: i64) -> Result<()> {
        // 檢查任務是否存在
        let existing = self.get_job(id).await?;
        if existing.is_none() {
            return Err(anyhow::anyhow!("排程任務 {} 不存在", id));
        }

        // 執行刪除
        let success = self.db_manager.delete_schedule_async(id).await?;
        if !success {
            return Err(anyhow::anyhow!("刪除排程任務 {} 失敗", id));
        }

        // 觸發狀態同步
        self.state_manager.notify_job_deleted(id).await?;

        Ok(())
    }

    /// 暫停排程任務
    pub async fn pause_job(&self, id: i64) -> Result<()> {
        let request = UpdateJobRequest {
            id,
            name: None,
            cron_expression: None,
            status: Some("paused".to_string()),
            description: None,
        };

        self.update_job(request).await
    }

    /// 恢復排程任務
    pub async fn resume_job(&self, id: i64) -> Result<()> {
        let request = UpdateJobRequest {
            id,
            name: None,
            cron_expression: None,
            status: Some("active".to_string()),
            description: None,
        };

        self.update_job(request).await
    }

    /// 手動執行任務
    pub async fn execute_job(
        &self,
        id: i64,
    ) -> Result<crate::enhanced_executor::EnhancedClaudeResponse> {
        let job = self.get_job(id).await?;
        if let Some(job_data) = job {
            // 獲取關聯的prompt
            let prompt_service = crate::services::PromptService::new().await?;
            let options = crate::unified_interface::UnifiedExecutionOptions {
                mode: "sync".to_string(),
                cron_expr: Some(job_data.cron_expression),
                retry_enabled: Some(true),
                cooldown_check: Some(true),
                working_directory: None,
            };

            let result = prompt_service
                .execute_prompt(job_data.prompt_id, options)
                .await?;

            // 觸發狀態同步 - 記錄執行
            self.state_manager.notify_job_executed(id, &result).await?;

            Ok(result)
        } else {
            Err(anyhow::anyhow!("排程任務 {} 不存在", id))
        }
    }
}

// Tauri命令包裝器
#[tauri::command]
pub async fn job_service_list_jobs() -> Result<Vec<JobServiceResponse>, String> {
    let service = JobService::new().await.map_err(|e| e.to_string())?;
    service.list_jobs().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn job_service_create_job(
    prompt_id: i64,
    name: String,
    cron_expression: String,
    description: Option<String>,
) -> Result<i64, String> {
    let service = JobService::new().await.map_err(|e| e.to_string())?;
    let request = CreateJobRequest {
        prompt_id,
        name,
        cron_expression,
        description,
    };
    service.create_job(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn job_service_delete_job(id: i64) -> Result<(), String> {
    let service = JobService::new().await.map_err(|e| e.to_string())?;
    service.delete_job(id).await.map_err(|e| e.to_string())
}
