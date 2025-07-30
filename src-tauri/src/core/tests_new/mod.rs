// 核心模組測試組織
//
// 這個模組包含所有核心功能的全面測試用例：
// - scheduler_tests: 排程系統測試（Cron、適應性、會話排程）
// - cooldown_tests: 冷卻檢測系統測試（模式識別、時間解析、智慧等待）
// - retry_tests: 重試策略系統測試（指數退避、錯誤分類、智慧重試）
// - process_tests: 進程編排系統測試（依賴管理、並行執行、生命週期）
// - integration_tests: 整合測試（模組間協作、端到端工作流程）

pub mod scheduler_tests;
pub mod cooldown_tests;
pub mod retry_tests;
pub mod process_tests;
pub mod integration_tests;

// 測試工具函數
#[cfg(test)]
pub mod test_utils {
    use std::time::Duration;
    use tokio::time::sleep;
    
    /// 測試用的短延遲，避免測試運行時間過長
    pub async fn short_delay() {
        sleep(Duration::from_millis(10)).await;
    }
    
    /// 測試用的中等延遲
    pub async fn medium_delay() {
        sleep(Duration::from_millis(100)).await;
    }
    
    /// 驗證時間範圍的輔助函數
    pub fn is_time_in_range(actual: Duration, expected: Duration, tolerance_ms: u64) -> bool {
        let tolerance = Duration::from_millis(tolerance_ms);
        actual >= expected.saturating_sub(tolerance) && 
        actual <= expected + tolerance
    }
    
    /// 生成測試用的隨機字符串
    pub fn random_test_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("test_{}", timestamp)
    }
}

// 測試配置常數
#[cfg(test)]
pub mod test_config {
    use std::time::Duration;
    
    /// 測試用的短超時時間
    pub const SHORT_TIMEOUT: Duration = Duration::from_secs(5);
    
    /// 測試用的中等超時時間
    pub const MEDIUM_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// 測試用的長超時時間
    pub const LONG_TIMEOUT: Duration = Duration::from_secs(120);
    
    /// 測試用的最大重試次數
    pub const MAX_TEST_RETRIES: u32 = 3;
    
    /// 測試用的基礎延遲
    pub const BASE_TEST_DELAY: Duration = Duration::from_millis(10);
}