// 🏥 Claude Night Pilot - 企業級健康檢查系統
// 系統健康監控與告警機制
// 創建時間: 2025-08-17T05:20:00+00:00

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// 企業級健康檢查管理器
pub struct HealthChecker {
    /// 健康檢查項目
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,

    /// 整體健康狀態
    overall_status: Arc<RwLock<HealthStatus>>,

    /// 檢查間隔
    check_interval: Duration,

    /// 是否運行中
    running: Arc<RwLock<bool>>,
}

/// 健康檢查項目
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// 檢查名稱
    pub name: String,

    /// 檢查函數
    pub check_fn: fn() -> HealthCheckResult,

    /// 檢查間隔
    pub interval: Duration,

    /// 最後檢查時間
    pub last_check: Option<Instant>,

    /// 最後檢查結果
    pub last_result: Option<HealthCheckResult>,

    /// 是否關鍵檢查（失敗時影響整體狀態）
    pub critical: bool,
}

/// 健康檢查結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// 檢查是否通過
    pub healthy: bool,

    /// 響應時間
    pub response_time: Duration,

    /// 檢查消息
    pub message: String,

    /// 詳細信息
    pub details: HashMap<String, String>,
}

/// 系統健康狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,

    /// 警告（非關鍵問題）
    Warning,

    /// 降級服務
    Degraded,

    /// 不健康
    Unhealthy,
}

/// 健康檢查摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    /// 整體狀態
    pub status: HealthStatus,

    /// 檢查結果
    pub checks: HashMap<String, HealthCheckResult>,

    /// 系統運行時間
    pub uptime: Duration,

    /// 檢查時間戳
    pub timestamp: String,
}

impl HealthChecker {
    /// 創建新的健康檢查管理器
    pub fn new() -> Self {
        HealthChecker {
            checks: Arc::new(RwLock::new(HashMap::new())),
            overall_status: Arc::new(RwLock::new(HealthStatus::Healthy)),
            check_interval: Duration::from_secs(60), // 默認1分鐘
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 啟動健康檢查
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("健康檢查已在運行中");
            return Ok(());
        }

        *running = true;
        info!("🏥 啟動健康檢查服務");

        // 啟動檢查循環 - 使用 Arc clone 共享 RwLock
        let checks = Arc::clone(&self.checks);
        let overall_status = Arc::clone(&self.overall_status);
        let running_flag = Arc::clone(&self.running);
        let interval = self.check_interval;

        tokio::spawn(async move {
            let mut check_interval = tokio::time::interval(interval);

            while *running_flag.read().await {
                check_interval.tick().await;

                // 執行所有健康檢查
                let mut check_results = HashMap::new();
                let mut has_critical_failure = false;
                let mut has_warning = false;

                {
                    let mut checks_guard = checks.write().await;
                    for (name, check) in checks_guard.iter_mut() {
                        // 檢查是否需要執行
                        let should_check = check
                            .last_check
                            .map(|last| last.elapsed() >= check.interval)
                            .unwrap_or(true);

                        if should_check {
                            let start = Instant::now();
                            let result = (check.check_fn)();

                            check.last_check = Some(start);
                            check.last_result = Some(result.clone());
                            check_results.insert(name.clone(), result.clone());

                            if !result.healthy {
                                if check.critical {
                                    has_critical_failure = true;
                                    error!(
                                        check = name,
                                        message = result.message,
                                        "關鍵健康檢查失敗"
                                    );
                                } else {
                                    has_warning = true;
                                    warn!(check = name, message = result.message, "健康檢查警告");
                                }
                            }
                        }
                    }
                }

                // 更新整體健康狀態
                let new_status = if has_critical_failure {
                    HealthStatus::Unhealthy
                } else if has_warning {
                    HealthStatus::Warning
                } else {
                    HealthStatus::Healthy
                };

                {
                    let mut status = overall_status.write().await;
                    if !matches!(*status, HealthStatus::Healthy)
                        || !matches!(new_status, HealthStatus::Healthy)
                    {
                        info!(
                            previous_status = format!("{:?}", *status),
                            new_status = format!("{:?}", new_status),
                            "健康狀態變更"
                        );
                    }
                    *status = new_status;
                }
            }
        });

        Ok(())
    }

