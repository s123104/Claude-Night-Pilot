// Claude 請求模型 - 參考 vibe-kanban 設計

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// Claude CLI 執行請求
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct ClaudeRequest {
    /// 請求 ID
    pub id: String,

    /// 提示內容
    pub prompt: String,

    /// 執行選項
    pub options: ExecutionOptions,

    /// 請求時間
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    /// 請求來源 (CLI/GUI)
    #[serde(default)]
    pub source: RequestSource,

    /// 會話 ID (用於續接)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub session_id: Option<String>,

    /// 元數據
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// 執行選項配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct ExecutionOptions {
    /// 跳過權限確認
    #[serde(default)]
    pub skip_permissions: bool,

    /// 輸出格式 (json/text/stream-json)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// 超時時間 (秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout_seconds: Option<u64>,

    /// 乾運行模式
    #[serde(default)]
    pub dry_run: bool,

    /// 工作目錄
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_directory: Option<String>,

    /// 允許的操作類型
    #[serde(default)]
    pub allowed_operations: Vec<String>,

    /// 安全檢查
    #[serde(default = "default_safety_check")]
    pub safety_check: bool,

    /// 最大重試次數
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// 使用 @ 符號文件引用
    #[serde(default)]
    pub enable_file_references: bool,

    /// 流式輸出處理
    #[serde(default = "default_stream_processing")]
    pub stream_processing: bool,
}

/// 請求來源類型
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "../../src/types/")]
pub enum RequestSource {
    #[default]
    Cli,
    Gui,
    Api,
    Scheduler,
}

impl ClaudeRequest {
    /// 創建新的 Claude 請求
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.into(),
            options: ExecutionOptions::default(),
            created_at: Utc::now(),
            source: RequestSource::default(),
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    /// 設置執行選項
    pub fn with_options(mut self, options: ExecutionOptions) -> Self {
        self.options = options;
        self
    }

    /// 設置請求來源
    pub fn with_source(mut self, source: RequestSource) -> Self {
        self.source = source;
        self
    }

    /// 設置會話 ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// 添加元數據
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 驗證請求有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() {
            return Err("提示內容不能為空".to_string());
        }

        if let Some(timeout) = self.options.timeout_seconds {
            if timeout == 0 {
                return Err("超時時間必須大於 0".to_string());
            }
        }

        Ok(())
    }

    /// 生成 Claude CLI 命令參數
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![];

        // 基本參數
        args.push("--output-format".to_string());
        args.push(self.options.output_format.clone());

        if self.options.skip_permissions {
            args.push("--dangerously-skip-permissions".to_string());
        }

        if self.options.dry_run {
            args.push("--dry-run".to_string());
        }

        if let Some(timeout) = self.options.timeout_seconds {
            args.push("--timeout".to_string());
            args.push(timeout.to_string());
        }

        if let Some(session_id) = &self.session_id {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        // 提示內容 (作為最後一個參數)
        args.push("--prompt".to_string());
        args.push(self.prompt.clone());

        args
    }
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            skip_permissions: false,
            output_format: default_output_format(),
            timeout_seconds: None,
            dry_run: false,
            working_directory: None,
            allowed_operations: vec![],
            safety_check: default_safety_check(),
            max_retries: default_max_retries(),
            enable_file_references: false,
            stream_processing: default_stream_processing(),
        }
    }
}

// 默認值函數
fn default_output_format() -> String {
    "stream-json".to_string()
}

fn default_safety_check() -> bool {
    true
}

fn default_max_retries() -> u32 {
    3
}

fn default_stream_processing() -> bool {
    true
}

impl std::fmt::Display for RequestSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestSource::Cli => write!(f, "CLI"),
            RequestSource::Gui => write!(f, "GUI"),
            RequestSource::Api => write!(f, "API"),
            RequestSource::Scheduler => write!(f, "Scheduler"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_request_creation() {
        let request = ClaudeRequest::new("測試提示");

        assert!(!request.id.is_empty());
        assert_eq!(request.prompt, "測試提示");
        assert!(matches!(request.source, RequestSource::Cli));
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_request_validation() {
        let empty_request = ClaudeRequest::new("");
        assert!(empty_request.validate().is_err());

        let valid_request = ClaudeRequest::new("有效提示");
        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_cli_args_generation() {
        let request = ClaudeRequest::new("測試命令")
            .with_options(ExecutionOptions {
                skip_permissions: true,
                dry_run: true,
                timeout_seconds: Some(30),
                ..Default::default()
            })
            .with_session_id("session123");

        let args = request.to_cli_args();

        assert!(args.contains(&"--dangerously-skip-permissions".to_string()));
        assert!(args.contains(&"--dry-run".to_string()));
        assert!(args.contains(&"--timeout".to_string()));
        assert!(args.contains(&"30".to_string()));
        assert!(args.contains(&"--resume".to_string()));
        assert!(args.contains(&"session123".to_string()));
    }

    #[test]
    fn test_request_builder_pattern() {
        let request = ClaudeRequest::new("構建器測試")
            .with_source(RequestSource::Gui)
            .with_metadata("user_id", "12345")
            .with_metadata("priority", "high");

        assert!(matches!(request.source, RequestSource::Gui));
        assert_eq!(request.metadata.get("user_id"), Some(&"12345".to_string()));
        assert_eq!(request.metadata.get("priority"), Some(&"high".to_string()));
    }
}
