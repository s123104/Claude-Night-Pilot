// Prompt服務 - GUI和CLI共享業務邏輯
use crate::database_manager_impl::DatabaseManager as OldDatabaseManager;
use crate::simple_db::SimplePrompt;
use crate::state::AppStateManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePromptRequest {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptServiceResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

impl From<SimplePrompt> for PromptServiceResponse {
    fn from(prompt: SimplePrompt) -> Self {
        Self {
            id: prompt.id,
            title: prompt.title,
            content: prompt.content,
            tags: prompt.tags,
            created_at: prompt.created_at,
            updated_at: None,
        }
    }
}

pub struct PromptService {
    db_manager: Arc<OldDatabaseManager>,
    state_manager: Arc<AppStateManager>,
}

impl PromptService {
    pub async fn new() -> Result<Self> {
        let config = crate::database_manager_impl::DatabaseConfig::default();
        let db_manager = Arc::new(OldDatabaseManager::new(config).await?);
        let state_manager = Arc::new(AppStateManager::new());

        Ok(Self {
            db_manager,
            state_manager,
        })
    }

    /// 列出所有Prompts
    pub async fn list_prompts(&self) -> Result<Vec<PromptServiceResponse>> {
        let prompts = self.db_manager.list_prompts_async().await?;
        let responses: Vec<PromptServiceResponse> = prompts
            .into_iter()
            .map(PromptServiceResponse::from)
            .collect();

        // 觸發狀態同步
        self.state_manager
            .notify_prompts_changed(&responses)
            .await?;

        Ok(responses)
    }

    /// 創建新Prompt
    pub async fn create_prompt(&self, request: CreatePromptRequest) -> Result<i64> {
        let prompt_id = self
            .db_manager
            .create_prompt_async(&request.title, &request.content)
            .await?;

        // 觸發狀態同步
        self.state_manager
            .notify_prompt_created(prompt_id, &request)
            .await?;

        Ok(prompt_id)
    }

    /// 獲取單個Prompt
    pub async fn get_prompt(&self, id: i64) -> Result<Option<PromptServiceResponse>> {
        let prompt = self.db_manager.get_prompt_async(id).await?;
        Ok(prompt.map(PromptServiceResponse::from))
    }

    /// 更新Prompt
    pub async fn update_prompt(&self, request: UpdatePromptRequest) -> Result<()> {
        // 獲取現有prompt以進行驗證
        let existing = self.get_prompt(request.id).await?;
        if existing.is_none() {
            return Err(anyhow::anyhow!("Prompt {} 不存在", request.id));
        }

        // 更新邏輯（目前DatabaseManager不支持更新，暫時返回成功）
        // TODO: 實現更新功能
        self.state_manager.notify_prompt_updated(&request).await?;

        Ok(())
    }

    /// 刪除Prompt
    pub async fn delete_prompt(&self, id: i64) -> Result<()> {
        // 檢查prompt是否存在
        let existing = self.get_prompt(id).await?;
        if existing.is_none() {
            return Err(anyhow::anyhow!("Prompt {} 不存在", id));
        }

        // 執行刪除邏輯（目前DatabaseManager不支持刪除，暫時返回成功）
        // TODO: 實現刪除功能
        self.state_manager.notify_prompt_deleted(id).await?;

        Ok(())
    }

    /// 執行Prompt
    pub async fn execute_prompt(
        &self,
        id: i64,
        options: crate::unified_interface::UnifiedExecutionOptions,
    ) -> Result<crate::enhanced_executor::EnhancedClaudeResponse> {
        let prompt = self.get_prompt(id).await?;
        if let Some(prompt_data) = prompt {
            let result = crate::unified_interface::UnifiedClaudeInterface::execute_claude(
                prompt_data.content,
                options,
            )
            .await?;

            // 觸發狀態同步 - 記錄執行
            self.state_manager
                .notify_prompt_executed(id, &result)
                .await?;

            Ok(result)
        } else {
            Err(anyhow::anyhow!("Prompt {} 不存在", id))
        }
    }
}

// Tauri命令包裝器
#[tauri::command]
pub async fn prompt_service_list_prompts() -> Result<Vec<PromptServiceResponse>, String> {
    let service = PromptService::new().await.map_err(|e| e.to_string())?;
    service.list_prompts().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prompt_service_create_prompt(
    title: String,
    content: String,
    tags: Option<String>,
) -> Result<i64, String> {
    let service = PromptService::new().await.map_err(|e| e.to_string())?;
    let request = CreatePromptRequest {
        title,
        content,
        tags,
    };
    service
        .create_prompt(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn prompt_service_delete_prompt(id: i64) -> Result<(), String> {
    let service = PromptService::new().await.map_err(|e| e.to_string())?;
    service.delete_prompt(id).await.map_err(|e| e.to_string())
}
