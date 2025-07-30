// 重試策略系統全面測試用例
use super::super::retry::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::{sleep, Instant};

#[tokio::test]
async fn test_retry_manager_creation() -> Result<()> {
    let manager = RetryManager::new()?;
    
    // 驗證預設配置
    assert_eq!(manager.stats.total_attempts, 0);
    assert_eq!(manager.stats.success_count, 0);
    assert_eq!(manager.stats.failure_count, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_exponential_backoff_calculation() -> Result<()> {
    let strategy = RetryStrategy::ExponentialBackoff;
    let config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    // 測試指數退避計算
    let delay1 = RetryManager::calculate_delay(&strategy, &config, 0);
    assert_eq!(delay1, Duration::from_millis(100));
    
    let delay2 = RetryManager::calculate_delay(&strategy, &config, 1);
    assert_eq!(delay2, Duration::from_millis(200));
    
    let delay3 = RetryManager::calculate_delay(&strategy, &config, 2);
    assert_eq!(delay3, Duration::from_millis(400));
    
    // 測試最大延遲限制
    let delay_max = RetryManager::calculate_delay(&strategy, &config, 10);
    assert_eq!(delay_max, Duration::from_secs(10));
    
    Ok(())
}

#[tokio::test]
async fn test_linear_backoff_calculation() -> Result<()> {
    let strategy = RetryStrategy::LinearBackoff;
    let config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    // 測試線性退避計算
    let delay1 = RetryManager::calculate_delay(&strategy, &config, 0);
    assert_eq!(delay1, Duration::from_millis(100));
    
    let delay2 = RetryManager::calculate_delay(&strategy, &config, 1);
    assert_eq!(delay2, Duration::from_millis(300)); // 100 + 2 * 100
    
    let delay3 = RetryManager::calculate_delay(&strategy, &config, 2);
    assert_eq!(delay3, Duration::from_millis(500)); // 100 + 4 * 100
    
    Ok(())
}

#[tokio::test]
async fn test_fixed_delay_calculation() -> Result<()> {
    let strategy = RetryStrategy::FixedDelay;
    let config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    // 固定延遲應該始終相同
    for attempt in 0..5 {
        let delay = RetryManager::calculate_delay(&strategy, &config, attempt);
        assert_eq!(delay, Duration::from_millis(500));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_jitter_application() -> Result<()> {
    let strategy = RetryStrategy::ExponentialBackoff;
    let config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: true,
        cooldown_aware: false,
    };
    
    // 測試抖動會產生不同的延遲
    let mut delays = Vec::new();
    for _ in 0..10 {
        let delay = RetryManager::calculate_delay(&strategy, &config, 1);
        delays.push(delay);
    }
    
    // 檢查是否有變化（至少有些不同）
    let all_same = delays.iter().all(|&d| d == delays[0]);
    assert!(!all_same, "抖動應該產生不同的延遲值");
    
    // 檢查延遲在合理範圍內（基礎延遲的50%-150%）
    for delay in delays {
        assert!(delay >= Duration::from_millis(100)); // 至少50%
        assert!(delay <= Duration::from_millis(300)); // 最多150%
    }
    
    Ok(())
}

#[test]
fn test_error_classification() {
    // 測試各種錯誤類型的分類
    let test_cases = vec![
        ("Claude usage limit reached", ErrorClassification::RateLimited),
        ("Rate limit exceeded", ErrorClassification::RateLimited),
        ("API quota exhausted", ErrorClassification::RateLimited),
        ("Connection refused", ErrorClassification::NetworkError),
        ("Timeout", ErrorClassification::TimeoutError),
        ("Internal server error", ErrorClassification::ServerError),
        ("Bad request", ErrorClassification::ClientError),
        ("Authentication failed", ErrorClassification::AuthenticationError),
        ("Some random error", ErrorClassification::UnknownError),
    ];
    
    for (error_msg, expected) in test_cases {
        let classification = RetryManager::classify_error(error_msg);
        assert_eq!(
            classification, expected,
            "錯誤訊息 '{}' 分類不正確", error_msg
        );
    }
}

#[test]
fn test_should_retry_decision() {
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    // 測試可重試的錯誤
    assert!(RetryManager::should_retry(&ErrorClassification::RateLimited, 1, &config));
    assert!(RetryManager::should_retry(&ErrorClassification::NetworkError, 1, &config));
    assert!(RetryManager::should_retry(&ErrorClassification::TimeoutError, 1, &config));
    assert!(RetryManager::should_retry(&ErrorClassification::ServerError, 1, &config));
    
    // 測試不可重試的錯誤
    assert!(!RetryManager::should_retry(&ErrorClassification::ClientError, 1, &config));
    assert!(!RetryManager::should_retry(&ErrorClassification::AuthenticationError, 1, &config));
    
    // 測試達到最大嘗試次數
    assert!(!RetryManager::should_retry(&ErrorClassification::NetworkError, 3, &config));
}

#[tokio::test]
async fn test_retry_execution_success() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let call_count_clone = call_count.clone();
    
    let operation = move || {
        let count = call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        async move {
            if count == 1 {
                Err(anyhow::anyhow!("Network error"))
            } else {
                Ok("Success".to_string())
            }
        }
    };
    
    let result = manager.retry_with_strategy(
        RetryStrategy::ExponentialBackoff,
        config,
        operation
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2); // 第一次失敗，第二次成功
    
    // 檢查統計
    assert_eq!(manager.stats.total_attempts, 2);
    assert_eq!(manager.stats.success_count, 1);
    assert_eq!(manager.stats.failure_count, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_retry_execution_max_attempts() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let call_count_clone = call_count.clone();
    
    let operation = move || {
        call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move {
            Err::<String, _>(anyhow::anyhow!("Persistent error"))
        }
    };
    
    let result = manager.retry_with_strategy(
        RetryStrategy::FixedDelay,
        config,
        operation
    ).await;
    
    assert!(result.is_err());
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2); // 達到最大嘗試次數
    
    // 檢查統計
    assert_eq!(manager.stats.total_attempts, 2);
    assert_eq!(manager.stats.success_count, 0);
    assert_eq!(manager.stats.failure_count, 2);
    
    Ok(())
}

#[tokio::test]
async fn test_adaptive_cooldown_strategy() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let attempt_count_clone = attempt_count.clone();
    
    let operation = move || {
        let count = attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        async move {
            if count <= 2 {
                Err(anyhow::anyhow!("Claude usage limit reached. Reset at 4:30 PM"))
            } else {
                Ok("Success after cooldown".to_string())
            }
        }
    };
    
    let start_time = Instant::now();
    let result = manager.retry_with_strategy(
        RetryStrategy::AdaptiveCooldown,
        config,
        operation
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    
    // 適應性冷卻應該花費更多時間（由於冷卻檢測）
    let elapsed = start_time.elapsed();
    assert!(elapsed >= Duration::from_millis(100)); // 至少有一些延遲
    
    Ok(())
}

#[tokio::test]
async fn test_smart_retry_strategy() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 4,
        base_delay: Duration::from_millis(20),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    let mut attempt_count = 0;
    let operation = || async {
        attempt_count += 1;
        match attempt_count {
            1 => Err(anyhow::anyhow!("Network error")),           // 網路錯誤
            2 => Err(anyhow::anyhow!("Rate limit exceeded")),     // 速率限制
            3 => Err(anyhow::anyhow!("Server internal error")),   // 伺服器錯誤
            _ => Ok("Success".to_string()),                       // 最終成功
        }
    };
    
    let result = manager.retry_with_strategy(
        RetryStrategy::SmartRetry,
        config,
        operation
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(attempt_count, 4);
    
    Ok(())
}

#[tokio::test]
async fn test_retry_with_cooldown_detection() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    let mut attempt_count = 0;
    let operation = || async {
        attempt_count += 1;
        if attempt_count == 1 {
            // 模擬冷卻錯誤，包含具體秒數
            Err(anyhow::anyhow!("Rate limited. Try again in 2 seconds"))
        } else {
            Ok("Success after cooldown".to_string())
        }
    };
    
    let start_time = Instant::now();
    let result = manager.retry_with_strategy(
        RetryStrategy::AdaptiveCooldown,
        config,
        operation
    ).await;
    
    assert!(result.is_ok());
    
    // 應該等待冷卻時間（至少2秒）
    let elapsed = start_time.elapsed();
    assert!(elapsed >= Duration::from_secs(2));
    
    Ok(())
}

#[tokio::test]
async fn test_retry_statistics_tracking() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(5),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    // 執行成功的重試
    let operation1 = || async {
        static mut COUNT: u32 = 0;
        unsafe {
            COUNT += 1;
            if COUNT == 1 {
                Err(anyhow::anyhow!("First failure"))
            } else {
                Ok("Success".to_string())
            }
        }
    };
    
    let _ = manager.retry_with_strategy(
        RetryStrategy::FixedDelay,
        config.clone(),
        operation1
    ).await;
    
    // 執行失敗的重試
    let operation2 = || async {
        Err::<String, _>(anyhow::anyhow!("Always fails"))
    };
    
    let _ = manager.retry_with_strategy(
        RetryStrategy::FixedDelay,
        config,
        operation2
    ).await;
    
    // 檢查統計
    let stats = manager.get_stats();
    assert_eq!(stats.total_attempts, 4); // 2 + 2
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 3);
    assert!(stats.success_rate > 0.0 && stats.success_rate < 1.0);
    assert!(stats.average_attempts > 1.0);
    
    Ok(())
}

