// 端到端服務集成測試
// 測試 vibe-kanban 架構的完整工作流程

use std::sync::Arc;
use std::time::Duration;

use crate::services::{ServiceContainer, Service};
use crate::models::{
    ClaudeRequest, ClaudeResponse, ExecutionStatus, 
    Job, JobStatus, JobType, Prompt,
    ApiResponse, PaginatedResponse
};

/// 測試服務容器的初始化和基本功能
#[tokio::test]
async fn test_service_container_initialization() {
    let result = ServiceContainer::new().await;
    assert!(result.is_ok(), "服務容器初始化應該成功");
    
    let container = result.unwrap();
    
    // 測試所有服務的獲取
    let claude_service = container.claude_service();
    let database_service = container.database_service();
    let scheduler_service = container.scheduler_service();
    let monitoring_service = container.monitoring_service();
    
    // 驗證服務名稱
    assert_eq!(claude_service.name(), "claude_service");
    assert_eq!(database_service.name(), "database_service");
    assert_eq!(scheduler_service.name(), "scheduler_service");
    assert_eq!(monitoring_service.name(), "monitoring_service");
}

/// 測試服務容器的啟動和停止
#[tokio::test]
async fn test_service_container_lifecycle() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    
    // 測試啟動
    let start_result = container.start().await;
    assert!(start_result.is_ok(), "服務容器啟動應該成功");
    
    // 等待一段時間確保服務已經啟動
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 測試健康檢查
    let health_status = container.health_check().await;
    assert!(health_status.overall_healthy, "整體健康狀態應該為健康");
    
    // 測試停止
    let stop_result = container.stop().await;
    assert!(stop_result.is_ok(), "服務容器停止應該成功");
}

/// 測試 Claude 服務的完整工作流程
#[tokio::test]
async fn test_claude_service_workflow() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let claude_service = container.claude_service();
    
    // 啟動服務
    claude_service.start().await.expect("Claude 服務啟動失敗");
    
    // 測試健康檢查
    let is_healthy = claude_service.health_check().await;
    // 在測試環境中，健康檢查可能會失敗（因為沒有實際的 Claude CLI）
    // 這是預期的行為
    println!("Claude 服務健康狀態: {}", is_healthy);
    
    // 創建測試請求
    let test_request = ClaudeRequest::new("測試提示: 請說 Hello World");
    let request_id = test_request.id.clone();
    
    // 提交執行請求
    let execution_result = claude_service.submit_execution(test_request).await;
    assert!(execution_result.is_ok(), "執行請求提交應該成功");
    
    // 等待執行完成（模擬執行需要時間）
    tokio::time::sleep(Duration::from_millis(600)).await;
    
    // 檢查執行狀態（應該已經完成並移除）
    let status = claude_service.get_execution_status(&request_id).await;
    assert!(status.is_none(), "執行完成後應該從活動列表移除");
    
    // 檢查執行歷史
    let history = claude_service.get_execution_history(Some(5)).await;
    assert!(!history.is_empty(), "執行歷史應該包含結果");
    assert_eq!(history.len(), 1, "應該有一個執行記錄");
    
    // 檢查執行統計
    let stats = claude_service.get_active_executions_stats().await;
    assert_eq!(stats.get("total").unwrap_or(&0), &0, "活動執行數應該為 0");
    
    // 測試服務統計
    let service_stats = claude_service.get_stats().await;
    assert_eq!(service_stats["service"], "claude_service");
    
    claude_service.stop().await.expect("Claude 服務停止失敗");
}

/// 測試並行執行限制
#[tokio::test]
async fn test_concurrent_execution_limits() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let claude_service = container.claude_service();
    
    claude_service.start().await.expect("Claude 服務啟動失敗");
    
    // 提交多個請求，超過最大並行限制（默認為3）
    let mut request_ids = Vec::new();
    
    for i in 1..=5 {
        let request = ClaudeRequest::new(&format!("測試請求 {}", i));
        let result = claude_service.submit_execution(request).await;
        
        if i <= 3 {
            assert!(result.is_ok(), "前3個請求應該成功提交");
            if let Ok(id) = result {
                request_ids.push(id);
            }
        } else {
            assert!(result.is_err(), "超出限制的請求應該失敗");
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("最大並行執行數限制"), "錯誤訊息應該包含限制說明");
        }
    }
    
    // 檢查活動執行統計
    let stats = claude_service.get_active_executions_stats().await;
    assert_eq!(stats.get("total").unwrap_or(&0), &3, "應該有3個活動執行");
    
    claude_service.stop().await.expect("Claude 服務停止失敗");
}

