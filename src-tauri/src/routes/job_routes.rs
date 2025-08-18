// Job 路由處理器
// 負責處理任務執行相關的所有 API 請求

use crate::models::{ApiResponse, Job, JobStatus, JobType, PaginatedResponse};
use crate::services::ServiceContainer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 創建任務請求
#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub prompt_id: String,
    pub job_type: JobType,
    pub cron_expression: String,
    pub priority: Option<u8>,
}

/// 更新任務請求
#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub name: Option<String>,
    pub status: Option<JobStatus>,
    pub cron_expression: Option<String>,
    pub priority: Option<u8>,
}

/// 任務執行統計
#[derive(Debug, Serialize)]
pub struct JobStats {
    pub total_jobs: u64,
    pub pending_jobs: u64,
    pub running_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub success_rate: f64,
    pub average_execution_time_ms: Option<u64>,
}

/// 列出任務
pub async fn list_jobs(
    _container: Arc<ServiceContainer>,
    status: Option<JobStatus>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, String> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);

    // 模擬任務數據
    let mock_jobs = vec![
        Job::new("Daily Report Job", "prompt_1", "0 0 9 * * *").with_priority(5), // 6欄位格式
        Job::new("Code Review Job", "prompt_2", "0 0 */2 * * *").with_priority(7), // 6欄位格式
        Job::one_time(
            "Immediate Analysis",
            "prompt_3",
            chrono::Utc::now() + chrono::Duration::minutes(30),
        )
        .with_priority(9),
    ];

    // 根據狀態過濾
    let filtered_jobs: Vec<Job> = if let Some(filter_status) = status {
        mock_jobs
            .into_iter()
            .filter(|job| job.status == filter_status)
            .collect()
    } else {
        mock_jobs
    };

    let total_count = filtered_jobs.len() as u64;
    let paginated = PaginatedResponse::new(filtered_jobs, total_count, page, limit);
    let response = ApiResponse::success(paginated);

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 創建任務
pub async fn create_job(
    _container: Arc<ServiceContainer>,
    request: CreateJobRequest,
) -> Result<String, String> {
    // 驗證輸入
    if request.name.trim().is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("任務名稱不能為空");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    if request.prompt_id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的提示詞 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 創建任務
    let mut job = match request.job_type {
        JobType::OneTime => Job::one_time(
            &request.name,
            &request.prompt_id,
            chrono::Utc::now() + chrono::Duration::hours(1),
        ),
        _ => Job::new(&request.name, &request.prompt_id, &request.cron_expression),
    };

    if let Some(priority) = request.priority {
        job = job.with_priority(priority);
    }

    let response = ApiResponse::success_with_message(job, "任務創建成功");
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 獲取特定任務
pub async fn get_job(_container: Arc<ServiceContainer>, id: String) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的任務 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 模擬獲取任務
    let mut job = Job::new(
        format!("Job {}", id),
        format!("prompt_{}", id.chars().take(3).collect::<String>()),
        "0 0 9 * * *", // 6欄位格式
    );
    job.id = id.clone();
    job.update_status(JobStatus::Completed);

    let response = ApiResponse::success(job);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 更新任務
pub async fn update_job(
    _container: Arc<ServiceContainer>,
    id: String,
    request: UpdateJobRequest,
) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的任務 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 模擬更新
    let mut job = Job::new(
        request
            .name
            .unwrap_or_else(|| format!("Updated Job {}", id)),
        format!("prompt_{}", id.chars().take(3).collect::<String>()),
        request
            .cron_expression
            .unwrap_or_else(|| "0 0 9 * * *".to_string()), // 6欄位預設值
    );
    job.id = id.clone();

    if let Some(status) = request.status {
        job.update_status(status);
    }

    if let Some(priority) = request.priority {
        job = job.with_priority(priority);
    }

    let response = ApiResponse::success_with_message(job, "任務更新成功");
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 取消任務
pub async fn cancel_job(container: Arc<ServiceContainer>, id: String) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的任務 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 嘗試取消正在執行的任務
    let claude_service = container.claude_service();
    match claude_service.cancel_execution(&id).await {
        Ok(_) => {
            let response = ApiResponse::success_with_message(
                serde_json::json!({"cancelled_job_id": id}),
                "任務取消成功",
            );
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        Err(e) => {
            let error_response: ApiResponse<()> =
                ApiResponse::error(format!("取消任務失敗: {}", e));
            serde_json::to_string(&error_response).map_err(|e| e.to_string())
        }
    }
}

/// 獲取任務統計
pub async fn get_job_stats(container: Arc<ServiceContainer>) -> Result<String, String> {
    let claude_service = container.claude_service();
    let execution_stats = claude_service.get_active_executions_stats().await;

    let stats = JobStats {
        total_jobs: 50,
        pending_jobs: *execution_stats.get("pending").unwrap_or(&0) as u64,
        running_jobs: *execution_stats.get("running").unwrap_or(&0) as u64,
        completed_jobs: 42,
        failed_jobs: 3,
        success_rate: 93.3,
        average_execution_time_ms: Some(1800),
    };

    let response = ApiResponse::success(stats);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_job_validation() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        // 測試無效的 prompt_id
        let invalid_request = CreateJobRequest {
            name: "Test Job".to_string(),
            prompt_id: "".to_string(),
            job_type: JobType::OneTime,
            cron_expression: "0 0 9 * * *".to_string(), // 6欄位格式
            priority: None,
        };

        let result = create_job(container.clone(), invalid_request).await;
        assert!(result.is_ok());
        let response_str = result.unwrap();
        assert!(response_str.contains("無效的提示詞 ID"));
    }

    #[tokio::test]
    async fn test_get_job_invalid_id() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        let result = get_job(container, "".to_string()).await;
        assert!(result.is_ok());
        let response_str = result.unwrap();
        assert!(response_str.contains("無效的任務 ID"));
    }

    #[tokio::test]
    async fn test_list_jobs_filtering() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        // 測試不帶過濾器的列表
        let result = list_jobs(container.clone(), None, None, None).await;
        assert!(result.is_ok());

        // 測試帶狀態過濾器的列表
        let result = list_jobs(container, Some(JobStatus::Active), None, None).await;
        assert!(result.is_ok());
    }
}