    /// 停止健康檢查
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("🛑 停止健康檢查服務");
        Ok(())
    }

    /// 註冊健康檢查
    pub async fn register_check(
        &self,
        name: String,
        check_fn: fn() -> HealthCheckResult,
        critical: bool,
    ) {
        let check = HealthCheck {
            name: name.clone(),
            check_fn,
            interval: Duration::from_secs(60),
            last_check: None,
            last_result: None,
            critical,
        };

        let mut checks = self.checks.write().await;
        checks.insert(name.clone(), check);

        info!(check_name = name, critical = critical, "註冊健康檢查");
    }

    /// 獲取健康狀態
    pub async fn get_status(&self) -> HealthStatus {
        self.overall_status.read().await.clone()
    }

    /// 獲取詳細健康摘要
    pub async fn get_health_summary(&self) -> HealthSummary {
        let status = self.get_status().await;
        let checks_guard = self.checks.read().await;

        let mut check_results = HashMap::new();
        for (name, check) in checks_guard.iter() {
            if let Some(result) = &check.last_result {
                check_results.insert(name.clone(), result.clone());
            }
        }

        HealthSummary {
            status,
            checks: check_results,
            uptime: Duration::from_secs(0), // TODO: 實際計算運行時間
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 註冊默認健康檢查
    async fn register_default_checks(&self) {
        // 資料庫連接檢查
        self.register_check(
            "database_connection".to_string(),
            check_database_connection,
            true, // 關鍵檢查
        )
        .await;

        // 記憶體使用檢查
        self.register_check(
            "memory_usage".to_string(),
            check_memory_usage,
            false, // 非關鍵檢查
        )
        .await;

        // 磁碟空間檢查
        self.register_check(
            "disk_space".to_string(),
            check_disk_space,
            false, // 非關鍵檢查
        )
        .await;

        // 排程器狀態檢查
        self.register_check(
            "scheduler_status".to_string(),
            check_scheduler_status,
            true, // 關鍵檢查
        )
        .await;

        info!("✅ 默認健康檢查註冊完成");
    }
}

/// 資料庫連接健康檢查
fn check_database_connection() -> HealthCheckResult {
    let start = Instant::now();

    // TODO: 實際檢查資料庫連接
    // 這裡暫時模擬檢查
    let healthy = true; // 實際應該嘗試連接資料庫
    let response_time = start.elapsed();

    let mut details = HashMap::new();
    details.insert("database_type".to_string(), "SQLite".to_string());
    details.insert("connection_pool".to_string(), "Available".to_string());

    HealthCheckResult {
        healthy,
        response_time,
        message: if healthy {
            "資料庫連接正常".to_string()
        } else {
            "資料庫連接失敗".to_string()
        },
        details,
    }
}

/// 記憶體使用健康檢查
fn check_memory_usage() -> HealthCheckResult {
    let start = Instant::now();

    // TODO: 實際檢查記憶體使用情況
    // 這裡使用模擬數據
    let memory_usage_mb = 45; // 模擬45MB使用
    let memory_limit_mb = 150; // 企業級限制150MB

    let usage_percentage = (memory_usage_mb as f64 / memory_limit_mb as f64) * 100.0;
    let healthy = usage_percentage < 80.0; // 80%以下認為健康

    let mut details = HashMap::new();
    details.insert("usage_mb".to_string(), memory_usage_mb.to_string());
    details.insert("limit_mb".to_string(), memory_limit_mb.to_string());
    details.insert(
        "usage_percentage".to_string(),
        format!("{:.1}%", usage_percentage),
    );

    HealthCheckResult {
        healthy,
        response_time: start.elapsed(),
        message: format!("記憶體使用: {:.1}%", usage_percentage),
        details,
    }
}

/// 磁碟空間健康檢查
fn check_disk_space() -> HealthCheckResult {
    let start = Instant::now();

    // TODO: 實際檢查磁碟空間
    // 這裡使用模擬數據
    let free_space_gb = 10.5; // 模擬10.5GB可用空間
    let total_space_gb = 100.0; // 模擬100GB總空間

    let usage_percentage = ((total_space_gb - free_space_gb) / total_space_gb) * 100.0;
    let healthy = usage_percentage < 85.0; // 85%以下認為健康

    let mut details = HashMap::new();
    details.insert("free_space_gb".to_string(), format!("{:.1}", free_space_gb));
    details.insert(
        "total_space_gb".to_string(),
        format!("{:.1}", total_space_gb),
    );
    details.insert(
        "usage_percentage".to_string(),
        format!("{:.1}%", usage_percentage),
    );

    HealthCheckResult {
        healthy,
        response_time: start.elapsed(),
        message: format!("磁碟使用: {:.1}%", usage_percentage),
        details,
    }
}

/// 排程器狀態健康檢查
fn check_scheduler_status() -> HealthCheckResult {
    let start = Instant::now();

    // TODO: 實際檢查排程器狀態
    // 這裡暫時模擬檢查
    let healthy = true; // 實際應該檢查排程器是否正常運行

    let mut details = HashMap::new();
    details.insert("scheduler_type".to_string(), "RealTimeExecutor".to_string());
    details.insert("active_jobs".to_string(), "9".to_string()); // 當前有9個活躍任務
    details.insert("last_execution".to_string(), "Normal".to_string());

    HealthCheckResult {
        healthy,
        response_time: start.elapsed(),
        message: if healthy {
            "排程器運行正常".to_string()
        } else {
            "排程器狀態異常".to_string()
        },
        details,
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}
