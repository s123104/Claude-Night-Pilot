/**
 * Claude 冷卻狀態檢測器
 * 基於對 research-projects 的分析和網路研究實現
 * 支援多種檢測方法：API 回應解析、時間窗口管理、使用量追蹤
 */

use chrono::{DateTime, Local, Duration, Timelike};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context};

/// Claude API 限制類型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitType {
    RequestsPerMinute,
    TokensPerMinute,
    DailyQuota,
    HourlyQuota,
}

/// 冷卻狀態資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownStatus {
    pub is_cooling: bool,
    pub limit_type: Option<LimitType>,
    pub remaining_time_seconds: u64,
    pub reset_time: Option<DateTime<Local>>,
    pub usage_info: UsageInfo,
    pub last_updated: DateTime<Local>,
}

/// 使用量資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub tokens_used_this_minute: u64,
    pub tokens_used_today: u64,
    pub requests_this_minute: u32,
    pub requests_today: u32,
    pub current_5hour_block_usage: u64,
    pub estimated_cost_usd: f64,
}

/// Claude 冷卻檢測器
pub struct ClaudeCooldownDetector {
    /// 錯誤模式匹配器
    error_patterns: Vec<Regex>,
    /// ccusage 命令路徑
    ccusage_cmd: Option<String>,
    /// 使用量追蹤
    usage_tracker: UsageTracker,
}

/// 使用量追蹤器
#[derive(Debug)]
struct UsageTracker {
    minute_window: HashMap<DateTime<Local>, u32>,
    daily_tokens: u64,
    daily_requests: u32,
    last_reset: DateTime<Local>,
}

impl ClaudeCooldownDetector {
    /// 創建新的冷卻檢測器
    pub fn new() -> Self {
        let error_patterns = Self::build_error_patterns();
        let ccusage_cmd = Self::detect_ccusage_command();
        
        Self {
            error_patterns,
            ccusage_cmd,
            usage_tracker: UsageTracker::new(),
        }
    }

    /// 建立錯誤模式匹配器
    fn build_error_patterns() -> Vec<Regex> {
        let patterns = [
            // 基本 429 錯誤
            r"429\s*Too\s*Many\s*Requests",
            r"Rate\s*limit\s*exceeded",
            r"Number\s*of\s*request\s*tokens\s*has\s*exceeded",
            r"Number\s*of\s*requests\s*has\s*exceeded",
            
            // Claude 特定錯誤
            r"Your\s*API\s*usage\s*has\s*exceeded",
            r"You\s*have\s*reached\s*your\s*(?:daily|monthly|hourly)\s*limit",
            r"Please\s*wait\s*(\d+)\s*seconds?\s*before\s*retrying",
            r"Retry-After:\s*(\d+)",
            
            // 冷卻時間指示
            r"Try\s*again\s*in\s*(\d+)\s*minutes?",
            r"Available\s*again\s*at\s*(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})",
            r"Reset\s*time:\s*(\d+)",
            
            // 使用量相關
            r"Token\s*quota\s*exceeded",
            r"Daily\s*limit\s*reached",
            r"Usage\s*window\s*expired",
            
            // ccusage 相關
            r"Time\s*remaining:\s*(\d+)h\s*(\d+)m",
            r"Block\s*resets?\s*in\s*(\d+)\s*minutes?",
        ];

        patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect()
    }

    /// 檢測 ccusage 命令
    fn detect_ccusage_command() -> Option<String> {
        // 嘗試不同的 ccusage 命令
        let commands = ["ccusage", "bunx ccusage", "npx ccusage@latest"];
        
        for cmd in &commands {
            if let Ok(output) = Command::new("sh")
                .arg("-c")
                .arg(&format!("command -v {}", cmd.split_whitespace().next()?))
                .output()
            {
                if output.status.success() {
                    return Some(cmd.to_string());
                }
            }
        }
        
        None
    }

    /// 檢查冷卻狀態 - 主要方法
    pub async fn check_cooldown(&mut self) -> Result<CooldownStatus> {
        let mut status = CooldownStatus {
            is_cooling: false,
            limit_type: None,
            remaining_time_seconds: 0,
            reset_time: None,
            usage_info: self.usage_tracker.get_current_usage(),
            last_updated: Local::now(),
        };

        // 方法1: 使用 ccusage 檢查（最準確）
        if let Some(ccusage_status) = self.check_with_ccusage().await? {
            status = ccusage_status;
        }
        
        // 方法2: 檢查 Claude 會話文件
        if !status.is_cooling {
            if let Some(session_status) = self.check_claude_session_files()? {
                status = session_status;
            }
        }

        // 方法3: 基於時間的推測檢查
        if !status.is_cooling {
            status = self.check_time_based_cooldown()?;
        }

        Ok(status)
    }

