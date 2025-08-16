// 調度服務 - 任務排程管理
// 採用 vibe-kanban 架構模式

use anyhow::Result;
use std::sync::Arc;

use super::{ClaudeService, DatabaseService, MonitoringService};

/// 調度服務
///
/// 負責管理和執行排程任務
pub struct SchedulerService {
    _database_service: Arc<DatabaseService>,
    _claude_service: Arc<ClaudeService>,
    _monitoring_service: Arc<MonitoringService>,
    is_running: std::sync::atomic::AtomicBool,
}

impl SchedulerService {
    /// 創建新的調度服務
    pub async fn new(
        database_service: Arc<DatabaseService>,
        claude_service: Arc<ClaudeService>,
        monitoring_service: Arc<MonitoringService>,
    ) -> Result<Self> {
        Ok(Self {
            _database_service: database_service,
            _claude_service: claude_service,
            _monitoring_service: monitoring_service,
            is_running: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// 啟動調度服務
    pub async fn start(&self) -> Result<()> {
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 停止調度服務
    pub async fn stop(&self) -> Result<()> {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 檢查是否正在運行
    pub async fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

// 實現 Service trait
#[async_trait::async_trait]
impl super::Service for SchedulerService {
    fn name(&self) -> &'static str {
        "scheduler_service"
    }

    async fn start(&self) -> Result<()> {
        self.is_running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("調度服務已啟動");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.is_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("調度服務已停止");
        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.is_running().await
    }

    async fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "service": self.name(),
            "is_running": self.is_running().await,
            "active_jobs": 0,
            "completed_jobs": 0
        })
    }
}