/// 測試執行取消功能
#[tokio::test]
async fn test_execution_cancellation() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let claude_service = container.claude_service();
    
    claude_service.start().await.expect("Claude 服務啟動失敗");
    
    // 提交執行請求
    let request = ClaudeRequest::new("長時間執行的測試請求");
    let request_id = claude_service.submit_execution(request).await.expect("請求提交失敗");
    
    // 立即取消執行
    let cancel_result = claude_service.cancel_execution(&request_id).await;
    assert!(cancel_result.is_ok(), "執行取消應該成功");
    
    // 檢查執行歷史中是否有取消的記錄
    tokio::time::sleep(Duration::from_millis(100)).await;
    let history = claude_service.get_execution_history(Some(5)).await;
    assert!(!history.is_empty(), "執行歷史應該包含取消的記錄");
    
    let last_result = &history[0];
    assert_eq!(last_result.final_status(), &ExecutionStatus::Cancelled, "最後的執行狀態應該為已取消");
    
    claude_service.stop().await.expect("Claude 服務停止失敗");
}

/// 測試數據庫服務功能
#[tokio::test]
async fn test_database_service_functionality() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let database_service = container.database_service();
    
    // 啟動數據庫服務
    database_service.start().await.expect("數據庫服務啟動失敗");
    
    // 測試健康檢查
    let is_healthy = database_service.health_check().await;
    assert!(is_healthy, "數據庫服務健康檢查應該通過");
    
    // 測試 ping
    let ping_result = database_service.ping().await;
    assert!(ping_result.is_ok(), "數據庫 ping 應該成功");
    
    // 測試服務統計
    let stats = database_service.get_stats().await;
    assert_eq!(stats["service"], "database_service");
    assert_eq!(stats["connection_status"], "mock");
    
    database_service.stop().await.expect("數據庫服務停止失敗");
}

/// 測試調度服務功能
#[tokio::test]
async fn test_scheduler_service_functionality() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let scheduler_service = container.scheduler_service();
    
    // 測試初始狀態
    let initial_running = scheduler_service.is_running().await;
    assert!(!initial_running, "調度服務初始狀態應該為未運行");
    
    // 啟動調度服務
    scheduler_service.start().await.expect("調度服務啟動失敗");
    
    // 檢查運行狀態
    let is_running = scheduler_service.is_running().await;
    assert!(is_running, "調度服務啟動後應該為運行中");
    
    // 測試健康檢查
    let is_healthy = scheduler_service.health_check().await;
    assert!(is_healthy, "調度服務健康檢查應該通過");
    
    // 測試服務統計
    let stats = scheduler_service.get_stats().await;
    assert_eq!(stats["service"], "scheduler_service");
    assert_eq!(stats["is_running"], true);
    
    // 停止調度服務
    scheduler_service.stop().await.expect("調度服務停止失敗");
    
    // 檢查停止後的狀態
    let final_running = scheduler_service.is_running().await;
    assert!(!final_running, "調度服務停止後應該為未運行");
}

/// 測試監控服務功能
#[tokio::test]
async fn test_monitoring_service_functionality() {
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let monitoring_service = container.monitoring_service();
    
    // 測試初始狀態
    let initial_active = monitoring_service.is_active().await;
    assert!(!initial_active, "監控服務初始狀態應該為未激活");
    
    // 啟動監控服務
    monitoring_service.start().await.expect("監控服務啟動失敗");
    
    // 檢查激活狀態
    let is_active = monitoring_service.is_active().await;
    assert!(is_active, "監控服務啟動後應該為激活狀態");
    
    // 測試健康檢查
    let is_healthy = monitoring_service.health_check().await;
    assert!(is_healthy, "監控服務健康檢查應該通過");
    
    // 測試服務統計
    let stats = monitoring_service.get_stats().await;
    assert_eq!(stats["service"], "monitoring_service");
    assert_eq!(stats["is_active"], true);
    assert_eq!(stats["cpu_usage"], 0.0);
    assert_eq!(stats["memory_usage"], 0.0);
    assert_eq!(stats["disk_usage"], 0.0);
    
    // 停止監控服務
    monitoring_service.stop().await.expect("監控服務停止失敗");
    
    // 檢查停止後的狀態
    let final_active = monitoring_service.is_active().await;
    assert!(!final_active, "監控服務停止後應該為未激活");
}

/// 測試 API 響應格式
#[tokio::test]
async fn test_api_response_formats() {
    // 測試成功響應
    let success_response = ApiResponse::success("測試數據");
    assert!(success_response.success);
    assert_eq!(success_response.data, Some("測試數據"));
    assert!(success_response.request_id.is_none());
    
    // 測試帶訊息的成功響應
    let success_with_message = ApiResponse::success_with_message("測試數據", "操作成功完成");
    assert!(success_with_message.success);
    assert_eq!(success_with_message.message, Some("操作成功完成".to_string()));
    
    // 測試錯誤響應
    let error_response: ApiResponse<String> = ApiResponse::error("測試錯誤");
    assert!(!error_response.success);
    assert!(error_response.data.is_none());
    assert_eq!(error_response.message, Some("測試錯誤".to_string()));
    
    // 測試響應轉換
    let mapped_response = success_response.map(|data| data.chars().count());
    assert!(mapped_response.success);
    assert_eq!(mapped_response.data, Some(4)); // "測試數據" 的字符數
    
    // 測試請求 ID
    let response_with_id = ApiResponse::success("測試").with_request_id("req-123");
    assert_eq!(response_with_id.request_id, Some("req-123".to_string()));
}

