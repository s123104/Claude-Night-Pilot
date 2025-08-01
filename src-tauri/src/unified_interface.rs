// 統一的CLI-GUI介面橋接器
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::enhanced_executor::{EnhancedClaudeExecutor, EnhancedClaudeResponse};
use crate::core::{CooldownInfo, ExecutionOptions};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedExecutionOptions {
    pub mode: String,                    // "sync", "async", "scheduled"
    pub cron_expr: Option<String>,       // 排程表達式
    pub retry_enabled: Option<bool>,     // 是否啟用重試
    pub cooldown_check: Option<bool>,    // 是否檢查冷卻
    pub working_directory: Option<String>, // 工作目錄
}

impl From<UnifiedExecutionOptions> for ExecutionOptions {
    fn from(options: UnifiedExecutionOptions) -> Self {
        Self {
            working_directory: options.working_directory,
            timeout_seconds: Some(300), // 預設5分鐘超時
            skip_permissions: false,
            output_format: "json".to_string(),
            dry_run: false,
            allowed_operations: vec!["claude_execute".to_string()],
            safety_check: options.cooldown_check.unwrap_or(true),
            max_retries: if options.retry_enabled.unwrap_or(true) { 3 } else { 0 },
        }
    }
}

/// 統一的Claude執行介面 - 供GUI和CLI共用
pub struct UnifiedClaudeInterface;

impl UnifiedClaudeInterface {
    /// 執行Claude命令 - GUI和CLI統一入口
    pub async fn execute_claude(
        prompt: String,
        options: UnifiedExecutionOptions,
    ) -> Result<EnhancedClaudeResponse> {
        let mut executor = EnhancedClaudeExecutor::with_smart_defaults()?;
        let execution_options = ExecutionOptions::from(options);
        
        executor.execute_with_full_enhancement(
            &prompt,
            execution_options,
        ).await
    }

    /// 檢查冷卻狀態 - GUI和CLI統一入口 (帶重試機制)
    pub async fn check_cooldown() -> Result<CooldownInfo> {
        Self::check_cooldown_with_retry(3).await
    }

    /// 帶重試機制的冷卻檢查
    pub async fn check_cooldown_with_retry(max_retries: u8) -> Result<CooldownInfo> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match Self::try_cooldown_check().await {
                Ok(result) => {
                    tracing::debug!("冷卻檢查成功 (嘗試 {}/{})", attempt, max_retries);
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        let delay = std::time::Duration::from_millis(500 * attempt as u64);
                        tracing::warn!("冷卻檢查失敗 (嘗試 {}/{}), {}ms 後重試", attempt, max_retries, delay.as_millis());
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("冷卻檢查失敗，超過最大重試次數")))
    }

    /// 單次冷卻檢查嘗試
    async fn try_cooldown_check() -> Result<CooldownInfo> {
        let executor = EnhancedClaudeExecutor::with_smart_defaults()?;
        executor.check_cooldown_status().await
    }

    /// 系統健康檢查 - GUI和CLI統一入口
    pub async fn health_check() -> Result<serde_json::Value> {
        let executor = EnhancedClaudeExecutor::with_smart_defaults()?;
        let health = executor.health_check().await?;
        
        Ok(serde_json::json!({
            "claude_cli_available": health.claude_cli_available,
            "cooldown_detection_working": health.cooldown_detection_working,
            "current_cooldown": health.current_cooldown,
            "active_processes": health.active_processes,
            "last_check": health.last_check,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_cooldown_check() {
        let result = UnifiedClaudeInterface::check_cooldown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unified_health_check() {
        let result = UnifiedClaudeInterface::health_check().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_options_conversion() {
        let unified_options = UnifiedExecutionOptions {
            mode: "sync".to_string(),
            cron_expr: None,
            retry_enabled: Some(true),
            cooldown_check: Some(true),
            working_directory: Some("/tmp".to_string()),
        };

        let execution_options = ExecutionOptions::from(unified_options);
        assert_eq!(execution_options.working_directory, Some("/tmp".to_string()));
        assert_eq!(execution_options.timeout_seconds, Some(300));
        assert!(execution_options.safety_check);
        assert_eq!(execution_options.max_retries, 3);
    }
}