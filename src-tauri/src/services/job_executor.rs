// Job 執行器 - 集成 ClaudeExecutor 實現任務執行
// 參考 Context7 最佳實踐和 vibe-kanban 執行模式

use crate::enhanced_executor::{EnhancedClaudeExecutor, EnhancedClaudeResponse};
use crate::models::job::{Job, JobExecutionOptions, ResourceLimits, RetryStrategy};
use crate::unified_interface::{UnifiedClaudeInterface, UnifiedExecutionOptions};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

/// 任務執行器 - 負責執行 Claude CLI 任務
pub struct JobExecutor {
    /// Claude 執行器
    claude_executor: Arc<EnhancedClaudeExecutor>,
    /// 統一介面
    unified_interface: Arc<UnifiedClaudeInterface>,
    /// 並發控制信號量
    concurrency_limiter: Arc<Semaphore>,
    /// 正在執行的任務
    running_executions: Arc<Mutex<HashMap<String, JobExecution>>>,
    /// 執行統計
    execution_stats: Arc<Mutex<ExecutionStats>>,
}

impl std::fmt::Debug for JobExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobExecutor")
            .field("concurrency_limit", &"[configured]")
            .field("running_executions_count", &"[computed]")
            .finish()
    }
}

/// 任務執行實例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    /// 執行 ID
    pub id: String,
    /// 任務 ID
    pub job_id: String,
    /// 任務名稱
    pub job_name: String,
    /// 開始時間
    pub started_at: DateTime<Utc>,
    /// 執行狀態
    pub status: ExecutionStatus,
    /// 進度百分比 (0-100)
    pub progress: Option<u8>,
    /// 當前步驟描述
    pub current_step: Option<String>,
    /// 執行選項
    pub execution_options: JobExecutionOptions,
    /// 資源使用情況
    pub resource_usage: Option<ResourceUsage>,
}

/// 執行狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// 準備中
    Preparing,
    /// 執行中
    Running,
    /// 已完成
    Completed,
    /// 失敗
    Failed,
    /// 已取消
    Cancelled,
    /// 超時
    Timeout,
}

/// 資源使用情況
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 記憶體使用量 (MB)
    pub memory_mb: Option<f64>,
    /// CPU 使用率 (%)
    pub cpu_percent: Option<f64>,
    /// 執行時間 (秒)
    pub execution_time_seconds: f64,
    /// 峰值記憶體 (MB)
    pub peak_memory_mb: Option<f64>,
}

/// 執行統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// 總執行次數
    pub total_executions: u64,
    /// 成功次數
    pub successful_executions: u64,
    /// 失敗次數
    pub failed_executions: u64,
    /// 平均執行時間 (秒)
    pub average_execution_time: f64,
    /// 並發執行次數
    pub concurrent_executions: u64,
    /// 上次重置時間
    pub last_reset: DateTime<Utc>,
}

/// 執行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionResult {
    /// 執行實例
    pub execution: JobExecution,
    /// Claude 回應
    pub claude_response: Option<EnhancedClaudeResponse>,
    /// 輸出內容
    pub output: Option<String>,
    /// 錯誤信息
    pub error: Option<String>,
    /// 重試次數
    pub retry_count: u32,
    /// 總耗時 (毫秒)
    pub total_duration_ms: u64,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            average_execution_time: 0.0,
            concurrent_executions: 0,
            last_reset: Utc::now(),
        }
    }
}

impl JobExecutor {
    /// 創建新的任務執行器
    pub async fn new() -> Result<Self> {
        let claude_executor = Arc::new(
            EnhancedClaudeExecutor::with_smart_defaults()
                .context("創建 EnhancedClaudeExecutor 失敗")?
        );
        
        let unified_interface = Arc::new(
            UnifiedClaudeInterface::new()
                .await
                .context("創建 UnifiedClaudeInterface 失敗")?
        );
        
        // 預設最大並發數為 10
        let concurrency_limiter = Arc::new(Semaphore::new(10));
        
        Ok(Self {
            claude_executor,
            unified_interface,
            concurrency_limiter,
            running_executions: Arc::new(Mutex::new(HashMap::new())),
            execution_stats: Arc::new(Mutex::new(ExecutionStats::default())),
        })
    }
    
