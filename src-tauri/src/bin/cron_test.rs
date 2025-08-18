// 簡單的Cron調試程序

use anyhow::Result;
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Testing tokio-cron-scheduler...");
    
    // 創建排程器
    let scheduler = JobScheduler::new().await?;
    println!("✅ Scheduler created successfully");
    
    // 測試不同的cron表達式 (可能需要6部分格式)
    let test_expressions = vec![
        // 標準5部分格式 (分 時 日 月 星期)
        ("*/10 * * * *", "每10分鐘"),
        ("*/5 * * * *", "每5分鐘"),
        ("0 * * * *", "每小時"),
        ("5 * * * *", "每小時5分"),
        ("0 0 * * *", "每日"),
        // 6部分格式 (秒 分 時 日 月 星期)
        ("0 */10 * * * *", "每10分鐘 (6部分)"),
        ("0 */5 * * * *", "每5分鐘 (6部分)"),
        ("0 0 * * * *", "每小時 (6部分)"),
        ("0 5 * * * *", "每小時5分 (6部分)"),
        ("0 0 0 * * *", "每日 (6部分)"),
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