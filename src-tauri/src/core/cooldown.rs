// 冷卻檢測與管理系統 - 基於研究分析的最佳實踐
use anyhow::Result;
use chrono::{DateTime, Local, NaiveTime, TimeZone};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownInfo {
    pub is_cooling: bool,
    pub seconds_remaining: u64,
    pub next_available_time: Option<SystemTime>,
    pub reset_time: Option<DateTime<Local>>,
    pub original_message: String,
    pub cooldown_pattern: Option<CooldownPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CooldownPattern {
    UsageLimitReached { reset_time: String },
    RateLimitExceeded { seconds: u64 },
    ApiQuotaExhausted { next_reset: DateTime<Local> },
    ClaudeSpecificError { code: String, message: String },
}

pub struct CooldownDetector {
    usage_limit_regex: Regex,
    time_parsing_regex: Regex,
    rate_limit_regex: Regex,
    cooldown_seconds_regex: Vec<Regex>,
}

impl CooldownDetector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // 基於 Claude-Autopilot 的高精度正則表達式
            usage_limit_regex: Regex::new(
                r"(?i)(Claude\s+)?usage\s+limit\s+reached.*?reset\s+at\s+(\d{1,2}[:\d]*(?:\s*[APM]{2})?(?:\s*\([^)]+\))?)",
            )?,
            time_parsing_regex: Regex::new(r"(\d{1,2})(?::(\d{2}))?\s*(am|pm)?")?,
            rate_limit_regex: Regex::new(r"(?i)rate\s+limit.*?(\d+)\s+(seconds?|minutes?|hours?)")?,
            cooldown_seconds_regex: vec![
                Regex::new(r"cooldown[:\s]+(\d+)s")?,
                Regex::new(r"wait\s+(\d+)\s+seconds?")?,
                Regex::new(r"retry\s+in\s+(\d+)\s+seconds?")?,
                Regex::new(r"(\d+)\s+seconds?\s+remaining")?,
                Regex::new(r"try\s+again\s+in\s+(\d+)\s+seconds?")?,
                Regex::new(r"cooldown.*?(\d+)")?, // 通用冷卻模式
            ],
        })
    }

    /// 主要冷卻檢測方法 - 整合多種檢測策略
    pub fn detect_cooldown(&self, claude_output: &str) -> Option<CooldownInfo> {
        // 1. 檢測使用限制錯誤（最常見）
        if let Some(cooldown) = self.detect_usage_limit(claude_output) {
            return Some(cooldown);
        }

        // 2. 檢測直接的秒數冷卻
        if let Some(cooldown) = self.detect_seconds_cooldown(claude_output) {
            return Some(cooldown);
        }

        // 3. 檢測速率限制錯誤
        if let Some(cooldown) = self.detect_rate_limit(claude_output) {
            return Some(cooldown);
        }

        // 4. 檢測 API 配額錯誤
        if let Some(cooldown) = self.detect_api_quota(claude_output) {
            return Some(cooldown);
        }

        None
    }

    /// 檢測使用限制錯誤 - 基於 Claude-Autopilot 邏輯增強
    fn detect_usage_limit(&self, output: &str) -> Option<CooldownInfo> {
        let matches: Vec<_> = self.usage_limit_regex.captures_iter(output).collect();

        if matches.is_empty() {
            return None;
        }

        // 取最後一個匹配（最新的錯誤訊息）
        let last_match = matches.last()?;
        let reset_time_str = last_match.get(2)?.as_str();
        let full_match = last_match.get(0)?.as_str();

        let reset_time = self.parse_reset_time(reset_time_str)?;
        let now = Local::now();

        // 檢查是否在 6 小時窗口內（Claude-Autopilot 的邏輯）
        let duration = reset_time.signed_duration_since(now);
        let hours_diff = duration.num_hours();

        if hours_diff > 6 || duration.num_seconds() <= 0 {
            return Some(CooldownInfo {
                is_cooling: false,
                seconds_remaining: 0,
                next_available_time: None,
                reset_time: Some(reset_time),
                original_message: full_match.to_string(),
                cooldown_pattern: Some(CooldownPattern::UsageLimitReached {
                    reset_time: reset_time_str.to_string(),
                }),
            });
        }

        let seconds_remaining = duration.num_seconds().max(0) as u64;

        Some(CooldownInfo {
            is_cooling: true,
            seconds_remaining,
            next_available_time: Some(reset_time.into()),
            reset_time: Some(reset_time),
            original_message: full_match.to_string(),
            cooldown_pattern: Some(CooldownPattern::UsageLimitReached {
                reset_time: reset_time_str.to_string(),
            }),
        })
    }

    /// 解析重置時間 - 基於 Claude-Autopilot 的複雜時間解析邏輯
    fn parse_reset_time(&self, reset_time_str: &str) -> Option<DateTime<Local>> {
        // 清理時區資訊和多餘空白
        let clean_time = reset_time_str
            .replace(char::is_whitespace, " ")
            .replace(|c: char| c == '(' || c == ')', "")
            .trim()
            .to_lowercase();

        let caps = self.time_parsing_regex.captures(&clean_time)?;

        let hours: u32 = caps.get(1)?.as_str().parse().ok()?;
        let minutes: u32 = caps
            .get(2)
            .map(|m| m.as_str().parse().unwrap_or(0))
            .unwrap_or(0);
        let ampm = caps.get(3).map(|m| m.as_str());

        // 驗證時間範圍
        if hours > 23 || minutes > 59 {
            return None;
        }

        let mut final_hours = hours;

        // AM/PM 處理 - 完全複製 Claude-Autopilot 邏輯
        if let Some(period) = ampm {
            match period {
                "pm" if hours != 12 => final_hours += 12,
                "am" if hours == 12 => final_hours = 0,
                _ => {}
            }
        }

        if final_hours >= 24 {
            return None;
        }

        let now = Local::now();
        let naive_time = NaiveTime::from_hms_opt(final_hours, minutes, 0)?;
        let today = now.date_naive();

        // 嘗試今天的時間
        let mut target = today.and_time(naive_time);
        let mut target_dt = Local.from_local_datetime(&target).single()?;

        // 如果時間已過，移到明天 - 使用 <= 以處理精確時間匹配
        if target_dt <= now {
            target = target + chrono::Duration::days(1);
            target_dt = Local.from_local_datetime(&target).single()?;
        }

        Some(target_dt)
    }

    /// 檢測直接秒數冷卻模式
    fn detect_seconds_cooldown(&self, output: &str) -> Option<CooldownInfo> {
        for regex in &self.cooldown_seconds_regex {
            if let Some(caps) = regex.captures(output) {
                if let Some(seconds_str) = caps.get(1) {
                    if let Ok(seconds) = seconds_str.as_str().parse::<u64>() {
                        let next_time = SystemTime::now() + std::time::Duration::from_secs(seconds);

                        return Some(CooldownInfo {
                            is_cooling: seconds > 0,
                            seconds_remaining: seconds,
                            next_available_time: Some(next_time),
                            reset_time: None,
                            original_message: caps.get(0)?.as_str().to_string(),
                            cooldown_pattern: Some(CooldownPattern::RateLimitExceeded { seconds }),
                        });
                    }
                }
            }
        }
        None
    }

    /// 檢測速率限制錯誤
    fn detect_rate_limit(&self, output: &str) -> Option<CooldownInfo> {
        if let Some(caps) = self.rate_limit_regex.captures(output) {
            let number: u64 = caps.get(1)?.as_str().parse().ok()?;
            let unit = caps.get(2)?.as_str().to_lowercase();

            let seconds = match unit.as_str() {
                s if s.starts_with("second") => number,
                s if s.starts_with("minute") => number * 60,
                s if s.starts_with("hour") => number * 3600,
                _ => return None,
            };

            let next_time = SystemTime::now() + std::time::Duration::from_secs(seconds);

            return Some(CooldownInfo {
                is_cooling: seconds > 0,
                seconds_remaining: seconds,
                next_available_time: Some(next_time),
                reset_time: None,
                original_message: caps.get(0)?.as_str().to_string(),
                cooldown_pattern: Some(CooldownPattern::RateLimitExceeded { seconds }),
            });
        }
        None
    }

    /// 檢測 API 配額錯誤
    fn detect_api_quota(&self, output: &str) -> Option<CooldownInfo> {
        // 檢測常見的 API 配額錯誤模式
        let quota_patterns = [
            r"(?i)api\s+quota\s+exceeded",
            r"(?i)monthly\s+limit\s+reached",
            r"(?i)billing\s+quota\s+exceeded",
            r"(?i)insufficient\s+credits",
        ];

        for pattern in &quota_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(m) = regex.find(output) {
                    // API 配額錯誤通常需要較長的等待時間
                    let reset_time = Local::now() + chrono::Duration::hours(1);
                    let seconds_remaining = 3600; // 1 小時

                    return Some(CooldownInfo {
                        is_cooling: true,
                        seconds_remaining,
                        next_available_time: Some(reset_time.into()),
                        reset_time: Some(reset_time),
                        original_message: m.as_str().to_string(),
                        cooldown_pattern: Some(CooldownPattern::ApiQuotaExhausted {
                            next_reset: reset_time,
                        }),
                    });
                }
            }
        }
        None
    }

    /// 從 claude doctor 命令檢查冷卻狀態
    pub async fn check_claude_doctor_cooldown(&self) -> Result<CooldownInfo> {
        let output = tokio::process::Command::new("claude")
            .arg("doctor")
            .arg("--json")
            .output()
            .await?;

        if !output.status.success() {
            return Ok(CooldownInfo {
                is_cooling: false,
                seconds_remaining: 0,
                next_available_time: None,
                reset_time: None,
                original_message: "claude doctor command failed".to_string(),
                cooldown_pattern: None,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 解析 JSON 診斷資訊
        if let Ok(diag) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(cooldown_secs) = diag.get("cooldown_seconds").and_then(|v| v.as_u64()) {
                let next_time = if cooldown_secs > 0 {
                    Some(SystemTime::now() + std::time::Duration::from_secs(cooldown_secs))
                } else {
                    None
                };

                return Ok(CooldownInfo {
                    is_cooling: cooldown_secs > 0,
                    seconds_remaining: cooldown_secs,
                    next_available_time: next_time,
                    reset_time: None,
                    original_message: stdout.to_string(),
                    cooldown_pattern: Some(CooldownPattern::ClaudeSpecificError {
                        code: "doctor_check".to_string(),
                        message: format!("Cooldown: {} seconds", cooldown_secs),
                    }),
                });
            }
        }

        // 如果無法解析，假設沒有冷卻
        Ok(CooldownInfo {
            is_cooling: false,
            seconds_remaining: 0,
            next_available_time: None,
            reset_time: None,
            original_message: "No cooldown detected".to_string(),
            cooldown_pattern: None,
        })
    }

    /// 智慧等待策略 - 基於冷卻時間決定等待方式
    pub async fn smart_wait(&self, cooldown_info: &CooldownInfo) {
        let seconds = cooldown_info.seconds_remaining;

        if seconds == 0 {
            return;
        }

        match seconds {
            // 短期冷卻：直接等待
            s if s <= 300 => {
                tracing::info!("短期冷卻 {} 秒，直接等待", s);
                tokio::time::sleep(std::time::Duration::from_secs(s)).await;
            }
            // 中期冷卻：分段等待並更新狀態
            s if s <= 1800 => {
                tracing::info!("中期冷卻 {} 秒，分段等待", s);
                self.segmented_wait(s).await;
            }
            // 長期冷卻：建議使用 ccusage 追蹤
            s => {
                tracing::warn!("長期冷卻 {} 秒，建議使用適應性排程", s);
                self.adaptive_wait(s).await;
            }
        }
    }

    /// 分段等待 - 中期冷卻時提供進度反饋
    async fn segmented_wait(&self, total_seconds: u64) {
        let segment_size = 60; // 每分鐘更新一次
        let mut remaining = total_seconds;

        while remaining > 0 {
            let wait_time = remaining.min(segment_size);
            tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
            remaining -= wait_time;

            if remaining > 0 {
                tracing::info!("冷卻進度: 剩餘 {} 秒", remaining);
            }
        }
    }

    /// 適應性等待 - 長期冷卻時的智慧策略
    async fn adaptive_wait(&self, total_seconds: u64) {
        // 對於長期冷卻，建議使用外部工具如 ccusage 進行更精確的追蹤
        tracing::info!("啟動適應性等待模式，總時長: {} 秒", total_seconds);

        // 初始等待 10 分鐘，然後檢查狀態
        let initial_wait = 600; // 10 分鐘
        tokio::time::sleep(std::time::Duration::from_secs(initial_wait)).await;

        let mut remaining = total_seconds.saturating_sub(initial_wait);

        // 之後每 5 分鐘檢查一次
        while remaining > 0 {
            // 檢查是否可以透過 claude doctor 獲得更準確的狀態
            if let Ok(current_status) = self.check_claude_doctor_cooldown().await {
                if !current_status.is_cooling {
                    tracing::info!("適應性等待檢測到冷卻已結束");
                    return;
                }

                // 更新剩餘時間
                if current_status.seconds_remaining < remaining {
                    remaining = current_status.seconds_remaining;
                    tracing::info!("適應性等待更新剩餘時間: {} 秒", remaining);
                }
            }

            let next_check = remaining.min(300); // 最多等待 5 分鐘
            tokio::time::sleep(std::time::Duration::from_secs(next_check)).await;
            remaining = remaining.saturating_sub(next_check);
        }
    }

    /// 檢查當前是否處於冷卻狀態
    pub async fn is_currently_cooling(&self) -> bool {
        match self.check_claude_doctor_cooldown().await {
            Ok(info) => info.is_cooling,
            Err(_) => false,
        }
    }

    /// 獲取格式化的冷卻狀態描述
    pub fn format_cooldown_status(&self, cooldown_info: &CooldownInfo) -> String {
        if !cooldown_info.is_cooling {
            return "✅ Claude CLI 可用".to_string();
        }

        let hours = cooldown_info.seconds_remaining / 3600;
        let minutes = (cooldown_info.seconds_remaining % 3600) / 60;
        let seconds = cooldown_info.seconds_remaining % 60;

        let time_str = if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        };

        match &cooldown_info.cooldown_pattern {
            Some(CooldownPattern::UsageLimitReached { reset_time }) => {
                format!(
                    "⏳ 使用限制冷卻中，重置時間: {} (剩餘: {})",
                    reset_time, time_str
                )
            }
            Some(CooldownPattern::RateLimitExceeded { .. }) => {
                format!("⏳ 速率限制冷卻中，剩餘: {}", time_str)
            }
            Some(CooldownPattern::ApiQuotaExhausted { .. }) => {
                format!("⏳ API 配額用盡，剩餘: {}", time_str)
            }
            _ => format!("⏳ Claude CLI 冷卻中，剩餘: {}", time_str),
        }
    }
}

