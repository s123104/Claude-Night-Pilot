// 執行結果模型 - 參考 vibe-kanban 設計

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{ClaudeRequest, ClaudeResponse, ExecutionStatus, UsageStats};

/// 完整的執行結果，包含請求、響應和審計資訊
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionResult {
    /// 執行結果 ID
    pub id: String,

    /// 原始請求
    pub request: ClaudeRequest,

    /// 執行響應
    pub response: ClaudeResponse,

    /// 安全審計資訊
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub security_audit: Option<SecurityAudit>,

    /// 性能指標
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub performance_metrics: Option<PerformanceMetrics>,

    /// 重試歷史
    #[serde(default)]
    pub retry_history: Vec<RetryAttempt>,

    /// 標籤和分類
    #[serde(default)]
    pub tags: Vec<String>,

    /// 創建時間
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    /// 最後更新時間
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

/// 安全審計資訊
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SecurityAudit {
    /// 風險等級
    pub risk_level: RiskLevel,

    /// 風險分數 (0.0-1.0)
    pub risk_score: f64,

    /// 檢測到的風險項目
    pub detected_risks: Vec<SecurityRisk>,

    /// 提示內容雜湊值 (SHA256)
    pub prompt_hash: String,

    /// 敏感資料檢測
    pub sensitive_data_detected: bool,

    /// 審計時間
    #[ts(type = "string")]
    pub audit_timestamp: DateTime<Utc>,

    /// 審計版本
    pub audit_version: String,
}

/// 風險等級
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(export)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 安全風險項目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SecurityRisk {
    /// 風險類型
    pub risk_type: String,

    /// 風險描述
    pub description: String,

    /// 嚴重性分數 (0.0-1.0)
    pub severity: f64,

    /// 建議措施
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mitigation: Option<String>,
}

/// 性能指標
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PerformanceMetrics {
    /// 總執行時間 (毫秒)
    pub total_execution_time_ms: u64,

    /// Claude CLI 啟動時間 (毫秒)
    pub cli_startup_time_ms: u64,

    /// 網路延遲 (毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub network_latency_ms: Option<u64>,

    /// 記憶體使用峰值 (MB)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub peak_memory_mb: Option<u64>,

    /// CPU 使用率峰值 (%)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub peak_cpu_percent: Option<f64>,

    /// 磁碟 I/O (MB)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub disk_io_mb: Option<u64>,

    /// 性能等級
    pub performance_grade: PerformanceGrade,

    /// 瓶頸分析
    #[serde(default)]
    pub bottlenecks: Vec<String>,
}

/// 性能等級
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PerformanceGrade {
    Excellent, // < 1秒
    Good,      // 1-3秒
    Average,   // 3-10秒
    Poor,      // 10-30秒
    Critical,  // > 30秒
}

/// 重試嘗試記錄
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RetryAttempt {
    /// 嘗試次數
    pub attempt_number: u32,

    /// 失敗原因
    pub failure_reason: String,

    /// 重試時間
    #[ts(type = "string")]
    pub retry_timestamp: DateTime<Utc>,

    /// 等待時間 (毫秒)
    pub wait_time_ms: u64,

    /// 重試策略
    pub retry_strategy: String,
}

impl ExecutionResult {
    /// 創建新的執行結果
    pub fn new(request: ClaudeRequest, response: ClaudeResponse) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request,
            response,
            security_audit: None,
            performance_metrics: None,
            retry_history: vec![],
            tags: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加安全審計
    pub fn with_security_audit(mut self, audit: SecurityAudit) -> Self {
        self.security_audit = Some(audit);
        self.updated_at = Utc::now();
        self
    }

    /// 添加性能指標
    pub fn with_performance_metrics(mut self, metrics: PerformanceMetrics) -> Self {
        self.performance_metrics = Some(metrics);
        self.updated_at = Utc::now();
        self
    }

    /// 添加重試記錄
    pub fn add_retry_attempt(&mut self, attempt: RetryAttempt) {
        self.retry_history.push(attempt);
        self.updated_at = Utc::now();
    }

    /// 添加標籤
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 檢查是否成功
    pub fn is_success(&self) -> bool {
        self.response.is_success()
    }

    /// 獲取最終狀態
    pub fn final_status(&self) -> &ExecutionStatus {
        &self.response.status
    }

    /// 獲取總重試次數
    pub fn total_retries(&self) -> usize {
        self.retry_history.len()
    }

    /// 計算總執行時間 (包含重試)
    pub fn total_execution_time_with_retries(&self) -> Option<i64> {
        let base_time = self.response.calculate_duration()?;
        let retry_time: i64 = self
            .retry_history
            .iter()
            .map(|retry| retry.wait_time_ms as i64)
            .sum();
        Some(base_time + retry_time)
    }

    /// 獲取風險等級
    pub fn risk_level(&self) -> RiskLevel {
        self.security_audit
            .as_ref()
            .map(|audit| audit.risk_level.clone())
            .unwrap_or(RiskLevel::Low)
    }

    /// 獲取性能等級
    pub fn performance_grade(&self) -> Option<&PerformanceGrade> {
        self.performance_metrics
            .as_ref()
            .map(|metrics| &metrics.performance_grade)
    }

    /// 生成執行摘要
    pub fn generate_summary(&self) -> ExecutionSummary {
        ExecutionSummary {
            id: self.id.clone(),
            status: self.response.status.clone(),
            execution_time_ms: self.response.execution_time_ms,
            usage_stats: self.response.usage_stats.clone(),
            risk_level: self.risk_level(),
            performance_grade: self.performance_grade().cloned(),
            retry_count: self.total_retries() as u32,
            created_at: self.created_at,
        }
    }
}

/// 執行摘要 (用於列表顯示)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExecutionSummary {
    pub id: String,
    pub status: ExecutionStatus,
    #[ts(optional)]
    pub execution_time_ms: Option<u64>,
    #[ts(optional)]
    pub usage_stats: Option<UsageStats>,
    pub risk_level: RiskLevel,
    #[ts(optional)]
    pub performance_grade: Option<PerformanceGrade>,
    pub retry_count: u32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

impl SecurityAudit {
    /// 創建新的安全審計
    pub fn new(prompt: &str) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let prompt_hash = format!("{:x}", hasher.finalize());

        Self {
            risk_level: RiskLevel::Low,
            risk_score: 0.0,
            detected_risks: vec![],
            prompt_hash,
            sensitive_data_detected: false,
            audit_timestamp: Utc::now(),
            audit_version: "1.0.0".to_string(),
        }
    }

    /// 添加風險項目
    pub fn add_risk(&mut self, risk: SecurityRisk) {
        self.detected_risks.push(risk);
        self.recalculate_risk_level();
    }

    /// 重新計算風險等級
    fn recalculate_risk_level(&mut self) {
        if self.detected_risks.is_empty() {
            self.risk_level = RiskLevel::Low;
            self.risk_score = 0.0;
            return;
        }

        // 計算平均風險分數
        let total_severity: f64 = self.detected_risks.iter().map(|risk| risk.severity).sum();

        self.risk_score = total_severity / self.detected_risks.len() as f64;

        // 根據分數確定等級
        self.risk_level = match self.risk_score {
            score if score >= 0.8 => RiskLevel::Critical,
            score if score >= 0.6 => RiskLevel::High,
            score if score >= 0.3 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };
    }
}

