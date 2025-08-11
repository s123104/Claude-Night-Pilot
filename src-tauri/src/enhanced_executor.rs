// 增強的 Claude 執行器 - 整合所有核心模組的統一執行介面
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;
use crate::core::{
    CooldownDetector, CooldownInfo, CooldownPattern,
    RetryOrchestrator, RetryConfig,
    ProcessOrchestrator, ProcessType, ProcessHandle, ProcessMetadata, ExecutionOptions,
    Scheduler, SchedulingConfig, SchedulerType
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedClaudeResponse {
    pub completion: String,
    pub model: Option<String>,
    pub usage: Option<Usage>,
    pub execution_metadata: ExecutionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub execution_id: Uuid,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub total_attempts: u32,
    pub cooldown_detected: Option<CooldownInfo>,
    pub process_handle: Option<Uuid>,
    pub scheduler_used: Option<String>,
}

pub struct EnhancedClaudeExecutor {
    cooldown_detector: CooldownDetector,
    retry_orchestrator: RetryOrchestrator,
    process_orchestrator: ProcessOrchestrator,
    config: ExecutorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    pub enable_cooldown_detection: bool,
    pub enable_smart_retry: bool,
    pub enable_process_orchestration: bool,
    pub default_retry_config: RetryConfig,
    pub cooldown_timeout_multiplier: f64,
    pub max_concurrent_executions: u32,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            enable_cooldown_detection: true,
            enable_smart_retry: true,
            enable_process_orchestration: true,
            default_retry_config: RetryConfig::default(),
            cooldown_timeout_multiplier: 1.2, // 20% 緩衝時間
            max_concurrent_executions: 3,
        }
    }
}

impl EnhancedClaudeExecutor {
    /// 創建新的增強執行器實例
    pub fn new(config: ExecutorConfig) -> Result<Self> {
        Ok(Self {
            cooldown_detector: CooldownDetector::new()?,
            retry_orchestrator: RetryOrchestrator::new(config.default_retry_config.clone())?,
            process_orchestrator: ProcessOrchestrator::new()?,
            config,
        })
    }

    /// 使用預設配置創建執行器
    pub fn with_smart_defaults() -> Result<Self> {
        Self::new(ExecutorConfig::default())
    }

    /// 主要執行方法 - 整合所有增強功能
    pub async fn execute_with_full_enhancement(
        &mut self,
        prompt: &str,
        options: ExecutionOptions,
    ) -> Result<EnhancedClaudeResponse> {
        let execution_id = Uuid::new_v4();
        let start_time = SystemTime::now();

        tracing::info!("開始增強執行 {}: {}", execution_id, &prompt[..50.min(prompt.len())]);

        // 1. 預檢查冷卻狀態
        if self.config.enable_cooldown_detection {
            if let Some(cooldown) = self.pre_execution_cooldown_check().await? {
                if cooldown.is_cooling {
                    tracing::info!("檢測到預冷卻狀態，將等待 {} 秒", cooldown.seconds_remaining);
                    self.cooldown_detector.smart_wait(&cooldown).await;
                }
            }
        }

        // 2. 使用進程編排執行（如果啟用）
        let process_handle = if self.config.enable_process_orchestration {
            Some(self.orchestrated_execution(prompt, &options).await?)
        } else {
            None
        };

        // 3. 執行主要邏輯（帶重試）
        let mut execution_metadata = ExecutionMetadata {
            execution_id,
            start_time,
            end_time: None,
            total_attempts: 0,
            cooldown_detected: None,
            process_handle: process_handle.map(|h| h.id),
            scheduler_used: None,
        };

        let result = if self.config.enable_smart_retry {
            self.execute_with_smart_retry(prompt, &options, &mut execution_metadata).await?
        } else {
            self.execute_direct(prompt, &options).await?
        };

        execution_metadata.end_time = Some(SystemTime::now());

        // 4. 後處理檢查
        if self.config.enable_cooldown_detection {
            if let Some(cooldown) = self.cooldown_detector.detect_cooldown(&result) {
                execution_metadata.cooldown_detected = Some(cooldown);
            }
        }

        tracing::info!("增強執行 {} 完成，總嘗試次數: {}", execution_id, execution_metadata.total_attempts);

        Ok(EnhancedClaudeResponse {
            completion: result,
            model: Some("claude-3-sonnet".to_string()), // 預設模型
            usage: None, // 需要從實際響應中提取
            execution_metadata,
        })
    }

