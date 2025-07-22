use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use std::time::SystemTime;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub completion: String,
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CooldownInfo {
    pub is_cooling: bool,
    pub seconds_remaining: u64,
    pub next_available_time: Option<SystemTime>,
}

pub struct ClaudeExecutor;

impl ClaudeExecutor {
    /// 同步執行 Claude prompt
    pub async fn run_sync(prompt: &str) -> Result<String> {
        let output = AsyncCommand::new("claude")
            .arg("-p")
            .arg(prompt)
            .arg("--output-format")
            .arg("json")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // 檢查是否為冷卻錯誤
            if stderr.contains("429") || stderr.contains("cooldown") {
                // 嘗試解析冷卻時間
                if let Some(seconds) = Self::parse_cooldown_from_error(&stderr) {
                    bail!("Claude CLI 正在冷卻中，剩餘 {} 秒", seconds);
                } else {
                    bail!("Claude CLI 冷卻中，請稍後再試");
                }
            }

            bail!("Claude CLI 執行失敗: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 嘗試解析 JSON 回應
        match serde_json::from_str::<ClaudeResponse>(&stdout) {
            Ok(response) => Ok(response.completion),
            Err(_) => {
                // 如果不是 JSON 格式，直接返回原始輸出
                Ok(stdout.trim().to_string())
            }
        }
    }

    /// 檢查 Claude CLI 狀態和冷卻時間
    pub async fn check_cooldown() -> Result<CooldownInfo> {
        let output = AsyncCommand::new("claude")
            .arg("doctor")
            .arg("--json")
            .output()
            .await?;

        if !output.status.success() {
            // 如果 doctor 命令失敗，假設沒有冷卻
            return Ok(CooldownInfo {
                is_cooling: false,
                seconds_remaining: 0,
                next_available_time: None,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 嘗試解析診斷資訊
        if let Ok(diag) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(cooldown_secs) = diag.get("cooldown_seconds").and_then(|v| v.as_u64()) {
                return Ok(CooldownInfo {
                    is_cooling: cooldown_secs > 0,
                    seconds_remaining: cooldown_secs,
                    next_available_time: if cooldown_secs > 0 {
                        Some(SystemTime::now() + std::time::Duration::from_secs(cooldown_secs))
                    } else {
                        None
                    },
                });
            }
        }

        // 如果無法解析，假設沒有冷卻
        Ok(CooldownInfo {
            is_cooling: false,
            seconds_remaining: 0,
            next_available_time: None,
        })
    }

    /// 驗證 Claude CLI 是否可用
    pub async fn verify_claude_cli() -> Result<bool> {
        let output = AsyncCommand::new("claude").arg("--version").output().await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 從錯誤訊息中解析冷卻時間
    pub fn parse_cooldown_from_error(error_message: &str) -> Option<u64> {
        // 尋找冷卻時間模式: "cooldown: 123s" 或 "wait 123 seconds"
        use regex::Regex;

        let patterns = [
            r"cooldown[:\s]+(\d+)s",
            r"wait\s+(\d+)\s+seconds?",
            r"retry\s+in\s+(\d+)\s+seconds?",
            r"(\d+)\s+seconds?\s+remaining",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(error_message) {
                    if let Some(seconds_str) = captures.get(1) {
                        if let Ok(seconds) = seconds_str.as_str().parse::<u64>() {
                            return Some(seconds);
                        }
                    }
                }
            }
        }

        None
    }

    /// 建立測試用的 mock executor（開發階段使用）
    #[cfg(debug_assertions)]
    pub async fn run_mock(prompt: &str) -> Result<String> {
        // 模擬執行時間
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(format!(
            "模擬回應 [{}]: 這是對 prompt '{}' 的模擬回應。此為開發階段的測試功能。",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            prompt.chars().take(50).collect::<String>()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cooldown_from_error() {
        let test_cases = [
            ("Error: cooldown: 123s", Some(123)),
            ("Please wait 45 seconds before retrying", Some(45)),
            ("Rate limited. retry in 60 seconds", Some(60)),
            ("30 seconds remaining", Some(30)),
            ("No cooldown message", None),
        ];

        for (error_msg, expected) in &test_cases {
            assert_eq!(
                ClaudeExecutor::parse_cooldown_from_error(error_msg),
                *expected,
                "Failed for: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_mock_executor() {
        let result = ClaudeExecutor::run_mock("Test prompt").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("模擬回應"));
        assert!(response.contains("Test prompt"));
    }
}
