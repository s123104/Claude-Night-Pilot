// 簡化的 Job 管理器 - 與現有架構完全兼容
// 優先實現功能性，後續可逐步遷移到完整的企業級系統

use crate::enhanced_executor::{EnhancedClaudeExecutor, EnhancedClaudeResponse};
use crate::models::job::{Job, JobType};
use crate::services::JobService;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler as TokioCronScheduler;
use tracing::{error, info, warn};
use uuid::Uuid;

/// 簡化的 Job 管理器 - 專注於實用性和穩定性
///
/// ⚠️ **棄用通知**: 此實作將在 v3.0.0 版本中移除
///
/// **建議遷移**: 使用新的 `UnifiedScheduler`
/// ```rust
/// use crate::scheduler::UnifiedScheduler;
/// let scheduler = UnifiedScheduler::new().await?;
/// ```
///
/// **遷移期限**: 2025年12月31日前完成遷移
/// **詳細指南**: 參考 `docs/SCHEDULER_MIGRATION_GUIDE.md`
#[deprecated(
    since = "2.1.0",
    note = "請使用 UnifiedScheduler 替代。此實作將在 v3.0.0 移除。詳見遷移指南。"
)]
pub struct SimpleJobManager {
    /// 基礎排程器
    scheduler: Arc<TokioCronScheduler>,
    /// 現有的 Job 服務
    job_service: Arc<JobService>,
    /// Claude 執行器
    claude_executor: Arc<EnhancedClaudeExecutor>,
    /// 運行狀態追蹤
    running_jobs: Arc<RwLock<HashMap<String, SimpleJobExecution>>>,
    /// 管理器狀態
    is_running: Arc<RwLock<bool>>,
}

/// 簡化的任務執行記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleJobExecution {
    pub job_id: String,
    pub job_name: String,
    pub started_at: DateTime<Utc>,
    pub status: ExecutionStatus,
    pub cron_job_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
}

/// 管理器統計信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerStats {
    pub total_scheduled_jobs: usize,
    pub active_executions: usize,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub uptime_seconds: u64,
}