    /// 預執行冷卻檢查
    async fn pre_execution_cooldown_check(&self) -> Result<Option<CooldownInfo>> {
        // 檢查 claude doctor 狀態
        let doctor_cooldown = self.cooldown_detector.check_claude_doctor_cooldown().await?;
        
        if doctor_cooldown.is_cooling {
            return Ok(Some(doctor_cooldown));
        }

        // 檢查是否目前正在冷卻中
        if self.cooldown_detector.is_currently_cooling().await {
            // 如果正在冷卻但 doctor 沒有檢測到，使用保守估計
            return Ok(Some(CooldownInfo {
                is_cooling: true,
                seconds_remaining: 60, // 保守估計 1 分鐘
                next_available_time: Some(SystemTime::now() + std::time::Duration::from_secs(60)),
                reset_time: None,
                original_message: "Pre-execution cooldown check".to_string(),
                cooldown_pattern: Some(CooldownPattern::RateLimitExceeded { seconds: 60 }),
            }));
        }

        Ok(None)
    }

    /// 編排執行 - 使用進程管理系統
    async fn orchestrated_execution(
        &mut self,
        prompt: &str,
        options: &ExecutionOptions,
    ) -> Result<ProcessHandle> {
        let process_type = ProcessType::ClaudeExecution {
            prompt: prompt.to_string(),
            options: options.clone(),
        };

        let metadata = ProcessMetadata {
            job_id: Some(Uuid::new_v4()),
            timeout: options.timeout_seconds.map(std::time::Duration::from_secs),
            retry_config: Some(self.config.default_retry_config.clone()),
            ..Default::default()
        };

        // 定義可能的前置條件
        let prerequisites = self.determine_prerequisites(options).await?;

        let _prompt_clone = prompt.to_string();
        let _options_clone = options.clone();
        
        let main_operation = move || {
            Box::pin(async move {
                // 在這裡使用簡單的執行邏輯
                Ok("模擬執行結果".to_string())
            })
        };

        self.process_orchestrator
            .execute_with_prerequisites(process_type, prerequisites, metadata, main_operation)
            .await
    }

    /// 決定執行前置條件
    async fn determine_prerequisites(&self, options: &ExecutionOptions) -> Result<Vec<ProcessType>> {
        let mut prerequisites = Vec::new();

        // 如果需要資料庫初始化
        if self.needs_database_setup(options).await? {
            prerequisites.push(ProcessType::DatabaseMigration {
                migration_type: "init".to_string(),
            });
        }

        // 如果需要清理
        if options.working_directory.is_some() {
            prerequisites.push(ProcessType::CleanupScript {
                script_path: "cleanup_temp.sh".to_string(),
                cleanup_type: crate::core::CleanupType::Temporary,
            });
        }

        Ok(prerequisites)
    }

    /// 檢查是否需要資料庫設置
    async fn needs_database_setup(&self, _options: &ExecutionOptions) -> Result<bool> {
        // 這裡可以檢查資料庫連接狀態
        // 暫時回傳 false
        Ok(false)
    }

