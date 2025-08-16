// Claude 響應模型 - 參考 vibe-kanban 設計

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Claude CLI 執行響應
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct ClaudeResponse {
    /// 響應 ID
    pub id: String,

    /// 對應的請求 ID
    pub request_id: String,

    /// 執行狀態
    pub status: ExecutionStatus,

    /// 輸出內容
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub output: Option<String>,

    /// 錯誤訊息
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error: Option<String>,

    /// 執行時間 (毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub execution_time_ms: Option<u64>,

    /// 使用統計
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub usage_stats: Option<UsageStats>,

    /// 流式數據片段 (stream-json 模式)
    #[serde(default)]
    pub stream_chunks: Vec<StreamChunk>,

    /// 響應時間
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    /// 完成時間
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "string | null")]
    pub completed_at: Option<DateTime<Utc>>,

    /// 元數據
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// 執行狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/")]
pub enum ExecutionStatus {
    /// 執行中
    Running,
    /// 執行成功
    Success,
    /// 執行失敗
    Failed,
    /// 被取消
    Cancelled,
    /// 超時
    Timeout,
    /// 冷卻中
    Cooldown,
}

/// 使用統計資訊
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct UsageStats {
    /// 輸入 token 數
    pub input_tokens: u64,

    /// 輸出 token 數
    pub output_tokens: u64,

    /// 總 token 數
    pub total_tokens: u64,

    /// 費用 (USD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,

    /// 模型名稱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// API 調用次數
    #[serde(default)]
    pub api_calls: u32,

    /// 緩存命中
    #[serde(default)]
    pub cache_hits: u32,
}

/// 流式數據片段
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct StreamChunk {
    /// 片段類型
    pub chunk_type: String,

    /// 內容
    pub content: String,

    /// 時間戳
    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,

    /// 序號
    pub sequence: u64,

    /// 是否最後一個片段
    #[serde(default)]
    pub is_final: bool,
}

impl ClaudeResponse {
    /// 創建新的響應
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            request_id: request_id.into(),
            status: ExecutionStatus::Running,
            output: None,
            error: None,
            execution_time_ms: None,
            usage_stats: None,
            stream_chunks: vec![],
            created_at: Utc::now(),
            completed_at: None,
            metadata: HashMap::new(),
        }
    }

    /// 標記為成功
    pub fn success(mut self, output: impl Into<String>) -> Self {
        self.status = ExecutionStatus::Success;
        self.output = Some(output.into());
        self.completed_at = Some(Utc::now());
        self
    }

    /// 標記為失敗
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.status = ExecutionStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(Utc::now());
        self
    }

    /// 標記為取消
    pub fn cancelled(mut self) -> Self {
        self.status = ExecutionStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self
    }

    /// 標記為超時
    pub fn timeout(mut self) -> Self {
        self.status = ExecutionStatus::Timeout;
        self.error = Some("執行超時".to_string());
        self.completed_at = Some(Utc::now());
        self
    }

    /// 標記為冷卻中
    pub fn cooldown(mut self, message: impl Into<String>) -> Self {
        self.status = ExecutionStatus::Cooldown;
        self.error = Some(message.into());
        self.completed_at = Some(Utc::now());
        self
    }

    /// 設置執行時間
    pub fn with_execution_time(mut self, start_time: DateTime<Utc>) -> Self {
        let now = Utc::now();
        self.execution_time_ms = Some((now - start_time).num_milliseconds() as u64);
        self
    }

    /// 設置使用統計
    pub fn with_usage_stats(mut self, stats: UsageStats) -> Self {
        self.usage_stats = Some(stats);
        self
    }

    /// 添加流式數據片段
    pub fn add_stream_chunk(&mut self, chunk_type: impl Into<String>, content: impl Into<String>) {
        let sequence = self.stream_chunks.len() as u64;
        self.stream_chunks.push(StreamChunk {
            chunk_type: chunk_type.into(),
            content: content.into(),
            timestamp: Utc::now(),
            sequence,
            is_final: false,
        });
    }

    /// 標記流式數據結束
    pub fn finalize_stream(&mut self) {
        if let Some(last_chunk) = self.stream_chunks.last_mut() {
            last_chunk.is_final = true;
        }
    }

    /// 添加元數據
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 檢查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatus::Success
                | ExecutionStatus::Failed
                | ExecutionStatus::Cancelled
                | ExecutionStatus::Timeout
                | ExecutionStatus::Cooldown
        )
    }

    /// 檢查是否成功
    pub fn is_success(&self) -> bool {
        self.status == ExecutionStatus::Success
    }

    /// 獲取完整輸出 (包含流式數據)
    pub fn get_full_output(&self) -> String {
        if let Some(output) = &self.output {
            output.clone()
        } else {
            self.stream_chunks
                .iter()
                .map(|chunk| chunk.content.as_str())
                .collect::<Vec<_>>()
                .join("")
        }
    }

    /// 計算總執行時間
    pub fn calculate_duration(&self) -> Option<i64> {
        self.completed_at
            .map(|completed| (completed - self.created_at).num_milliseconds())
    }
}

