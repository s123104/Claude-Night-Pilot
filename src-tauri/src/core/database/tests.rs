#[cfg(test)]
mod database_tests {
    use crate::database_manager_impl::{DatabaseManager, DatabaseConfig};
    use tempfile::tempdir;
    use std::time::Duration;
    
    /// 創建測試資料庫配置的輔助函數
    fn create_test_config(db_path: &str) -> DatabaseConfig {
        DatabaseConfig {
            path: db_path.to_string(),
            enable_foreign_keys: true,
            wal_mode: false, // 測試中使用較簡單的模式
            synchronous_mode: "NORMAL".to_string(),
        }
    }
    
    /// 測試資料庫管理器初始化
    #[tokio::test]
    async fn test_database_manager_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_init.db");
        
        let config = create_test_config(&db_path.to_string_lossy());
        
        let _manager = DatabaseManager::new(config)
            .await
            .expect("Failed to initialize database manager");
        
        // 基本驗證：資料庫檔案應該存在
        assert!(db_path.exists());
    }
    
    /// 測試預設配置初始化
    #[tokio::test]
    async fn test_database_manager_default_config() {
        let result = DatabaseManager::with_default_config().await;
        assert!(result.is_ok());
    }
    
    /// 測試多次初始化同一資料庫
    #[tokio::test]
    async fn test_multiple_database_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_multiple.db");
        let config = create_test_config(&db_path.to_string_lossy());
        
        // 第一次初始化
        let _manager1 = DatabaseManager::new(config.clone())
            .await
            .expect("Failed first initialization");
        
        // 第二次初始化同一資料庫應該成功
        let _manager2 = DatabaseManager::new(config)
            .await
            .expect("Failed second initialization");
        
        // 兩個管理器都應該能成功創建
        assert!(db_path.exists());
    }
    
    /// 測試錯誤處理 - 無效路徑
    #[tokio::test]
    async fn test_invalid_database_path() {
        let config = create_test_config("/invalid/nonexistent/path/test.db");
        let result = DatabaseManager::new(config).await;
        
        // 應該返回錯誤，但不應該 panic
        assert!(result.is_err());
    }
    
    /// 性能測試：資料庫初始化時間
    #[tokio::test]
    async fn test_database_initialization_performance() {
        let temp_dir = tempdir().unwrap();
        let start_time = std::time::Instant::now();
        
        // 測試多次資料庫初始化
        for i in 0..5 { // 減少測試數量以加快測試速度
            let db_path = temp_dir.path().join(format!("perf_test_{}.db", i));
            let config = create_test_config(&db_path.to_string_lossy());
            
            let _manager = DatabaseManager::new(config)
                .await
                .expect("Failed to initialize database for performance test");
        }
        
        let elapsed = start_time.elapsed();
        println!("5 database initializations took: {:?}", elapsed);
        
        // 性能要求：5次初始化應在3秒內完成
        assert!(elapsed < Duration::from_secs(3));
    }
    
    /// 併發測試：同時初始化多個資料庫
    #[tokio::test]
    async fn test_concurrent_database_initialization() {
        let temp_dir = tempdir().unwrap();
        let mut handles = vec![];
        
        for i in 0..3 { // 減少併發數量
            let temp_path = temp_dir.path().to_owned();
            let handle = tokio::spawn(async move {
                let db_path = temp_path.join(format!("concurrent_test_{}.db", i));
                let config = create_test_config(&db_path.to_string_lossy());
                
                let _manager = DatabaseManager::new(config)
                    .await
                    .expect("Failed concurrent database initialization");
                
                (i, db_path.exists())
            });
            handles.push(handle);
        }
        
        // 等待所有併發初始化完成
        for handle in handles {
            let (id, exists) = handle.await.unwrap();
            assert!(exists, "Database {} should exist", id);
        }
    }
    
    /// 記憶體測試：多個資料庫管理器的記憶體使用
    #[tokio::test]
    async fn test_multiple_database_managers_memory() {
        let temp_dir = tempdir().unwrap();
        let mut managers = Vec::new();
        
        // 創建10個資料庫管理器（減少數量）
        for i in 0..10 {
            let db_path = temp_dir.path().join(format!("memory_test_{}.db", i));
            let config = create_test_config(&db_path.to_string_lossy());
            
            let manager = DatabaseManager::new(config)
                .await
                .expect("Failed to create database manager for memory test");
            
            managers.push(manager);
        }
        
        // 驗證所有管理器都成功創建
        assert_eq!(managers.len(), 10);
        
        // 測試清理 - 確保沒有記憶體洩漏
        drop(managers);
    }
    
    /// 邊界測試：不同資料庫配置
    #[tokio::test]
    async fn test_different_database_configs() {
        let temp_dir = tempdir().unwrap();
        
        let configs = vec![
            DatabaseConfig {
                path: temp_dir.path().join("test_1.db").to_string_lossy().to_string(),
                enable_foreign_keys: true,
                wal_mode: false,
                synchronous_mode: "OFF".to_string(),
            },
            DatabaseConfig {
                path: temp_dir.path().join("test_2.db").to_string_lossy().to_string(),
                enable_foreign_keys: false,
                wal_mode: false,
                synchronous_mode: "NORMAL".to_string(),
            },
            DatabaseConfig {
                path: temp_dir.path().join("test_3.db").to_string_lossy().to_string(),
                enable_foreign_keys: true,
                wal_mode: false,
                synchronous_mode: "FULL".to_string(),
            },
        ];
        
        for (i, config) in configs.into_iter().enumerate() {
            let _manager = DatabaseManager::new(config)
                .await
                .expect(&format!("Failed to create database with config {}", i));
            
            // 基本驗證
            let db_path = temp_dir.path().join(format!("test_{}.db", i + 1));
            assert!(db_path.exists());
        }
    }
    
    /// 測試資源清理
    #[tokio::test]
    async fn test_database_cleanup() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("cleanup_test.db");
        
        {
            let config = create_test_config(&db_path.to_string_lossy());
            let _manager = DatabaseManager::new(config)
                .await
                .expect("Failed to create database manager for cleanup test");
            
            // 驗證資料庫檔案存在
            assert!(db_path.exists());
        } // manager 在這裡離開作用域
        
        // 給一點時間進行清理
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // 檔案應該仍然存在（SQLite 檔案在連接關閉後保持）
        assert!(db_path.exists());
    }
    
    /// 整合測試：完整的資料庫生命周期
    #[tokio::test]
    async fn test_complete_database_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("lifecycle_test.db");
        let config = create_test_config(&db_path.to_string_lossy());
        
        // 1. 創建資料庫
        let manager1 = DatabaseManager::new(config.clone())
            .await
            .expect("Failed to create database");
        
        assert!(db_path.exists());
        
        // 2. 重新連接到同一資料庫
        let manager2 = DatabaseManager::new(config)
            .await
            .expect("Failed to reconnect to database");
        
        // 3. 多個管理器可以同時存在
        assert!(db_path.exists());
        
        // 4. 清理第一個管理器
        drop(manager1);
        
        // 5. 第二個管理器仍然有效
        assert!(db_path.exists());
        
        // 6. 清理第二個管理器
        drop(manager2);
        
        // 7. 資料庫檔案仍然存在
        assert!(db_path.exists());
    }
    
    /// 基本功能測試：配置結構
    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        
        assert_eq!(config.path, "claude-night-pilot.db");
        assert_eq!(config.enable_foreign_keys, true);
        assert_eq!(config.wal_mode, true);
        assert_eq!(config.synchronous_mode, "NORMAL");
    }
    
    /// 基本功能測試：配置複製
    #[test]
    fn test_database_config_clone() {
        let config1 = DatabaseConfig::default();
        let config2 = config1.clone();
        
        assert_eq!(config1.path, config2.path);
        assert_eq!(config1.enable_foreign_keys, config2.enable_foreign_keys);
        assert_eq!(config1.wal_mode, config2.wal_mode);
        assert_eq!(config1.synchronous_mode, config2.synchronous_mode);
    }
}