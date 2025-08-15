// Claude 執行服務 - 核心業務邏輯
// 採用 vibe-kanban 架構模式

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::models::{
    ClaudeRequest, ClaudeResponse, ExecutionResult, 
    ExecutionStatus, SecurityAudit, PerformanceMetrics
};

/// Claude 執行服務
/// 
/// 負責 Claude CLI 的執行管理、狀態追蹤和結果處理
pub struct ClaudeService {
    /// 執行中的任務追蹤
    active_executions: Arc<RwLock<HashMap<String, ClaudeExecution>>>,
    
    /// 執行歷史記錄
    execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
    
    /// 服務配置
    config: ClaudeServiceConfig,
}

/// Claude 執行任務
#[derive(Debug, Clone)]
pub struct ClaudeExecution {
    pub request_id: String,
    pub status: ExecutionStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub process_handle: Option<String>,
}

/// Claude 服務配置
#[derive(Debug, Clone)]
pub struct ClaudeServiceConfig {
    /// 最大並行執行數
    pub max_concurrent_executions: u32,
    
    /// 預設超時時間 (秒)
    pub default_timeout_seconds: u64,
    
    /// 啟用安全檢查
    pub enable_security_audit: bool,
    
    /// 啟用性能監控
    pub enable_performance_monitoring: bool,
    
    /// Claude CLI 路徑
    pub claude_cli_path: String,
}

impl ClaudeService {
    /// 創建新的 Claude 服務實例
    pub fn new(config: ClaudeServiceConfig) -> Self {
        Self {
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }
    
    /// 使用預設配置創建服務
    pub fn with_defaults() -> Self {
        let config = ClaudeServiceConfig {
            max_concurrent_executions: 3,
            default_timeout_seconds: 300, // 5 分鐘
            enable_security_audit: true,
            enable_performance_monitoring: true,
            claude_cli_path: "npx @anthropic-ai/claude-code@latest".to_string(),
        };
        
        Self::new(config)
    }
    
    /// 提交執行請求
    pub async fn submit_execution(
        &self, 
        request: ClaudeRequest
    ) -> Result<String> {
        // 檢查並行執行限制
        let active_count = self.active_executions.read().await.len() as u32;
        if active_count >= self.config.max_concurrent_executions {
            return Err(anyhow::anyhow!(
                "已達到最大並行執行數限制: {}", 
                self.config.max_concurrent_executions
            ));
        }
        
        // 安全檢查
        if self.config.enable_security_audit {
            let audit_result = self.perform_security_audit(&request).await?;
            if audit_result.risk_level >= crate::models::RiskLevel::High {
                return Err(anyhow::anyhow!(
                    "安全檢查失敗: 風險等級過高 ({})", 
                    audit_result.risk_level
                ));
            }
        }
        
        // 創建執行任務
        let execution = ClaudeExecution {
            request_id: request.id.clone(),
            status: ExecutionStatus::Running,
            start_time: chrono::Utc::now(),
            process_handle: None,
        };
        
        // 添加到活動執行列表
        self.active_executions.write().await.insert(
            request.id.clone(), 
            execution
        );
        
        // 異步執行 Claude CLI
        let service = self.clone();
        let request_clone = request.clone();
        tokio::spawn(async move {
            let _ = service.execute_claude_cli(request_clone).await;
        });
        
        Ok(request.id)
    }
    
    /// 獲取執行狀態
    pub async fn get_execution_status(&self, request_id: &str) -> Option<ExecutionStatus> {
        self.active_executions
            .read()
            .await
            .get(request_id)
            .map(|exec| exec.status.clone())
    }
    
    /// 取消執行
    pub async fn cancel_execution(&self, request_id: &str) -> Result<()> {
        let mut executions = self.active_executions.write().await;
        
        if let Some(mut execution) = executions.remove(request_id) {
            execution.status = ExecutionStatus::Cancelled;
            
            // 這裡應該實現實際的進程取消邏輯
            // 例如發送 SIGTERM 信號給 Claude CLI 進程
            
            // 記錄取消的執行結果
            let response = ClaudeResponse::new(request_id).cancelled();
            let result = ExecutionResult::new(
                ClaudeRequest::new("已取消的請求"), 
                response
            );
            
            self.execution_history.write().await.push(result);
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("執行任務不存在: {}", request_id))
        }
    }
    
