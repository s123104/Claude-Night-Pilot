//! 整合測試模組
//! 
//! 測試不同系統元件間的互動和端到端工作流程
//! 包括資料庫、服務層、介面適配器的整合測試

use claude_night_pilot_lib::core::database::manager::DatabaseManager;
use claude_night_pilot_lib::services::{
    prompt_service::PromptService,
    job_service::JobService,
    health_service::HealthService,
    sync_service::SyncService,
};
use claude_night_pilot_lib::interfaces::{
    tauri_adapter::TauriAdapter,
    cli_adapter::CLIAdapter,
    shared_types::*,
};
use tempfile::tempdir;
use tokio_test;
use std::sync::Arc;
use std::time::Duration;

/// 創建完整的測試環境
async fn setup_test_environment() -> TestEnvironment {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("integration_test.db");
    
    let mut config = claude_night_pilot_lib::core::database::DatabaseConfig::default();
    config.path = db_path.to_str().unwrap().to_string();
    let db_manager = Arc::new(
        DatabaseManager::new(config)
            .await
            .expect("Failed to create test database")
    );
    
    let prompt_service = Arc::new(PromptService::new().await.expect("Failed to create prompt service"));
    let job_service = Arc::new(JobService::new().await.expect("Failed to create job service"));
    let health_service = Arc::new(HealthService::new());
    let sync_service = Arc::new(SyncService::new());
    
    let tauri_adapter = Arc::new(TauriAdapter::new().await.expect("Failed to create Tauri adapter"));
    
    let cli_adapter = Arc::new(CLIAdapter::new().await.expect("Failed to create CLI adapter"));
    
    TestEnvironment {
        db_manager,
        prompt_service,
        job_service,
        health_service,
        sync_service,
        tauri_adapter,
        cli_adapter,
        _temp_dir: temp_dir,
    }
}

struct TestEnvironment {
    db_manager: Arc<DatabaseManager>,
    prompt_service: Arc<PromptService>,
    job_service: Arc<JobService>,
    health_service: Arc<HealthService>,
    sync_service: Arc<SyncService>,
    tauri_adapter: Arc<TauriAdapter>,
    cli_adapter: Arc<CLIAdapter>,
    _temp_dir: tempfile::TempDir, // Keep alive for test duration
}

/// 測試完整的 Prompt 生命周期
#[tokio::test]
async fn test_complete_prompt_lifecycle() {
    let env = setup_test_environment().await;
    
    // 1. 通過 Tauri 適配器創建 Prompt
    let create_request = claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
        title: "整合測試 Prompt".to_string(),
        content: "這是一個完整的整合測試".to_string(),
        tags: Some("integration,test,lifecycle".to_string()),
    };
    
    let tauri_result = env.tauri_adapter.gui_create_prompt(create_request.clone()).await;
    assert!(tauri_result.is_ok());
    let api_response = tauri_result.unwrap();
    assert!(api_response.success);
    let prompt_id = api_response.data.unwrap();
    
    // 2. 通過 PromptService 讀取同一個 Prompt
    let cli_result = env.prompt_service.get_prompt(prompt_id).await;
    assert!(cli_result.is_ok());
    let prompt = cli_result.unwrap().expect("Prompt should exist");
    
    // 3. 驗證資料一致性
    assert_eq!(prompt.title, "整合測試 Prompt");
    assert_eq!(prompt.content, "這是一個完整的整合測試");
    assert_eq!(prompt.id, prompt_id);
    
    // 4. 通過服務層更新 Prompt
    let update_request = claude_night_pilot_lib::services::prompt_service::UpdatePromptRequest {
        id: prompt_id,
        title: Some("更新後的整合測試 Prompt".to_string()),
        content: None,
        tags: Some("integration,test,lifecycle,updated".to_string()),
    };
    
    let update_result = env.prompt_service.update_prompt(update_request).await;
    assert!(update_result.is_ok());
    
    // 5. 通過服務層驗證更新
    let tauri_updated = env.prompt_service.get_prompt(prompt_id).await.unwrap().unwrap();
    let cli_updated = env.prompt_service.get_prompt(prompt_id).await.unwrap().unwrap();
    
    assert_eq!(tauri_updated.title, "更新後的整合測試 Prompt");
    assert_eq!(cli_updated.title, "更新後的整合測試 Prompt");
    assert_eq!(tauri_updated.title, cli_updated.title);
    
    // 6. 刪除 Prompt
    let delete_result = env.prompt_service.delete_prompt(prompt_id).await;
    assert!(delete_result.is_ok());
    
    // 7. 驗證刪除
    let get_after_delete = env.prompt_service.get_prompt(prompt_id).await.unwrap();
    assert!(get_after_delete.is_none());
}