    /// 使用自定義並發限制創建執行器
    pub async fn with_concurrency_limit(max_concurrent: usize) -> Result<Self> {
        let mut executor = Self::new().await?;
        executor.concurrency_limiter = Arc::new(Semaphore::new(max_concurrent));
        Ok(executor)
    }
    
    /// 執行任務
    pub async fn execute_job(&self, job: &Job) -> Result<JobExecutionResult> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let started_at = Utc::now();
        
        info!("開始執行任務: {} ({})", job.name, job.id);
        
        // 獲取並發許可
        let _permit = self.concurrency_limiter
            .acquire()
            .await
            .context("獲取並發許可失敗")?;
        
        // 創建執行實例
        let mut execution = JobExecution {
            id: execution_id.clone(),
            job_id: job.id.clone(),
            job_name: job.name.clone(),
            started_at,
            status: ExecutionStatus::Preparing,
            progress: Some(0),
            current_step: Some("準備執行環境".to_string()),
            execution_options: job.execution_options.clone(),
            resource_usage: None,
        };
        
        // 註冊執行實例
        {
            let mut running_executions = self.running_executions.lock().await;
            running_executions.insert(execution_id.clone(), execution.clone());
        }
        
        // 準備執行選項
        let unified_options = self.prepare_execution_options(job).await?;
        
        // 執行任務 (包含重試邏輯)
        let result = self.execute_with_retry(job, &mut execution, unified_options).await;
        
        // 計算執行時間
        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        
        // 更新執行狀態
        execution.status = match &result {
            Ok(_) => ExecutionStatus::Completed,
            Err(_) => ExecutionStatus::Failed,
        };
        
        // 更新資源使用情況
        execution.resource_usage = Some(ResourceUsage {
            memory_mb: None, // TODO: 實際測量記憶體使用
            cpu_percent: None, // TODO: 實際測量 CPU 使用
            execution_time_seconds: total_duration_ms as f64 / 1000.0,
            peak_memory_mb: None,
        });
        
        // 移除執行實例
        {
            let mut running_executions = self.running_executions.lock().await;
            running_executions.remove(&execution_id);
        }
        
        // 更新統計信息
        self.update_execution_stats(result.is_ok(), total_duration_ms as f64 / 1000.0).await;
        
        // 構建結果
        let execution_result = match result {
            Ok((claude_response, retry_count)) => JobExecutionResult {
                execution: execution.clone(),
                claude_response: Some(claude_response),
                output: None, // 從 claude_response 中提取
                error: None,
                retry_count,
                total_duration_ms,
            },
            Err(e) => JobExecutionResult {
                execution: execution.clone(),
                claude_response: None,
                output: None,
                error: Some(e.to_string()),
                retry_count: 0, // TODO: 從錯誤上下文中獲取
                total_duration_ms,
            }
        };
        