impl Default for CooldownDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create CooldownDetector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_limit_detection() {
        let detector = CooldownDetector::new().unwrap();

        let test_cases = [
            "Claude usage limit reached. Your limit will reset at 4:30 PM (EST)",
            "usage limit reached, reset at 11pm",
            "Claude usage limit reached. Your limit will reset at 9 am",
        ];

        for case in &test_cases {
            let result = detector.detect_cooldown(case);
            assert!(result.is_some(), "Failed to detect cooldown in: {}", case);

            let cooldown = result.unwrap();
            assert!(matches!(
                cooldown.cooldown_pattern,
                Some(CooldownPattern::UsageLimitReached { .. })
            ));
        }
    }

    #[test]
    fn test_seconds_cooldown_detection() {
        let detector = CooldownDetector::new().unwrap();

        let test_cases = [
            ("Error: cooldown: 123s", Some(123)),
            ("Please wait 45 seconds before retrying", Some(45)),
            ("Rate limited. retry in 60 seconds", Some(60)),
            ("30 seconds remaining", Some(30)),
            ("No cooldown message", None),
        ];

        for (input, expected_seconds) in &test_cases {
            let result = detector.detect_cooldown(input);

            match expected_seconds {
                Some(expected) => {
                    assert!(result.is_some(), "Failed to detect cooldown in: {}", input);
                    let cooldown = result.unwrap();
                    assert_eq!(cooldown.seconds_remaining, *expected as u64);
                }
                None => {
                    assert!(result.is_none(), "False positive for: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_time_parsing() {
        let detector = CooldownDetector::new().unwrap();

        let test_cases = [
            ("4:30 PM", true),
            ("11pm", true),
            ("9 am", true),
            ("14:30", true),
            ("25:00", false), // Invalid hour
            ("12:70", false), // Invalid minute
        ];

        for (time_str, should_parse) in &test_cases {
            let result = detector.parse_reset_time(time_str);

            if *should_parse {
                assert!(result.is_some(), "Failed to parse time: {}", time_str);
            } else {
                assert!(
                    result.is_none(),
                    "Should not parse invalid time: {}",
                    time_str
                );
            }
        }
    }

    #[test]
    fn test_rate_limit_detection() {
        let detector = CooldownDetector::new().unwrap();

        let test_cases = [
            "Rate limit exceeded. Try again in 30 seconds",
            "API rate limit: wait 5 minutes",
            "Rate limited for 2 hours",
        ];

        for case in &test_cases {
            let result = detector.detect_cooldown(case);
            assert!(result.is_some(), "Failed to detect rate limit in: {}", case);

            let cooldown = result.unwrap();
            assert!(matches!(
                cooldown.cooldown_pattern,
                Some(CooldownPattern::RateLimitExceeded { .. })
            ));
        }
    }

    #[test]
    fn test_cooldown_formatting() {
        let detector = CooldownDetector::new().unwrap();

        let cooldown_info = CooldownInfo {
            is_cooling: true,
            seconds_remaining: 3661, // 1h 1m 1s
            next_available_time: None,
            reset_time: None,
            original_message: "test".to_string(),
            cooldown_pattern: Some(CooldownPattern::UsageLimitReached {
                reset_time: "4:30 PM".to_string(),
            }),
        };

        let formatted = detector.format_cooldown_status(&cooldown_info);
        assert!(formatted.contains("1h 1m 1s"));
        assert!(formatted.contains("4:30 PM"));
    }
}