impl PerformanceMetrics {
    /// 創建新的性能指標
    pub fn new(total_time_ms: u64, cli_startup_ms: u64) -> Self {
        let grade = Self::calculate_grade(total_time_ms);

        Self {
            total_execution_time_ms: total_time_ms,
            cli_startup_time_ms: cli_startup_ms,
            network_latency_ms: None,
            peak_memory_mb: None,
            peak_cpu_percent: None,
            disk_io_mb: None,
            performance_grade: grade,
            bottlenecks: vec![],
        }
    }

    /// 根據執行時間計算性能等級
    fn calculate_grade(time_ms: u64) -> PerformanceGrade {
        match time_ms {
            0..=1000 => PerformanceGrade::Excellent,
            1001..=3000 => PerformanceGrade::Good,
            3001..=10000 => PerformanceGrade::Average,
            10001..=30000 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Critical,
        }
    }

    /// 添加瓶頸分析
    pub fn add_bottleneck(&mut self, bottleneck: impl Into<String>) {
        self.bottlenecks.push(bottleneck.into());
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "低風險"),
            RiskLevel::Medium => write!(f, "中風險"),
            RiskLevel::High => write!(f, "高風險"),
            RiskLevel::Critical => write!(f, "極高風險"),
        }
    }
}

impl std::fmt::Display for PerformanceGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceGrade::Excellent => write!(f, "優秀"),
            PerformanceGrade::Good => write!(f, "良好"),
            PerformanceGrade::Average => write!(f, "一般"),
            PerformanceGrade::Poor => write!(f, "較差"),
            PerformanceGrade::Critical => write!(f, "極差"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClaudeRequest, ClaudeResponse, ExecutionOptions};

    #[test]
    fn test_execution_result_creation() {
        let request = ClaudeRequest::new("測試請求");
        let response = ClaudeResponse::new(&request.id).success("測試輸出");

        let result = ExecutionResult::new(request, response);

        assert!(!result.id.is_empty());
        assert!(result.is_success());
        assert_eq!(result.total_retries(), 0);
    }

    #[test]
    fn test_security_audit() {
        let mut audit = SecurityAudit::new("測試提示");

        audit.add_risk(SecurityRisk {
            risk_type: "敏感資料".to_string(),
            description: "檢測到密碼".to_string(),
            severity: 0.8,
            mitigation: Some("移除敏感資訊".to_string()),
        });

        assert_eq!(audit.risk_level, RiskLevel::High);
        assert_eq!(audit.detected_risks.len(), 1);
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new(500, 100);

        assert!(matches!(
            metrics.performance_grade,
            PerformanceGrade::Excellent
        ));
        assert_eq!(metrics.total_execution_time_ms, 500);
        assert_eq!(metrics.cli_startup_time_ms, 100);
    }

    #[test]
    fn test_retry_history() {
        let request = ClaudeRequest::new("測試請求");
        let response = ClaudeResponse::new(&request.id).failed("網路錯誤");
        let mut result = ExecutionResult::new(request, response);

        result.add_retry_attempt(RetryAttempt {
            attempt_number: 1,
            failure_reason: "連線超時".to_string(),
            retry_timestamp: Utc::now(),
            wait_time_ms: 1000,
            retry_strategy: "指數退避".to_string(),
        });

        assert_eq!(result.total_retries(), 1);
    }
}
