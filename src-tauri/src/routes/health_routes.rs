// Health 路由處理器
// 負責處理系統健康檢查相關的 API 請求

use crate::models::ApiResponse;
use crate::services::{Service, ServiceContainer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 系統健康狀態
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_healthy: bool,
    pub claude_service: bool,
    pub database_service: bool,
    pub scheduler_service: bool,
    pub monitoring_service: bool,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_connections: u32,
}

/// 服務狀態詳情
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub health: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

/// 獲取系統整體健康狀態
pub async fn get_system_health(container: Arc<ServiceContainer>) -> Result<String, String> {
    // 執行完整的健康檢查
    let health_status = container.health_check().await;

    let system_health = SystemHealth {
        overall_healthy: health_status.overall_healthy,
        claude_service: health_status.claude_service,
        database_service: health_status.database_service,
        scheduler_service: health_status.scheduler_service,
        monitoring_service: health_status.monitoring_service,
        uptime_seconds: 3600, // 模擬 1 小時運行時間
        memory_usage_mb: 45.2,
        cpu_usage_percent: 12.5,
        active_connections: 3,
    };

    let response = ApiResponse::success(system_health);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 獲取詳細的服務狀態
pub async fn get_services_status(container: Arc<ServiceContainer>) -> Result<String, String> {
    let now = chrono::Utc::now();

    let services = vec![
        ServiceStatus {
            name: "claude_service".to_string(),
            status: "running".to_string(),
            health: container.claude_service().health_check().await,
            last_check: now,
            details: {
                let mut details = HashMap::new();
                details.insert("active_executions".to_string(), serde_json::Value::from(0));
                details.insert("total_executions".to_string(), serde_json::Value::from(42));
                details
            },
        },
        ServiceStatus {
            name: "database_service".to_string(),
            status: "connected".to_string(),
            health: container.database_service().health_check().await,
            last_check: now,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "connection_pool_size".to_string(),
                    serde_json::Value::from(5),
                );
                details.insert("active_queries".to_string(), serde_json::Value::from(0));
                details
            },
        },
        ServiceStatus {
            name: "scheduler_service".to_string(),
            status: if container.scheduler_service().is_running().await {
                "running"
            } else {
                "stopped"
            }
            .to_string(),
            health: container.scheduler_service().health_check().await,
            last_check: now,
            details: {
                let mut details = HashMap::new();
                details.insert("pending_jobs".to_string(), serde_json::Value::from(2));
                details.insert("completed_jobs".to_string(), serde_json::Value::from(15));
                details
            },
        },
        ServiceStatus {
            name: "monitoring_service".to_string(),
            status: if container.monitoring_service().is_active().await {
                "active"
            } else {
                "inactive"
            }
            .to_string(),
            health: container.monitoring_service().health_check().await,
            last_check: now,
            details: {
                let mut details = HashMap::new();
                details.insert(
                    "metrics_collected".to_string(),
                    serde_json::Value::from(1234),
                );
                details.insert("alerts_active".to_string(), serde_json::Value::from(0));
                details
            },
        },
    ];

    let response = ApiResponse::success(services);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 快速健康檢查 (輕量級)
pub async fn quick_health_check(_container: Arc<ServiceContainer>) -> Result<String, String> {
    let health = serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "response_time_ms": 1
    });

    let response = ApiResponse::success(health);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

/// 獲取系統資源使用情況
pub async fn get_system_resources(container: Arc<ServiceContainer>) -> Result<String, String> {
    let monitoring_service = container.monitoring_service();
    let stats = monitoring_service.get_stats().await;

    let resources = serde_json::json!({
        "memory": {
            "used_mb": stats.get("memory_usage").unwrap_or(&serde_json::Value::from(0.0)),
            "available_mb": 512.0,
            "percentage": (stats.get("memory_usage").unwrap_or(&serde_json::Value::from(0.0)).as_f64().unwrap_or(0.0) / 512.0) * 100.0
        },
        "cpu": {
            "usage_percent": stats.get("cpu_usage").unwrap_or(&serde_json::Value::from(0.0)),
            "cores": 8
        },
        "disk": {
            "usage_percent": stats.get("disk_usage").unwrap_or(&serde_json::Value::from(0.0)),
            "free_gb": 50.5,
            "total_gb": 100.0
        },
        "network": {
            "connections": 3,
            "bytes_sent": 1024567,
            "bytes_received": 2048123
        }
    });

    let response = ApiResponse::success(resources);
    serde_json::to_string(&response).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_quick_health_check() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        let result = quick_health_check(container).await;
        assert!(result.is_ok());

        let response_str = result.unwrap();
        assert!(response_str.contains("ok"));
        assert!(response_str.contains("timestamp"));
    }

    #[tokio::test]
    async fn test_get_system_health() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        let result = get_system_health(container).await;
        assert!(result.is_ok());

        let response_str = result.unwrap();
        assert!(response_str.contains("overall_healthy"));
        assert!(response_str.contains("claude_service"));
    }

    #[tokio::test]
    async fn test_get_services_status() {
        let container = Arc::new(ServiceContainer::new().await.unwrap());

        let result = get_services_status(container).await;
        assert!(result.is_ok());

        let response_str = result.unwrap();
        assert!(response_str.contains("claude_service"));
        assert!(response_str.contains("database_service"));
        assert!(response_str.contains("scheduler_service"));
        assert!(response_str.contains("monitoring_service"));
    }
}
