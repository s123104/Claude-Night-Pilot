// 🔍 Claude Night Pilot - 企業級監控與可觀測性系統
// 基於Context7 Tokio tracing最佳實踐
// 創建時間: 2025-08-17T05:20:00+00:00

pub mod health;
pub mod logging;
pub mod metrics;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// 企業級監控管理器
///
/// 功能特性：
/// - 結構化日誌記錄
/// - 效能指標收集
/// - 健康檢查監控
/// - 告警機制
/// - 可觀測性儀表板
pub struct MonitoringManager {
    /// 效能指標收集器
    metrics_collector: Arc<RwLock<metrics::MetricsCollector>>,

    /// 健康檢查管理器
    health_checker: Arc<health::HealthChecker>,

    /// 日誌配置
    log_config: logging::LogConfig,

    /// 監控是否啟用
    enabled: bool,
}

impl MonitoringManager {
    /// 創建新的監控管理器
    pub async fn new() -> Result<Self> {
        // 初始化日誌系統
        let log_config = logging::LogConfig::new();
        logging::init_logging(&log_config)?;

        info!("🔍 初始化企業級監控系統");

        Ok(MonitoringManager {
            metrics_collector: Arc::new(RwLock::new(metrics::MetricsCollector::new())),
            health_checker: Arc::new(health::HealthChecker::new()),
            log_config,
            enabled: true,
        })
    }

    /// 啟動監控服務
    pub async fn start(&self) -> Result<()> {
        if !self.enabled {
            warn!("監控系統已禁用");
            return Ok(());
        }

        info!("🚀 啟動企業級監控服務");

        // 啟動指標收集
        self.start_metrics_collection().await?;

        // 啟動健康檢查
        self.health_checker.start().await?;

        info!("✅ 監控服務啟動完成");
        Ok(())
    }

    /// 停止監控服務
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 停止監控服務");

        // 停止健康檢查
        self.health_checker.stop().await?;

        // 最終指標報告
        self.generate_final_report().await?;

        info!("✅ 監控服務已停止");
        Ok(())
    }

    /// 記錄操作指標
    pub async fn record_operation(
        &self,
        operation: &str,
        duration: std::time::Duration,
        success: bool,
    ) {
        if !self.enabled {
            return;
        }

        let mut collector = self.metrics_collector.write().await;
        collector.record_operation(operation, duration, success);

        if success {
            info!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "操作成功完成"
            );
        } else {
            warn!(
                operation = operation,
                duration_ms = duration.as_millis(),
                "操作執行失敗"
            );
        }
    }

    /// 記錄錯誤事件
    pub async fn record_error(&self, component: &str, error: &str, context: Option<&str>) {
        error!(
            component = component,
            error = error,
            context = context,
            "系統錯誤事件"
        );

        if self.enabled {
            let mut collector = self.metrics_collector.write().await;
            collector.record_error(component, error);
        }
    }

    /// 獲取系統指標摘要
    pub async fn get_metrics_summary(&self) -> Result<metrics::MetricsSummary> {
        let collector = self.metrics_collector.read().await;
        Ok(collector.get_summary())
    }

    /// 獲取健康狀態
    pub async fn get_health_status(&self) -> health::HealthStatus {
        self.health_checker.get_status().await
    }

    /// 啟動指標收集
    async fn start_metrics_collection(&self) -> Result<()> {
        let collector = self.metrics_collector.clone();

        // 定期指標報告（每5分鐘）
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));

            loop {
                interval.tick().await;

                let collector_guard = collector.read().await;
                let summary = collector_guard.get_summary();

                info!(
                    total_operations = summary.total_operations,
                    success_rate = format!("{:.2}%", summary.success_rate * 100.0),
                    avg_response_time = format!("{:.2}ms", summary.avg_response_time),
                    "📊 定期指標報告"
                );
            }
        });

        Ok(())
    }

    /// 生成最終報告
    async fn generate_final_report(&self) -> Result<()> {
        let summary = self.get_metrics_summary().await?;
        let health = self.get_health_status().await;

        info!(
            total_operations = summary.total_operations,
            success_rate = format!("{:.2}%", summary.success_rate * 100.0),
            avg_response_time = format!("{:.2}ms", summary.avg_response_time),
            error_count = summary.error_count,
            health_status = format!("{:?}", health),
            "📋 最終監控報告"
        );

        Ok(())
    }
}

/// 監控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// 是否啟用監控
    pub enabled: bool,

    /// 指標報告間隔（秒）
    pub metrics_interval: u64,

    /// 健康檢查間隔（秒）
    pub health_check_interval: u64,

    /// 日誌級別
    pub log_level: String,

    /// 日誌輸出路徑
    pub log_path: Option<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        MonitoringConfig {
            enabled: true,
            metrics_interval: 300,     // 5分鐘
            health_check_interval: 60, // 1分鐘
            log_level: "info".to_string(),
            log_path: None,
        }
    }
}
