use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use claude_night_pilot_lib::database_manager_impl::{DatabaseManager, DatabaseConfig};
use claude_night_pilot_lib::services::{
    prompt_service::PromptService,
    health_service::HealthService,
    sync_service::SyncService,
};
use tempfile::tempdir;
use std::sync::Arc;

/// 基準測試：應用程式啟動時間
fn benchmark_application_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_performance");
    
    // 測試資料庫初始化時間
    group.bench_function("database_initialization", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("startup_bench.db");
            
            let config = DatabaseConfig {
                path: db_path.to_string_lossy().to_string(),
                enable_foreign_keys: true,
                wal_mode: false, // 測試中使用較簡單的模式
                synchronous_mode: "NORMAL".to_string(),
            };
            
            black_box(DatabaseManager::new(config).await.unwrap())
        });
    });
    
    // 測試服務初始化時間
    group.bench_function("service_initialization", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let prompt_service = PromptService::new().await.unwrap();
            let health_service = HealthService::new();
            let sync_service = SyncService::new();
            
            black_box((prompt_service, health_service, sync_service))
        });
    });
    
    // 測試完整應用啟動流程
    group.bench_function("complete_application_startup", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            // 1. 資料庫初始化
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("complete_startup_bench.db");
            
            let config = DatabaseConfig {
                path: db_path.to_string_lossy().to_string(),
                enable_foreign_keys: true,
                wal_mode: false,
                synchronous_mode: "NORMAL".to_string(),
            };
            
            let _db_manager = DatabaseManager::new(config).await.unwrap();
            
            // 2. 服務初始化
            let prompt_service = PromptService::new().await.unwrap();
            let health_service = HealthService::new();
            let sync_service = SyncService::new();
            
            black_box((prompt_service, health_service, sync_service))
        });
    });
    
    group.finish();
}

/// 基準測試：記憶體使用效率
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // 測試多個服務實例的記憶體效率
    for instance_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiple_service_instances", instance_count),
            instance_count,
            |b, &instance_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                    let mut services = Vec::new();
                    
                    for _ in 0..instance_count {
                        let prompt_service = PromptService::new().await.unwrap();
                        let health_service = HealthService::new();
                        let sync_service = SyncService::new();
                        
                        services.push((prompt_service, health_service, sync_service));
                    }
                    
                    black_box(services)
                });
            },
        );
    }
    
    group.finish();
}

/// 基準測試：併發性能
fn benchmark_concurrent_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_performance");
    
    // 測試併發服務創建
    for concurrent_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_service_creation", concurrent_count),
            concurrent_count,
            |b, &concurrent_count| {
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                    let handles: Vec<_> = (0..concurrent_count).map(|_| {
                        tokio::spawn(async {
                            let prompt_service = PromptService::new().await.unwrap();
                            let health_service = HealthService::new();
                            let sync_service = SyncService::new();
                            
                            (prompt_service, health_service, sync_service)
                        })
                    }).collect();
                    
                    let mut results = Vec::new();
                    for handle in handles {
                        results.push(handle.await.unwrap());
                    }
                    
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

/// 基準測試：冷啟動 vs 熱啟動
fn benchmark_cold_vs_warm_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_vs_warm_startup");
    
    // 冷啟動：每次都創建新的資料庫
    group.bench_function("cold_startup", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("cold_startup.db");
            
            let config = DatabaseConfig {
                path: db_path.to_string_lossy().to_string(),
                enable_foreign_keys: true,
                wal_mode: false,
                synchronous_mode: "NORMAL".to_string(),
            };
            
            let db_manager = DatabaseManager::new(config).await.unwrap();
            let prompt_service = PromptService::new().await.unwrap();
            
            black_box((db_manager, prompt_service))
        });
    });
    
    // 模擬熱啟動：重複使用相同的資料庫路徑
    let temp_dir = tempdir().unwrap();
    let warm_db_path = temp_dir.path().join("warm_startup.db");
    let warm_path_str = warm_db_path.to_string_lossy().to_string();
    
    group.bench_function("warm_startup", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let config = DatabaseConfig {
                path: warm_path_str.clone(),
                enable_foreign_keys: true,
                wal_mode: false,
                synchronous_mode: "NORMAL".to_string(),
            };
            
            let db_manager = DatabaseManager::new(config).await.unwrap();
            let prompt_service = PromptService::new().await.unwrap();
            
            black_box((db_manager, prompt_service))
        });
    });
    
    group.finish();
}

/// 基準測試：配置選項對性能的影響
fn benchmark_configuration_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_impact");
    
    let configs = vec![
        ("default", DatabaseConfig::default()),
        ("fast_unsafe", DatabaseConfig {
            path: "bench_fast.db".to_string(),
            enable_foreign_keys: false,
            wal_mode: false,
            synchronous_mode: "OFF".to_string(),
        }),
        ("secure_slow", DatabaseConfig {
            path: "bench_secure.db".to_string(),
            enable_foreign_keys: true,
            wal_mode: true,
            synchronous_mode: "FULL".to_string(),
        }),
    ];
    
    for (config_name, config) in configs {
        group.bench_function(config_name, |b| {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join(format!("{}.db", config_name));
            
            let mut test_config = config;
            test_config.path = db_path.to_string_lossy().to_string();
            
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                let db_manager = DatabaseManager::new(test_config.clone()).await.unwrap();
                black_box(db_manager)
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    startup_benches,
    benchmark_application_startup,
    benchmark_memory_efficiency,
    benchmark_concurrent_performance,
    benchmark_cold_vs_warm_startup,
    benchmark_configuration_impact
);

criterion_main!(startup_benches);