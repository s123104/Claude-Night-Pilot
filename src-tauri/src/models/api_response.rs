// API 響應統一格式 - 參考 vibe-kanban 設計

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use ts_rs::TS;

/// 統一的 API 響應格式
/// 確保所有 API 端點返回一致的響應結構
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct ApiResponse<T> {
    /// 操作是否成功
    pub success: bool,
    
    /// 響應資料（成功時包含）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// 錯誤或成功訊息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    
    /// 響應時間戳
    #[ts(type = "string")]
    pub timestamp: DateTime<Utc>,
    
    /// 請求 ID（用於追蹤和偵錯）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 創建成功響應
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    /// 創建成功響應並附帶訊息
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    /// 創建錯誤響應
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
            timestamp: Utc::now(),
            request_id: None,
        }
    }
    
    /// 設置請求 ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
    
    /// 轉換資料類型
    pub fn map<U, F>(self, f: F) -> ApiResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        ApiResponse {
            success: self.success,
            data: self.data.map(f),
            message: self.message,
            timestamp: self.timestamp,
            request_id: self.request_id,
        }
    }
}

impl<T> Default for ApiResponse<T> {
    fn default() -> Self {
        Self::error("Unknown error occurred")
    }
}

/// 分頁響應資料結構
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/")]
pub struct PaginatedResponse<T> {
    /// 當前頁面資料
    pub items: Vec<T>,
    
    /// 總記錄數
    pub total_count: u64,
    
    /// 當前頁碼（從 1 開始）
    pub page: u32,
    
    /// 每頁大小
    pub page_size: u32,
    
    /// 總頁數
    pub total_pages: u32,
    
    /// 是否有下一頁
    pub has_next: bool,
    
    /// 是否有上一頁
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// 創建分頁響應
    pub fn new(items: Vec<T>, total_count: u64, page: u32, page_size: u32) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as u32;
        
        Self {
            items,
            total_count,
            page,
            page_size,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// HTTP 狀態碼對應
impl<T> ApiResponse<T> {
    /// 取得對應的 HTTP 狀態碼
    pub fn status_code(&self) -> u16 {
        if self.success {
            200 // OK
        } else {
            500 // Internal Server Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_success_response() {
        let response = ApiResponse::success("test data");
        
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.message.is_none());
        assert_eq!(response.status_code(), 200);
    }
    
    #[test]
    fn test_error_response() {
        let response: ApiResponse<()> = ApiResponse::error("Test error");
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("Test error".to_string()));
        assert_eq!(response.status_code(), 500);
    }
    
    #[test]
    fn test_map_transformation() {
        let response = ApiResponse::success(42);
        let mapped = response.map(|x| x.to_string());
        
        assert!(mapped.success);
        assert_eq!(mapped.data, Some("42".to_string()));
    }
    
    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let paginated = PaginatedResponse::new(items, 15, 2, 5);
        
        assert_eq!(paginated.total_count, 15);
        assert_eq!(paginated.total_pages, 3);
        assert!(paginated.has_prev);
        assert!(paginated.has_next);
    }
}