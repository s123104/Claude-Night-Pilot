// 簡單的資料庫測試，不使用 DatabaseManager
use std::path::Path;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 創建臨時目錄
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    
    println!("測試資料庫路徑: {:?}", db_path);
    
    // 直接使用 SimpleDatabase
    use claude_night_pilot_lib::simple_db::SimpleDatabase;
    
    // 測試創建資料庫
    let db = SimpleDatabase::new(db_path.to_str().unwrap())?;
    println!("✅ 資料庫創建成功");
    
    // 測試創建 prompt
    let result = db.create_prompt("測試標題", "測試內容")?;
    println!("✅ Prompt 創建成功，ID: {}", result);
    
    Ok(())
}