        info!("任務執行完成: {} (耗時: {}ms)", job.name, total_duration_ms);
        Ok(execution_result)
    }
    
    /// 帶重試邏輯的執行
    async fn execute_with_retry(
        &self,
        job: &Job,
        execution: &mut JobExecution,
        unified_options: UnifiedExecutionOptions,
    ) -> Result<(EnhancedClaudeResponse, u32)> {
        let max_retries = job.retry_config.max_retries;
        let mut retry_count = 0;
        
        loop {
            execution.status = ExecutionStatus::Running;
            execution.current_step = Some(format!(
                "執行任務 (嘗試 {}/{})",
                retry_count + 1,
                max_retries + 1
            ));
            execution.progress = Some((retry_count as f32 / (max_retries + 1) as f32 * 100.0) as u8);
            
            // 更新執行狀態
            {
                let mut running_executions = self.running_executions.lock().await;
                running_executions.insert(execution.id.clone(), execution.clone());
            }
            
            debug!("執行任務嘗試 {}: {}", retry_count + 1, job.name);
            
            // 執行任務
            match self.execute_claude_task(job, &unified_options).await {
                Ok(response) => {
                    info!("任務執行成功: {} (嘗試 {})", job.name, retry_count + 1);
                    return Ok((response, retry_count));
                }
                Err(e) => {
                    warn!("任務執行失敗: {} (嘗試 {}): {}", job.name, retry_count + 1, e);
                    
                    retry_count += 1;
                    
                    // 檢查是否還有重試機會
                    if retry_count > max_retries {
                        error!("任務執行徹底失敗: {} (重試 {} 次)", job.name, max_retries);
                        return Err(e);
                    }
                    
                    // 計算重試延遲
                    let delay = self.calculate_retry_delay(&job.retry_config, retry_count);
                    
                    execution.current_step = Some(format!(
                        "等待重試 ({} 秒後重試)",
                        delay.as_secs()
                    ));
                    
                    info!("等待 {:?} 後重試任務: {}", delay, job.name);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    /// 執行 Claude 任務
    async fn execute_claude_task(
        &self,
        job: &Job,
        unified_options: &UnifiedExecutionOptions,
    ) -> Result<EnhancedClaudeResponse> {
        // 檢查資源限制
        self.check_resource_limits(&job.execution_options.resource_limits).await?;
        
        // TODO: 從 prompt_id 獲取實際的 prompt 內容
        // 目前使用臨時的 prompt 內容
        let prompt_content = format!("執行任務: {}", job.name);
        
        // 使用統一介面執行
        let response = self.unified_interface
            .execute_prompt_with_options(&prompt_content, (*unified_options).clone())
            .await
            .context("Claude 任務執行失敗")?;
        
        Ok(response)
    }
    
    /// 準備執行選項
    async fn prepare_execution_options(&self, job: &Job) -> Result<UnifiedExecutionOptions> {
        let options = UnifiedExecutionOptions {
            mode: "sync".to_string(),
            cron_expr: if job.cron_expression.is_empty() {
                None
            } else {
                Some(job.cron_expression.clone())
            },
            retry_enabled: Some(job.retry_config.max_retries > 0),
            cooldown_check: Some(true),
            working_directory: job.execution_options.working_directory.clone(),
        };
        
        debug!("準備執行選項: {:?}", options);
        Ok(options)
    }
    
    /// 檢查資源限制
    async fn check_resource_limits(&self, limits: &Option<ResourceLimits>) -> Result<()> {
        if let Some(limits) = limits {
            // TODO: 實際檢查系統資源
            if let Some(max_memory) = limits.max_memory_mb {
                debug!("檢查記憶體限制: {} MB", max_memory);
                // 檢查當前記憶體使用
            }
            
            if let Some(max_cpu) = limits.max_cpu_percent {
                debug!("檢查 CPU 限制: {}%", max_cpu);
                // 檢查當前 CPU 使用
            }
        }
        
        Ok(())
    }
    
    /// 計算重試延遲
    fn calculate_retry_delay(
        &self,
        retry_config: &crate::models::job::RetryConfig,
        retry_count: u32,
    ) -> Duration {
        let base_interval = retry_config.retry_interval_seconds;
        
        let delay_seconds = match retry_config.retry_strategy {
            RetryStrategy::Fixed => base_interval,
            RetryStrategy::ExponentialBackoff => {
                let backoff = (base_interval as f64 
                    * retry_config.backoff_multiplier.powi(retry_count as i32 - 1)) as u64;
                backoff.min(retry_config.max_backoff_seconds)
            }
            RetryStrategy::Linear => base_interval * retry_count as u64,
            RetryStrategy::Custom(ref intervals) => {
                intervals.get((retry_count - 1) as usize)
                    .copied()
                    .unwrap_or(base_interval)
            }
        };
        
        Duration::from_secs(delay_seconds)
    }
    
    /// 取消執行
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<()> {
        let mut running_executions = self.running_executions.lock().await;
        
        if let Some(mut execution) = running_executions.remove(execution_id) {
            execution.status = ExecutionStatus::Cancelled;
            execution.current_step = Some("已取消".to_string());
            
            info!("已取消執行: {}", execution_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("找不到執行實例: {}", execution_id))
        }
    }
    
    /// 獲取執行狀態
    pub async fn get_execution_status(&self, execution_id: &str) -> Option<JobExecution> {
        let running_executions = self.running_executions.lock().await;
        running_executions.get(execution_id).cloned()
    }
    
    /// 獲取所有正在執行的任務
    pub async fn get_running_executions(&self) -> HashMap<String, JobExecution> {
        let running_executions = self.running_executions.lock().await;
        running_executions.clone()
    }
    
    /// 獲取執行統計
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        let stats = self.execution_stats.lock().await;
        stats.clone()
    }
    
    /// 重置執行統計
    pub async fn reset_execution_stats(&self) {
        let mut stats = self.execution_stats.lock().await;
        *stats = ExecutionStats::default();
        info!("執行統計已重置");
    }
    
    /// 更新執行統計
    async fn update_execution_stats(&self, success: bool, execution_time: f64) {
        let mut stats = self.execution_stats.lock().await;
        
        stats.total_executions += 1;
        
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }
        
        // 更新平均執行時間
        let total_time = stats.average_execution_time * (stats.total_executions - 1) as f64 + execution_time;
        stats.average_execution_time = total_time / stats.total_executions as f64;
        
        // 更新並發統計
        let running_count = {
            let running_executions = self.running_executions.lock().await;
            running_executions.len() as u64
        };
        stats.concurrent_executions = stats.concurrent_executions.max(running_count);
    }
    
    /// 健康檢查
    pub async fn health_check(&self) -> Result<bool> {
        // 檢查 Claude 執行器是否正常
        match self.claude_executor.health_check().await {
            Ok(health_status) => {
                debug!("Claude 執行器健康狀態: {:?}", health_status);
                Ok(true)
            }
            Err(e) => {
                error!("Claude 執行器健康檢查失敗: {}", e);
                Ok(false)
            }
        }
    }
    
    /// 清理已完成的執行記錄
    pub async fn cleanup_completed_executions(&self) -> usize {
        let mut running_executions = self.running_executions.lock().await;
        let initial_count = running_executions.len();
        
        // 移除已完成、失敗或取消的執行
        running_executions.retain(|_, execution| {
            matches!(execution.status, ExecutionStatus::Preparing | ExecutionStatus::Running)
        });
        
        let removed_count = initial_count - running_executions.len();
        debug!("清理了 {} 個已完成的執行記錄", removed_count);
        removed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::job::{JobType, RetryConfig, RetryStrategy};

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = JobExecutor::new().await.unwrap();
        assert!(executor.health_check().await.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_execution_stats() {
        let executor = JobExecutor::new().await.unwrap();
        
        let stats = executor.get_execution_stats().await;
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.successful_executions, 0);
        assert_eq!(stats.failed_executions, 0);
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let executor = JobExecutor::new().await.unwrap();
        
        let retry_config = RetryConfig {
            max_retries: 3,
            retry_interval_seconds: 10,
            retry_strategy: RetryStrategy::ExponentialBackoff,
            backoff_multiplier: 2.0,
            max_backoff_seconds: 60,
        };
        
        let delay1 = executor.calculate_retry_delay(&retry_config, 1);
        let delay2 = executor.calculate_retry_delay(&retry_config, 2);
        
        assert_eq!(delay1.as_secs(), 10); // 第一次重試：10秒
        assert_eq!(delay2.as_secs(), 20); // 第二次重試：20秒
    }
}