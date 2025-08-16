// 應用程式狀態管理器 - 集中管理GUI和CLI狀態
use crate::services::{job_service::*, prompt_service::*};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub prompts: HashMap<i64, PromptServiceResponse>,
    pub jobs: HashMap<i64, JobServiceResponse>,
    pub sync_events: Vec<crate::services::sync_service::SyncEvent>,
    pub pending_changes: u32,
    pub last_sync: String,
    pub health_status: Option<crate::services::health_service::HealthStatus>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            prompts: HashMap::new(),
            jobs: HashMap::new(),
            sync_events: Vec::new(),
            pending_changes: 0,
            last_sync: chrono::Utc::now().to_rfc3339(),
            health_status: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateChangeEvent {
    PromptsChanged(Vec<PromptServiceResponse>),
    PromptCreated(i64, CreatePromptRequest),
    PromptUpdated(UpdatePromptRequest),
    PromptDeleted(i64),
    PromptExecuted(i64, crate::enhanced_executor::EnhancedClaudeResponse),

    JobsChanged(Vec<JobServiceResponse>),
    JobCreated(i64, CreateJobRequest),
    JobUpdated(UpdateJobRequest),
    JobDeleted(i64),
    JobExecuted(i64, crate::enhanced_executor::EnhancedClaudeResponse),

    HealthCheckCompleted(crate::services::health_service::HealthStatus),
    SyncEventRecorded(crate::services::sync_service::SyncEvent),
}

#[derive(Debug)]
pub struct AppStateManager {
    state: Arc<RwLock<AppState>>,
    event_sender: broadcast::Sender<StateChangeEvent>,
    _event_receiver: broadcast::Receiver<StateChangeEvent>,
}

impl Default for AppStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStateManager {
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);

        Self {
            state: Arc::new(RwLock::new(AppState::default())),
            event_sender,
            _event_receiver: event_receiver,
        }
    }

    /// 獲取當前狀態的只讀副本
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }

    /// 創建狀態變更事件訂閱者
    pub fn subscribe_to_changes(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.event_sender.subscribe()
    }

    /// 廣播狀態變更事件
    async fn broadcast_event(&self, event: StateChangeEvent) -> Result<()> {
        if self.event_sender.send(event).is_err() {
            tracing::warn!("廣播狀態變更事件失敗: 沒有訂閱者");
        }
        Ok(())
    }

    /// 通知Prompts變更
    pub async fn notify_prompts_changed(&self, prompts: &[PromptServiceResponse]) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.prompts.clear();
            for prompt in prompts {
                state.prompts.insert(prompt.id, prompt.clone());
            }
            state.pending_changes += 1;
            state.last_sync = chrono::Utc::now().to_rfc3339();
        }

        self.broadcast_event(StateChangeEvent::PromptsChanged(prompts.to_vec()))
            .await
    }

    /// 通知Prompt創建
    pub async fn notify_prompt_created(
        &self,
        id: i64,
        request: &CreatePromptRequest,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::PromptCreated(id, request.clone()))
            .await
    }

    /// 通知Prompt更新
    pub async fn notify_prompt_updated(&self, request: &UpdatePromptRequest) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::PromptUpdated(request.clone()))
            .await
    }

    /// 通知Prompt刪除
    pub async fn notify_prompt_deleted(&self, id: i64) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.prompts.remove(&id);
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::PromptDeleted(id))
            .await
    }

    /// 通知Prompt執行
    pub async fn notify_prompt_executed(
        &self,
        id: i64,
        response: &crate::enhanced_executor::EnhancedClaudeResponse,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::PromptExecuted(id, response.clone()))
            .await
    }

    /// 通知Jobs變更
    pub async fn notify_jobs_changed(&self, jobs: &[JobServiceResponse]) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.jobs.clear();
            for job in jobs {
                state.jobs.insert(job.id, job.clone());
            }
            state.pending_changes += 1;
            state.last_sync = chrono::Utc::now().to_rfc3339();
        }

        self.broadcast_event(StateChangeEvent::JobsChanged(jobs.to_vec()))
            .await
    }

    /// 通知Job創建
    pub async fn notify_job_created(&self, id: i64, request: &CreateJobRequest) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::JobCreated(id, request.clone()))
            .await
    }

    /// 通知Job更新
    pub async fn notify_job_updated(&self, request: &UpdateJobRequest) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::JobUpdated(request.clone()))
            .await
    }

    /// 通知Job刪除
    pub async fn notify_job_deleted(&self, id: i64) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.jobs.remove(&id);
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::JobDeleted(id)).await
    }

    /// 通知Job執行
    pub async fn notify_job_executed(
        &self,
        id: i64,
        response: &crate::enhanced_executor::EnhancedClaudeResponse,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pending_changes += 1;
        }

        self.broadcast_event(StateChangeEvent::JobExecuted(id, response.clone()))
            .await
    }

    /// 通知健康檢查完成
    pub async fn notify_health_check_completed(
        &self,
        health: &crate::services::health_service::HealthStatus,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.health_status = Some(health.clone());
            state.last_sync = chrono::Utc::now().to_rfc3339();
        }

        self.broadcast_event(StateChangeEvent::HealthCheckCompleted(health.clone()))
            .await
    }

    /// 記錄同步事件
    pub async fn record_sync_event(
        &self,
        event: crate::services::sync_service::SyncEvent,
    ) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.sync_events.push(event.clone());

            // 保持最近100個事件
            let len = state.sync_events.len();
            if len > 100 {
                state.sync_events.drain(0..len - 100);
            }
        }

        self.broadcast_event(StateChangeEvent::SyncEventRecorded(event))
            .await
    }

    /// 獲取待處理變更數量
    pub async fn get_pending_changes_count(&self) -> Result<u32> {
        let state = self.state.read().await;
        Ok(state.pending_changes)
    }

    /// 獲取同步衝突數量
    pub async fn get_sync_conflicts_count(&self) -> Result<u32> {
        // 簡化實現，實際應該檢查真實的衝突
        Ok(0)
    }

    /// 清理已處理的變更
    pub async fn clear_processed_changes(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.pending_changes = 0;
        state.last_sync = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    /// 解決衝突
    pub async fn resolve_conflict(&self, _conflict_id: u64, _resolution: &str) -> Result<()> {
        // 簡化實現，實際應該處理具體的衝突解決邏輯
        Ok(())
    }

    /// 獲取狀態統計信息
    pub async fn get_state_statistics(&self) -> Result<serde_json::Value> {
        let state = self.state.read().await;
        Ok(serde_json::json!({
            "prompts_count": state.prompts.len(),
            "jobs_count": state.jobs.len(),
            "sync_events_count": state.sync_events.len(),
            "pending_changes": state.pending_changes,
            "last_sync": state.last_sync,
            "has_health_status": state.health_status.is_some(),
        }))
    }
}

// 單例模式的全局狀態管理器
use std::sync::OnceLock;

static APP_STATE_MANAGER: OnceLock<AppStateManager> = OnceLock::new();

impl AppStateManager {
    pub fn global() -> &'static AppStateManager {
        APP_STATE_MANAGER.get_or_init(AppStateManager::new)
    }
}
