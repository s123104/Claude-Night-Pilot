// 監控服務 - 系統監控和指標收集
// 採用 vibe-kanban 架構模式

use anyhow::Result;

/// 監控服務
///
/// 負責系統監控、指標收集和健康檢查
pub struct MonitoringService {
    is_active: std::sync::atomic::AtomicBool,
}

impl MonitoringService {
    /// 創建新的監控服務
    pub fn new() -> Self {
        Self {
            is_active: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 啟動監控服務
    pub async fn start(&self) -> Result<()> {
        self.is_active
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 停止監控服務
    pub async fn stop(&self) -> Result<()> {
        self.is_active
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 檢查是否處於活動狀態
    pub async fn is_active(&self) -> bool {
        self.is_active.load(std::sync::atomic::Ordering::SeqCst)
    }
}

// 實現 Service trait
#[async_trait::async_trait]
impl super::Service for MonitoringService {
    fn name(&self) -> &'static str {
        "monitoring_service"
    }

    async fn start(&self) -> Result<()> {
        self.is_active
            .store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("監控服務已啟動");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.is_active
            .store(false, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("監控服務已停止");
        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.is_active().await
    }

    async fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "service": self.name(),
            "is_active": self.is_active().await,
            "cpu_usage": 0.0,
            "memory_usage": 0.0,
            "disk_usage": 0.0
        })
    }
}
