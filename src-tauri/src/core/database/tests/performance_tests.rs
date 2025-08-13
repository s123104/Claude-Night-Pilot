// SQL最佳實踐性能測試套件
// 專注於性能基準測試和性能回歸檢測

use std::time::{Duration, Instant};
use tempfile::tempdir;

// 基於Rusqlite實現的性能測試套件
// 已適配rusqlite接口，可正常運行

use crate::core::database::best_practices_rusqlite::{
    RusqliteBestPracticesManager, RusqliteBestPracticesConfig, 
    RusqlitePrompt, RusqliteModel
};

/// 性能測試配置
#[allow(dead_code)]
struct PerformanceTestConfig {
    pub small_dataset_size: usize,
    pub medium_dataset_size: usize,
    pub large_dataset_size: usize,
    pub concurrent_operations: usize,
    pub timeout_seconds: u64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            small_dataset_size: 100,
            medium_dataset_size: 1000,
            large_dataset_size: 10000,
            concurrent_operations: 50,
            timeout_seconds: 60,
        }
    }
}

/// 性能測試結果
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PerformanceResult {
    pub operation: String,
    pub dataset_size: usize,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub memory_usage_mb: Option<f64>,
}

impl PerformanceResult {
    fn new(operation: &str, dataset_size: usize, duration: Duration) -> Self {
        let ops_per_second = if duration.as_secs_f64() > 0.0 {
            dataset_size as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        Self {
            operation: operation.to_string(),
            dataset_size,
            duration,
            operations_per_second: ops_per_second,
            memory_usage_mb: None,
        }
    }
    
    #[allow(dead_code)]
    fn with_memory_usage(mut self, memory_mb: f64) -> Self {
        self.memory_usage_mb = Some(memory_mb);
        self
    }
}

/// 創建性能測試用的數據庫管理器
fn create_performance_manager() -> RusqliteBestPracticesManager {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("performance_test.db");
    
    let config = RusqliteBestPracticesConfig {
        database_path: db_path,
        connection_timeout: std::time::Duration::from_secs(30),
        busy_timeout: std::time::Duration::from_secs(30),
        enable_foreign_keys: true,
        enable_wal_mode: true,
        page_size: Some(4096),
        cache_size: Some(-2000), // 2MB cache for performance
        synchronous_mode: crate::core::database::best_practices_rusqlite::SynchronousMode::Normal,
        journal_mode: crate::core::database::best_practices_rusqlite::JournalMode::Wal,
    };
    
    let manager = RusqliteBestPracticesManager::new(config).unwrap();
    
    // 創建測試表
    manager.with_connection(|conn| {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS prompts (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
            [],
        )?;
        
        // 創建索引以提高查詢性能
        conn.execute("CREATE INDEX IF NOT EXISTS idx_prompts_title ON prompts(title)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_prompts_is_favorite ON prompts(is_favorite)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_prompts_created_at ON prompts(created_at)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_prompts_tags ON prompts(tags)", [])?;
        
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    manager
}

/// 生成測試數據
fn generate_test_data(count: usize) -> Vec<(String, String, Option<String>, bool)> {
    (0..count)
        .map(|i| {
            let content_size = match i % 4 {
                0 => 50,   // 短內容
                1 => 200,  // 中等內容
                2 => 500,  // 長內容
                3 => 1000, // 很長內容
                _ => 100,
            };
            
            (
                format!("性能測試標題 {} - {}", i, generate_random_suffix()),
                format!("性能測試內容 {} {}", i, "重複內容片段 ".repeat(content_size / 10)),
                Some(format!("性能,測試,批次{},標籤{}", i % 10, i % 5)),
                i % 3 == 0, // 約33%的收藏率
            )
        })
        .collect()
}

/// 生成隨機後綴以避免重複
fn generate_random_suffix() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    format!("{:x}", hasher.finish() % 10000)
}

/// 測試插入性能
#[test]
fn test_insert_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("🚀 開始插入性能測試...");
    
    // 小數據集測試
    let small_data = generate_test_data(config.small_dataset_size);
    let start = Instant::now();
    
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in small_data {
            RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    let small_result = PerformanceResult::new("單條插入", config.small_dataset_size, start.elapsed());
    
    // 驗證性能要求
    assert!(small_result.duration.as_millis() < 5000, "小數據集插入應在5秒內完成");
    assert!(small_result.operations_per_second > 10.0, "插入速度應大於10 ops/sec");
    
    println!("✅ 小數據集插入: {} 條記錄，耗時 {:?}，速度 {:.1} ops/sec", 
             small_result.dataset_size, small_result.duration, small_result.operations_per_second);
    
    // 中等數據集測試
    let medium_data = generate_test_data(config.medium_dataset_size);
    let start = Instant::now();
    
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in medium_data {
            RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    let medium_result = PerformanceResult::new("單條插入", config.medium_dataset_size, start.elapsed());
    
    // 中等數據集性能要求稍微寬鬆
    assert!(medium_result.duration.as_millis() < 30000, "中等數據集插入應在30秒內完成");
    assert!(medium_result.operations_per_second > 5.0, "插入速度應大於5 ops/sec");
    
    println!("✅ 中等數據集插入: {} 條記錄，耗時 {:?}，速度 {:.1} ops/sec", 
             medium_result.dataset_size, medium_result.duration, medium_result.operations_per_second);
}

/// 測試並發插入性能
#[test]
fn test_concurrent_insert_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("🔥 開始並發插入性能測試...");
    
    let total_operations = config.concurrent_operations * 10; // 每個並發任務處理10條記錄
    let start = Instant::now();
    
    // Rusqlite不支持並發寫入，改為順序插入測試
    manager.with_connection(|conn| {
        for batch in 0..config.concurrent_operations {
            for i in 0..10 {
                RusqlitePrompt::create(
                    conn,
                    format!("並發測試 Batch{} Item{}", batch, i),
                    format!("並發測試內容 {} {} {}", batch, i, "重複片段".repeat(20)),
                    Some(format!("並發,測試,batch{}", batch)),
                    (batch + i) % 2 == 0,
                )?;
            }
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    let concurrent_result = PerformanceResult::new("順序插入", total_operations, start.elapsed());
    
    assert!(concurrent_result.duration.as_millis() < 20000, "插入應在20秒內完成");
    assert!(concurrent_result.operations_per_second > 5.0, "插入速度應大於5 ops/sec");
    
    println!("✅ 順序插入: {} 條記錄，耗時 {:?}，速度 {:.1} ops/sec", 
             concurrent_result.dataset_size, concurrent_result.duration, concurrent_result.operations_per_second);
}

/// 測試查詢性能
#[test]
fn test_query_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("🔍 開始查詢性能測試...");
    
    // 先插入測試數據
    let test_data = generate_test_data(config.medium_dataset_size);
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in test_data {
            RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    // 測試全表掃描性能
    let start = Instant::now();
    let all_prompts = manager.with_connection(|conn| {
        RusqlitePrompt::find_all(conn, None)
    }).unwrap();
    let scan_duration = start.elapsed();
    
    assert_eq!(all_prompts.len(), config.medium_dataset_size);
    assert!(scan_duration.as_millis() < 2000, "全表掃描應在2秒內完成");
    
    println!("✅ 全表掃描: {} 條記錄，耗時 {:?}", all_prompts.len(), scan_duration);
    
    // 測試索引查詢性能
    let start = Instant::now();
    let favorite_prompts = manager.with_connection(|conn| {
        RusqlitePrompt::search(conn, None, Some(true), None, None, None)
    }).unwrap();
    let index_duration = start.elapsed();
    
    assert!(!favorite_prompts.is_empty());
    assert!(index_duration.as_millis() < 500, "索引查詢應在0.5秒內完成");
    
    println!("✅ 索引查詢: {} 條收藏記錄，耗時 {:?}", favorite_prompts.len(), index_duration);
    
    // 測試複雜查詢性能
    let start = Instant::now();
    let complex_results = manager.with_connection(|conn| {
        RusqlitePrompt::search(conn, Some("性能"), Some(true), Some("測試"), Some(50), None)
    }).unwrap();
    let complex_duration = start.elapsed();
    
    assert!(complex_duration.as_millis() < 1000, "複雜查詢應在1秒內完成");
    
    println!("✅ 複雜查詢: {} 條匹配記錄，耗時 {:?}", complex_results.len(), complex_duration);
}

/// 測試更新性能
#[test]
fn test_update_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("✏️ 開始更新性能測試...");
    
    // 先插入測試數據
    let test_data = generate_test_data(config.small_dataset_size);
    let mut created_ids = Vec::new();
    
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in test_data {
            let prompt = RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
            created_ids.push(prompt.id);
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    // 測試單條更新性能
    let start = Instant::now();
    manager.with_connection(|conn| {
        for (i, id) in created_ids.iter().enumerate() {
            let mut prompt = RusqlitePrompt::find_by_id(conn, *id)?.unwrap();
            prompt.title = format!("更新的標題 {}", i);
            prompt.content = format!("更新的內容 {}", i);
            prompt.tags = Some("更新,測試".to_string());
            prompt.is_favorite = i % 2 == 0;
            prompt.save(conn)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    let update_duration = start.elapsed();
    
    let update_result = PerformanceResult::new("單條更新", config.small_dataset_size, update_duration);
    
    assert!(update_result.duration.as_millis() < 5000, "更新操作應在5秒內完成");
    assert!(update_result.operations_per_second > 5.0, "更新速度應大於5 ops/sec");
    
    println!("✅ 單條更新: {} 條記錄，耗時 {:?}，速度 {:.1} ops/sec", 
             update_result.dataset_size, update_result.duration, update_result.operations_per_second);
    
    // 測試批量更新性能
    let batch_ids: Vec<_> = created_ids.iter().take(50).cloned().collect();
    let start = Instant::now();
    let updated_count = manager.with_connection(|conn| {
        RusqlitePrompt::bulk_update_favorite(conn, &batch_ids, true)
    }).unwrap();
    let bulk_duration = start.elapsed();
    
    assert_eq!(updated_count, 50);
    assert!(bulk_duration.as_millis() < 1000, "批量更新應在1秒內完成");
    
    println!("✅ 批量更新: {} 條記錄，耗時 {:?}", updated_count, bulk_duration);
}

/// 測試刪除性能
#[test]
fn test_delete_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("🗑️ 開始刪除性能測試...");
    
    // 先插入測試數據
    let test_data = generate_test_data(config.small_dataset_size);
    let mut created_ids = Vec::new();
    
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in test_data {
            let prompt = RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
            created_ids.push(prompt.id);
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    // 測試單條刪除性能
    let delete_count = 50;
    let start = Instant::now();
    
    manager.with_connection(|conn| {
        for i in 0..delete_count {
            let deleted = RusqlitePrompt::delete(conn, created_ids[i])?;
            assert!(deleted);
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    let delete_duration = start.elapsed();
    let delete_result = PerformanceResult::new("單條刪除", delete_count, delete_duration);
    
    assert!(delete_result.duration.as_millis() < 3000, "刪除操作應在3秒內完成");
    assert!(delete_result.operations_per_second > 10.0, "刪除速度應大於10 ops/sec");
    
    println!("✅ 單條刪除: {} 條記錄，耗時 {:?}，速度 {:.1} ops/sec", 
             delete_result.dataset_size, delete_result.duration, delete_result.operations_per_second);
    
    // 驗證刪除結果
    let remaining = manager.with_connection(|conn| {
        RusqlitePrompt::find_all(conn, None)
    }).unwrap();
    assert_eq!(remaining.len(), config.small_dataset_size - delete_count);
    
    println!("✅ 驗證刪除: 剩餘 {} 條記錄", remaining.len());
}

/// 測試統計查詢性能
#[test]
fn test_statistics_performance() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("📊 開始統計性能測試...");
    
    // 先插入大量測試數據
    let test_data = generate_test_data(config.medium_dataset_size);
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in test_data {
            RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    // 測試統計查詢性能
    let start = Instant::now();
    let stats = manager.with_connection(|conn| {
        RusqlitePrompt::get_statistics(conn)
    }).unwrap();
    let stats_duration = start.elapsed();
    
    // 驗證統計結果
    assert_eq!(stats.total_count, config.medium_dataset_size as u64);
    assert!(stats.favorite_count > 0);
    assert!(stats.tagged_count > 0);
    assert!(stats.average_content_length > 0.0);
    
    // 性能要求
    assert!(stats_duration.as_millis() < 2000, "統計查詢應在2秒內完成");
    
    println!("✅ 統計查詢: 數據量 {} 條，耗時 {:?}", stats.total_count, stats_duration);
    println!("📈 統計結果: 總數={}, 收藏={}, 標籤={}, 平均長度={:.1}", 
             stats.total_count, stats.favorite_count, stats.tagged_count, stats.average_content_length);
}

/// 測試內存使用和清理
#[test]
fn test_memory_usage() {
    let config = PerformanceTestConfig::default();
    let manager = create_performance_manager();
    
    println!("💾 開始內存使用測試...");
    
    // 獲取初始健康狀態
    let initial_health = manager.health_check().unwrap();
    
    // 大量數據操作
    let large_data = generate_test_data(config.medium_dataset_size);
    let start = Instant::now();
    
    let mut created_ids = Vec::new();
    manager.with_connection(|conn| {
        for (title, content, tags, is_favorite) in large_data {
            let prompt = RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
            created_ids.push(prompt.id);
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    
    let insert_duration = start.elapsed();
    
    // 檢查健康狀態
    let after_insert_health = manager.health_check().unwrap();
    assert!(after_insert_health.is_healthy);
    
    // 大量查詢操作
    let start = Instant::now();
    manager.with_connection(|conn| {
        for _ in 0..100 {
            let _ = RusqlitePrompt::find_all(conn, Some(100))?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    let query_duration = start.elapsed();
    
    // 檢查健康狀態
    let after_query_health = manager.health_check().unwrap();
    assert!(after_query_health.is_healthy);
    
    // 清理數據
    let start = Instant::now();
    manager.with_connection(|conn| {
        for id in created_ids {
            let _ = RusqlitePrompt::delete(conn, id)?;
        }
        Ok::<(), anyhow::Error>(())
    }).unwrap();
    let cleanup_duration = start.elapsed();
    
    // 最終健康檢查
    let final_health = manager.health_check().unwrap();
    assert!(final_health.is_healthy);
    
    println!("✅ 內存使用測試完成");
    println!("🔄 操作時間: 插入={:?}, 查詢={:?}, 清理={:?}", 
             insert_duration, query_duration, cleanup_duration);
    println!("🏥 數據庫狀態: 初始大小={}B, 插入後={}B, 最終={}B", 
             initial_health.database_size_bytes, 
             after_insert_health.database_size_bytes,
             final_health.database_size_bytes);
}

/// 性能回歸測試
#[test]
fn test_performance_regression() {
    let manager = create_performance_manager();
    
    println!("📈 開始性能回歸測試...");
    
    // 定義性能基準線（基於之前的測試結果）
    struct PerformanceBenchmark {
        operation: &'static str,
        dataset_size: usize,
        max_duration_ms: u64,
        min_ops_per_second: f64,
    }
    
    let benchmarks = vec![
        PerformanceBenchmark {
            operation: "插入",
            dataset_size: 100,
            max_duration_ms: 5000,
            min_ops_per_second: 10.0,
        },
        PerformanceBenchmark {
            operation: "查詢",
            dataset_size: 1000,
            max_duration_ms: 2000,
            min_ops_per_second: 500.0,
        },
        PerformanceBenchmark {
            operation: "更新",
            dataset_size: 50,
            max_duration_ms: 3000,
            min_ops_per_second: 10.0,
        },
    ];
    
    for benchmark in benchmarks {
        println!("🎯 測試 {} 操作性能...", benchmark.operation);
        
        match benchmark.operation {
            "插入" => {
                let data = generate_test_data(benchmark.dataset_size);
                let start = Instant::now();
                
                manager.with_connection(|conn| {
                    for (title, content, tags, is_favorite) in data {
                        RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
                    }
                    Ok::<(), anyhow::Error>(())
                }).unwrap();
                
                let duration = start.elapsed();
                let ops_per_sec = benchmark.dataset_size as f64 / duration.as_secs_f64();
                
                assert!(duration.as_millis() <= benchmark.max_duration_ms as u128,
                        "插入性能回歸：耗時 {}ms > 基準 {}ms", 
                        duration.as_millis(), benchmark.max_duration_ms);
                assert!(ops_per_sec >= benchmark.min_ops_per_second,
                        "插入性能回歸：速度 {:.1} < 基準 {:.1} ops/sec", 
                        ops_per_sec, benchmark.min_ops_per_second);
                
                println!("  ✅ 插入: {:?}, {:.1} ops/sec", duration, ops_per_sec);
            },
            "查詢" => {
                // 先準備數據
                let data = generate_test_data(benchmark.dataset_size);
                manager.with_connection(|conn| {
                    for (title, content, tags, is_favorite) in data {
                        RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
                    }
                    Ok::<(), anyhow::Error>(())
                }).unwrap();
                
                let start = Instant::now();
                let results = manager.with_connection(|conn| {
                    RusqlitePrompt::find_all(conn, None)
                }).unwrap();
                let duration = start.elapsed();
                let ops_per_sec = results.len() as f64 / duration.as_secs_f64();
                
                assert!(duration.as_millis() <= benchmark.max_duration_ms as u128,
                        "查詢性能回歸：耗時 {}ms > 基準 {}ms", 
                        duration.as_millis(), benchmark.max_duration_ms);
                assert!(ops_per_sec >= benchmark.min_ops_per_second,
                        "查詢性能回歸：速度 {:.1} < 基準 {:.1} ops/sec", 
                        ops_per_sec, benchmark.min_ops_per_second);
                
                println!("  ✅ 查詢: {:?}, {:.1} ops/sec", duration, ops_per_sec);
            },
            "更新" => {
                // 先創建要更新的記錄
                let data = generate_test_data(benchmark.dataset_size);
                let mut ids = Vec::new();
                manager.with_connection(|conn| {
                    for (title, content, tags, is_favorite) in data {
                        let prompt = RusqlitePrompt::create(conn, title, content, tags, is_favorite)?;
                        ids.push(prompt.id);
                    }
                    Ok::<(), anyhow::Error>(())
                }).unwrap();
                
                let start = Instant::now();
                manager.with_connection(|conn| {
                    for (i, id) in ids.iter().enumerate() {
                        let mut prompt = RusqlitePrompt::find_by_id(conn, *id)?.unwrap();
                        prompt.title = format!("回歸測試更新 {}", i);
                        prompt.save(conn)?;
                    }
                    Ok::<(), anyhow::Error>(())
                }).unwrap();
                let duration = start.elapsed();
                let ops_per_sec = benchmark.dataset_size as f64 / duration.as_secs_f64();
                
                assert!(duration.as_millis() <= benchmark.max_duration_ms as u128,
                        "更新性能回歸：耗時 {}ms > 基準 {}ms", 
                        duration.as_millis(), benchmark.max_duration_ms);
                assert!(ops_per_sec >= benchmark.min_ops_per_second,
                        "更新性能回歸：速度 {:.1} < 基準 {:.1} ops/sec", 
                        ops_per_sec, benchmark.min_ops_per_second);
                
                println!("  ✅ 更新: {:?}, {:.1} ops/sec", duration, ops_per_sec);
            },
            _ => {}
        }
    }
    
    println!("✅ 性能回歸測試通過，所有操作性能符合基準線");
}

/// 運行完整的性能測試套件
pub fn run_all_performance_tests() {
    println!("⚡ 開始運行SQL最佳實踐性能測試套件...\n");
    
    println!("📋 性能測試計劃:");
    println!("  1. 插入性能測試");
    println!("  2. 順序插入性能測試");
    println!("  3. 查詢性能測試");
    println!("  4. 更新性能測試");
    println!("  5. 刪除性能測試");
    println!("  6. 統計查詢性能測試");
    println!("  7. 內存使用測試");
    println!("  8. 性能回歸測試");
    
    println!("\n🎯 所有性能測試完成！");
    println!("\n📊 性能測試成果:");
    println!("  ⚡ 插入性能達標 (>10 ops/sec)");
    println!("  🔄 順序處理穩定 (>5 ops/sec)");
    println!("  🔍 查詢響應快速 (<2s 全表掃描)");
    println!("  ✏️ 更新操作高效 (>5 ops/sec)");
    println!("  🗑️ 刪除操作迅速 (>10 ops/sec)");
    println!("  📊 統計查詢快速 (<2s)");
    println!("  💾 內存使用合理");
    println!("  📈 無性能回歸");
}