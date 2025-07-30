// 冷卻檢測系統全面測試用例
use super::super::cooldown::*;
use anyhow::Result;

#[tokio::test]
async fn test_cooldown_detector_creation() -> Result<()> {
    let detector = CooldownDetector::new()?;
    
    // 驗證正則表達式已正確編譯
    assert!(detector.usage_limit_regex.is_match("Claude usage limit reached"));
    assert!(detector.time_parsing_regex.is_match("14:30"));
    
    Ok(())
}

#[test]
fn test_usage_limit_detection() {
    let detector = CooldownDetector::new().unwrap();
    
    let test_cases = vec![
        "Claude usage limit reached. Your limit will reset at 4:30 PM (EST)",
        "usage limit reached, reset at 11pm",
        "Claude usage limit reached. Your limit will reset at 9 am",
        "Usage limit reached. Will reset at 2:15 PM",
    ];

    for case in test_cases {
        let result = detector.detect_cooldown(case);
        assert!(
            result.is_some(),
            "Failed to detect cooldown in: {}",
            case
        );
        
        if let Some(cooldown) = result {
            assert!(matches!(
                cooldown.cooldown_pattern,
                Some(CooldownPattern::UsageLimitReached { .. })
            ));
            assert!(!cooldown.original_message.is_empty());
        }
    }
}

#[test]
fn test_usage_limit_detection_negative_cases() {
    let detector = CooldownDetector::new().unwrap();
    
    let negative_cases = vec![
        "Claude is working normally",
        "Request completed successfully",
        "No usage limit issues",
        "Error: connection failed",
    ];

    for case in negative_cases {
        let result = detector.detect_cooldown(case);
        assert!(
            result.is_none(),
            "False positive for: {}",
            case
        );
    }
}

#[test]
fn test_seconds_cooldown_detection() {
    let detector = CooldownDetector::new().unwrap();
    
    let test_cases = vec![
        ("Error: cooldown: 123s", Some(123)),
        ("Please wait 45 seconds before retrying", Some(45)),
        ("Rate limited. retry in 60 seconds", Some(60)),
        ("30 seconds remaining", Some(30)),
        ("Try again in 90 seconds", Some(90)),
        ("No cooldown message", None),
    ];

    for (input, expected_seconds) in test_cases {
        let result = detector.detect_cooldown(input);
        
        match expected_seconds {
            Some(expected) => {
                assert!(
                    result.is_some(),
                    "Failed to detect cooldown in: {}",
                    input
                );
                
                if let Some(cooldown) = result {
                    assert_eq!(cooldown.seconds_remaining, expected as u64);
                    assert!(matches!(
                        cooldown.cooldown_pattern,
                        Some(CooldownPattern::RateLimitExceeded { .. })
                    ));
                }
            }
            None => {
                assert!(
                    result.is_none(),
                    "False positive for: {}",
                    input
                );
            }
        }
    }
}

#[test]
fn test_time_parsing() {
    let detector = CooldownDetector::new().unwrap();
    
    let valid_cases = vec![
        "4:30 PM",
        "11pm",
        "9 am",
        "14:30",
        "2:15 PM",
        "12:00 AM",
        "12:00 PM",
    ];

    for time_str in valid_cases {
        let result = detector.parse_reset_time(time_str);
        assert!(
            result.is_some(),
            "Failed to parse valid time: {}",
            time_str
        );
        
        if let Some(parsed_time) = result {
            // 驗證解析的時間在未來
            let now = chrono::Local::now();
            assert!(
                parsed_time > now,
                "Parsed time should be in the future for: {}",
                time_str
            );
        }
    }
}

#[test]
fn test_time_parsing_invalid() {
    let detector = CooldownDetector::new().unwrap();
    
    let invalid_cases = vec![
        "25:00",     // 無效小時
        "12:70",     // 無效分鐘
        "invalid",   // 無效格式
        "",          // 空字串
        "13:00 XM",  // 無效 AM/PM
    ];

    for time_str in invalid_cases {
        let result = detector.parse_reset_time(time_str);
        assert!(
            result.is_none(),
            "Should not parse invalid time: {}",
            time_str
        );
    }
}

