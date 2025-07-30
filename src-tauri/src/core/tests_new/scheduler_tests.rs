// 排程系統全面測試用例
use super::super::scheduler::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_cron_scheduler_creation() -> Result<()> {
    let scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    // 測試基本創建
    assert!(!scheduler.jobs.is_empty() == false); // 應該為空
    Ok(())
}

#[tokio::test]
async fn test_cron_scheduler_schedule_job() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let config = CronConfig {
        expression: "0/1 * * * * *".to_string(), // 每秒執行
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    let handle = scheduler.schedule(config).await?;
    
    // 驗證任務已被排程
    assert!(scheduler.jobs.contains_key(&handle));
    
    // 測試是否正在運行
    assert!(!scheduler.is_running(&handle)); // 初始狀態不應該在運行
    
    // 清理
    scheduler.cancel(handle).await?;
    Ok(())
}

#[tokio::test]
async fn test_cron_scheduler_cancel_job() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let config = CronConfig {
        expression: "0/5 * * * * *".to_string(), // 每5秒執行
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    let handle = scheduler.schedule(config).await?;
    
    // 取消任務
    scheduler.cancel(handle).await?;
    
    // 驗證任務狀態已更新
    if let Some(job) = scheduler.jobs.get(&handle) {
        assert!(matches!(job.status, JobStatus::Cancelled));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_cron_scheduler_list_active() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let config1 = CronConfig {
        expression: "0/10 * * * * *".to_string(),
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    let config2 = CronConfig {
        expression: "0/15 * * * * *".to_string(),
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    let handle1 = scheduler.schedule(config1).await?;
    let handle2 = scheduler.schedule(config2).await?;
    
    let active_jobs = scheduler.list_active().await;
    assert_eq!(active_jobs.len(), 2);
    assert!(active_jobs.contains(&handle1));
    assert!(active_jobs.contains(&handle2));
    
    // 清理
    scheduler.cancel(handle1).await?;
    scheduler.cancel(handle2).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_adaptive_scheduler_creation() -> Result<()> {
    let config = AdaptiveConfig {
        intervals: vec![(30, 600), (5, 120), (0, 30)],
        ccusage_integration: false,
        fallback_to_time_based: true,
    };
    
    let scheduler = AdaptiveScheduler::new(config.clone());
    
    // 測試配置正確保存
    assert_eq!(scheduler.config.intervals.len(), 3);
    assert!(!scheduler.config.ccusage_integration);
    assert!(scheduler.config.fallback_to_time_based);
    
    Ok(())
}

#[tokio::test]
async fn test_adaptive_scheduler_interval_calculation() -> Result<()> {
    let intervals = vec![(30, 600), (5, 120), (0, 30)];
    
    // 測試不同剩餘時間的間隔計算
    let long_interval = AdaptiveScheduler::calculate_adaptive_interval(&intervals, 45);
    assert_eq!(long_interval, Duration::from_secs(600));
    
    let medium_interval = AdaptiveScheduler::calculate_adaptive_interval(&intervals, 10);
    assert_eq!(medium_interval, Duration::from_secs(120));
    
    let short_interval = AdaptiveScheduler::calculate_adaptive_interval(&intervals, 2);
    assert_eq!(short_interval, Duration::from_secs(30));
    
    Ok(())
}

#[tokio::test]
async fn test_adaptive_scheduler_schedule() -> Result<()> {
    let config = AdaptiveConfig {
        intervals: vec![(30, 600), (5, 120), (0, 30)],
        ccusage_integration: false,
        fallback_to_time_based: true,
    };
    
    let mut scheduler = AdaptiveScheduler::new(config.clone());
    
    // 排程一個任務
    let handle = scheduler.schedule(config).await?;
    
    // 驗證任務已被創建
    assert!(scheduler.timers.contains_key(&handle));
    
    // 短暫等待以確保任務開始執行
    sleep(Duration::from_millis(100)).await;
    
    // 取消任務
    scheduler.cancel(handle).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_session_scheduler_time_parsing() -> Result<()> {
    let scheduler = SessionScheduler::new();
    
    // 測試有效時間格式
    let (hours, minutes) = scheduler.parse_time("14:30")?;
    assert_eq!(hours, 14);
    assert_eq!(minutes, 30);
    
    let (hours, minutes) = scheduler.parse_time("09:05")?;
    assert_eq!(hours, 9);
    assert_eq!(minutes, 5);
    
    // 測試邊界值
    let (hours, minutes) = scheduler.parse_time("00:00")?;
    assert_eq!(hours, 0);
    assert_eq!(minutes, 0);
    
    let (hours, minutes) = scheduler.parse_time("23:59")?;
    assert_eq!(hours, 23);
    assert_eq!(minutes, 59);
    
    Ok(())
}

#[tokio::test]
async fn test_session_scheduler_invalid_time_parsing() -> Result<()> {
    let scheduler = SessionScheduler::new();
    
    // 測試無效時間格式應該失敗
    assert!(scheduler.parse_time("25:00").is_err()); // 無效小時
    assert!(scheduler.parse_time("12:60").is_err()); // 無效分鐘
    assert!(scheduler.parse_time("invalid").is_err()); // 無效格式
    assert!(scheduler.parse_time("12").is_err()); // 缺少分鐘
    
    Ok(())
}

#[tokio::test]
async fn test_session_scheduler_next_execution_calculation() -> Result<()> {
    let scheduler = SessionScheduler::new();
    let now = chrono::Local::now();
    
    // 測試明天的執行時間
    let future_time = scheduler.calculate_next_execution(
        (now.hour() + 1) % 24,
        now.minute()
    );
    
    // 如果是明天同一時間，應該大於現在
    assert!(future_time > now);
    
    Ok(())
}

#[tokio::test]
async fn test_session_scheduler_schedule() -> Result<()> {
    let mut scheduler = SessionScheduler::new();
    
    let config = SessionConfig {
        scheduled_time: "23:59".to_string(), // 明天的 23:59
        auto_reschedule: true,
        daily_repeat: true,
    };
    
    let handle = scheduler.schedule(config).await?;
    
    // 驗證會話已被創建
    assert!(scheduler.sessions.contains_key(&handle));
    
    // 取消會話
    scheduler.cancel(handle).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_session_scheduler_reschedule() -> Result<()> {
    let mut scheduler = SessionScheduler::new();
    
    let config1 = SessionConfig {
        scheduled_time: "12:00".to_string(),
        auto_reschedule: false,
        daily_repeat: false,
    };
    
    let config2 = SessionConfig {
        scheduled_time: "18:00".to_string(),
        auto_reschedule: true,
        daily_repeat: true,
    };
    
    let handle1 = scheduler.schedule(config1).await?;
    let handle2 = scheduler.reschedule(handle1, config2).await?;
    
    // 驗證新的會話已創建
    assert!(scheduler.sessions.contains_key(&handle2));
    
    // 清理
    scheduler.cancel(handle2).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_scheduler_trait_consistency() -> Result<()> {
    // 測試所有排程器都實現了相同的 trait
    let mut cron_scheduler = CronScheduler::new().await?;
    cron_scheduler.start().await?;
    
    let adaptive_config = AdaptiveConfig {
        intervals: vec![(30, 600)],
        ccusage_integration: false,
        fallback_to_time_based: true,
    };
    let mut adaptive_scheduler = AdaptiveScheduler::new(adaptive_config.clone());
    
    let mut session_scheduler = SessionScheduler::new();
    
    // 測試所有排程器都能執行基本操作
    let cron_handle = cron_scheduler.schedule(CronConfig {
        expression: "0/30 * * * * *".to_string(),
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    }).await?;
    
    let adaptive_handle = adaptive_scheduler.schedule(adaptive_config).await?;
    
    let session_handle = session_scheduler.schedule(SessionConfig {
        scheduled_time: "23:58".to_string(),
        auto_reschedule: false,
        daily_repeat: false,
    }).await?;
    
    // 測試取消操作
    cron_scheduler.cancel(cron_handle).await?;
    adaptive_scheduler.cancel(adaptive_handle).await?;
    session_scheduler.cancel(session_handle).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_scheduler_operations() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let config = CronConfig {
        expression: "0/1 * * * * *".to_string(),
        timezone: "UTC".to_string(),
        max_concurrent: 3,
        timeout_seconds: 30,
    };
    
    // 並發創建多個任務
    let mut handles = Vec::new();
    for _ in 0..5 {
        let handle = scheduler.schedule(config.clone()).await?;
        handles.push(handle);
    }
    
    // 驗證所有任務都已創建
    assert_eq!(handles.len(), 5);
    for handle in &handles {
        assert!(scheduler.jobs.contains_key(handle));
    }
    
    // 並發取消所有任務
    for handle in handles {
        scheduler.cancel(handle).await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_scheduler_timeout_handling() -> Result<()> {
    let mut scheduler = SessionScheduler::new();
    
    let config = SessionConfig {
        scheduled_time: "00:01".to_string(), // 明天很早的時間
        auto_reschedule: false,
        daily_repeat: false,
    };
    
    // 使用超時來測試排程器不會無限期等待
    let result = timeout(
        Duration::from_millis(100),
        scheduler.schedule(config)
    ).await;
    
    // 應該在超時內完成排程（不等待實際執行）
    assert!(result.is_ok());
    
    if let Ok(handle) = result {
        scheduler.cancel(handle).await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_scheduler_error_handling() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    // 測試無效的 cron 表達式
    let invalid_config = CronConfig {
        expression: "invalid cron".to_string(),
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    // 應該返回錯誤
    let result = scheduler.schedule(invalid_config).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_scheduling_config_serialization() -> Result<()> {
    let config = SchedulingConfig {
        scheduler_type: SchedulerType::Cron,
        cron: Some(CronConfig {
            expression: "0 0 12 * * *".to_string(),
            timezone: "Asia/Taipei".to_string(),
            max_concurrent: 2,
            timeout_seconds: 300,
        }),
        adaptive: None,
        session: None,
    };
    
    // 測試序列化
    let serialized = serde_json::to_string(&config)?;
    assert!(serialized.contains("Cron"));
    assert!(serialized.contains("Asia/Taipei"));
    
    // 測試反序列化
    let deserialized: SchedulingConfig = serde_json::from_str(&serialized)?;
    assert!(matches!(deserialized.scheduler_type, SchedulerType::Cron));
    assert!(deserialized.cron.is_some());
    assert!(deserialized.adaptive.is_none());
    
    Ok(())
}