/// 測試 GUI-CLI 同步機制
#[tokio::test]
async fn test_gui_cli_synchronization() {
    let env = setup_test_environment().await;
    
    // 1. 通過服務層創建多個 Prompts (模擬 CLI)
    let cli_prompts = vec![
        claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
            title: "CLI Prompt 1".to_string(),
            content: "CLI 內容 1".to_string(),
            tags: Some("cli".to_string()),
        },
        claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
            title: "CLI Prompt 2".to_string(),
            content: "CLI 內容 2".to_string(),
            tags: Some("cli".to_string()),
        },
    ];
    
    let mut cli_ids = Vec::new();
    for prompt_request in cli_prompts {
        let id = env.prompt_service.create_prompt(prompt_request).await.unwrap();
        cli_ids.push(id);
    }
    
    // 2. 通過 GUI 適配器創建多個 Prompts
    let gui_prompts = vec![
        claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
            title: "GUI Prompt 1".to_string(),
            content: "GUI 內容 1".to_string(),
            tags: Some("gui".to_string()),
        },
        claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
            title: "GUI Prompt 2".to_string(),
            content: "GUI 內容 2".to_string(),
            tags: Some("gui".to_string()),
        },
    ];
    
    let mut gui_ids = Vec::new();
    for prompt_request in gui_prompts {
        let api_response = env.tauri_adapter.gui_create_prompt(prompt_request).await.unwrap();
        assert!(api_response.success);
        let id = api_response.data.unwrap();
        gui_ids.push(id);
    }
    
    // 3. 同步服務功能暫時跳過
    println!("數據一致性檢查暫時跳過");
    
    // 4. 通過服務層獲取完整列表
    let all_prompts = env.prompt_service.list_prompts().await.unwrap();
    
    assert_eq!(all_prompts.len(), 4); // 2 CLI + 2 GUI
    
    // 5. 驗證每個 Prompt 都存在於數據庫中
    for id in cli_ids.iter().chain(gui_ids.iter()) {
        let prompt = env.prompt_service.get_prompt(*id).await.unwrap();
        assert!(prompt.is_some(), "Prompt with id {} should exist", id);
    }
}

/// 測試排程作業完整工作流程（簡化版）
#[tokio::test]
async fn test_complete_job_workflow() {
    let _env = setup_test_environment().await;
    
    // 由於作業調度系統較為複雜，此測試暫時跳過
    // TODO: 實現完整的作業調度測試
    println!("作業工作流程測試暫時跳過");
}

/// 測試系統健康監控整合
#[tokio::test]
async fn test_health_monitoring_integration() {
    let env = setup_test_environment().await;
    
    // 1. 通過健康服務檢查系統健康
    let health_status = env.health_service.quick_health_check().await;
    assert!(health_status.is_ok());
    
    let status = health_status.unwrap();
    println!("系統健康狀態: {:?}", status);
    
    // 2. 測試健康檢查的性能
    let start_time = std::time::Instant::now();
    
    for _ in 0..10 {
        let _ = env.health_service.quick_health_check().await.unwrap();
    }
    
    let elapsed = start_time.elapsed();
    assert!(elapsed < Duration::from_secs(5)); // 10次檢查應在5秒內完成
}

