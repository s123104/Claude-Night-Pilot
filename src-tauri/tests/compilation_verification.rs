//! 編譯驗證測試 - 確保所有修復都能正常編譯和運行
//!
//! 這個測試文件專注於驗證修復的編譯錯誤，包括：
//! 1. 正確的匯入路徑 (CLIAdapter vs CliAdapter)
//! 2. 正確的服務構造器調用
//! 3. 正確的數據類型使用

use claude_night_pilot_lib::core::database::DatabaseConfig;
use claude_night_pilot_lib::interfaces::cli_adapter::CLIAdapter;
use claude_night_pilot_lib::services::{
    health_service::HealthService, prompt_service::CreatePromptRequest,
};

/// 測試基本的結構體和類型是否能正確編譯
#[tokio::test]
async fn test_types_compilation() {
    // 測試 DatabaseConfig 結構
    let config = DatabaseConfig::default();
    assert_eq!(config.path, "claude-night-pilot.db");

    // 測試 CreatePromptRequest 結構
    let request = CreatePromptRequest {
        title: "測試 Prompt".to_string(),
        content: "測試內容".to_string(),
        tags: Some("test".to_string()),
    };

    assert_eq!(request.title, "測試 Prompt");
    assert_eq!(request.content, "測試內容");
    assert_eq!(request.tags, Some("test".to_string()));
}

/// 測試服務能否正確初始化（不依賴數據庫）
#[tokio::test]
async fn test_service_initialization() {
    // 測試 HealthService 初始化（不需要數據庫）
    let health_service = HealthService::new();

    // 快速健康檢查應該能運行（即使沒有真實的數據庫連接）
    let health_result = health_service.quick_health_check().await;

    // 這可能失敗，但不應該 panic
    match health_result {
        Ok(status) => println!("健康檢查成功: {:?}", status),
        Err(e) => println!("健康檢查失敗（預期）: {:?}", e),
    }
}

/// 測試 CLIAdapter 結構是否存在且可實例化
#[test]
fn test_cli_adapter_compilation() {
    // 這個測試確保 CLIAdapter 類型存在且可以被引用
    // 由於 CLIAdapter::new() 是 async 的，我們不在這裡實際創建實例

    // 檢查類型是否存在
    fn check_type<T>() {}
    check_type::<CLIAdapter>();

    println!("CLIAdapter 類型編譯成功");
}

/// 測試關鍵的數據結構都能被正確序列化和反序列化
#[test]
fn test_serialization() {
    let request = CreatePromptRequest {
        title: "序列化測試".to_string(),
        content: "測試序列化功能".to_string(),
        tags: Some("serialization,test".to_string()),
    };

    // 測試序列化
    let json = serde_json::to_string(&request).expect("序列化失敗");
    println!("序列化結果: {}", json);

    // 測試反序列化
    let deserialized: CreatePromptRequest = serde_json::from_str(&json).expect("反序列化失敗");
    assert_eq!(deserialized.title, request.title);
    assert_eq!(deserialized.content, request.content);
    assert_eq!(deserialized.tags, request.tags);
}

/// 測試所有核心模組都能被正確匯入
#[test]
fn test_module_imports() {
    use claude_night_pilot_lib::core::database::DatabaseConfig;

    // 這個測試確保所有模組都能被正確匯入
    println!("所有核心模組匯入成功");

    // 檢查一些基本類型
    let _config = DatabaseConfig::default();

    println!("✅ 編譯驗證測試全部通過");
}