    /// 使用 ccusage 檢查冷卻狀態
    async fn check_with_ccusage(&self) -> Result<Option<CooldownStatus>> {
        let ccusage_cmd = match &self.ccusage_cmd {
            Some(cmd) => cmd,
            None => return Ok(None),
        };

        // 執行 ccusage blocks 命令
        let output = Command::new("sh")
            .arg("-c")
            .arg(&format!("{} blocks", ccusage_cmd))
            .output()
            .context("執行 ccusage 命令失敗")?;

        if !output.status.success() {
            return Ok(None);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // 解析時間剩餘
        if let Some(remaining_minutes) = self.parse_time_remaining(&output_str) {
            let is_cooling = remaining_minutes <= 5; // 5分鐘內視為即將冷卻
            
            return Ok(Some(CooldownStatus {
                is_cooling,
                limit_type: Some(LimitType::HourlyQuota),
                remaining_time_seconds: (remaining_minutes * 60) as u64,
                reset_time: Some(Local::now() + Duration::minutes(remaining_minutes)),
                usage_info: self.usage_tracker.get_current_usage(),
                last_updated: Local::now(),
            }));
        }

        Ok(None)
    }

    /// 解析時間剩餘
    fn parse_time_remaining(&self, output: &str) -> Option<i64> {
        // 嘗試匹配 "Time remaining: 2h 30m" 格式
        let time_regex = Regex::new(r"(?i)time\s+remaining:?\s*(\d+)h\s*(\d+)m").ok()?;
        if let Some(captures) = time_regex.captures(output) {
            let hours: i64 = captures[1].parse().ok()?;
            let minutes: i64 = captures[2].parse().ok()?;
            return Some(hours * 60 + minutes);
        }

        // 嘗試匹配 "30m" 格式
        let min_regex = Regex::new(r"(?i)(\d+)m\s*remaining").ok()?;
        if let Some(captures) = min_regex.captures(output) {
            return captures[1].parse().ok();
        }

        None
    }

    /// 檢查 Claude 會話文件
    fn check_claude_session_files(&self) -> Result<Option<CooldownStatus>> {
        // Claude 會話文件通常在 ~/.claude/projects/ 目錄
        let home_dir = dirs::home_dir().context("無法獲取 home 目錄")?;
        let claude_dir = home_dir.join(".claude").join("projects");

        if !claude_dir.exists() {
            return Ok(None);
        }

        // 尋找最新的會話文件
        let mut latest_file: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
        
        for entry in fs::read_dir(&claude_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if latest_file.as_ref().map_or(true, |(_, time)| modified > *time) {
                            latest_file = Some((path, modified));
                        }
                    }
                }
            }
        }

        if let Some((file_path, _)) = latest_file {
            return self.analyze_session_file(&file_path);
        }

        Ok(None)
    }

    /// 分析會話文件
    fn analyze_session_file(&self, file_path: &Path) -> Result<Option<CooldownStatus>> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        // 分析最後幾行，查找錯誤訊息
        for line in lines.iter().rev().take(20) {
            for pattern in &self.error_patterns {
                if pattern.is_match(line) {
                    // 找到冷卻相關錯誤
                    let remaining_time = self.extract_cooldown_time(line);
                    
                    return Ok(Some(CooldownStatus {
                        is_cooling: true,
                        limit_type: Some(self.detect_limit_type(line)),
                        remaining_time_seconds: remaining_time,
                        reset_time: Some(Local::now() + Duration::seconds(remaining_time as i64)),
                        usage_info: self.usage_tracker.get_current_usage(),
                        last_updated: Local::now(),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// 提取冷卻時間
    fn extract_cooldown_time(&self, error_message: &str) -> u64 {
        // 嘗試提取具體的等待時間
        let wait_patterns = [
            r"wait\s*(\d+)\s*seconds?",
            r"retry\s*in\s*(\d+)\s*seconds?",
            r"retry-after:\s*(\d+)",
            r"(\d+)\s*minutes?",
        ];

        for pattern_str in &wait_patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                if let Some(captures) = pattern.captures(error_message) {
                    if let Ok(seconds) = captures[1].parse::<u64>() {
                        return if pattern_str.contains("minutes") {
                            seconds * 60
                        } else {
                            seconds
                        };
                    }
                }
            }
        }

        // 預設冷卻時間（基於 Claude 的典型限制）
        300 // 5 分鐘
    }

    /// 檢測限制類型
    fn detect_limit_type(&self, error_message: &str) -> LimitType {
        let message_lower = error_message.to_lowercase();
        
        if message_lower.contains("daily") {
            LimitType::DailyQuota
        } else if message_lower.contains("minute") {
            if message_lower.contains("token") {
                LimitType::TokensPerMinute
            } else {
                LimitType::RequestsPerMinute
            }
        } else if message_lower.contains("hour") {
            LimitType::HourlyQuota
        } else {
            LimitType::RequestsPerMinute
        }
    }

    /// 基於時間的冷卻檢查
    fn check_time_based_cooldown(&self) -> Result<CooldownStatus> {
        let usage_info = self.usage_tracker.get_current_usage();
        let now = Local::now();
        
        // 檢查是否接近 5 小時邊界（Claude 的計費窗口）
        let hours_since_midnight = now.time().hour();
        let minutes_since_midnight = now.time().minute();
        let total_minutes = (hours_since_midnight * 60 + minutes_since_midnight) as i64;
        
        // Claude 的 5 小時區塊: 0-5, 5-10, 10-15, 15-20, 20-24
        let current_block_start = (total_minutes / 300) * 300; // 300 分鐘 = 5 小時
        let next_block_start = current_block_start + 300;
        let minutes_to_reset = next_block_start - total_minutes;
        
        let is_cooling = minutes_to_reset <= 5 && usage_info.current_5hour_block_usage > 1000000; // 假設超過 100 萬 token 為高使用量
        
        Ok(CooldownStatus {
            is_cooling,
            limit_type: Some(LimitType::HourlyQuota),
            remaining_time_seconds: (minutes_to_reset * 60) as u64,
            reset_time: Some(now + Duration::minutes(minutes_to_reset)),
            usage_info,
            last_updated: now,
        })
    }

    /// 解析 Claude CLI 錯誤輸出
    pub fn parse_claude_error(&self, error_output: &str) -> Option<CooldownStatus> {
        for pattern in &self.error_patterns {
            if pattern.is_match(error_output) {
                let remaining_time = self.extract_cooldown_time(error_output);
                let limit_type = self.detect_limit_type(error_output);
                
                return Some(CooldownStatus {
                    is_cooling: true,
                    limit_type: Some(limit_type),
                    remaining_time_seconds: remaining_time,
                    reset_time: Some(Local::now() + Duration::seconds(remaining_time as i64)),
                    usage_info: self.usage_tracker.get_current_usage(),
                    last_updated: Local::now(),
                });
            }
        }
        
        None
    }

    /// 更新使用量統計
    pub fn update_usage(&mut self, tokens_used: u64, request_count: u32) {
        self.usage_tracker.update_usage(tokens_used, request_count);
    }
}

impl UsageTracker {
    fn new() -> Self {
        Self {
            minute_window: HashMap::new(),
            daily_tokens: 0,
            daily_requests: 0,
            last_reset: Local::now(),
        }
    }

    fn update_usage(&mut self, tokens_used: u64, request_count: u32) {
        let now = Local::now();
        
        // 重置每日統計（如果需要）
        if now.date_naive() != self.last_reset.date_naive() {
            self.daily_tokens = 0;
            self.daily_requests = 0;
            self.last_reset = now;
        }

        // 更新統計
        self.daily_tokens += tokens_used;
        self.daily_requests += request_count;
        
        // 更新分鐘窗口
        let minute_key = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        *self.minute_window.entry(minute_key).or_insert(0) += request_count;
        
        // 清理舊的分鐘數據（保留最近 5 分鐘）
        let cutoff = now - Duration::minutes(5);
        self.minute_window.retain(|&time, _| time > cutoff);
    }

    fn get_current_usage(&self) -> UsageInfo {
        let now = Local::now();
        let current_minute = now.with_second(0).unwrap().with_nanosecond(0).unwrap();
        
        // 計算當前分鐘的請求數
        let requests_this_minute = self.minute_window.get(&current_minute).copied().unwrap_or(0);
        
        // 估算當前 5 小時區塊的使用量（簡化計算）
        let _hours_since_block_start = now.time().hour() % 5;
        let estimated_block_usage = self.daily_tokens / 5; // 簡化估算
        
        // 估算成本 (基於 Claude 3.5 Sonnet 的價格: $3/M input tokens, $15/M output tokens)
        let estimated_cost = (self.daily_tokens as f64 / 1_000_000.0) * 9.0; // 假設平均價格

        UsageInfo {
            tokens_used_this_minute: 0, // 需要更詳細的追蹤
            tokens_used_today: self.daily_tokens,
            requests_this_minute,
            requests_today: self.daily_requests,
            current_5hour_block_usage: estimated_block_usage,
            estimated_cost_usd: estimated_cost,
        }
    }
}

impl Default for ClaudeCooldownDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_pattern_matching() {
        let detector = ClaudeCooldownDetector::new();
        
        let error_messages = [
            "429 Too Many Requests - Rate limit exceeded",
            "Number of request tokens has exceeded your per-minute rate limit",
            "Please wait 60 seconds before retrying",
            "Your API usage has exceeded the daily limit",
        ];

        for message in &error_messages {
            let has_match = detector.error_patterns.iter().any(|pattern| pattern.is_match(message));
            assert!(has_match, "應該匹配錯誤訊息: {}", message);
        }
    }

    #[test]
    fn test_time_extraction() {
        let detector = ClaudeCooldownDetector::new();
        
        assert_eq!(detector.extract_cooldown_time("Please wait 120 seconds"), 120);
        assert_eq!(detector.extract_cooldown_time("Retry in 5 minutes"), 300);
        assert_eq!(detector.extract_cooldown_time("Retry-After: 60"), 60);
    }

    #[test]
    fn test_limit_type_detection() {
        let detector = ClaudeCooldownDetector::new();
        
        assert!(matches!(
            detector.detect_limit_type("daily limit exceeded"),
            LimitType::DailyQuota
        ));
        
        assert!(matches!(
            detector.detect_limit_type("tokens per minute exceeded"),
            LimitType::TokensPerMinute
        ));
    }
}