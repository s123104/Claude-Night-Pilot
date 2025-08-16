// 重試策略系統 - 智慧指數退避與錯誤恢復
use crate::core::cooldown::{CooldownDetector, CooldownInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
    pub cooldown_aware: bool,
    pub strategy: RetryStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    ExponentialBackoff,
    LinearBackoff,
    FixedDelay,
    AdaptiveCooldown,
    SmartRetry, // 綜合策略
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    pub attempt_number: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub error_type: ErrorType,
    pub delay_used: Duration,
    pub cooldown_detected: Option<CooldownInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    CooldownError,
    RateLimitError,
    NetworkError,
    AuthenticationError,
    TimeoutError,
    UnknownError,
    SystemError,
}

#[derive(Debug)]
pub struct RetryOrchestrator {
    config: RetryConfig,
    cooldown_detector: CooldownDetector,
    attempt_history: Vec<RetryAttempt>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: true,
            cooldown_aware: true,
            strategy: RetryStrategy::SmartRetry,
        }
    }
}

impl RetryOrchestrator {
    pub fn new(config: RetryConfig) -> Result<Self> {
        Ok(Self {
            config,
            cooldown_detector: CooldownDetector::new()?,
            attempt_history: Vec::new(),
        })
    }

    pub fn with_smart_defaults() -> Result<Self> {
        let config = RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(300), // 5 分鐘最大延遲
            multiplier: 2.0,
            jitter: true,
            cooldown_aware: true,
            strategy: RetryStrategy::SmartRetry,
        };

