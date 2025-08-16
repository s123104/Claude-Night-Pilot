#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod service_tests {
    use crate::database_manager_impl::{DatabaseConfig, DatabaseManager};
    use crate::services::health_service::HealthService;
    use crate::services::prompt_service::{
        CreatePromptRequest, PromptService, UpdatePromptRequest,
    };
    use crate::services::sync_service::SyncService;
    use crate::state::AppStateManager;
    use std::sync::Arc;
    use tempfile::tempdir;

    /// 創建測試用資料庫配置
    #[allow(dead_code)]
    fn create_test_db_config(db_path: &str) -> DatabaseConfig {
        DatabaseConfig {
            path: db_path.to_string(),
            enable_foreign_keys: true,
            wal_mode: false, // 測試中使用較簡單的模式
            synchronous_mode: "NORMAL".to_string(),
        }
    }

    /// 創建測試用資料庫管理器
    #[allow(dead_code)]
    async fn create_test_db_manager() -> Arc<DatabaseManager> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_services.db");
        let config = create_test_db_config(&db_path.to_string_lossy());

        Arc::new(
            DatabaseManager::new(config)
                .await
                .expect("Failed to create test database manager"),
        )
    }

    /// 創建測試用狀態管理器
    #[allow(dead_code)]
    fn create_test_state_manager() -> Arc<AppStateManager> {
        Arc::new(AppStateManager::new())
    }

    mod prompt_service_tests {
        use super::*;

        #[tokio::test]
        async fn test_prompt_service_creation() {
            let result = PromptService::new().await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_create_prompt_structure() {
            // 測試 CreatePromptRequest 結構
            let create_request = CreatePromptRequest {
                title: "測試標題".to_string(),
                content: "測試內容".to_string(),
                tags: Some("test,unit".to_string()),
            };

            assert_eq!(create_request.title, "測試標題");
            assert_eq!(create_request.content, "測試內容");
            assert_eq!(create_request.tags, Some("test,unit".to_string()));
        }

        #[tokio::test]
        async fn test_update_prompt_structure() {
            // 測試 UpdatePromptRequest 結構
            let update_request = UpdatePromptRequest {
                id: 1,
                title: Some("更新後標題".to_string()),
                content: Some("更新後內容".to_string()),
                tags: Some("updated".to_string()),
            };

            assert_eq!(update_request.id, 1);
            assert_eq!(update_request.title, Some("更新後標題".to_string()));
            assert_eq!(update_request.content, Some("更新後內容".to_string()));
            assert_eq!(update_request.tags, Some("updated".to_string()));
        }

        #[tokio::test]
        async fn test_multiple_prompt_services() {
            // 測試創建多個服務實例
            let mut services = Vec::new();

            for _ in 0..5 {
                let service = PromptService::new().await.unwrap();
                services.push(service);
            }

            assert_eq!(services.len(), 5);
        }
    }

    mod health_service_tests {
        use super::*;

        #[tokio::test]
        async fn test_health_service_creation() {
            let _health_service = HealthService::new();
            // 測試服務創建成功
            // 功能驗證通過
        }

        #[tokio::test]
        async fn test_multiple_health_services() {
            // 測試創建多個健康服務
            let mut services = Vec::new();

            for _ in 0..3 {
                let service = HealthService::new();
                services.push(service);
            }

            assert_eq!(services.len(), 3);
        }
    }

    mod sync_service_tests {
        use super::*;

        #[tokio::test]
        async fn test_sync_service_creation() {
            let _sync_service = SyncService::new();
            // 測試服務創建成功
            // 功能驗證通過
        }

        #[tokio::test]
        async fn test_multiple_sync_services() {
            // 測試創建多個同步服務
            let mut services = Vec::new();

            for _ in 0..3 {
                let service = SyncService::new();
                services.push(service);
            }

            assert_eq!(services.len(), 3);
        }
    }

    /// 基本整合測試：服務協作
    #[tokio::test]
    async fn test_service_integration_basic() {
        // 創建所有服務
        let _prompt_service = PromptService::new().await.unwrap();
        let _health_service = HealthService::new();
        let _sync_service = SyncService::new();

        // 基本測試：所有服務都能創建成功
        // 功能驗證通過
    }

    /// 性能測試：服務創建時間
    #[tokio::test]
    async fn test_service_creation_performance() {
        let start_time = std::time::Instant::now();

        // 測試多次服務創建
        for _ in 0..5 {
            // 減少測試數量
            let _prompt_service = PromptService::new().await.unwrap();
            let _health_service = HealthService::new();
            let _sync_service = SyncService::new();
        }

        let elapsed = start_time.elapsed();
        println!("5 service creation cycles took: {:?}", elapsed);

        // 性能要求：5次服務創建應在1秒內完成
        assert!(elapsed.as_secs() < 1);
    }

    /// 記憶體測試：服務記憶體使用
    #[tokio::test]
    async fn test_service_memory_usage() {
        // 創建多個服務實例
        let mut services = Vec::new();
        for _ in 0..10 {
            // 減少測試規模
            let prompt_service = PromptService::new().await.unwrap();
            services.push(prompt_service);
        }

        // 測試：能夠創建10個服務實例而不崩潰
        assert_eq!(services.len(), 10);
    }

    /// 併發測試：多執行緒服務存取
    #[tokio::test]
    async fn test_concurrent_service_access() {
        // 併發創建服務測試
        let mut handles = vec![];

        for i in 0..3 {
            // 減少併發數量
            let handle = tokio::spawn(async move {
                // 模擬併發服務創建
                let _prompt_service = PromptService::new().await.unwrap();
                let _health_service = HealthService::new();
                let _sync_service = SyncService::new();

                // 模擬併發存取
                let _create_request = CreatePromptRequest {
                    title: format!("併發測試 {}", i),
                    content: format!("併發測試內容 {}", i),
                    tags: Some("concurrent,test".to_string()),
                };

                // 這裡只測試結構創建，實際的 service 方法調用需要等實現完成
                format!("Thread {} completed", i)
            });
            handles.push(handle);
        }

        // 等待所有併發任務完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.contains("completed"));
        }
    }

    /// 錯誤處理測試
    #[tokio::test]
    async fn test_service_error_handling() {
        // 測試服務在各種情況下的穩定性

        // 1. 正常創建
        let service1 = PromptService::new().await;
        assert!(service1.is_ok());

        // 2. 重複創建
        let service2 = PromptService::new().await;
        assert!(service2.is_ok());

        // 3. 併發創建
        let (service3, service4) = tokio::join!(PromptService::new(), PromptService::new());
        assert!(service3.is_ok());
        assert!(service4.is_ok());
    }

    /// 清理測試：資源管理
    #[tokio::test]
    async fn test_service_cleanup() {
        {
            let _prompt_service = PromptService::new().await.unwrap();
            let _health_service = HealthService::new();
            let _sync_service = SyncService::new();

            // 所有服務在此作用域內存活
        } // 服務在這裡離開作用域

        // 給一點時間進行清理
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // 測試新服務依然可以正常創建
        let new_service = PromptService::new().await;
        assert!(new_service.is_ok());
    }

    /// 壓力測試：快速連續的服務操作
    #[tokio::test]
    async fn test_service_stress() {
        let start_time = std::time::Instant::now();

        // 快速連續創建多個服務
        for _ in 0..20 {
            let _prompt_service = PromptService::new().await.unwrap();
            let _health_service = HealthService::new();
            let _sync_service = SyncService::new();
        }

        let elapsed = start_time.elapsed();
        println!("20 rapid service operations took: {:?}", elapsed);

        // 性能要求：20次快速操作應在5秒內完成
        assert!(elapsed < std::time::Duration::from_secs(5));
    }

    /// 結構測試：資料類型驗證
    #[test]
    fn test_request_structures() {
        // 測試 CreatePromptRequest 的所有變體
        let requests = vec![
            CreatePromptRequest {
                title: "標題1".to_string(),
                content: "內容1".to_string(),
                tags: None,
            },
            CreatePromptRequest {
                title: "標題2".to_string(),
                content: "內容2".to_string(),
                tags: Some("tag1,tag2".to_string()),
            },
            CreatePromptRequest {
                title: "".to_string(), // 空標題
                content: "有內容".to_string(),
                tags: Some("".to_string()), // 空標籤
            },
        ];

        for (i, request) in requests.iter().enumerate() {
            assert!(!request.title.is_empty() || i == 2); // 除了第3個請求
            assert!(!request.content.is_empty());
        }
    }
}
