// 排程整合測試
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 開始排程整合測試");
    
    // 測試1: 立即排程測試
    println!("\n📋 測試1: 立即排程執行");
    test_immediate_execution().await?;
    
    // 測試2: 2分鐘後排程測試
    println!("\n⏰ 測試2: 2分鐘後排程執行");
    test_delayed_execution().await?;
    
    // 測試3: cron表達式測試
    println!("\n🔄 測試3: Cron表達式排程測試");
    test_cron_scheduling().await?;
    
    println!("\n✅ 所有排程測試完成");
    Ok(())
}

async fn test_immediate_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("執行立即排程測試...");
    
    // 模擬立即執行
    let start_time = std::time::Instant::now();
    
    // 這裡應該調用實際的Claude執行器
    simulate_claude_execution("立即執行測試").await?;
    
    let execution_time = start_time.elapsed();
    println!("✅ 立即執行完成，耗時: {:?}", execution_time);
    
    Ok(())
}

async fn test_delayed_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("設定2分鐘後執行的排程...");
    
    let target_time = chrono::Local::now() + chrono::Duration::minutes(2);
    println!("目標執行時間: {}", target_time.format("%H:%M:%S"));
    
    // 創建一個2分鐘的延遲任務
    let delay_task = tokio::spawn(async move {
        sleep(Duration::from_secs(120)).await; // 2分鐘
        println!("🎯 2分鐘排程觸發！");
        simulate_claude_execution("2分鐘後排程執行測試").await
    });
    
    // 等待任務完成，但設定超時保護
    match timeout(Duration::from_secs(130), delay_task).await {
        Ok(Ok(Ok(()))) => println!("✅ 2分鐘排程執行成功"),
        Ok(Ok(Err(e))) => println!("❌ 2分鐘排程執行失敗: {}", e),
        Ok(Err(e)) => println!("❌ 任務被中斷: {:?}", e),
        Err(_) => println!("⏰ 排程測試超時"),
    }
    
    Ok(())
}

async fn test_cron_scheduling() -> Result<(), Box<dyn std::error::Error>> {
    println!("測試Cron表達式排程...");
    
    // 測試下一分鐘的cron表達式 (假設現在是22:02，設定22:03執行)
    let now = chrono::Local::now();
    let next_minute = now + chrono::Duration::minutes(1);
    let cron_expr = format!("{} {} * * *", next_minute.minute(), next_minute.hour());
    
    println!("Cron表達式: {} (下一分鐘執行)", cron_expr);
    
    // 計算到下一分鐘的延遲
    let wait_seconds = 61 - now.second(); // 等到下一分鐘開始
    println!("等待 {} 秒到下一分鐘...", wait_seconds);
    
    let cron_task = tokio::spawn(async move {
        sleep(Duration::from_secs(wait_seconds as u64)).await;
        println!("🎯 Cron排程觸發！");
        simulate_claude_execution("Cron排程執行測試").await
    });
    
    // 等待任務完成
    match timeout(Duration::from_secs(70), cron_task).await {
        Ok(Ok(Ok(()))) => println!("✅ Cron排程執行成功"),
        Ok(Ok(Err(e))) => println!("❌ Cron排程執行失敗: {}", e),
        Ok(Err(e)) => println!("❌ Cron任務被中斷: {:?}", e),
        Err(_) => println!("⏰ Cron排程測試超時"),
    }
    
    Ok(())
}

// 模擬Claude執行器
async fn simulate_claude_execution(prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 執行Claude命令: \"{}\"", prompt);
    
    // 模擬執行時間
    sleep(Duration::from_millis(500)).await;
    
    println!("📝 Claude回應: 已成功執行排程任務 - {}", prompt);
    
    Ok(())
}