        Self::new(config)
    }

    /// 執行帶重試的操作
    pub async fn execute_with_retry<F, T, E>(&mut self, operation: F) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send,
        E: std::error::Error + Send + Sync + 'static,
        T: Send,
    {
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 1..=self.config.max_attempts {
            let start_time = chrono::Utc::now();

            match operation().await {
                Ok(result) => {
                    // 成功，清理重試歷史
                    self.attempt_history.clear();
                    return Ok(result);
                }
                Err(error) => {
                    let error_anyhow = anyhow::anyhow!("{}", error);
                    let error_type = self.classify_error(&error_anyhow);

                    // 檢測冷卻
                    let cooldown_info = if self.config.cooldown_aware {
                        self.cooldown_detector
                            .detect_cooldown(&error_anyhow.to_string())
                    } else {
                        None
                    };

                    // 記錄重試嘗試
                    let retry_attempt = RetryAttempt {
                        attempt_number: attempt,
                        timestamp: start_time,
                        error_type: error_type.clone(),
                        delay_used: Duration::from_secs(0), // 將在下面更新
                        cooldown_detected: cooldown_info.clone(),
                    };

                    last_error = Some(error_anyhow);

                    // 如果是最後一次嘗試，不再重試
                    if attempt >= self.config.max_attempts {
                        self.attempt_history.push(retry_attempt);
                        break;
                    }

                    // 計算延遲
                    let delay = self
                        .calculate_retry_delay(attempt, &error_type, &cooldown_info)
                        .await;

                    // 更新記錄
                    let mut retry_attempt = retry_attempt;
                    retry_attempt.delay_used = delay;
                    self.attempt_history.push(retry_attempt);

                    tracing::info!(
                        "重試 {}/{}: {} - 將等待 {:?}",
                        attempt,
                        self.config.max_attempts,
                        error_type.description(),
                        delay
                    );

                    // 智慧等待
                    self.smart_wait(delay, &cooldown_info).await;
                }
            }
        }

        // 所有重試都失敗
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("所有重試都失敗")))
    }

    /// 計算重試延遲 - 基於策略和錯誤類型
    async fn calculate_retry_delay(
        &self,
        attempt: u32,
        error_type: &ErrorType,
        cooldown_info: &Option<CooldownInfo>,
    ) -> Duration {
        // 如果檢測到冷卻，優先使用冷卻時間
        if let Some(cooldown) = cooldown_info {
            if cooldown.is_cooling {
                return Duration::from_secs(cooldown.seconds_remaining);
            }
        }

        match self.config.strategy {
            RetryStrategy::ExponentialBackoff => self.exponential_backoff_delay(attempt),
            RetryStrategy::LinearBackoff => self.linear_backoff_delay(attempt),
            RetryStrategy::FixedDelay => self.config.base_delay,
            RetryStrategy::AdaptiveCooldown => {
                self.adaptive_cooldown_delay(attempt, cooldown_info).await
            }
            RetryStrategy::SmartRetry => {
                self.smart_retry_delay(attempt, error_type, cooldown_info)
                    .await
            }
        }
    }

    /// 指數退避延遲計算
    fn exponential_backoff_delay(&self, attempt: u32) -> Duration {
        let base_ms = self.config.base_delay.as_millis() as f64;
        let delay_ms = base_ms * self.config.multiplier.powi(attempt as i32 - 1);

        let mut final_delay = Duration::from_millis(delay_ms as u64);

        // 限制最大延遲
        if final_delay > self.config.max_delay {
            final_delay = self.config.max_delay;
        }

        // 添加抖動以避免雷群效應
        if self.config.jitter {
            final_delay = self.add_jitter(final_delay);
        }

        final_delay
    }

    /// 線性退避延遲計算
    fn linear_backoff_delay(&self, attempt: u32) -> Duration {
        let base_ms = self.config.base_delay.as_millis() as u64;
        let delay_ms = base_ms * attempt as u64;

        let mut final_delay = Duration::from_millis(delay_ms);

        if final_delay > self.config.max_delay {
            final_delay = self.config.max_delay;
        }

        if self.config.jitter {
            final_delay = self.add_jitter(final_delay);
        }

        final_delay
    }

    /// 適應性冷卻延遲 - 基於 ccusage 整合
    async fn adaptive_cooldown_delay(
        &self,
        _attempt: u32,
        _cooldown_info: &Option<CooldownInfo>,
    ) -> Duration {
        // 嘗試從 ccusage 獲取更精確的時間
        if let Ok(remaining_minutes) = self.get_ccusage_remaining_time().await {
            if remaining_minutes > 0 {
                // 根據剩餘時間調整策略
                return match remaining_minutes {
                    m if m > 30 => Duration::from_secs(600), // >30分鐘: 10分鐘後重試
                    m if m > 5 => Duration::from_secs(120),  // 5-30分鐘: 2分鐘後重試
                    _ => Duration::from_secs(30),            // <5分鐘: 30秒後重試
                };
            }
        }

        // 回退到指數退避
        self.exponential_backoff_delay(_attempt)
    }

    /// 智慧重試延遲 - 綜合策略
    async fn smart_retry_delay(
        &self,
        attempt: u32,
        error_type: &ErrorType,
        cooldown_info: &Option<CooldownInfo>,
    ) -> Duration {
        match error_type {
            ErrorType::CooldownError => {
                // 冷卻錯誤：使用適應性策略
                self.adaptive_cooldown_delay(attempt, cooldown_info).await
            }
            ErrorType::RateLimitError => {
                // 速率限制：較長的指數退避
                let base_delay = Duration::from_secs(30); // 30秒基礎延遲
                let multiplied = base_delay.as_secs() * (2_u64.pow(attempt - 1));
                Duration::from_secs(multiplied.min(300)) // 最大5分鐘
            }
            ErrorType::NetworkError => {
                // 網路錯誤：短期線性退避
                Duration::from_secs((attempt * 5).min(60) as u64) // 5s, 10s, 15s... 最大60s
            }
            ErrorType::TimeoutError => {
                // 超時錯誤：逐步增加
                Duration::from_secs((attempt * 10).min(120) as u64) // 10s, 20s, 30s... 最大120s
            }
            ErrorType::AuthenticationError => {
                // 認證錯誤：快速重試（可能是臨時問題）
                Duration::from_secs(5)
            }
            ErrorType::SystemError => {
                // 系統錯誤：標準指數退避
                self.exponential_backoff_delay(attempt)
            }
            ErrorType::UnknownError => {
                // 未知錯誤：保守的指數退避
                let delay = self.exponential_backoff_delay(attempt);
                if delay < Duration::from_secs(30) {
                    Duration::from_secs(30) // 最少30秒
                } else {
                    delay
                }
            }
        }
    }

    /// 添加抖動以避免雷群效應
    fn add_jitter(&self, delay: Duration) -> Duration {
        use rand::Rng;

        let base_ms = delay.as_millis() as f64;
        let jitter_range = base_ms * 0.1; // ±10% 抖動
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_range..=jitter_range);

        let final_ms = (base_ms + jitter).max(100.0) as u64; // 最少100ms
        Duration::from_millis(final_ms)
    }

    /// 智慧等待 - 結合冷卻檢測
    async fn smart_wait(&self, delay: Duration, cooldown_info: &Option<CooldownInfo>) {
        if let Some(cooldown) = cooldown_info {
            if cooldown.is_cooling {
                // 使用冷卻檢測器的智慧等待
                self.cooldown_detector.smart_wait(cooldown).await;
                return;
            }
        }

        // 普通等待
        tokio::time::sleep(delay).await;
    }

    /// 錯誤分類 - 基於錯誤訊息判斷錯誤類型
    fn classify_error(&self, error: &anyhow::Error) -> ErrorType {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("cooldown") || error_str.contains("usage limit") {
            ErrorType::CooldownError
        } else if error_str.contains("rate limit") || error_str.contains("429") {
            ErrorType::RateLimitError
        } else if error_str.contains("network") || error_str.contains("connection") {
            ErrorType::NetworkError
        } else if error_str.contains("auth")
            || error_str.contains("401")
            || error_str.contains("403")
        {
            ErrorType::AuthenticationError
        } else if error_str.contains("timeout") || error_str.contains("timed out") {
            ErrorType::TimeoutError
        } else if error_str.contains("system") || error_str.contains("internal") {
            ErrorType::SystemError
        } else {
            ErrorType::UnknownError
        }
    }

    /// 從 ccusage 獲取剩餘時間（分鐘）
    async fn get_ccusage_remaining_time(&self) -> Result<u32> {
        // 嘗試多種 ccusage 命令
        let commands = ["ccusage", "claude-usage", "ccu"];

        for cmd in &commands {
            match tokio::process::Command::new(cmd)
                .arg("blocks")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(minutes) = self.parse_ccusage_output(&stdout) {
                        return Ok(minutes);
                    }
                }
                _ => continue,
            }
        }

        Err(anyhow::anyhow!("無法執行 ccusage 命令"))
    }

    /// 解析 ccusage 輸出
    fn parse_ccusage_output(&self, output: &str) -> Option<u32> {
        use regex::Regex;

        // 解析 "3h 45m" 格式
        if let Ok(re) = Regex::new(r"(\d+)h\s*(\d+)m") {
            if let Some(caps) = re.captures(output) {
                let hours: u32 = caps[1].parse().ok()?;
                let minutes: u32 = caps[2].parse().ok()?;
                return Some(hours * 60 + minutes);
            }
        }

        // 解析只有分鐘的格式 "45m"
        if let Ok(re) = Regex::new(r"(\d+)m") {
            if let Some(caps) = re.captures(output) {
                let minutes: u32 = caps[1].parse().ok()?;
                return Some(minutes);
            }
        }

        None
    }

    /// 獲取重試統計信息
    pub fn get_retry_stats(&self) -> RetryStats {
        let total_attempts = self.attempt_history.len() as u32;
        let mut error_counts = std::collections::HashMap::new();
        let mut total_delay = Duration::from_secs(0);

        for attempt in &self.attempt_history {
            *error_counts.entry(attempt.error_type.clone()).or_insert(0) += 1;
            total_delay += attempt.delay_used;
        }

        RetryStats {
            total_attempts,
            error_counts,
            total_delay,
            success_rate: if total_attempts > 0 {
                0.0 // 如果還在重試階段，成功率為0
            } else {
                1.0 // 沒有重試記錄意味著首次成功
            },
        }
    }

    /// 重置重試狀態
    pub fn reset(&mut self) {
        self.attempt_history.clear();
    }

    /// 檢查是否應該繼續重試
    pub fn should_continue_retry(&self, attempt: u32, error_type: &ErrorType) -> bool {
        if attempt >= self.config.max_attempts {
            return false;
        }

        // 某些錯誤類型不應該重試
        match error_type {
            ErrorType::AuthenticationError => {
                // 認證錯誤可能是臨時的，允許少量重試
                attempt <= 2
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub error_counts: std::collections::HashMap<ErrorType, u32>,
    pub total_delay: Duration,
    pub success_rate: f64,
}

impl ErrorType {
    pub fn description(&self) -> &'static str {
        match self {
            ErrorType::CooldownError => "冷卻錯誤",
            ErrorType::RateLimitError => "速率限制錯誤",
            ErrorType::NetworkError => "網路錯誤",
            ErrorType::AuthenticationError => "認證錯誤",
            ErrorType::TimeoutError => "超時錯誤",
            ErrorType::SystemError => "系統錯誤",
            ErrorType::UnknownError => "未知錯誤",
        }
    }

    pub fn is_retriable(&self) -> bool {
        match self {
            ErrorType::CooldownError => true,
            ErrorType::RateLimitError => true,
            ErrorType::NetworkError => true,
            ErrorType::TimeoutError => true,
            ErrorType::SystemError => true,
            ErrorType::AuthenticationError => true, // 可能是臨時問題
            ErrorType::UnknownError => true,
        }
    }
}

// 用於序列化的 Hash 實現
impl std::hash::Hash for ErrorType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for ErrorType {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for ErrorType {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        let orchestrator = RetryOrchestrator::new(config).unwrap();

        assert_eq!(
            orchestrator.exponential_backoff_delay(1),
            Duration::from_millis(1000)
        );
        assert_eq!(
            orchestrator.exponential_backoff_delay(2),
            Duration::from_millis(2000)
        );
        assert_eq!(
            orchestrator.exponential_backoff_delay(3),
            Duration::from_millis(4000)
        );

        // 測試最大延遲限制
        let long_delay = orchestrator.exponential_backoff_delay(10);
        assert!(long_delay <= Duration::from_secs(60));
    }

    #[test]
    fn test_error_classification() {
        let orchestrator = RetryOrchestrator::with_smart_defaults().unwrap();

        let test_cases = [
            ("Claude usage limit reached", ErrorType::CooldownError),
            ("Rate limit exceeded", ErrorType::RateLimitError),
            ("Network connection failed", ErrorType::NetworkError),
            ("Authentication failed", ErrorType::AuthenticationError),
            ("Request timed out", ErrorType::TimeoutError),
            ("Internal system error", ErrorType::SystemError),
            ("Something went wrong", ErrorType::UnknownError),
        ];

        for (error_msg, expected_type) in &test_cases {
            let error = anyhow::anyhow!("{}", error_msg);
            let classified = orchestrator.classify_error(&error);
            assert_eq!(classified, *expected_type, "Failed for: {}", error_msg);
        }
    }

    #[test]
    fn test_ccusage_output_parsing() {
        let orchestrator = RetryOrchestrator::with_smart_defaults().unwrap();

        let test_cases = [
            ("Time remaining: 3h 45m", Some(225)), // 3*60 + 45 = 225
            ("Remaining: 1h 30m", Some(90)),       // 1*60 + 30 = 90
            ("45m remaining", Some(45)),
            ("No time info", None),
        ];

        for (output, expected) in &test_cases {
            let result = orchestrator.parse_ccusage_output(output);
            assert_eq!(result, *expected, "Failed for: {}", output);
        }
    }

    #[tokio::test]
    async fn test_retry_orchestrator_success() {
        let mut orchestrator = RetryOrchestrator::with_smart_defaults().unwrap();

        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = orchestrator
            .execute_with_retry(move || {
                let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                Box::pin(async move {
                    if count < 3 {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Temporary failure",
                        ))
                    } else {
                        Ok("Success!")
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success!");
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();

        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(1000));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.multiplier, 2.0);
        assert!(config.jitter);
        assert!(config.cooldown_aware);
        assert!(matches!(config.strategy, RetryStrategy::SmartRetry));
    }
}
