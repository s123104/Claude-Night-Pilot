// 測試增強執行器的範例程式
use claude_night_pilot_lib::enhanced_executor::EnhancedClaudeExecutor;
use claude_night_pilot_lib::core::ExecutionOptions;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    println!("🚀 測試增強執行器");

    // 創建增強執行器
    let mut executor = EnhancedClaudeExecutor::with_smart_defaults()?;
    println!("✅ 增強執行器創建成功");

    // 健康檢查
    let health = executor.health_check().await?;
    println!("🏥 健康檢查結果:");
    println!("   Claude CLI 可用: {}", health.claude_cli_available);
    println!("   冷卻檢測工作: {}", health.cooldown_detection_working);
    println!("   活躍進程: {}", health.active_processes);

    // 檢查冷卻狀態
    let cooldown = executor.check_cooldown_status().await?;
    println!("❄️  冷卻狀態:");
    println!("   正在冷卻: {}", cooldown.is_cooling);
    if cooldown.is_cooling {
        println!("   剩餘秒數: {}", cooldown.seconds_remaining);
    }

    // 測試執行（使用模擬）
    let test_prompt = "Hello, Claude! 這是一個測試 prompt。";
    let options = ExecutionOptions {
        dry_run: true, // 使用乾跑模式進行測試
        max_retries: 2,
        timeout_seconds: Some(30),
        ..Default::default()
    };

    println!("🔄 執行測試 prompt...");
    match executor.execute_with_full_enhancement(test_prompt, options).await {
        Ok(response) => {
            println!("✅ 執行成功!");
            println!("   執行 ID: {}", response.execution_metadata.execution_id);
            println!("   嘗試次數: {}", response.execution_metadata.total_attempts);
            println!("   回應: {}", &response.completion[..50.min(response.completion.len())]);
        }
        Err(e) => {
            println!("❌ 執行失敗: {}", e);
        }
    }

    // 獲取統計信息
    let retry_stats = executor.get_retry_stats();
    let process_stats = executor.get_process_stats();

    println!("📊 統計信息:");
    println!("   重試統計:");
    println!("     總嘗試次數: {}", retry_stats.total_attempts);
    println!("     成功率: {:.2}%", retry_stats.success_rate * 100.0);
    println!("   進程統計:");
    println!("     總進程: {}", process_stats.total);
    println!("     已完成: {}", process_stats.completed);
    println!("     運行中: {}", process_stats.running);

    // 優雅關閉
    println!("🛑 關閉執行器...");
    executor.shutdown().await?;
    println!("✅ 關閉完成");

    Ok(())
}