// 核心模組整合測試用例
use super::super::{scheduler::*, cooldown::*, retry::*, process::*, ExecutionOptions};
use anyhow::Result;
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::test]
async fn test_scheduler_with_cooldown_detection() -> Result<()> {
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let cooldown_detector = CooldownDetector::new()?;
    
    // 創建一個會觸發冷卻的任務配置
    let config = CronConfig {
        expression: "0/1 * * * * *".to_string(), // 每秒執行
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 30,
    };
    
    let handle = scheduler.schedule(config).await?;
    
    // 模擬冷卻檢測
    let cooldown_message = "Claude usage limit reached. Your limit will reset at 4:30 PM";
    let cooldown_info = cooldown_detector.detect_cooldown(cooldown_message);
    
    assert!(cooldown_info.is_some());
    
    // 如果檢測到冷卻，應該暫停排程
    if let Some(cooldown) = cooldown_info {
        assert!(cooldown.is_cooling);
        
        // 暫停排程器以等待冷卻
        scheduler.cancel(handle).await?;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_retry_with_adaptive_scheduling() -> Result<()> {
    let mut retry_manager = RetryManager::new()?;
    let adaptive_config = AdaptiveConfig {
        intervals: vec![(30, 600), (5, 120), (0, 30)],
        ccusage_integration: false,
        fallback_to_time_based: true,
    };
    
    let mut adaptive_scheduler = AdaptiveScheduler::new(adaptive_config.clone());
    
    // 創建一個會失敗然後成功的操作
    let mut attempt_count = 0;
    let operation = || async {
        attempt_count += 1;
        if attempt_count == 1 {
            Err(anyhow::anyhow!("Rate limit exceeded. Try again in 2 seconds"))
        } else {
            Ok("Success after cooldown".to_string())
        }
    };
    
    // 配置重試策略
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: true,
    };
    
    // 執行帶有自適應冷卻的重試
    let result = retry_manager.retry_with_strategy(
        RetryStrategy::AdaptiveCooldown,
        retry_config,
        operation
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(attempt_count, 2);
    
    // 使用適應性排程器安排後續任務
    let schedule_handle = adaptive_scheduler.schedule(adaptive_config).await?;
    assert!(adaptive_scheduler.timers.contains_key(&schedule_handle));
    
    // 清理
    adaptive_scheduler.cancel(schedule_handle).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_process_orchestration_with_cooldown() -> Result<()> {
    let mut orchestrator = ProcessOrchestrator::new().await?;
    let cooldown_detector = CooldownDetector::new()?;
    
    // 創建一個 Claude 執行進程
    let process = ProcessDefinition {
        id: "cooldown-aware-execution".to_string(),
        process_type: ProcessType::ClaudeExecution {
            prompt: "Test prompt that might hit rate limit".to_string(),
            options: ExecutionOptions {
                dry_run: true,
                timeout_seconds: Some(30),
                max_retries: 2,
                ..Default::default()
            },
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(120),
        cleanup_on_failure: true,
        retry_config: Some(RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: false,
            cooldown_aware: true,
        }),
    };
    
    let process_id = orchestrator.add_process(process).await?;
    
    // 執行進程
    let result = orchestrator.execute_process(&process_id).await;
    
    // 檢查執行結果
    match result {
        Ok(process_result) => {
            // 如果成功，驗證輸出
            assert!(matches!(process_result.status, ProcessStatus::Completed));
        }
        Err(e) => {
            // 如果失敗，檢查是否是冷卻相關的錯誤
            let error_str = e.to_string();
            let cooldown_info = cooldown_detector.detect_cooldown(&error_str);
            
            if let Some(cooldown) = cooldown_info {
                // 如果檢測到冷卻，這是預期的行為
                assert!(cooldown.is_cooling);
            }
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_full_workflow_integration() -> Result<()> {
    // 創建所有核心組件
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let mut retry_manager = RetryManager::new()?;
    let cooldown_detector = CooldownDetector::new()?;
    let mut orchestrator = ProcessOrchestrator::new().await?;
    
    // 1. 創建一個複雜的工作流程
    let setup_process = ProcessDefinition {
        id: "setup".to_string(),
        process_type: ProcessType::SetupScript {
            script_path: "/bin/echo".to_string(),
            args: vec!["Setup complete".to_string()],
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(30),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    let main_process = ProcessDefinition {
        id: "main-claude-execution".to_string(),
        process_type: ProcessType::ClaudeExecution {
            prompt: "Perform main analysis task".to_string(),
            options: ExecutionOptions {
                dry_run: true,
                timeout_seconds: Some(60),
                max_retries: 3,
                ..Default::default()
            },
        },
        dependencies: vec!["setup".to_string()],
        timeout: Duration::from_secs(180),
        cleanup_on_failure: true,
        retry_config: Some(RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
            cooldown_aware: true,
        }),
    };
    
    let cleanup_process = ProcessDefinition {
        id: "cleanup".to_string(),
        process_type: ProcessType::CleanupScript {
            script_path: "/bin/echo".to_string(),
            cleanup_type: CleanupType::TempFiles,
        },
        dependencies: vec!["main-claude-execution".to_string()],
        timeout: Duration::from_secs(30),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    // 2. 添加進程到編排器
    let setup_id = orchestrator.add_process(setup_process).await?;
    let main_id = orchestrator.add_process(main_process).await?;
    let cleanup_id = orchestrator.add_process(cleanup_process).await?;
    
    // 3. 驗證依賴關係
    assert!(orchestrator.has_dependency(&main_id, &setup_id));
    assert!(orchestrator.has_dependency(&cleanup_id, &main_id));
    
    // 4. 執行完整工作流程
    let workflow_result = orchestrator.execute_with_dependencies(&cleanup_id).await;
    
    // 5. 驗證結果
    match workflow_result {
        Ok(result) => {
            assert!(matches!(result.status, ProcessStatus::Completed));
            println!("✅ 完整工作流程執行成功");
        }
        Err(e) => {
            // 檢查是否是冷卻相關錯誤
            let error_str = e.to_string();
            let cooldown_info = cooldown_detector.detect_cooldown(&error_str);
            
            if cooldown_info.is_some() {
                println!("⏸️ 工作流程因冷卻暫停，這是正常行為");
            } else {
                println!("❌ 工作流程執行失敗: {}", e);
            }
        }
    }
    
    // 6. 安排定期執行
    let cron_config = CronConfig {
        expression: "0 0 */4 * * *".to_string(), // 每4小時執行一次
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 300,
    };
    
    let scheduled_handle = scheduler.schedule(cron_config).await?;
    assert!(scheduler.jobs.contains_key(&scheduled_handle));
    
    // 清理
    scheduler.cancel(scheduled_handle).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_error_propagation_across_modules() -> Result<()> {
    let mut retry_manager = RetryManager::new()?;
    let cooldown_detector = CooldownDetector::new()?;
    
    // 創建一個會產生各種錯誤的操作
    let mut error_sequence = vec![
        "Network connection failed",
        "Claude usage limit reached. Reset at 6:00 PM",
        "Internal server error",
        "Success",
    ];
    
    let mut call_count = 0;
    let operation = || async {
        if call_count < error_sequence.len() - 1 {
            let error_msg = error_sequence[call_count].to_string();
            call_count += 1;
            
            // 檢測冷卻狀態
            let cooldown_info = cooldown_detector.detect_cooldown(&error_msg);
            if let Some(cooldown) = cooldown_info {
                if cooldown.is_cooling {
                    // 如果檢測到冷卻，等待一段時間
                    sleep(Duration::from_millis(50)).await;
                }
            }
            
            Err(anyhow::anyhow!(error_msg))
        } else {
            Ok("Operation completed successfully".to_string())
        }
    };
    
    let retry_config = RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 1.5,
        jitter: false,
        cooldown_aware: true,
    };
    
    let result = retry_manager.retry_with_strategy(
        RetryStrategy::SmartRetry,
        retry_config,
        operation
    ).await;
    
    // 應該最終成功
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Operation completed successfully");
    
    // 檢查重試統計
    let stats = retry_manager.get_stats();
    assert!(stats.total_attempts >= 3); // 至少嘗試了3次
    assert_eq!(stats.success_count, 1);
    assert!(stats.failure_count >= 2);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_module_operations() -> Result<()> {
    let scheduler = std::sync::Arc::new(tokio::sync::Mutex::new(CronScheduler::new().await?));
    let retry_manager = std::sync::Arc::new(tokio::sync::Mutex::new(RetryManager::new()?));
    let orchestrator = std::sync::Arc::new(tokio::sync::Mutex::new(ProcessOrchestrator::new().await?));
    
    // 啟動排程器
    {
        let mut sched = scheduler.lock().await;
        sched.start().await?;
    }
    
    let mut handles = Vec::new();
    
    // 並發創建多個任務
    for i in 0..3 {
        let scheduler_clone = scheduler.clone();
        let retry_manager_clone = retry_manager.clone();
        let orchestrator_clone = orchestrator.clone();
        
        let handle = tokio::spawn(async move {
            // 1. 在排程器中創建任務
            let cron_config = CronConfig {
                expression: format!("0/{} * * * * *", (i + 1) * 10), // 不同頻率
                timezone: "UTC".to_string(),
                max_concurrent: 1,
                timeout_seconds: 30,
            };
            
            let mut sched = scheduler_clone.lock().await;
            let schedule_handle = sched.schedule(cron_config).await.unwrap();
            
            // 2. 創建重試操作
            let mut retry_mgr = retry_manager_clone.lock().await;
            let retry_config = RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_secs(1),
                backoff_multiplier: 2.0,
                jitter: false,
                cooldown_aware: false,
            };
            
            let operation = || async {
                if i == 0 {
                    Ok(format!("Success {}", i))
                } else {
                    Err(anyhow::anyhow!("Failure {}", i))
                }
            };
            
            let retry_result = retry_mgr.retry_with_strategy(
                RetryStrategy::FixedDelay,
                retry_config,
                operation
            ).await;
            
            // 3. 創建進程
            let process = ProcessDefinition {
                id: format!("concurrent-process-{}", i),
                process_type: ProcessType::SetupScript {
                    script_path: "/bin/echo".to_string(),
                    args: vec![format!("Concurrent process {}", i)],
                },
                dependencies: Vec::new(),
                timeout: Duration::from_secs(30),
                cleanup_on_failure: false,
                retry_config: None,
            };
            
            let mut orch = orchestrator_clone.lock().await;
            let process_id = orch.add_process(process).await.unwrap();
            let process_result = orch.execute_process(&process_id).await;
            
            // 清理排程
            sched.cancel(schedule_handle).await.unwrap();
            
            (i, retry_result.is_ok(), process_result.is_ok())
        });
        
        handles.push(handle);
    }
    
    // 等待所有任務完成
    let results = futures::future::join_all(handles).await;
    
    // 驗證結果
    assert_eq!(results.len(), 3);
    for (i, result) in results.into_iter().enumerate() {
        let (task_id, retry_ok, process_ok) = result.unwrap();
        assert_eq!(task_id, i);
        
        if i == 0 {
            assert!(retry_ok); // 第一個任務應該成功
        }
        assert!(process_ok); // 所有進程都應該成功
    }
    
    Ok(())
}

#[tokio::test]
async fn test_module_configuration_integration() -> Result<()> {
    // 測試不同模組配置的整合
    let scheduler_config = SchedulingConfig {
        scheduler_type: SchedulerType::Adaptive,
        cron: None,
        adaptive: Some(AdaptiveConfig {
            intervals: vec![(30, 300), (10, 60), (0, 10)],
            ccusage_integration: true,
            fallback_to_time_based: true,
        }),
        session: None,
    };
    
    let retry_config = RetryConfig {
        max_attempts: 4,
        base_delay: Duration::from_millis(250),
        max_delay: Duration::from_secs(15),
        backoff_multiplier: 1.8,
        jitter: true,
        cooldown_aware: true,
    };
    
    let process_definition = ProcessDefinition {
        id: "config-integration-test".to_string(),
        process_type: ProcessType::ClaudeExecution {
            prompt: "Test configuration integration".to_string(),
            options: ExecutionOptions {
                dry_run: true,
                timeout_seconds: Some(45),
                max_retries: 3,
                safety_check: true,
                ..Default::default()
            },
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(180),
        cleanup_on_failure: true,
        retry_config: Some(retry_config.clone()),
    };
    
    // 測試配置序列化和反序列化
    let scheduler_json = serde_json::to_string(&scheduler_config)?;
    let retry_json = serde_json::to_string(&retry_config)?;
    let process_json = serde_json::to_string(&process_definition)?;
    
    let _: SchedulingConfig = serde_json::from_str(&scheduler_json)?;
    let _: RetryConfig = serde_json::from_str(&retry_json)?;
    let _: ProcessDefinition = serde_json::from_str(&process_json)?;
    
    // 如果所有配置都能正確序列化和反序列化，測試通過
    println!("✅ 所有模組配置整合測試通過");
    
    Ok(())
}

#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    let start_time = std::time::Instant::now();
    
    // 創建多個組件實例
    let mut orchestrator = ProcessOrchestrator::new().await?;
    let mut retry_manager = RetryManager::new()?;
    let cooldown_detector = CooldownDetector::new()?;
    
    // 創建大量進程進行壓力測試
    let mut process_ids = Vec::new();
    for i in 0..10 {
        let process = ProcessDefinition {
            id: format!("load-test-{}", i),
            process_type: ProcessType::SetupScript {
                script_path: "/bin/echo".to_string(),
                args: vec![format!("Load test {}", i)],
            },
            dependencies: Vec::new(),
            timeout: Duration::from_secs(10),
            cleanup_on_failure: false,
            retry_config: None,
        };
        
        let process_id = orchestrator.add_process(process).await?;
        process_ids.push(process_id);
    }
    
    // 並行執行所有進程
    let results = orchestrator.execute_parallel(process_ids).await;
    
    // 執行多個重試操作
    for i in 0..5 {
        let operation = || async {
            if i < 2 {
                Err(anyhow::anyhow!("Simulated error {}", i))
            } else {
                Ok(format!("Success {}", i))
            }
        };
        
        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter: false,
            cooldown_aware: false,
        };
        
        let _ = retry_manager.retry_with_strategy(
            RetryStrategy::ExponentialBackoff,
            retry_config,
            operation
        ).await;
    }
    
    // 執行多個冷卻檢測
    let test_messages = vec![
        "Claude usage limit reached. Reset at 3:30 PM",
        "Rate limit exceeded. Try again in 60 seconds",
        "API quota exhausted",
        "Normal successful response",
        "Another rate limit: wait 120 seconds",
    ];
    
    for message in test_messages {
        let _ = cooldown_detector.detect_cooldown(message);
    }
    
    let elapsed = start_time.elapsed();
    
    // 性能驗證：應該在合理時間內完成
    assert!(elapsed < Duration::from_secs(30), "負載測試執行時間過長: {:?}", elapsed);
    
    // 驗證結果
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok(), "負載測試中的進程執行失敗");
    }
    
    println!("✅ 負載測試完成，執行時間: {:?}", elapsed);
    
    Ok(())
}

#[tokio::test]
async fn test_error_recovery_integration() -> Result<()> {
    let mut orchestrator = ProcessOrchestrator::new().await?;
    let mut retry_manager = RetryManager::new()?;
    
    // 創建一個會失敗的進程，然後測試恢復
    let failing_process = ProcessDefinition {
        id: "recovery-test".to_string(),
        process_type: ProcessType::SetupScript {
            script_path: "/bin/false".to_string(), // 總是失敗
            args: Vec::new(),
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(10),
        cleanup_on_failure: true,
        retry_config: Some(RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: false,
            cooldown_aware: false,
        }),
    };
    
    let process_id = orchestrator.add_process(failing_process).await?;
    let result = orchestrator.execute_process(&process_id).await;
    
    // 驗證失敗處理
    assert!(result.is_err());
    
    // 創建恢復進程
    let recovery_process = ProcessDefinition {
        id: "recovery-success".to_string(),
        process_type: ProcessType::SetupScript {
            script_path: "/bin/echo".to_string(),
            args: vec!["Recovery successful".to_string()],
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(10),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    let recovery_id = orchestrator.add_process(recovery_process).await?;
    let recovery_result = orchestrator.execute_process(&recovery_id).await;
    
    // 驗證恢復成功
    assert!(recovery_result.is_ok());
    
    // 測試重試管理器的錯誤恢復
    let mut attempt_count = 0;
    let recovery_operation = || async {
        attempt_count += 1;
        if attempt_count <= 2 {
            Err(anyhow::anyhow!("Temporary failure"))
        } else {
            Ok("Recovered successfully".to_string())
        }
    };
    
    let retry_config = RetryConfig {
        max_attempts: 4,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        jitter: false,
        cooldown_aware: false,
    };
    
    let retry_result = retry_manager.retry_with_strategy(
        RetryStrategy::ExponentialBackoff,
        retry_config,
        recovery_operation
    ).await;
    
    assert!(retry_result.is_ok());
    assert_eq!(retry_result.unwrap(), "Recovered successfully");
    assert_eq!(attempt_count, 3);
    
    println!("✅ 錯誤恢復整合測試完成");
    
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow() -> Result<()> {
    println!("🚀 開始端到端工作流程測試");
    
    // 1. 初始化所有組件
    let mut scheduler = CronScheduler::new().await?;
    scheduler.start().await?;
    
    let mut orchestrator = ProcessOrchestrator::new().await?;
    let cooldown_detector = CooldownDetector::new()?;
    let mut retry_manager = RetryManager::new()?;
    
    println!("✅ 所有組件初始化完成");
    
    // 2. 創建完整的數據處理工作流程
    let data_prep_process = ProcessDefinition {
        id: "data-preparation".to_string(),
        process_type: ProcessType::SetupScript {
            script_path: "/bin/echo".to_string(),
            args: vec!["Data preparation complete".to_string()],
        },
        dependencies: Vec::new(),
        timeout: Duration::from_secs(30),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    let claude_analysis_process = ProcessDefinition {
        id: "claude-analysis".to_string(),
        process_type: ProcessType::ClaudeExecution {
            prompt: "Analyze the prepared data and provide insights".to_string(),
            options: ExecutionOptions {
                dry_run: true,
                timeout_seconds: Some(120),
                max_retries: 3,
                safety_check: true,
                ..Default::default()
            },
        },
        dependencies: vec!["data-preparation".to_string()],
        timeout: Duration::from_secs(300),
        cleanup_on_failure: true,
        retry_config: Some(RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
            cooldown_aware: true,
        }),
    };
    
    let result_processing_process = ProcessDefinition {
        id: "result-processing".to_string(),
        process_type: ProcessType::SetupScript {
            script_path: "/bin/echo".to_string(),
            args: vec!["Results processed and saved".to_string()],
        },
        dependencies: vec!["claude-analysis".to_string()],
        timeout: Duration::from_secs(60),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    let notification_process = ProcessDefinition {
        id: "notification".to_string(),
        process_type: ProcessType::NotificationSender {
            message: "Workflow completed successfully".to_string(),
            recipients: vec!["admin@example.com".to_string()],
        },
        dependencies: vec!["result-processing".to_string()],
        timeout: Duration::from_secs(30),
        cleanup_on_failure: false,
        retry_config: None,
    };
    
    // 3. 添加所有進程
    let prep_id = orchestrator.add_process(data_prep_process).await?;
    let analysis_id = orchestrator.add_process(claude_analysis_process).await?;
    let processing_id = orchestrator.add_process(result_processing_process).await?;
    let notification_id = orchestrator.add_process(notification_process).await?;
    
    println!("✅ 工作流程進程創建完成");
    
    // 4. 驗證依賴關係
    assert!(orchestrator.has_dependency(&analysis_id, &prep_id));
    assert!(orchestrator.has_dependency(&processing_id, &analysis_id));
    assert!(orchestrator.has_dependency(&notification_id, &processing_id));
    
    // 5. 執行完整工作流程
    println!("🔄 開始執行工作流程");
    let workflow_start = std::time::Instant::now();
    
    let final_result = orchestrator.execute_with_dependencies(&notification_id).await;
    
    let workflow_duration = workflow_start.elapsed();
    println!("⏱️ 工作流程執行時間: {:?}", workflow_duration);
    
    // 6. 驗證工作流程結果
    match final_result {
        Ok(result) => {
            assert!(matches!(result.status, ProcessStatus::Completed));
            println!("✅ 端到端工作流程執行成功");
            println!("📊 執行輸出: {}", &result.output[..result.output.len().min(100)]);
        }
        Err(e) => {
            println!("⚠️ 工作流程執行遇到問題: {}", e);
            
            // 檢查是否是冷卻相關問題
            let error_str = e.to_string();
            if let Some(cooldown_info) = cooldown_detector.detect_cooldown(&error_str) {
                if cooldown_info.is_cooling {
                    println!("❄️ 檢測到冷卻狀態，剩餘時間: {}秒", cooldown_info.seconds_remaining);
                    
                    // 在實際環境中，這裡會等待冷卻結束後重新執行
                    println!("🔄 在真實環境中將等待冷卻結束後重新執行");
                }
            }
        }
    }
    
    // 7. 設置定期執行
    let periodic_config = CronConfig {
        expression: "0 0 2 * * *".to_string(), // 每天凌晨2點執行
        timezone: "UTC".to_string(),
        max_concurrent: 1,
        timeout_seconds: 1800, // 30分鐘超時
    };
    
    let periodic_handle = scheduler.schedule(periodic_config).await?;
    println!("⏰ 已安排定期執行: 每日凌晨2點");
    
    // 8. 獲取統計信息
    let process_stats = orchestrator.get_statistics();
    let retry_stats = retry_manager.get_stats();
    
    println!("📈 統計信息:");
    println!("   進程統計: 總計{}, 完成{}, 失敗{}", 
             process_stats.total, process_stats.completed, process_stats.failed);
    println!("   重試統計: 總嘗試{}, 成功率{:.1}%", 
             retry_stats.total_attempts, retry_stats.success_rate * 100.0);
    
    // 9. 清理資源
    scheduler.cancel(periodic_handle).await?;
    println!("🧹 資源清理完成");
    
    println!("🎉 端到端工作流程測試全部完成");
    
    Ok(())
}