    /// 智慧重試執行
    async fn execute_with_smart_retry(
        &mut self,
        prompt: &str,
        options: &ExecutionOptions,
        metadata: &mut ExecutionMetadata,
    ) -> Result<String> {
        // 暫時使用簡單的重試邏輯
        let mut attempts = 0;
        let max_attempts = options.max_retries;
        
        loop {
            attempts += 1;
            metadata.total_attempts = attempts;
            
            match self.execute_claude_command(prompt, options).await {
                Ok(result) => return Ok(result),
                Err(e) if attempts < max_attempts => {
                    tracing::warn!("執行失敗，重試 {}/{}: {}", attempts, max_attempts, e);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 直接執行（無重試）
    async fn execute_direct(&self, prompt: &str, options: &ExecutionOptions) -> Result<String> {
        self.execute_claude_command(prompt, options).await
    }

    /// 核心 Claude 命令執行
    async fn execute_claude_command(&self, prompt: &str, options: &ExecutionOptions) -> Result<String> {
        let timeout = std::time::Duration::from_secs(options.timeout_seconds.unwrap_or(300));

        let execution_future = async {
            let mut cmd = tokio::process::Command::new("claude");
            cmd.arg("-p").arg(prompt);

            // 添加選項
            if options.skip_permissions {
                cmd.arg("--dangerously-skip-permissions");
            }

            if options.output_format == "json" {
                cmd.arg("--output-format").arg("json");
            }

            if let Some(work_dir) = &options.working_directory {
                cmd.current_dir(work_dir);
            }

            let output = cmd.output().await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // 檢測冷卻並提供詳細資訊
                if let Some(cooldown) = self.cooldown_detector.detect_cooldown(&stderr) {
                    let formatted_status = self.cooldown_detector.format_cooldown_status(&cooldown);
                    return Err(anyhow::anyhow!("Claude CLI 執行失敗: {}", formatted_status));
                }

                return Err(anyhow::anyhow!("Claude CLI 執行失敗: {}", stderr));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);

            // 嘗試解析 JSON 回應
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => {
                    if let Some(completion) = json.get("completion").and_then(|c| c.as_str()) {
                        Ok(completion.to_string())
                    } else {
                        Ok(stdout.trim().to_string())
                    }
                }
                Err(_) => Ok(stdout.trim().to_string()),
            }
        };

        match tokio::time::timeout(timeout, execution_future).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("執行超時，超過 {} 秒", timeout.as_secs())),
        }
    }

    /// 檢查當前冷卻狀態
    pub async fn check_cooldown_status(&self) -> Result<CooldownInfo> {
        self.cooldown_detector.check_claude_doctor_cooldown().await
    }

    /// 獲取重試統計
    pub fn get_retry_stats(&self) -> crate::core::RetryStats {
        self.retry_orchestrator.get_retry_stats()
    }

    /// 獲取進程統計
    pub fn get_process_stats(&self) -> crate::core::ProcessStats {
        self.process_orchestrator.get_process_stats()
    }

    /// 取消正在執行的進程
    pub async fn cancel_process(&mut self, process_id: Uuid) -> Result<()> {
        self.process_orchestrator.cancel_process(process_id).await
    }

    /// 列出活躍進程
    pub fn list_active_processes(&self) -> Vec<&ProcessHandle> {
        self.process_orchestrator.list_active_processes()
    }

    /// 清理已完成的進程
    pub fn cleanup_completed_processes(&mut self) {
        self.process_orchestrator.cleanup_completed_processes();
    }

    /// 執行排程任務
    pub async fn execute_scheduled(
        &mut self,
        prompt: &str,
        options: ExecutionOptions,
        scheduling_config: SchedulingConfig,
    ) -> Result<Uuid> {
        use crate::core::{CronScheduler, AdaptiveScheduler, SessionScheduler};

        match scheduling_config.scheduler_type {
            SchedulerType::Cron => {
                if let Some(cron_config) = scheduling_config.cron {
                    let mut scheduler = CronScheduler::new().await?;
                    scheduler.start().await?;
                    
                    let handle = scheduler.schedule(cron_config).await?;
                    tracing::info!("Cron 排程任務已創建: {}", handle);
                    Ok(handle)
                } else {
                    Err(anyhow::anyhow!("Cron 配置缺失"))
                }
            }
            SchedulerType::Adaptive => {
                if let Some(adaptive_config) = scheduling_config.adaptive {
                    let mut scheduler = AdaptiveScheduler::new(adaptive_config.clone());
                    let handle = scheduler.schedule(adaptive_config).await?;
                    tracing::info!("適應性排程任務已創建: {}", handle);
                    Ok(handle)
                } else {
                    Err(anyhow::anyhow!("適應性配置缺失"))
                }
            }
            SchedulerType::Session => {
                if let Some(session_config) = scheduling_config.session {
                    let mut scheduler = SessionScheduler::new();
                    let handle = scheduler.schedule(session_config).await?;
                    tracing::info!("會話排程任務已創建: {}", handle);
                    Ok(handle)
                } else {
                    Err(anyhow::anyhow!("會話配置缺失"))
                }
            }
            SchedulerType::Immediate => {
                // 立即執行
                let response = self.execute_with_full_enhancement(prompt, options).await?;
                tracing::info!("立即執行完成: {}", response.execution_metadata.execution_id);
                Ok(response.execution_metadata.execution_id)
            }
        }
    }

