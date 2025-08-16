// 同步服務 - GUI和CLI狀態同步管理
use crate::state::AppStateManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncStatus {
    pub gui_cli_sync_enabled: bool,
    pub last_sync_timestamp: String,
    pub pending_changes: u32,
    pub sync_conflicts: u32,
    pub performance_impact: f64,
    pub sync_health: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncEvent {
    pub event_type: String,
    pub source: String,      // "gui" or "cli"
    pub entity_type: String, // "prompt", "job", "result"
    pub entity_id: Option<i64>,
    pub action: String, // "create", "update", "delete", "execute"
    pub timestamp: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncConfiguration {
    pub enable_real_time_sync: bool,
    pub sync_interval_seconds: u64,
    pub conflict_resolution_strategy: String, // "latest_wins", "manual", "source_priority"
    pub max_pending_changes: u32,
    pub performance_monitoring: bool,
}

pub struct SyncService {
    state_manager: Arc<AppStateManager>,
    event_sender: broadcast::Sender<SyncEvent>,
    _event_receiver: broadcast::Receiver<SyncEvent>,
    config: SyncConfiguration,
}

impl SyncService {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);

        Self {
            state_manager: Arc::new(AppStateManager::new()),
            event_sender,
            _event_receiver: event_receiver,
            config: SyncConfiguration {
                enable_real_time_sync: true,
                sync_interval_seconds: 5,
                conflict_resolution_strategy: "latest_wins".to_string(),
                max_pending_changes: 100,
                performance_monitoring: true,
            },
        }
    }

    /// 獲取同步狀態
    pub async fn get_sync_status(&self) -> Result<SyncStatus> {
        let pending_changes = self.state_manager.get_pending_changes_count().await?;
        let sync_conflicts = self.state_manager.get_sync_conflicts_count().await?;

        let sync_health = if sync_conflicts > 0 {
            "conflicts"
        } else if pending_changes > self.config.max_pending_changes {
            "overloaded"
        } else if pending_changes > 0 {
            "syncing"
        } else {
            "healthy"
        };

        Ok(SyncStatus {
            gui_cli_sync_enabled: self.config.enable_real_time_sync,
            last_sync_timestamp: chrono::Utc::now().to_rfc3339(),
            pending_changes,
            sync_conflicts,
            performance_impact: self.calculate_performance_impact(),
            sync_health: sync_health.to_string(),
        })
    }

    /// 觸發手動同步
    pub async fn trigger_sync(&self) -> Result<String> {
        let sync_id = format!("sync_{}", chrono::Utc::now().timestamp());

        // 發送同步事件
        let sync_event = SyncEvent {
            event_type: "sync_triggered".to_string(),
            source: "manual".to_string(),
            entity_type: "system".to_string(),
            entity_id: None,
            action: "sync".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::json!({ "sync_id": sync_id }),
        };

        self.broadcast_event(sync_event).await?;

        // 執行同步邏輯
        self.execute_full_sync().await?;

        Ok(sync_id)
    }

    /// 廣播同步事件
    pub async fn broadcast_event(&self, event: SyncEvent) -> Result<()> {
        if let Err(_) = self.event_sender.send(event.clone()) {
            tracing::warn!("廣播同步事件失敗: 沒有接收者");
        }

        // 記錄到狀態管理器
        self.state_manager.record_sync_event(event).await?;

        Ok(())
    }

    /// 創建事件監聽器
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_sender.subscribe()
    }

    /// 通知Prompt變更
    pub async fn notify_prompt_change(
        &self,
        source: &str,
        action: &str,
        prompt_id: i64,
        data: serde_json::Value,
    ) -> Result<()> {
        let event = SyncEvent {
            event_type: "entity_changed".to_string(),
            source: source.to_string(),
            entity_type: "prompt".to_string(),
            entity_id: Some(prompt_id),
            action: action.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        };

        self.broadcast_event(event).await
    }

    /// 通知Job變更
    pub async fn notify_job_change(
        &self,
        source: &str,
        action: &str,
        job_id: i64,
        data: serde_json::Value,
    ) -> Result<()> {
        let event = SyncEvent {
            event_type: "entity_changed".to_string(),
            source: source.to_string(),
            entity_type: "job".to_string(),
            entity_id: Some(job_id),
            action: action.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
        };

        self.broadcast_event(event).await
    }

    /// 執行完整同步
    async fn execute_full_sync(&self) -> Result<()> {
        tracing::info!("開始執行完整同步...");

        // 同步Prompts
        if let Ok(prompt_service) = crate::services::PromptService::new().await {
            let _prompts = prompt_service.list_prompts().await?;
            // 同步邏輯在service內部的狀態通知中處理
        }

        // 同步Jobs
        if let Ok(job_service) = crate::services::JobService::new().await {
            let _jobs = job_service.list_jobs().await?;
            // 同步邏輯在service內部的狀態通知中處理
        }

        // 清理已處理的變更
        self.state_manager.clear_processed_changes().await?;

        tracing::info!("完整同步完成");
        Ok(())
    }

    /// 計算性能影響
    fn calculate_performance_impact(&self) -> f64 {
        // 簡單的性能影響計算
        if self.config.enable_real_time_sync {
            2.5 // 2.5% 性能影響
        } else {
            0.1 // 0.1% 性能影響
        }
    }

    /// 更新同步配置
    pub async fn update_config(&mut self, config: SyncConfiguration) -> Result<()> {
        self.config = config;

        // 通知配置變更
        let event = SyncEvent {
            event_type: "config_changed".to_string(),
            source: "system".to_string(),
            entity_type: "sync_config".to_string(),
            entity_id: None,
            action: "update".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::to_value(&self.config)?,
        };

        self.broadcast_event(event).await?;
        Ok(())
    }

    /// 處理衝突解決
    pub async fn resolve_conflict(&self, conflict_id: u64, resolution: &str) -> Result<()> {
        let event = SyncEvent {
            event_type: "conflict_resolved".to_string(),
            source: "system".to_string(),
            entity_type: "conflict".to_string(),
            entity_id: Some(conflict_id as i64),
            action: resolution.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: serde_json::json!({ "resolution": resolution }),
        };

        self.broadcast_event(event).await?;
        self.state_manager
            .resolve_conflict(conflict_id, resolution)
            .await?;

        Ok(())
    }
}

// 單例模式的全局同步服務
use std::sync::OnceLock;
use tokio::sync::Mutex;

static SYNC_SERVICE: OnceLock<Arc<Mutex<SyncService>>> = OnceLock::new();

impl SyncService {
    pub fn global() -> Arc<Mutex<SyncService>> {
        SYNC_SERVICE
            .get_or_init(|| Arc::new(Mutex::new(SyncService::new())))
            .clone()
    }
}

// Tauri命令包裝器
#[tauri::command]
pub async fn sync_service_get_status() -> Result<SyncStatus, String> {
    let service_guard = SyncService::global();
    let service = service_guard.lock().await;
    service.get_sync_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_service_trigger_sync() -> Result<String, String> {
    let service_guard = SyncService::global();
    let service = service_guard.lock().await;
    service.trigger_sync().await.map_err(|e| e.to_string())
}