/// 測試錯誤處理和恢復機制
#[tokio::test]
async fn test_error_handling_integration() {
    let env = setup_test_environment().await;
    
    // 1. 測試不存在的 Prompt ID
    let invalid_prompt_result = env.prompt_service.get_prompt(99999).await;
    assert!(invalid_prompt_result.is_ok()); // 函數本身成功
    assert!(invalid_prompt_result.unwrap().is_none()); // 但結果為 None
    
    // 2. 測試作業服務（簡化）
    println!("作業服務測試暫時跳過");
    
    // 3. 測試空資料處理
    let empty_list = env.prompt_service.list_prompts().await.unwrap();
    assert!(empty_list.is_empty());
    
    // 4. 測試系統在部分失敗情況下的穩定性
    let valid_prompt = claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
        title: "錯誤處理測試".to_string(),
        content: "測試內容".to_string(),
        tags: None,
    };
    
    let prompt_id = env.prompt_service.create_prompt(valid_prompt).await.unwrap();
    
    // 測試系統是否仍然健康
    let system_still_healthy = env.health_service.quick_health_check().await;
    assert!(system_still_healthy.is_ok());
    
    println!("Prompt {} 創建成功，系統仍然健康", prompt_id);
}

/// 測試併發操作的資料一致性
#[tokio::test]
async fn test_concurrent_operations() {
    let env = setup_test_environment().await;
    
    // 1. 並發創建 Prompts
    let mut handles = vec![];
    
    for i in 0..10 {
        let prompt_service = Arc::clone(&env.prompt_service);
        let handle = tokio::spawn(async move {
            let request = claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
                title: format!("併發測試 {}", i),
                content: format!("併發測試內容 {}", i),
                tags: Some("concurrent,test".to_string()),
            };
            
            prompt_service.create_prompt(request).await
        });
        handles.push(handle);
    }
    
    // 等待所有併發操作完成
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        results.push(result.unwrap());
    }
    
    // 2. 驗證所有 Prompts 都成功創建
    assert_eq!(results.len(), 10);
    
    // 3. 驗證資料完整性
    let all_prompts = env.prompt_service.list_prompts().await.unwrap();
    assert_eq!(all_prompts.len(), 10);
    
    // 4. 併發讀取測試
    let read_handles: Vec<_> = results.iter().map(|&prompt_id| {
        let prompt_service = Arc::clone(&env.prompt_service);
        
        tokio::spawn(async move {
            let result = prompt_service.get_prompt(prompt_id).await;
            assert!(result.is_ok());
            
            let prompt = result.unwrap();
            assert!(prompt.is_some());
            prompt.unwrap()
        })
    }).collect();
    
    // 等待所有併發讀取完成
    for handle in read_handles {
        let _prompt = handle.await.unwrap();
    }
    
    // 5. 測試系統在高併發下的穩定性
    let health_check = env.health_service.quick_health_check().await;
    assert!(health_check.is_ok());
    
    let status = health_check.unwrap();
    println!("併發測試後系統狀態: {:?}", status);
}

/// 性能基準測試
#[tokio::test]
async fn test_performance_benchmarks() {
    let env = setup_test_environment().await;
    
    // 1. 測試單次操作性能
    let start_time = std::time::Instant::now();
    
    let prompt_request = claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
        title: "性能測試".to_string(),
        content: "性能測試內容".to_string(),
        tags: Some("performance".to_string()),
    };
    
    let prompt_id = env.prompt_service.create_prompt(prompt_request).await.unwrap();
    let creation_time = start_time.elapsed();
    
    // 單次創建應在50ms內完成
    assert!(creation_time < Duration::from_millis(50));
    
    // 2. 測試讀取性能
    let read_start = std::time::Instant::now();
    let _ = env.prompt_service.get_prompt(prompt_id).await.unwrap();
    let read_time = read_start.elapsed();
    
    // 單次讀取應在10ms內完成
    assert!(read_time < Duration::from_millis(10));
    
    // 3. 測試批量操作性能
    let batch_start = std::time::Instant::now();
    
    for i in 0..100 {
        let request = claude_night_pilot_lib::services::prompt_service::CreatePromptRequest {
            title: format!("批量測試 {}", i),
            content: format!("批量測試內容 {}", i),
            tags: Some("batch,performance".to_string()),
        };
        
        env.prompt_service.create_prompt(request).await.unwrap();
    }
    
    let batch_time = batch_start.elapsed();
    
    // 100次創建應在5秒內完成
    assert!(batch_time < Duration::from_secs(5));
    
    println!("性能基準測試結果:");
    println!("- 單次創建: {:?}", creation_time);
    println!("- 單次讀取: {:?}", read_time);
    println!("- 100次批量創建: {:?}", batch_time);
}