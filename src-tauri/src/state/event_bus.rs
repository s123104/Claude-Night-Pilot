// 事件匯流排系統 - GUI和CLI事件協調
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: String,
    pub event_type: String,
    pub source: String,         // "gui", "cli", "system"
    pub target: Option<String>, // 目標接收者
    pub payload: serde_json::Value,
    pub timestamp: String,
    pub priority: EventPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

pub struct EventHandler {
    pub id: String,
    pub event_types: Vec<String>,
    pub handler_func: Arc<dyn Fn(SystemEvent) -> Result<()> + Send + Sync>,
}

impl std::fmt::Debug for EventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandler")
            .field("id", &self.id)
            .field("event_types", &self.event_types)
            .field("handler_func", &"<function>")
            .finish()
    }
}

impl Clone for EventHandler {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            event_types: self.event_types.clone(),
            handler_func: Arc::clone(&self.handler_func),
        }
    }
}

pub struct EventBus {
    event_sender: broadcast::Sender<SystemEvent>,
    handlers: Arc<RwLock<HashMap<String, EventHandler>>>,
    event_history: Arc<RwLock<Vec<SystemEvent>>>,
    max_history_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(10000);

        Self {
            event_sender,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    /// 發布事件
    pub async fn publish(&self, event: SystemEvent) -> Result<()> {
        // 記錄到歷史
        self.add_to_history(event.clone()).await?;

        // 廣播事件
        if let Err(_) = self.event_sender.send(event.clone()) {
            tracing::warn!("發布事件失敗: 沒有訂閱者 - {}", event.id);
        }

        // 處理特定處理器
        self.handle_event(event).await?;

        Ok(())
    }

    /// 創建事件訂閱者
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_sender.subscribe()
    }