    /// 獲取執行歷史
    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<ExecutionResult> {
        let history = self.execution_history.read().await;
        
        match limit {
            Some(limit) => history.iter().rev().take(limit).cloned().collect(),
            None => history.clone(),
        }
    }
    
    /// 獲取活動執行統計
    pub async fn get_active_executions_stats(&self) -> HashMap<String, u32> {
        let executions = self.active_executions.read().await;
        let mut stats = HashMap::new();
        
        for execution in executions.values() {
            let status_key = format!("{:?}", execution.status);
            *stats.entry(status_key).or_insert(0) += 1;
        }
        
        stats.insert("total".to_string(), executions.len() as u32);
        stats
    }
    
    /// 實際執行 Claude CLI
    async fn execute_claude_cli(&self, request: ClaudeRequest) -> Result<()> {
        let start_time = chrono::Utc::now();
        
        // 構建 CLI 命令
        let args = request.to_cli_args();
        
        // 執行 Claude CLI (這裡使用模擬實現)
        let result = self.simulate_claude_execution(&request, &args).await;
        
        // 計算執行時間
        let execution_time = (chrono::Utc::now() - start_time).num_milliseconds() as u64;
        
        // 創建響應
        let response = match result {
            Ok(output) => ClaudeResponse::new(&request.id)
                .success(output)
                .with_execution_time(start_time),
            Err(error) => ClaudeResponse::new(&request.id)
                .failed(error.to_string())
                .with_execution_time(start_time),
        };
        
        // 性能監控
        let performance_metrics = if self.config.enable_performance_monitoring {
            Some(PerformanceMetrics::new(execution_time, 100)) // 假設 CLI 啟動時間為 100ms
        } else {
            None
        };
        
        // 創建執行結果
        let mut execution_result = ExecutionResult::new(request.clone(), response);
        
        if let Some(metrics) = performance_metrics {
            execution_result = execution_result.with_performance_metrics(metrics);
        }
        
        // 移除活動執行並添加到歷史
        self.active_executions.write().await.remove(&request.id);
        self.execution_history.write().await.push(execution_result);
        
        Ok(())
    }
    
    /// 模擬 Claude 執行 (開發階段使用)
    async fn simulate_claude_execution(
        &self, 
        request: &ClaudeRequest, 
        _args: &[String]
    ) -> Result<String> {
        // 模擬執行延遲
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // 模擬成功響應
        Ok(format!(
            "模擬 Claude 響應 for 提示: '{}'\n執行時間: {}",
            request.prompt.chars().take(50).collect::<String>(),
            chrono::Utc::now().format("%H:%M:%S")
        ))
    }
    
    /// 執行安全審計
    async fn perform_security_audit(&self, request: &ClaudeRequest) -> Result<SecurityAudit> {
        let mut audit = SecurityAudit::new(&request.prompt);
        
        // 檢查敏感資訊
        if self.contains_sensitive_data(&request.prompt) {
            audit.sensitive_data_detected = true;
            audit.add_risk(crate::models::SecurityRisk {
                risk_type: "敏感資料".to_string(),
                description: "檢測到可能的敏感資訊".to_string(),
                severity: 0.7,
                mitigation: Some("請移除敏感資訊後重新提交".to_string()),
            });
        }
        
        // 檢查危險指令
        if self.contains_dangerous_commands(&request.prompt) {
            audit.add_risk(crate::models::SecurityRisk {
                risk_type: "危險指令".to_string(),
                description: "檢測到可能的危險系統指令".to_string(),
                severity: 0.9,
                mitigation: Some("請確認指令安全性".to_string()),
            });
        }
        
        Ok(audit)
    }
    
    /// 檢查敏感資料
    fn contains_sensitive_data(&self, prompt: &str) -> bool {
        let sensitive_patterns = [
            r"password", r"secret", r"token", r"key", 
            r"api[_-]?key", r"credential", r"auth"
        ];
        
        for pattern in &sensitive_patterns {
            if regex::Regex::new(pattern)
                .unwrap()
                .is_match(&prompt.to_lowercase()) 
            {
                return true;
            }
        }
        
        false
    }
    