impl UsageStats {
    /// 創建新的使用統計
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: None,
            model: None,
            api_calls: 1,
            cache_hits: 0,
        }
    }

    /// 設置費用
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_usd = Some(cost);
        self
    }

    /// 設置模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 增加 API 調用次數
    pub fn increment_api_calls(&mut self) {
        self.api_calls += 1;
    }

    /// 增加緩存命中
    pub fn increment_cache_hits(&mut self) {
        self.cache_hits += 1;
    }

    /// 計算平均每個 token 的費用
    pub fn cost_per_token(&self) -> Option<f64> {
        self.cost_usd.map(|cost| {
            if self.total_tokens > 0 {
                cost / self.total_tokens as f64
            } else {
                0.0
            }
        })
    }

    /// 計算緩存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        if self.api_calls > 0 {
            self.cache_hits as f64 / self.api_calls as f64
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Running => write!(f, "執行中"),
            ExecutionStatus::Success => write!(f, "成功"),
            ExecutionStatus::Failed => write!(f, "失敗"),
            ExecutionStatus::Cancelled => write!(f, "已取消"),
            ExecutionStatus::Timeout => write!(f, "超時"),
            ExecutionStatus::Cooldown => write!(f, "冷卻中"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_response_creation() {
        let response = ClaudeResponse::new("req123");

        assert!(!response.id.is_empty());
        assert_eq!(response.request_id, "req123");
        assert_eq!(response.status, ExecutionStatus::Running);
        assert!(!response.is_completed());
    }

    #[test]
    fn test_response_status_transitions() {
        let response = ClaudeResponse::new("req123");

        let success_response = response.clone().success("測試輸出");
        assert!(success_response.is_success());
        assert!(success_response.is_completed());
        assert_eq!(success_response.output.unwrap(), "測試輸出");

        let failed_response = response.failed("測試錯誤");
        assert!(!failed_response.is_success());
        assert!(failed_response.is_completed());
        assert_eq!(failed_response.error.unwrap(), "測試錯誤");
    }

    #[test]
    fn test_stream_chunks() {
        let mut response = ClaudeResponse::new("req123");

        response.add_stream_chunk("assistant", "Hello");
        response.add_stream_chunk("assistant", " World");
        response.finalize_stream();

        assert_eq!(response.stream_chunks.len(), 2);
        assert!(response.stream_chunks[1].is_final);
        assert_eq!(response.get_full_output(), "Hello World");
    }

    #[test]
    fn test_usage_stats() {
        let stats = UsageStats::new(100, 200)
            .with_cost(0.05)
            .with_model("claude-3-5-sonnet");

        assert_eq!(stats.total_tokens, 300);
        assert_eq!(stats.cost_usd, Some(0.05));
        assert_eq!(stats.cost_per_token(), Some(0.05 / 300.0));
        assert_eq!(stats.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_execution_time_calculation() {
        let start_time = Utc::now() - chrono::Duration::seconds(5);
        let response = ClaudeResponse::new("req123").with_execution_time(start_time);

        assert!(response.execution_time_ms.is_some());
        assert!(response.execution_time_ms.unwrap() >= 5000); // 至少 5 秒
    }
}