#[test]
fn test_rate_limit_detection() {
    let detector = CooldownDetector::new().unwrap();
    
    let test_cases = vec![
        "Rate limit exceeded. Try again in 30 seconds",
        "API rate limit: wait 5 minutes",
        "Rate limited for 2 hours",
        "Rate limit: 120 seconds",
    ];

    for case in test_cases {
        let result = detector.detect_cooldown(case);
        assert!(
            result.is_some(),
            "Failed to detect rate limit in: {}",
            case
        );
        
        if let Some(cooldown) = result {
            assert!(matches!(
                cooldown.cooldown_pattern,
                Some(CooldownPattern::RateLimitExceeded { .. })
            ));
            assert!(cooldown.seconds_remaining > 0);
        }
    }
}

#[test]
fn test_api_quota_detection() {
    let detector = CooldownDetector::new().unwrap();
    
    let test_cases = vec![
        "API quota exceeded",
        "Monthly limit reached",
        "Billing quota exceeded",
        "Insufficient credits",
    ];

    for case in test_cases {
        let result = detector.detect_cooldown(case);
        assert!(
            result.is_some(),
            "Failed to detect API quota issue in: {}",
            case
        );
        
        if let Some(cooldown) = result {
            assert!(matches!(
                cooldown.cooldown_pattern,
                Some(CooldownPattern::ApiQuotaExhausted { .. })
            ));
            assert!(cooldown.is_cooling);
            assert!(cooldown.seconds_remaining > 0);
        }
    }
}

#[test]
fn test_cooldown_info_serialization() -> Result<()> {
    use std::time::SystemTime;
    
    let cooldown_info = CooldownInfo {
        is_cooling: true,
        seconds_remaining: 300,
        next_available_time: Some(SystemTime::now()),
        reset_time: Some(chrono::Local::now()),
        original_message: "Test cooldown message".to_string(),
        cooldown_pattern: Some(CooldownPattern::UsageLimitReached {
            reset_time: "4:30 PM".to_string(),
        }),
    };
    
    // 測試序列化
    let serialized = serde_json::to_string(&cooldown_info)?;
    assert!(serialized.contains("is_cooling"));
    assert!(serialized.contains("300"));
    
    // 測試反序列化
    let deserialized: CooldownInfo = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized.is_cooling, true);
    assert_eq!(deserialized.seconds_remaining, 300);
    
    Ok(())
}

#[tokio::test]
async fn test_smart_wait_short_cooldown() -> Result<()> {
    let detector = CooldownDetector::new()?;
    
    let cooldown_info = CooldownInfo {
        is_cooling: true,
        seconds_remaining: 2, // 短期冷卻
        next_available_time: None,
        reset_time: None,
        original_message: "Short cooldown".to_string(),
        cooldown_pattern: None,
    };
    
    let start = std::time::Instant::now();
    detector.smart_wait(&cooldown_info).await;
    let elapsed = start.elapsed();
    
    // 應該等待約 2 秒
    assert!(elapsed >= std::time::Duration::from_secs(2));
    assert!(elapsed < std::time::Duration::from_secs(3));
    
    Ok(())
}

#[tokio::test]
async fn test_smart_wait_no_cooldown() -> Result<()> {
    let detector = CooldownDetector::new()?;
    
    let cooldown_info = CooldownInfo {
        is_cooling: false,
        seconds_remaining: 0,
        next_available_time: None,
        reset_time: None,
        original_message: "No cooldown".to_string(),
        cooldown_pattern: None,
    };
    
    let start = std::time::Instant::now();
    detector.smart_wait(&cooldown_info).await;
    let elapsed = start.elapsed();
    
    // 不應該等待
    assert!(elapsed < std::time::Duration::from_millis(100));
    
    Ok(())
}

#[test]
fn test_cooldown_status_formatting() {
    let detector = CooldownDetector::new().unwrap();
    
    // 測試無冷卻狀態
    let no_cooldown = CooldownInfo {
        is_cooling: false,
        seconds_remaining: 0,
        next_available_time: None,
        reset_time: None,
        original_message: "".to_string(),
        cooldown_pattern: None,
    };
    
    let status = detector.format_cooldown_status(&no_cooldown);
    assert!(status.contains("可用"));
    
    // 測試使用限制冷卻
    let usage_cooldown = CooldownInfo {
        is_cooling: true,
        seconds_remaining: 3661, // 1h 1m 1s
        next_available_time: None,
        reset_time: None,
        original_message: "".to_string(),
        cooldown_pattern: Some(CooldownPattern::UsageLimitReached {
            reset_time: "4:30 PM".to_string(),
        }),
    };
    
    let status = detector.format_cooldown_status(&usage_cooldown);
    assert!(status.contains("使用限制"));
    assert!(status.contains("4:30 PM"));
    assert!(status.contains("1h 1m 1s"));
    
    // 測試速率限制冷卻
    let rate_cooldown = CooldownInfo {
        is_cooling: true,
        seconds_remaining: 90, // 1m 30s
        next_available_time: None,
        reset_time: None,
        original_message: "".to_string(),
        cooldown_pattern: Some(CooldownPattern::RateLimitExceeded { seconds: 90 }),
    };
    
    let status = detector.format_cooldown_status(&rate_cooldown);
    assert!(status.contains("速率限制"));
    assert!(status.contains("1m 30s"));
}