    /// 檢查危險指令
    fn contains_dangerous_commands(&self, prompt: &str) -> bool {
        let dangerous_patterns = [
            r"rm\s+-rf", r"sudo\s+rm", r"format", r"del\s+/s",
            r"rmdir\s+/s", r"mkfs", r"fdisk"
        ];
        
        for pattern in &dangerous_patterns {
            if regex::Regex::new(pattern)
                .unwrap()
                .is_match(&prompt.to_lowercase())
            {
                return true;
            }
        }
        
        false
    }
}

impl Clone for ClaudeService {
    fn clone(&self) -> Self {
        Self {
            active_executions: Arc::clone(&self.active_executions),
            execution_history: Arc::clone(&self.execution_history),
            config: self.config.clone(),
        }
    }
}

// 實現 Service trait
#[async_trait::async_trait]
impl super::Service for ClaudeService {
    fn name(&self) -> &'static str {
        "claude_service"
    }
    
    async fn start(&self) -> Result<()> {
        tracing::info!("Claude 服務已啟動");
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // 停止所有活動執行
        let mut executions = self.active_executions.write().await;
        for (request_id, _) in executions.drain() {
            tracing::info!("正在停止執行: {}", request_id);
        }
        tracing::info!("Claude 服務已停止");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // 檢查 Claude CLI 是否可用
        if let Ok(output) = tokio::process::Command::new("which")
            .arg("npx")
            .output()
            .await 
        {
            output.status.success()
        } else {
            false
        }
    }
    
    async fn get_stats(&self) -> serde_json::Value {
        let active_stats = self.get_active_executions_stats().await;
        let history = self.execution_history.read().await;
        
        serde_json::json!({
            "service": self.name(),
            "active_executions": active_stats,
            "total_history_count": history.len(),
            "config": {
                "max_concurrent": self.config.max_concurrent_executions,
                "default_timeout": self.config.default_timeout_seconds,
                "security_audit": self.config.enable_security_audit,
                "performance_monitoring": self.config.enable_performance_monitoring
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClaudeRequest, ExecutionOptions, RequestSource};
    
    #[tokio::test]
    async fn test_claude_service_creation() {
        let service = ClaudeService::with_defaults();
        assert_eq!(service.config.max_concurrent_executions, 3);
        assert!(service.config.enable_security_audit);
    }
    
    #[tokio::test]
    async fn test_execution_submission() {
        let service = ClaudeService::with_defaults();
        let request = ClaudeRequest::new("測試提示")
            .with_source(RequestSource::Cli);
        
        let execution_id = service.submit_execution(request).await.unwrap();
        assert!(!execution_id.is_empty());
        
        // 等待執行完成
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        
        let status = service.get_execution_status(&execution_id).await;
        assert!(status.is_none()); // 執行完成後應該從活動列表移除
        
        let history = service.get_execution_history(Some(1)).await;
        assert_eq!(history.len(), 1);
    }
    
    #[tokio::test]
    async fn test_security_audit() {
        let service = ClaudeService::with_defaults();
        
        // 測試包含敏感資訊的請求
        let sensitive_request = ClaudeRequest::new("Please use this API key: sk-123456");
        let audit = service.perform_security_audit(&sensitive_request).await.unwrap();
        
        assert!(audit.sensitive_data_detected);
        assert!(!audit.detected_risks.is_empty());
    }
    
    #[tokio::test]
    async fn test_concurrent_execution_limit() {
        let config = ClaudeServiceConfig {
            max_concurrent_executions: 1, // 限制為 1
            ..ClaudeServiceConfig {
                max_concurrent_executions: 1,
                default_timeout_seconds: 300,
                enable_security_audit: false,
                enable_performance_monitoring: false,
                claude_cli_path: "mock".to_string(),
            }
        };
        
        let service = ClaudeService::new(config);
        
        // 提交第一個請求
        let request1 = ClaudeRequest::new("第一個請求");
        let _execution_id1 = service.submit_execution(request1).await.unwrap();
        
        // 提交第二個請求應該失敗
        let request2 = ClaudeRequest::new("第二個請求");
        let result = service.submit_execution(request2).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("最大並行執行數限制"));
    }
}