impl SimpleJobManager {
    /// 創建新的簡化 Job 管理器
    pub async fn new() -> Result<Self> {
        let scheduler = Arc::new(TokioCronScheduler::new().await?);
        let job_service = Arc::new(JobService::new().await?);
        let claude_executor = Arc::new(EnhancedClaudeExecutor::with_smart_defaults()?);

        Ok(Self {
            scheduler,
            job_service,
            claude_executor,
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// 啟動管理器
    pub async fn start(&self) -> Result<()> {
        info!("啟動簡化 Job 管理器");

        // 啟動排程器
        self.scheduler.start().await?;

        // 標記為運行中
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        // 載入並排程現有的活躍任務
        match self.load_and_schedule_active_jobs().await {
            Ok(count) => info!("成功載入 {} 個活躍任務", count),
            Err(e) => warn!("載入活躍任務時發生錯誤: {}", e),
        }

        info!("簡化 Job 管理器已啟動");
        Ok(())
    }

    /// 停止管理器
    pub async fn shutdown(&self) -> Result<()> {
        info!("停止簡化 Job 管理器");

        // 標記停止排程器（tokio_cron_scheduler不支持直接shutdown）

        // 標記為停止
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        // 清理運行中的任務記錄
        {
            let mut running_jobs = self.running_jobs.write().await;
            running_jobs.clear();
        }

        info!("簡化 Job 管理器已停止");
        Ok(())
    }

    /// 載入並排程活躍任務
    async fn load_and_schedule_active_jobs(&self) -> Result<usize> {
        let jobs = self.job_service.list_jobs().await?;
        let mut scheduled_count = 0;

        for job_response in jobs {
            // 只處理活躍且有 cron 表達式的任務
            if job_response.status == "active" && !job_response.cron_expression.is_empty() {
                match self
                    .schedule_job_internal(
                        job_response.id.to_string(),
                        &job_response.name,
                        &job_response.cron_expression,
                        job_response.prompt_id,
                    )
                    .await
                {
                    Ok(_) => scheduled_count += 1,
                    Err(e) => warn!("排程任務失敗 {}: {}", job_response.name, e),
                }
            }
        }

        Ok(scheduled_count)
    }

    /// 內部排程任務方法
    async fn schedule_job_internal(
        &self,
        job_id: String,
        job_name: &str,
        cron_expression: &str,
        prompt_id: i64,
    ) -> Result<Uuid> {
        let job_id_clone = job_id.clone();
        let job_name_clone = job_name.to_string();
        let job_service = Arc::clone(&self.job_service);
        let claude_executor = Arc::clone(&self.claude_executor);
        let running_jobs = Arc::clone(&self.running_jobs);

        // 創建 Cron 任務
        let cron_job =
            tokio_cron_scheduler::Job::new_async(cron_expression, move |_uuid, _lock| {
                let job_id = job_id_clone.clone();
                let job_name = job_name_clone.clone();
                let job_service = Arc::clone(&job_service);
                let claude_executor = Arc::clone(&claude_executor);
                let running_jobs = Arc::clone(&running_jobs);

                Box::pin(async move {
                    info!("開始執行任務: {} ({})", job_name, job_id);

                    // 記錄執行開始
                    let execution = SimpleJobExecution {
                        job_id: job_id.clone(),
                        job_name: job_name.clone(),
                        started_at: Utc::now(),
                        status: ExecutionStatus::Running,
                        cron_job_id: None,
                    };

                    {
                        let mut running = running_jobs.write().await;
                        running.insert(job_id.clone(), execution);
                    }

                    // 執行任務邏輯
                    let result =
                        execute_job_logic(&job_service, &claude_executor, &job_id, prompt_id).await;

                    // 更新執行狀態
                    {
                        let mut running = running_jobs.write().await;
                        if let Some(mut execution) = running.remove(&job_id) {
                            execution.status = match result {
                                Ok(_) => {
                                    info!("任務執行成功: {} ({})", job_name, job_id);
                                    ExecutionStatus::Completed
                                }
                                Err(ref e) => {
                                    error!("任務執行失敗: {} ({}): {}", job_name, job_id, e);
                                    ExecutionStatus::Failed
                                }
                            };
                            // 可以選擇將執行記錄保存到數據庫
                        }
                    }
                })
            })?;

        // 添加到排程器
        let cron_job_id = self.scheduler.add(cron_job).await?;
        info!("任務已排程: {} (Cron ID: {})", job_name, cron_job_id);

        Ok(cron_job_id)
    }

    /// 添加新任務到排程器
    pub async fn schedule_job(&self, job_id: String, job: &Job) -> Result<()> {
        match job.job_type {
            JobType::Scheduled => {
                if !job.cron_expression.is_empty() {
                    // 從 job.prompt_id 字符串轉換為 i64
                    let prompt_id: i64 = job
                        .prompt_id
                        .parse()
                        .map_err(|_| anyhow::anyhow!("無效的 prompt_id: {}", job.prompt_id))?;

                    self.schedule_job_internal(job_id, &job.name, &job.cron_expression, prompt_id)
                        .await?;

                    info!("新任務已加入排程: {}", job.name);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("排程任務缺少 cron 表達式"))
                }
            }
            _ => {
                warn!("暫不支援的任務類型: {:?}", job.job_type);
                Err(anyhow::anyhow!("暫不支援的任務類型"))
            }
        }
    }

    /// 手動觸發任務執行
    pub async fn trigger_job(&self, job_id: &str) -> Result<EnhancedClaudeResponse> {
        info!("手動觸發任務: {}", job_id);

        // 嘗試解析 job_id 為數字 ID
        let numeric_job_id: i64 = job_id
            .parse()
            .map_err(|_| anyhow::anyhow!("無效的任務 ID 格式: {}", job_id))?;

        // 使用現有的 job_service 執行任務
        self.job_service.execute_job(numeric_job_id).await
    }

    /// 獲取當前運行中的任務
    pub async fn get_running_jobs(&self) -> HashMap<String, SimpleJobExecution> {
        let running_jobs = self.running_jobs.read().await;
        running_jobs.clone()
    }

    /// 獲取管理器狀態
    pub async fn is_running(&self) -> bool {
        let is_running = self.is_running.read().await;
        *is_running
    }

    /// 獲取管理器統計信息
    pub async fn get_stats(&self) -> ManagerStats {
        let running_jobs = self.running_jobs.read().await;

        ManagerStats {
            total_scheduled_jobs: 0, // TODO: 從排程器獲取
            active_executions: running_jobs.len(),
            total_executions: 0,      // TODO: 從歷史記錄計算
            successful_executions: 0, // TODO: 從歷史記錄計算
            failed_executions: 0,     // TODO: 從歷史記錄計算
            uptime_seconds: 0,        // TODO: 計算運行時間
        }
    }

    /// 健康檢查
    pub async fn health_check(&self) -> bool {
        self.is_running().await
    }
}

/// 執行任務的具體邏輯
async fn execute_job_logic(
    job_service: &JobService,
    _claude_executor: &EnhancedClaudeExecutor,
    job_id: &str,
    _prompt_id: i64,
) -> Result<EnhancedClaudeResponse> {
    // 使用現有的 job_service 執行邏輯
    // 這裡重用了現有的實現，確保兼容性
    let numeric_job_id: i64 = job_id
        .parse()
        .map_err(|_| anyhow::anyhow!("無效的任務 ID: {}", job_id))?;

    job_service.execute_job(numeric_job_id).await
}

/// Tauri 命令包裝器
#[tauri::command]
pub async fn simple_job_manager_start() -> Result<(), String> {
    // 這裡需要全局狀態管理，暫時返回成功
    info!("收到啟動簡化 Job 管理器的請求");
    Ok(())
}

#[tauri::command]
pub async fn simple_job_manager_status() -> Result<bool, String> {
    // TODO: 實現全局狀態檢查
    Ok(false)
}

#[tauri::command]
pub async fn simple_job_manager_trigger_job(job_id: String) -> Result<String, String> {
    info!("收到手動觸發任務請求: {}", job_id);
    // TODO: 實現實際的任務觸發
    Ok(format!("任務 {} 觸發請求已接收", job_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_job_manager_creation() {
        let manager = SimpleJobManager::new().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_manager_start_stop() {
        let manager = SimpleJobManager::new().await.unwrap();

        assert!(!manager.is_running().await);

        manager.start().await.unwrap();
        assert!(manager.is_running().await);

        manager.shutdown().await.unwrap();
        assert!(!manager.is_running().await);
    }
}