    /// 優雅關閉執行器
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("開始關閉增強執行器");

        // 清理所有進程
        self.cleanup_completed_processes();

        // 等待活躍進程完成
        let active_processes = self.list_active_processes();
        if !active_processes.is_empty() {
            tracing::info!("等待 {} 個活躍進程完成", active_processes.len());
            
            // 給進程一些時間完成
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            
            // 強制取消剩餘進程
            let remaining_ids: Vec<Uuid> = self.list_active_processes()
                .iter()
                .map(|p| p.id)
                .collect();
            
            for process_id in remaining_ids {
                if let Err(e) = self.cancel_process(process_id).await {
                    tracing::warn!("取消進程 {} 失敗: {}", process_id, e);
                }
            }
        }

        tracing::info!("增強執行器已關閉");
        Ok(())
    }

    /// 健康檢查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus {
            claude_cli_available: false,
            cooldown_detection_working: false,
            current_cooldown: None,
            active_processes: 0,
            last_check: SystemTime::now(),
        };

        // 檢查 Claude CLI 可用性
        match tokio::process::Command::new("claude").arg("--version").output().await {
            Ok(output) if output.status.success() => {
                status.claude_cli_available = true;
            }
            _ => {
                status.claude_cli_available = false;
            }
        }

        // 檢查冷卻檢測
        match self.cooldown_detector.check_claude_doctor_cooldown().await {
            Ok(cooldown_info) => {
                status.cooldown_detection_working = true;
                if cooldown_info.is_cooling {
                    status.current_cooldown = Some(cooldown_info);
                }
            }
            Err(_) => {
                status.cooldown_detection_working = false;
            }
        }

        // 統計活躍進程
        status.active_processes = self.list_active_processes().len() as u32;

        Ok(status)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub claude_cli_available: bool,
    pub cooldown_detection_working: bool,
    pub current_cooldown: Option<CooldownInfo>,
    pub active_processes: u32,
    pub last_check: SystemTime,
}

// Tauri 命令介面
#[tauri::command]
pub async fn execute_enhanced_claude(
    prompt: String,
    options: ExecutionOptions,
) -> Result<EnhancedClaudeResponse, String> {
    let mut executor = EnhancedClaudeExecutor::with_smart_defaults()
        .map_err(|e| e.to_string())?;
    
    executor
        .execute_with_full_enhancement(&prompt, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_enhanced_cooldown() -> Result<CooldownInfo, String> {
    let executor = EnhancedClaudeExecutor::with_smart_defaults()
        .map_err(|e| e.to_string())?;
    
    executor
        .check_cooldown_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn health_check_enhanced() -> Result<HealthStatus, String> {
    let executor = EnhancedClaudeExecutor::with_smart_defaults()
        .map_err(|e| e.to_string())?;
    
    executor
        .health_check()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_executor_creation() {
        let executor = EnhancedClaudeExecutor::with_smart_defaults();
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let executor = EnhancedClaudeExecutor::with_smart_defaults().unwrap();
        let health = executor.health_check().await;
        assert!(health.is_ok());
    }

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert!(config.enable_cooldown_detection);
        assert!(config.enable_smart_retry);
        assert!(config.enable_process_orchestration);
        assert_eq!(config.max_concurrent_executions, 3);
    }

    #[test]
    fn test_execution_metadata_creation() {
        let metadata = ExecutionMetadata {
            execution_id: Uuid::new_v4(),
            start_time: SystemTime::now(),
            end_time: None,
            total_attempts: 0,
            cooldown_detected: None,
            process_handle: None,
            scheduler_used: None,
        };

        assert_eq!(metadata.total_attempts, 0);
        assert!(metadata.end_time.is_none());
    }
}