    /// 註冊事件處理器
    pub async fn register_handler(&self, handler: EventHandler) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(handler.id.clone(), handler);
        Ok(())
    }

    /// 取消註冊事件處理器
    pub async fn unregister_handler(&self, handler_id: &str) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        handlers.remove(handler_id);
        Ok(())
    }

    /// 處理事件（調用相關處理器）
    async fn handle_event(&self, event: SystemEvent) -> Result<()> {
        let handlers = self.handlers.read().await;

        for handler in handlers.values() {
            if handler.event_types.contains(&event.event_type)
                || handler.event_types.contains(&"*".to_string())
            {
                if let Err(e) = (handler.handler_func)(event.clone()) {
                    tracing::error!(
                        "事件處理器 {} 處理事件 {} 失敗: {}",
                        handler.id,
                        event.id,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// 添加到歷史記錄
    async fn add_to_history(&self, event: SystemEvent) -> Result<()> {
        let mut history = self.event_history.write().await;
        history.push(event);

        // 保持歷史大小限制
        let len = history.len();
        if len > self.max_history_size {
            history.drain(0..len - self.max_history_size);
        }

        Ok(())
    }

    /// 獲取事件歷史
    pub async fn get_event_history(&self, limit: Option<usize>) -> Result<Vec<SystemEvent>> {
        let history = self.event_history.read().await;
        let limit = limit.unwrap_or(100);

        Ok(history.iter().rev().take(limit).cloned().collect())
    }

    /// 清理歷史記錄
    pub async fn clear_history(&self) -> Result<()> {
        let mut history = self.event_history.write().await;
        history.clear();
        Ok(())
    }

    /// 創建系統事件的便利方法
    pub fn create_event(
        event_type: &str,
        source: &str,
        payload: serde_json::Value,
        priority: EventPriority,
    ) -> SystemEvent {
        SystemEvent {
            id: format!("{}_{}", event_type, chrono::Utc::now().timestamp_millis()),
            event_type: event_type.to_string(),
            source: source.to_string(),
            target: None,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
            priority,
        }
    }

    /// 創建GUI到CLI的事件
    pub fn create_gui_to_cli_event(event_type: &str, payload: serde_json::Value) -> SystemEvent {
        let mut event = Self::create_event(event_type, "gui", payload, EventPriority::Normal);
        event.target = Some("cli".to_string());
        event
    }

    /// 創建CLI到GUI的事件
    pub fn create_cli_to_gui_event(event_type: &str, payload: serde_json::Value) -> SystemEvent {
        let mut event = Self::create_event(event_type, "cli", payload, EventPriority::Normal);
        event.target = Some("gui".to_string());
        event
    }

    /// 發布Prompt變更事件
    pub async fn publish_prompt_change(
        &self,
        source: &str,
        action: &str,
        prompt_id: i64,
        data: serde_json::Value,
    ) -> Result<()> {
        let event = Self::create_event(
            "prompt_changed",
            source,
            serde_json::json!({
                "action": action,
                "prompt_id": prompt_id,
                "data": data
            }),
            EventPriority::Normal,
        );

        self.publish(event).await
    }

    /// 發布Job變更事件
    pub async fn publish_job_change(
        &self,
        source: &str,
        action: &str,
        job_id: i64,
        data: serde_json::Value,
    ) -> Result<()> {
        let event = Self::create_event(
            "job_changed",
            source,
            serde_json::json!({
                "action": action,
                "job_id": job_id,
                "data": data
            }),
            EventPriority::Normal,
        );

        self.publish(event).await
    }

    /// 發布系統狀態事件
    pub async fn publish_system_status(
        &self,
        source: &str,
        status: serde_json::Value,
    ) -> Result<()> {
        let event = Self::create_event("system_status_update", source, status, EventPriority::High);

        self.publish(event).await
    }

    /// 發布同步事件
    pub async fn publish_sync_event(
        &self,
        sync_type: &str,
        source: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        let event = Self::create_event(
            "sync_event",
            source,
            serde_json::json!({
                "sync_type": sync_type,
                "data": data
            }),
            EventPriority::High,
        );

        self.publish(event).await
    }

    /// 獲取事件統計
    pub async fn get_event_statistics(&self) -> Result<serde_json::Value> {
        let history = self.event_history.read().await;
        let handlers = self.handlers.read().await;

        let mut event_type_counts = HashMap::new();
        let mut source_counts = HashMap::new();

        for event in history.iter() {
            *event_type_counts
                .entry(event.event_type.clone())
                .or_insert(0) += 1;
            *source_counts.entry(event.source.clone()).or_insert(0) += 1;
        }

        Ok(serde_json::json!({
            "total_events": history.len(),
            "registered_handlers": handlers.len(),
            "event_types": event_type_counts,
            "sources": source_counts,
            "max_history_size": self.max_history_size,
        }))
    }
}

// 單例模式的全局事件匯流排
use std::sync::OnceLock;

static EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

impl EventBus {
    pub fn global() -> &'static EventBus {
        EVENT_BUS.get_or_init(|| EventBus::new())
    }
}

// 預定義的事件處理器工廠
pub struct StandardEventHandlers;

impl StandardEventHandlers {
    /// 創建GUI同步處理器
    pub fn create_gui_sync_handler() -> EventHandler {
        EventHandler {
            id: "gui_sync_handler".to_string(),
            event_types: vec!["prompt_changed".to_string(), "job_changed".to_string()],
            handler_func: Arc::new(|event: SystemEvent| {
                tracing::info!("GUI同步處理器收到事件: {}", event.event_type);
                // 實際的GUI同步邏輯將在這裡實現
                Ok(())
            }),
        }
    }

    /// 創建CLI同步處理器
    pub fn create_cli_sync_handler() -> EventHandler {
        EventHandler {
            id: "cli_sync_handler".to_string(),
            event_types: vec!["system_status_update".to_string(), "sync_event".to_string()],
            handler_func: Arc::new(|event: SystemEvent| {
                tracing::info!("CLI同步處理器收到事件: {}", event.event_type);
                // 實際的CLI同步邏輯將在這裡實現
                Ok(())
            }),
        }
    }

    /// 創建日志記錄處理器
    pub fn create_logging_handler() -> EventHandler {
        EventHandler {
            id: "logging_handler".to_string(),
            event_types: vec!["*".to_string()], // 監聽所有事件
            handler_func: Arc::new(|event: SystemEvent| {
                match event.priority {
                    EventPriority::Critical => {
                        tracing::error!("關鍵事件: {} from {}", event.event_type, event.source)
                    }
                    EventPriority::High => {
                        tracing::warn!("高優先級事件: {} from {}", event.event_type, event.source)
                    }
                    EventPriority::Normal => {
                        tracing::info!("事件: {} from {}", event.event_type, event.source)
                    }
                    EventPriority::Low => {
                        tracing::debug!("低優先級事件: {} from {}", event.event_type, event.source)
                    }
                }
                Ok(())
            }),
        }
    }
}
