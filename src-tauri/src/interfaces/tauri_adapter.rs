// Tauri GUI適配器 - GUI介面的統一入口
use crate::interfaces::shared_types::*;
use crate::services::health_service::HealthService;
use crate::services::{JobService, PromptService, SyncService};
use crate::state::AppStateManager;
use anyhow::Result;
use std::sync::Arc;

pub struct TauriAdapter {
    prompt_service: Arc<PromptService>,
    job_service: Arc<JobService>,
    sync_service: Arc<SyncService>,
    health_service: Arc<HealthService>,
    state_manager: Arc<AppStateManager>,
}

impl TauriAdapter {
    pub async fn new() -> Result<Self> {
        let prompt_service = Arc::new(PromptService::new().await?);
        let job_service = Arc::new(JobService::new().await?);
        let sync_service = Arc::new(SyncService::new());
        let health_service = Arc::new(HealthService::new());
        let state_manager = Arc::new(AppStateManager::new());

        Ok(Self {
            prompt_service,
            job_service,
            sync_service,
            health_service,
            state_manager,
        })
    }

    /// GUI專用的Prompt操作 - 包含UI狀態管理
    pub async fn gui_list_prompts(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<
        ApiResponse<PaginatedResponse<crate::services::prompt_service::PromptServiceResponse>>,
    > {
        let context = self.create_gui_context(OperationType::List);

        match self.prompt_service.list_prompts().await {
            Ok(prompts) => {
                // 應用分頁邏輯
                let (items, total) = self.apply_pagination(prompts, pagination.clone());

                let page = pagination.as_ref().map(|p| p.page).unwrap_or(1);
                let page_size = pagination.as_ref().map(|p| p.page_size).unwrap_or(50);
                let total_pages = (total as f64 / page_size as f64).ceil() as u32;

                let paginated = PaginatedResponse {
                    items,
                    total: total as u64,
                    page,
                    page_size,
                    total_pages,
                };

                Ok(ApiResponse::success(paginated, context))
            }
            Err(e) => {
                let error = UnifiedError {
                    code: "PROMPT_LIST_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    pub async fn gui_create_prompt(
        &self,
        request: crate::services::prompt_service::CreatePromptRequest,
    ) -> Result<ApiResponse<i64>> {
        let context = self.create_gui_context(OperationType::Create);

        match self.prompt_service.create_prompt(request.clone()).await {
            Ok(id) => {
                // 觸發UI狀態更新
                self.notify_ui_state_change(
                    "prompt_created",
                    serde_json::json!({
                        "id": id,
                        "request": request
                    }),
                )
                .await?;

                Ok(ApiResponse::success(id, context))
            }
            Err(e) => {
                let error = UnifiedError {
                    code: "PROMPT_CREATE_ERROR".to_string(),
                    message: e.to_string(),
                    details: Some(serde_json::to_value(&request)?),
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    /// GUI專用的Job操作
    pub async fn gui_list_jobs(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<ApiResponse<PaginatedResponse<crate::services::job_service::JobServiceResponse>>>
    {
        let context = self.create_gui_context(OperationType::List);

        match self.job_service.list_jobs().await {
            Ok(jobs) => {
                let (items, total) = self.apply_pagination(jobs, pagination.clone());

                let page = pagination.as_ref().map(|p| p.page).unwrap_or(1);
                let page_size = pagination.as_ref().map(|p| p.page_size).unwrap_or(50);
                let total_pages = (total as f64 / page_size as f64).ceil() as u32;

                let paginated = PaginatedResponse {
                    items,
                    total: total as u64,
                    page,
                    page_size,
                    total_pages,
                };

                Ok(ApiResponse::success(paginated, context))
            }
            Err(e) => {
                let error = UnifiedError {
                    code: "JOB_LIST_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    /// GUI專用的同步操作
    pub async fn gui_get_sync_status(
        &self,
    ) -> Result<ApiResponse<crate::services::sync_service::SyncStatus>> {
        let context = self.create_gui_context(OperationType::Read);

        match self.sync_service.get_sync_status().await {
            Ok(status) => Ok(ApiResponse::success(status, context)),
            Err(e) => {
                let error = UnifiedError {
                    code: "SYNC_STATUS_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    pub async fn gui_trigger_sync(&self) -> Result<ApiResponse<String>> {
        let context = self.create_gui_context(OperationType::Execute);

        match self.sync_service.trigger_sync().await {
            Ok(sync_id) => {
                // 通知GUI刷新所有數據
                self.notify_ui_refresh_all().await?;
                Ok(ApiResponse::success(sync_id, context))
            }
            Err(e) => {
                let error = UnifiedError {
                    code: "SYNC_TRIGGER_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    /// GUI專用的健康檢查
    pub async fn gui_health_check(
        &self,
    ) -> Result<ApiResponse<crate::services::health_service::HealthStatus>> {
        let context = self.create_gui_context(OperationType::Read);

        match self.health_service.comprehensive_health_check().await {
            Ok(health) => Ok(ApiResponse::success(health, context)),
            Err(e) => {
                let error = UnifiedError {
                    code: "HEALTH_CHECK_ERROR".to_string(),
                    message: e.to_string(),
                    details: None,
                    source: "gui".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                Ok(ApiResponse::error(error, context))
            }
        }
    }

    /// 工具方法 - 創建GUI執行上下文
    fn create_gui_context(&self, operation: OperationType) -> ExecutionContext {
        ExecutionContext {
            source: "gui".to_string(),
            user_id: None,
            session_id: format!("gui_session_{}", chrono::Utc::now().timestamp()),
            operation,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 應用分頁邏輯
    fn apply_pagination<T: Clone>(
        &self,
        items: Vec<T>,
        pagination: Option<PaginationRequest>,
    ) -> (Vec<T>, usize) {
        let total = items.len();

        if let Some(page_req) = pagination {
            let page = page_req.page.max(1);
            let page_size = page_req.page_size.min(1000).max(1); // 限制頁面大小
            let start = ((page - 1) * page_size) as usize;
            let end = (start + page_size as usize).min(total);

            if start < total {
                (items[start..end].to_vec(), total)
            } else {
                (Vec::new(), total)
            }
        } else {
            (items, total)
        }
    }

    /// 通知UI狀態變更
    async fn notify_ui_state_change(
        &self,
        change_type: &str,
        data: serde_json::Value,
    ) -> Result<()> {
        // 發送狀態變更事件
        // 實際實現中可以通過WebSocket或其他方式通知前端
        tracing::info!("GUI狀態變更: {} - {:?}", change_type, data);
        Ok(())
    }

    /// 通知GUI刷新所有數據
    async fn notify_ui_refresh_all(&self) -> Result<()> {
        self.notify_ui_state_change(
            "refresh_all",
            serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        )
        .await
    }

    /// 獲取適配器統計信息
    pub async fn get_adapter_statistics(&self) -> Result<serde_json::Value> {
        let state_stats = self.state_manager.get_state_statistics().await?;

        Ok(serde_json::json!({
            "adapter_type": "gui",
            "services_initialized": true,
            "state_manager": state_stats,
            "last_activity": chrono::Utc::now().to_rfc3339(),
        }))
    }
}
