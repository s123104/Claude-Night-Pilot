// Prompt 路由處理器
// 負責處理提示詞相關的所有 API 請求

use crate::models::{ApiResponse, PaginatedResponse, Prompt};
use crate::services::ServiceContainer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 創建提示詞請求
#[derive(Debug, Deserialize)]
pub struct CreatePromptRequest {
    pub title: String,
    pub content: String,
    pub tags: Option<String>,
    pub variables: Option<Vec<PromptVariable>>,
}

/// 更新提示詞請求
#[derive(Debug, Deserialize)]
pub struct UpdatePromptRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub variables: Option<Vec<PromptVariable>>,
}

/// 提示詞變數（簡化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    pub name: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
}

/// 列出提示詞
pub async fn list_prompts(
    _container: Arc<ServiceContainer>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, String> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(20);

    // 模擬分頁數據
    let mock_prompts = vec![
        Prompt::new("Daily Report", "Generate a daily summary report"),
        Prompt::new("Code Review", "Review the following code for improvements"),
        Prompt::new("Email Draft", "Draft a professional email response"),
    ];

    let total_count = mock_prompts.len() as u64;
    let paginated = PaginatedResponse::new(mock_prompts, total_count, page, limit);
    let response = ApiResponse::success(paginated);

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 創建提示詞
pub async fn create_prompt(
    _container: Arc<ServiceContainer>,
    request: CreatePromptRequest,
) -> Result<String, String> {
    // 驗證輸入
    if request.title.trim().is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("標題不能為空");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    if request.content.trim().is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("內容不能為空");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 創建提示詞
    let mut prompt = Prompt::new(&request.title, &request.content);

    // 模擬 ID 分配
    prompt.id = "prompt_123".to_string();

    let response = ApiResponse::success_with_message(prompt, "提示詞創建成功");
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 獲取特定提示詞
pub async fn get_prompt(_container: Arc<ServiceContainer>, id: String) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的提示詞 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 模擬獲取提示詞
    let mut prompt = Prompt::new(
        format!("Prompt {}", id),
        format!("This is the content of prompt {}", id),
    );
    prompt.id = id;

    let response = ApiResponse::success(prompt);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 更新提示詞
pub async fn update_prompt(
    _container: Arc<ServiceContainer>,
    id: String,
    _request: UpdatePromptRequest,
) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的提示詞 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 模擬更新
    let mut prompt = Prompt::new("Updated Prompt", "Updated content");
    prompt.id = id.clone();

    let response = ApiResponse::success_with_message(prompt, "提示詞更新成功");
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 刪除提示詞
pub async fn delete_prompt(
    _container: Arc<ServiceContainer>,
    id: String,
) -> Result<String, String> {
    if id.is_empty() {
        let error_response: ApiResponse<()> = ApiResponse::error("無效的提示詞 ID");
        return serde_json::to_string(&error_response).map_err(|e| e.to_string());
    }

    // 模擬刪除操作
    let response =
        ApiResponse::success_with_message(serde_json::json!({"deleted_id": id}), "提示詞刪除成功");

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_prompt_validation() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        // 測試空標題
        let empty_title_request = CreatePromptRequest {
            title: "".to_string(),
            content: "Valid content".to_string(),
            tags: None,
            variables: None,
        };

        let result = create_prompt(container.clone(), empty_title_request).await;
        assert!(result.is_ok());
        let response_str = result.unwrap();
        assert!(response_str.contains("標題不能為空"));

        // 測試空內容
        let empty_content_request = CreatePromptRequest {
            title: "Valid title".to_string(),
            content: "".to_string(),
            tags: None,
            variables: None,
        };

        let result = create_prompt(container, empty_content_request).await;
        assert!(result.is_ok());
        let response_str = result.unwrap();
        assert!(response_str.contains("內容不能為空"));
    }

    #[tokio::test]
    async fn test_get_prompt_invalid_id() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        let result = get_prompt(container, "".to_string()).await;
        assert!(result.is_ok());
        let response_str = result.unwrap();
        assert!(response_str.contains("無效的提示詞 ID"));
    }
}