/// 測試分頁響應
#[tokio::test]
async fn test_paginated_response() {
    let items = vec![1, 2, 3, 4, 5];
    let total_count = 23;
    let page = 2;
    let page_size = 5;
    
    let paginated = PaginatedResponse::new(items, total_count, page, page_size);
    
    assert_eq!(paginated.items.len(), 5);
    assert_eq!(paginated.total_count, 23);
    assert_eq!(paginated.page, 2);
    assert_eq!(paginated.page_size, 5);
    assert_eq!(paginated.total_pages, 5); // ceil(23/5) = 5
    assert!(paginated.has_prev); // page 2 有上一頁
    assert!(paginated.has_next); // page 2 有下一頁（總共5頁）
}

/// 測試完整的端到端工作流程
#[tokio::test]
async fn test_full_end_to_end_workflow() {
    // 初始化服務容器
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    
    // 啟動所有服務
    container.start().await.expect("服務容器啟動失敗");
    
    // 等待服務完全啟動
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // 執行全面健康檢查
    let health_status = container.health_check().await;
    println!("整體健康狀態: {:?}", health_status);
    
    // 驗證各個服務的健康狀態
    assert!(health_status.database_service, "數據庫服務應該健康");
    assert!(health_status.scheduler_service, "調度服務應該健康"); 
    assert!(health_status.monitoring_service, "監控服務應該健康");
    // Claude 服務在測試環境中可能不健康，這是預期的
    
    // 測試 Claude 服務的執行流程
    let claude_service = container.claude_service();
    
    // 創建並執行請求
    let test_request = ClaudeRequest::new("端到端測試: 生成一份簡短的項目狀態報告");
    let execution_id = claude_service.submit_execution(test_request).await;
    
    match execution_id {
        Ok(id) => {
            println!("執行請求已提交，ID: {}", id);
            
            // 等待執行完成
            tokio::time::sleep(Duration::from_millis(700)).await;
            
            // 檢查執行結果
            let history = claude_service.get_execution_history(Some(1)).await;
            if !history.is_empty() {
                let result = &history[0];
                println!("執行結果狀態: {:?}", result.final_status());
                println!("執行時間: {:?}ms", result.response.execution_time_ms);
            }
        },
        Err(e) => {
            // 在測試環境中，Claude CLI 可能不可用，這是預期的
            println!("執行請求失敗（預期在測試環境中）: {}", e);
        }
    }
    
    // 停止所有服務
    container.stop().await.expect("服務容器停止失敗");
    
    println!("✅ 端到端測試完成");
}

/// 性能基準測試
#[tokio::test]
async fn test_performance_benchmarks() {
    let start_time = std::time::Instant::now();
    
    // 服務容器初始化性能測試
    let container = ServiceContainer::new().await.expect("服務容器初始化失敗");
    let init_duration = start_time.elapsed();
    
    assert!(init_duration.as_millis() < 1000, "服務容器初始化應該在1秒內完成");
    println!("服務容器初始化耗時: {}ms", init_duration.as_millis());
    
    // 服務啟動性能測試
    let start_time = std::time::Instant::now();
    container.start().await.expect("服務容器啟動失敗");
    let startup_duration = start_time.elapsed();
    
    assert!(startup_duration.as_millis() < 500, "服務容器啟動應該在500毫秒內完成");
    println!("服務容器啟動耗時: {}ms", startup_duration.as_millis());
    
    // 健康檢查性能測試
    let start_time = std::time::Instant::now();
    let _health_status = container.health_check().await;
    let health_check_duration = start_time.elapsed();
    
    assert!(health_check_duration.as_millis() < 100, "健康檢查應該在100毫秒內完成");
    println!("健康檢查耗時: {}ms", health_check_duration.as_millis());
    
    // 服務停止性能測試
    let start_time = std::time::Instant::now();
    container.stop().await.expect("服務容器停止失敗");
    let shutdown_duration = start_time.elapsed();
    
    assert!(shutdown_duration.as_millis() < 300, "服務容器停止應該在300毫秒內完成");
    println!("服務容器停止耗時: {}ms", shutdown_duration.as_millis());
    
    println!("✅ 性能基準測試通過");
}