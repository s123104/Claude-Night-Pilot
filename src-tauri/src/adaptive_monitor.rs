// src-tauri/src/adaptive_monitor.rs
// 自適應監控系統 - 根據Claude使用量動態調整監控頻率
// 基於ClaudeNightsWatch與CCAutoRenew最佳實踐 [2025-07-24T00:55:47+08:00]

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration as TokioDuration};

use crate::usage_tracker::{UsageTracker, UsageInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonitoringMode {
    Normal,      // > 30分鐘剩餘 - 10分鐘檢查一次
    Approaching, // 5-30分鐘剩餘 - 2分鐘檢查一次
    Imminent,    // 1-5分鐘剩餘 - 30秒檢查一次
    Critical,    // < 1分鐘剩餘 - 10秒檢查一次
    Unavailable, // 0分鐘剩餘 - 1分鐘檢查一次恢復
    Unknown,     // 無法確定狀態 - 5分鐘檢查一次
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub normal_interval_minutes: u64,      // 正常模式間隔 (預設: 10分鐘)
    pub approaching_interval_minutes: u64, // 接近模式間隔 (預設: 2分鐘)
    pub imminent_interval_seconds: u64,    // 緊急模式間隔 (預設: 30秒)
    pub critical_interval_seconds: u64,    // 危急模式間隔 (預設: 10秒)
    pub unavailable_interval_minutes: u64, // 不可用模式間隔 (預設: 1分鐘)
    pub unknown_interval_minutes: u64,     // 未知模式間隔 (預設: 5分鐘)
    pub approaching_threshold_minutes: u64, // 接近模式閾值 (預設: 30分鐘)
    pub imminent_threshold_minutes: u64,   // 緊急模式閾值 (預設: 5分鐘)
    pub critical_threshold_minutes: u64,   // 危急模式閾值 (預設: 1分鐘)
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            normal_interval_minutes: 10,      // 10分鐘
            approaching_interval_minutes: 2,  // 2分鐘
            imminent_interval_seconds: 30,    // 30秒
            critical_interval_seconds: 10,    // 10秒
            unavailable_interval_minutes: 1,  // 1分鐘
            unknown_interval_minutes: 5,      // 5分鐘
            approaching_threshold_minutes: 30, // 30分鐘閾值
            imminent_threshold_minutes: 5,    // 5分鐘閾值
            critical_threshold_minutes: 1,    // 1分鐘閾值
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatus {
    pub current_mode: MonitoringMode,
    pub next_check_at: DateTime<Utc>,
    pub last_usage_info: Option<UsageInfo>,
    pub check_count: u64,
    pub mode_changes: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: MonitoringEventType,
    pub mode: MonitoringMode,
    pub usage_info: Option<UsageInfo>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEventType {
    ModeChanged,
    StatusUpdated,
    AvailabilityChanged,
    Error,
    ConfigUpdated,
}

pub struct AdaptiveMonitor {
    config: MonitoringConfig,
    current_mode: MonitoringMode,
    last_check: Option<DateTime<Utc>>,
    last_usage_info: Option<UsageInfo>,
    usage_tracker: Arc<Mutex<UsageTracker>>,
    event_sender: broadcast::Sender<MonitoringEvent>,
    check_count: u64,
    mode_changes: u64,
    start_time: DateTime<Utc>,
    is_running: Arc<Mutex<bool>>,
}

impl AdaptiveMonitor {
    pub fn new(
        usage_tracker: Arc<Mutex<UsageTracker>>,
        config: Option<MonitoringConfig>,
    ) -> (Self, broadcast::Receiver<MonitoringEvent>) {
        let (event_sender, event_receiver) = broadcast::channel(100);
        
        let monitor = Self {
            config: config.unwrap_or_default(),
            current_mode: MonitoringMode::Unknown,
            last_check: None,
            last_usage_info: None,
            usage_tracker,
            event_sender,
            check_count: 0,
            mode_changes: 0,
            start_time: Utc::now(),
            is_running: Arc::new(Mutex::new(false)),
        };

        (monitor, event_receiver)
    }

    /// 啟動自適應監控系統
    /// [最佳實踐:CCAutoRenew:自適應監控:2025-07-24T00:55:47+08:00]
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut running = self.is_running.lock().await;
            if *running {
                return Ok(()); // 已經在運行
            }
            *running = true;
        }

        println!("🚀 啟動自適應監控系統");
        
        self.send_event(MonitoringEventType::StatusUpdated, None, "監控系統啟動".to_string()).await;

        loop {
            // 檢查是否應該停止
            {
                let running = self.is_running.lock().await;
                if !*running {
                    break;
                }
            }

            // 執行監控檢查
            if let Err(e) = self.perform_check().await {
                eprintln!("監控檢查失敗: {}", e);
                self.send_event(
                    MonitoringEventType::Error,
                    None,
                    format!("監控檢查失敗: {}", e)
                ).await;
            }

            // 計算下次檢查間隔
            let sleep_duration = self.calculate_sleep_duration();
            
            // 更新統計
            self.check_count += 1;

            println!("🔍 監控模式: {:?} | 下次檢查: {} 後", 
                self.current_mode,
                self.format_duration(sleep_duration)
            );

            // 等待下次檢查
            sleep(sleep_duration).await;
        }

        println!("⏹️ 自適應監控系統已停止");
        Ok(())
    }

    /// 停止監控系統
    pub async fn stop(&self) {
        let mut running = self.is_running.lock().await;
        *running = false;
    }

    /// 執行單次監控檢查
    async fn perform_check(&mut self) -> Result<()> {
        self.last_check = Some(Utc::now());

        // 獲取使用量資訊
        let usage_info = {
            let mut tracker = self.usage_tracker.lock().await;
            tracker.get_usage_info().await?
        };

        // 判斷新的監控模式
        let new_mode = self.determine_monitoring_mode(&usage_info);

        // 檢查模式是否發生變化
        if new_mode != self.current_mode {
            let old_mode = self.current_mode.clone();
            self.current_mode = new_mode.clone();
            self.mode_changes += 1;

            println!("🔄 監控模式變更: {:?} → {:?}", old_mode, new_mode);
            
            self.send_event(
                MonitoringEventType::ModeChanged,
                Some(usage_info.clone()),
                format!("監控模式從 {:?} 變更為 {:?}", old_mode, new_mode)
            ).await;
        }

        // 檢查可用性變化
        if let Some(last_info) = &self.last_usage_info {
            if usage_info.is_available != last_info.is_available {
                self.send_event(
                    MonitoringEventType::AvailabilityChanged,
                    Some(usage_info.clone()),
                    format!(
                        "Claude可用性變更: {} → {}",
                        if last_info.is_available { "可用" } else { "不可用" },
                        if usage_info.is_available { "可用" } else { "不可用" }
                    )
                ).await;
            }
        }

        // 更新狀態
        self.last_usage_info = Some(usage_info.clone());

        // 發送狀態更新事件
        self.send_event(
            MonitoringEventType::StatusUpdated,
            Some(usage_info),
            "狀態更新".to_string()
        ).await;

        Ok(())
    }

    /// 根據使用量資訊決定監控模式
    /// [最佳實踐:ClaudeNightsWatch:監控策略:2025-07-24T00:55:47+08:00]
    fn determine_monitoring_mode(&self, usage_info: &UsageInfo) -> MonitoringMode {
        // 如果無法確定狀態
        if usage_info.source == "fallback-unknown" {
            return MonitoringMode::Unknown;
        }

        // 檢查是否可用
        if !usage_info.is_available {
            return MonitoringMode::Unavailable;
        }

        // 根據剩餘時間決定模式
        if let Some(block) = &usage_info.current_block {
            let remaining_minutes = block.remaining_minutes;

            if remaining_minutes == 0 {
                MonitoringMode::Unavailable
            } else if remaining_minutes < self.config.critical_threshold_minutes as u32 {
                MonitoringMode::Critical
            } else if remaining_minutes < self.config.imminent_threshold_minutes as u32 {
                MonitoringMode::Imminent
            } else if remaining_minutes < self.config.approaching_threshold_minutes as u32 {
                MonitoringMode::Approaching
            } else {
                MonitoringMode::Normal
            }
        } else {
            MonitoringMode::Unknown
        }
    }

    /// 計算睡眠持續時間
    fn calculate_sleep_duration(&self) -> TokioDuration {
        match self.current_mode {
            MonitoringMode::Normal => {
                TokioDuration::from_secs(self.config.normal_interval_minutes * 60)
            }
            MonitoringMode::Approaching => {
                TokioDuration::from_secs(self.config.approaching_interval_minutes * 60)
            }
            MonitoringMode::Imminent => {
                TokioDuration::from_secs(self.config.imminent_interval_seconds)
            }
            MonitoringMode::Critical => {
                TokioDuration::from_secs(self.config.critical_interval_seconds)
            }
            MonitoringMode::Unavailable => {
                TokioDuration::from_secs(self.config.unavailable_interval_minutes * 60)
            }
            MonitoringMode::Unknown => {
                TokioDuration::from_secs(self.config.unknown_interval_minutes * 60)
            }
        }
    }

    /// 發送監控事件
    async fn send_event(&self, event_type: MonitoringEventType, usage_info: Option<UsageInfo>, message: String) {
        let event = MonitoringEvent {
            timestamp: Utc::now(),
            event_type,
            mode: self.current_mode.clone(),
            usage_info,
            message,
        };

        if self.event_sender.send(event).is_err() {
            // 沒有接收者，忽略錯誤
        }
    }

    /// 獲取當前監控狀態
    pub fn get_status(&self) -> MonitoringStatus {
        let next_check_duration = self.calculate_sleep_duration();
        let next_check_at = self.last_check
            .unwrap_or_else(Utc::now)
            .checked_add_signed(Duration::seconds(next_check_duration.as_secs() as i64))
            .unwrap_or_else(Utc::now);

        MonitoringStatus {
            current_mode: self.current_mode.clone(),
            next_check_at,
            last_usage_info: self.last_usage_info.clone(),
            check_count: self.check_count,
            mode_changes: self.mode_changes,
            uptime_seconds: (Utc::now() - self.start_time).num_seconds() as u64,
        }
    }

    /// 更新監控配置
    pub async fn update_config(&mut self, new_config: MonitoringConfig) {
        self.config = new_config;
        self.send_event(
            MonitoringEventType::ConfigUpdated,
            None,
            "監控配置已更新".to_string()
        ).await;
    }

    /// 手動觸發檢查
    pub async fn trigger_check(&mut self) -> Result<()> {
        self.perform_check().await
    }

    /// 格式化持續時間為可讀字符串
    fn format_duration(&self, duration: TokioDuration) -> String {
        let total_seconds = duration.as_secs();
        
        if total_seconds >= 60 {
            let minutes = total_seconds / 60;
            if minutes >= 60 {
                let hours = minutes / 60;
                let remaining_minutes = minutes % 60;
                if remaining_minutes > 0 {
                    format!("{}小時{}分鐘", hours, remaining_minutes)
                } else {
                    format!("{}小時", hours)
                }
            } else {
                format!("{}分鐘", minutes)
            }
        } else {
            format!("{}秒", total_seconds)
        }
    }

    /// 獲取監控統計資訊
    pub fn get_statistics(&self) -> serde_json::Value {
        serde_json::json!({
            "check_count": self.check_count,
            "mode_changes": self.mode_changes,
            "uptime_seconds": (Utc::now() - self.start_time).num_seconds(),
            "current_mode": self.current_mode,
            "last_check": self.last_check,
            "config": self.config
        })
    }
}

// Tauri命令介面 [最佳實踐:tauri-docs:2025-07-24T00:55:47+08:00]
#[tauri::command]
pub async fn get_monitoring_status(
    state: tauri::State<'_, Arc<Mutex<AdaptiveMonitor>>>
) -> Result<MonitoringStatus, String> {
    let monitor = state.lock().await;
    Ok(monitor.get_status())
}

#[tauri::command]
pub async fn trigger_monitoring_check(
    state: tauri::State<'_, Arc<Mutex<AdaptiveMonitor>>>
) -> Result<(), String> {
    let mut monitor = state.lock().await;
    monitor.trigger_check().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_monitoring_config(
    config: MonitoringConfig,
    state: tauri::State<'_, Arc<Mutex<AdaptiveMonitor>>>
) -> Result<(), String> {
    let mut monitor = state.lock().await;
    monitor.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn get_monitoring_statistics(
    state: tauri::State<'_, Arc<Mutex<AdaptiveMonitor>>>
) -> Result<serde_json::Value, String> {
    let monitor = state.lock().await;
    Ok(monitor.get_statistics())
}

// 監控系統管理器 - 用於整合到主應用程式
pub struct MonitoringManager {
    monitor: Arc<Mutex<AdaptiveMonitor>>,
    event_receiver: Arc<Mutex<broadcast::Receiver<MonitoringEvent>>>,
}

impl MonitoringManager {
    pub fn new(usage_tracker: Arc<Mutex<UsageTracker>>) -> Self {
        let (monitor, receiver) = AdaptiveMonitor::new(usage_tracker, None);
        
        Self {
            monitor: Arc::new(Mutex::new(monitor)),
            event_receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let monitor = Arc::clone(&self.monitor);
        
        tokio::spawn(async move {
            let mut mon = monitor.lock().await;
            if let Err(e) = mon.start().await {
                eprintln!("監控系統執行錯誤: {}", e);
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let monitor = self.monitor.lock().await;
        monitor.stop().await;
    }

    pub fn get_monitor(&self) -> Arc<Mutex<AdaptiveMonitor>> {
        Arc::clone(&self.monitor)
    }

    /// 訂閱監控事件
    pub async fn subscribe_events(&self) -> broadcast::Receiver<MonitoringEvent> {
        let receiver = self.event_receiver.lock().await;
        receiver.resubscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_tracker::{UsageBlock, UsageInfo};

    fn create_test_usage_info(remaining_minutes: u32) -> UsageInfo {
        UsageInfo {
            current_block: Some(UsageBlock {
                remaining_minutes,
                total_minutes: 300,
                reset_time: None,
                usage_percentage: 0.5,
            }),
            next_block_starts: None,
            is_available: remaining_minutes > 0,
            source: "test".to_string(),
            last_updated: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_monitoring_mode_determination() {
        let config = MonitoringConfig::default();
        let usage_tracker = Arc::new(Mutex::new(
            crate::usage_tracker::UsageTracker::new(
                Arc::new(crate::db::Database::new_mock().await.unwrap())
            )
        ));
        
        let (monitor, _) = AdaptiveMonitor::new(usage_tracker, Some(config));

        // 測試不同剩餘時間的模式判斷
        assert_eq!(
            monitor.determine_monitoring_mode(&create_test_usage_info(60)),
            MonitoringMode::Normal
        );
        
        assert_eq!(
            monitor.determine_monitoring_mode(&create_test_usage_info(20)),
            MonitoringMode::Approaching
        );
        
        assert_eq!(
            monitor.determine_monitoring_mode(&create_test_usage_info(3)),
            MonitoringMode::Imminent
        );
        
        assert_eq!(
            monitor.determine_monitoring_mode(&create_test_usage_info(0)),
            MonitoringMode::Unavailable
        );
    }

    #[tokio::test]
    async fn test_sleep_duration_calculation() {
        let config = MonitoringConfig::default();
        let usage_tracker = Arc::new(Mutex::new(
            crate::usage_tracker::UsageTracker::new(
                Arc::new(crate::db::Database::new_mock().await.unwrap())
            )
        ));
        
        let (mut monitor, _) = AdaptiveMonitor::new(usage_tracker, Some(config));

        monitor.current_mode = MonitoringMode::Normal;
        assert_eq!(monitor.calculate_sleep_duration(), TokioDuration::from_secs(600)); // 10分鐘

        monitor.current_mode = MonitoringMode::Approaching;
        assert_eq!(monitor.calculate_sleep_duration(), TokioDuration::from_secs(120)); // 2分鐘

        monitor.current_mode = MonitoringMode::Imminent;
        assert_eq!(monitor.calculate_sleep_duration(), TokioDuration::from_secs(30)); // 30秒
    }

    #[tokio::test]
    async fn test_duration_formatting() {
        let config = MonitoringConfig::default();
        let usage_tracker = Arc::new(Mutex::new(
            crate::usage_tracker::UsageTracker::new(
                Arc::new(crate::db::Database::new_mock().await.unwrap())
            )
        ));
        
        let (monitor, _) = AdaptiveMonitor::new(usage_tracker, Some(config));

        assert_eq!(monitor.format_duration(TokioDuration::from_secs(30)), "30秒");
        assert_eq!(monitor.format_duration(TokioDuration::from_secs(120)), "2分鐘");
        assert_eq!(monitor.format_duration(TokioDuration::from_secs(3600)), "1小時");
        assert_eq!(monitor.format_duration(TokioDuration::from_secs(3720)), "1小時2分鐘");
    }

    #[tokio::test]
    async fn test_monitoring_config_update() {
        let usage_tracker = Arc::new(Mutex::new(
            crate::usage_tracker::UsageTracker::new(
                Arc::new(crate::db::Database::new_mock().await.unwrap())
            )
        ));
        
        let (mut monitor, _) = AdaptiveMonitor::new(usage_tracker, None);

        let new_config = MonitoringConfig {
            normal_interval_minutes: 15,
            ..Default::default()
        };

        monitor.update_config(new_config.clone()).await;
        assert_eq!(monitor.config.normal_interval_minutes, 15);
    }
} 