#[tokio::test]
async fn test_retry_timeout_handling() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50), // 短最大延遲
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    let operation = || async {
        // 模擬長時間運行的操作
        sleep(Duration::from_millis(100)).await;
        Err::<String, _>(anyhow::anyhow!("Operation timeout"))
    };
    
    let start_time = Instant::now();
    let result = manager.retry_with_strategy(
        RetryStrategy::ExponentialBackoff,
        config,
        operation
    ).await;
    
    assert!(result.is_err());
    
    // 應該快速失敗，不等待太長時間
    let elapsed = start_time.elapsed();
    assert!(elapsed < Duration::from_secs(1));
    
    Ok(())
}

#[test]
fn test_retry_config_validation() {
    // 測試有效配置
    let valid_config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    assert!(valid_config.max_attempts > 0);
    assert!(valid_config.base_delay > Duration::ZERO);
    assert!(valid_config.max_delay >= valid_config.base_delay);
    assert!(valid_config.backoff_multiplier > 0.0);
}

#[test]
fn test_retry_config_serialization() -> Result<()> {
    let config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(30),
        backoff_multiplier: 1.5,
        jitter: true,
        cooldown_aware: true,
    };
    
    // 測試序列化
    let serialized = serde_json::to_string(&config)?;
    assert!(serialized.contains("max_attempts"));
    assert!(serialized.contains("500"));
    
    // 測試反序列化
    let deserialized: RetryConfig = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized.max_attempts, 5);
    assert_eq!(deserialized.base_delay, Duration::from_millis(500));
    assert_eq!(deserialized.backoff_multiplier, 1.5);
    assert!(deserialized.jitter);
    assert!(deserialized.cooldown_aware);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_retry_operations() -> Result<()> {
    let manager = std::sync::Arc::new(tokio::sync::Mutex::new(RetryManager::new()?));
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    let mut handles = Vec::new();
    
    // 並發執行多個重試操作
    for i in 0..3 {
        let manager_clone = manager.clone();
        let config_clone = config.clone();
        
        let handle = tokio::spawn(async move {
            let mut mgr = manager_clone.lock().await;
            let operation = || async {
                if i == 0 {
                    Ok(format!("Success {}", i))
                } else {
                    Err(anyhow::anyhow!("Failure {}", i))
                }
            };
            
            mgr.retry_with_strategy(
                RetryStrategy::FixedDelay,
                config_clone,
                operation
            ).await
        });
        
        handles.push(handle);
    }
    
    // 等待所有操作完成
    let results = futures::future::join_all(handles).await;
    
    // 檢查結果
    assert_eq!(results.len(), 3);
    assert!(results[0].is_ok() && results[0].as_ref().unwrap().is_ok()); // 第一個應該成功
    assert!(results[1].is_ok() && results[1].as_ref().unwrap().is_err()); // 其他應該失敗
    assert!(results[2].is_ok() && results[2].as_ref().unwrap().is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_retry_manager_reset() -> Result<()> {
    let mut manager = RetryManager::new()?;
    let config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    // 執行一些操作以產生統計
    let operation = || async {
        Err::<String, _>(anyhow::anyhow!("Test error"))
    };
    
    let _ = manager.retry_with_strategy(
        RetryStrategy::FixedDelay,
        config,
        operation
    ).await;
    
    // 確認有統計數據
    assert!(manager.stats.total_attempts > 0);
    
    // 重置統計
    manager.reset_stats();
    
    // 確認統計已重置
    assert_eq!(manager.stats.total_attempts, 0);
    assert_eq!(manager.stats.success_count, 0);
    assert_eq!(manager.stats.failure_count, 0);
    assert_eq!(manager.stats.success_rate, 0.0);
    
    Ok(())
}

#[test]
fn test_retry_strategy_variants() {
    // 測試所有重試策略變體
    let strategies = vec![
        RetryStrategy::ExponentialBackoff,
        RetryStrategy::LinearBackoff,
        RetryStrategy::FixedDelay,
        RetryStrategy::AdaptiveCooldown,
        RetryStrategy::SmartRetry,
    ];
    
    for strategy in strategies {
        // 測試序列化不會失敗
        let serialized = serde_json::to_string(&strategy);
        assert!(serialized.is_ok());
        
        // 測試反序列化
        if let Ok(json) = serialized {
            let deserialized: Result<RetryStrategy, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok());
        }
    }
}

#[test]
fn test_error_classification_variants() {
    // 測試所有錯誤分類變體
    let classifications = vec![
        ErrorClassification::RateLimited,
        ErrorClassification::NetworkError,
        ErrorClassification::TimeoutError,
        ErrorClassification::ServerError,
        ErrorClassification::ClientError,
        ErrorClassification::AuthenticationError,
        ErrorClassification::UnknownError,
    ];
    
    for classification in classifications {
        // 測試序列化
        let serialized = serde_json::to_string(&classification);
        assert!(serialized.is_ok());
        
        // 測試反序列化
        if let Ok(json) = serialized {
            let deserialized: Result<ErrorClassification, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok());
        }
    }
}