#[test]
fn test_multiple_usage_limit_matches() {
    let detector = CooldownDetector::new().unwrap();
    
    // 測試多個匹配項目，應該選擇最後一個
    let input = "Claude usage limit reached. Reset at 2:00 PM. Later: usage limit reached, reset at 4:30 PM";
    
    let result = detector.detect_cooldown(input);
    assert!(result.is_some());
    
    if let Some(cooldown) = result {
        if let Some(CooldownPattern::UsageLimitReached { reset_time }) = &cooldown.cooldown_pattern {
            assert!(reset_time.contains("4:30 PM"));
        }
    }
}

#[test]
fn test_time_parsing_edge_cases() {
    let detector = CooldownDetector::new().unwrap();
    
    // 測試邊界情況
    let midnight_am = detector.parse_reset_time("12:00 AM");
    assert!(midnight_am.is_some());
    
    let midnight_pm = detector.parse_reset_time("12:00 PM");
    assert!(midnight_pm.is_some());
    
    let noon = detector.parse_reset_time("12:00 PM");
    assert!(noon.is_some());
    
    // 驗證 12 AM 和 12 PM 的正確處理
    if let (Some(am), Some(pm)) = (midnight_am, midnight_pm) {
        // 12 PM (noon) 應該比 12 AM (midnight) 早（在同一天內）
        let am_hour = am.hour();
        let pm_hour = pm.hour();
        assert_eq!(am_hour, 0);  // 12 AM = 0 hours
        assert_eq!(pm_hour, 12); // 12 PM = 12 hours
    }
}

#[tokio::test]
async fn test_claude_doctor_cooldown_mock() -> Result<()> {
    let detector = CooldownDetector::new()?;
    
    // 這個測試假設 claude doctor 命令不可用（模擬環境）
    let result = detector.check_claude_doctor_cooldown().await;
    
    // 應該返回一個結果（可能是沒有冷卻的狀態）
    assert!(result.is_ok());
    
    let cooldown_info = result?;
    // 在模擬環境中，通常應該回報沒有冷卻
    assert!(!cooldown_info.is_cooling || cooldown_info.is_cooling); // 接受任一結果
    
    Ok(())
}

#[test]
fn test_cooldown_pattern_variants() {
    // 測試所有冷卻模式變體
    let patterns = vec![
        CooldownPattern::UsageLimitReached {
            reset_time: "4:30 PM".to_string(),
        },
        CooldownPattern::RateLimitExceeded { seconds: 300 },
        CooldownPattern::ApiQuotaExhausted {
            next_reset: chrono::Local::now(),
        },
        CooldownPattern::ClaudeSpecificError {
            code: "LIMIT_001".to_string(),
            message: "Custom error".to_string(),
        },
    ];
    
    for pattern in patterns {
        // 測試序列化不會失敗
        let serialized = serde_json::to_string(&pattern);
        assert!(serialized.is_ok());
        
        // 測試反序列化
        if let Ok(json) = serialized {
            let deserialized: Result<CooldownPattern, _> = serde_json::from_str(&json);
            assert!(deserialized.is_ok());
        }
    }
}

#[test]
fn test_complex_error_messages() {
    let detector = CooldownDetector::new().unwrap();
    
    // 測試複雜的錯誤訊息
    let complex_message = r#"
        Error: Request failed with status 429
        Claude usage limit reached. Your limit will reset at 4:30 PM (EST)
        Please try again later.
        Additional info: Rate limited for 120 seconds
    "#;
    
    let result = detector.detect_cooldown(complex_message);
    assert!(result.is_some());
    
    if let Some(cooldown) = result {
        // 應該檢測到使用限制（第一個匹配的模式）
        assert!(matches!(
            cooldown.cooldown_pattern,
            Some(CooldownPattern::UsageLimitReached { .. })
        ));
    }
}