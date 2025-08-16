// 共享資料類型 - GUI和CLI通用的數據結構
use serde::{Deserialize, Serialize};

/// 統一的執行結果類型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedExecutionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub timestamp: String,
}

/// 統一的錯誤類型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub source: String, // "gui", "cli", "system"
    pub timestamp: String,
}

/// 統一的分頁請求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationRequest {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 統一的分頁響應
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 統一的過濾條件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterRequest {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    NotIn,
}

/// 統一的搜索請求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub fields: Option<Vec<String>>,
    pub filters: Option<Vec<FilterRequest>>,
    pub pagination: Option<PaginationRequest>,
}

/// 統一的狀態響應
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusResponse {
    pub status: String,
    pub healthy: bool,
    pub components: std::collections::HashMap<String, ComponentStatus>,
    pub timestamp: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComponentStatus {
    pub name: String,
    pub status: String,
    pub healthy: bool,
    pub message: Option<String>,
    pub last_check: String,
}

/// 統一的配置類型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationConfig {
    pub gui: GuiConfig,
    pub cli: CliConfig,
    pub database: DatabaseConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuiConfig {
    pub theme: String,
    pub auto_refresh_interval: u64,
    pub enable_notifications: bool,
    pub max_concurrent_operations: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CliConfig {
    pub default_output_format: String,
    pub color_output: bool,
    pub verbose_logging: bool,
    pub command_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub enable_foreign_keys: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncConfig {
    pub enable_real_time: bool,
    pub sync_interval: u64,
    pub conflict_resolution: String,
    pub max_retry_attempts: u32,
}

/// 統一的操作類型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OperationType {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    List,
    Search,
    Export,
    Import,
}

/// 統一的執行上下文
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionContext {
    pub source: String, // "gui" or "cli"
    pub user_id: Option<String>,
    pub session_id: String,
    pub operation: OperationType,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 統一的回應包裝器
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<UnifiedError>,
    pub context: ExecutionContext,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, context: ExecutionContext) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            context,
        }
    }

    pub fn error(error: UnifiedError, context: ExecutionContext) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            context,
        }
    }
}

/// 統一的批量操作請求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchRequest<T> {
    pub operations: Vec<BatchOperation<T>>,
    pub abort_on_error: bool,
    pub context: ExecutionContext,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchOperation<T> {
    pub id: String,
    pub operation: OperationType,
    pub data: T,
}

/// 統一的批量操作響應
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchResponse<T> {
    pub results: Vec<BatchResult<T>>,
    pub success_count: u32,
    pub error_count: u32,
    pub total_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchResult<T> {
    pub id: String,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<UnifiedError>,
    pub execution_time_ms: u64,
}
