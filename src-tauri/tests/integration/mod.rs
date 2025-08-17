// 🧪 Claude Night Pilot - 企業級整合測試模塊
// 創建時間: 2025-08-17T05:20:00+00:00

pub mod scheduler_tests;
pub mod cli_tests;
pub mod database_tests;

// 測試工具函數
pub mod utils {
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    /// 創建臨時測試資料庫
    pub fn create_test_db() -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        (temp_dir, db_path)
    }
    
    /// 測試環境清理
    pub fn cleanup_test_env() {
        // 清理測試環境
        println!("🧹 清理測試環境");
    }
}
