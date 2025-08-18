// 簡單的Cron調試程序

use anyhow::Result;
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing tokio-cron-scheduler...");
    
    // 創建排程器
    let _scheduler = JobScheduler::new().await?;
    println!("✅ Scheduler created successfully");
    
    // 測試不同的cron表達式 (統一使用6欄位格式)
    let test_expressions = vec![
        // ❌ 舊的5欄位格式 (分 時 日 月 星期) - 應該失敗
        ("*/10 * * * *", "每10分鐘 (5欄位) - 應失敗"),
        ("*/5 * * * *", "每5分鐘 (5欄位) - 應失敗"),
        ("0 * * * *", "每小時 (5欄位) - 應失敗"),
        ("5 * * * *", "每小時5分 (5欄位) - 應失敗"),
        ("0 0 * * *", "每日 (5欄位) - 應失敗"),
        // ✅ 正確的6欄位格式 (秒 分 時 日 月 星期) - 應該成功
        ("0 */10 * * * *", "每10分鐘 (6欄位)"),
        ("0 */5 * * * *", "每5分鐘 (6欄位)"),
        ("0 0 * * * *", "每小時 (6欄位)"),
        ("0 5 * * * *", "每小時5分 (6欄位)"),
        ("0 0 9 * * *", "每日9點 (6欄位)"),
        ("30 0 12,18 * * *", "每日12點和18點半 (6欄位)"),
        ("0 0 9 * * 1", "每週一9點 (6欄位)"),
    ];
    
    for (expr, description) in test_expressions {
        println!("🧪 Testing cron expression: {} ({})", expr, description);
        
        match CronJob::new_async(expr, |_uuid, _l| {
            Box::pin(async move {
                println!("Job executed!");
            })
        }) {
            Ok(_job) => println!("✅ Cron expression '{}' parsed successfully", expr),
            Err(e) => println!("❌ Cron expression '{}' failed: {}", expr, e),
        }
    }
    
    println!("🏁 Test completed");
    Ok(())
}