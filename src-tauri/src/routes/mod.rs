// Routes 模組 - API 路由定義和處理器
// 採用 vibe-kanban 架構模式

pub mod health_routes;
pub mod job_routes;
pub mod prompt_routes;

// 重新導出主要路由處理器
pub use health_routes::*;
pub use job_routes::*;
pub use prompt_routes::*;

use crate::models::ApiResponse;

/// 路由錯誤處理
pub type RouteResult<T> = Result<ApiResponse<T>, String>;

/// 統一錯誤響應生成
pub fn error_response(message: impl Into<String>) -> String {
    let response: ApiResponse<()> = ApiResponse::error(message);
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

/// 統一成功響應生成
pub fn success_response<T: serde::Serialize>(data: T) -> Result<String, String> {
    let response = ApiResponse::success(data);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}
