use claude_night_pilot_lib::{
    database_manager::{DatabaseError, DatabaseManager},
    simple_db::SimpleDatabase,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;

// 基準測試：數據庫初始化性能
fn benchmark_database_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("database_initialization");

    group.bench_function("simple_database_new", |b| {
        b.iter(|| {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("bench_init.db");
            let _db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();
        });
    });

    group.bench_function("database_manager_new", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let temp_dir = tempdir().unwrap();
            let db_path = temp_dir.path().join("bench_manager.db");
            let _db = DatabaseManager::new(db_path.to_str().unwrap())
                .await
                .unwrap();
        });
    });

    group.finish();
}

// 基準測試：CRUD 操作性能
fn benchmark_crud_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crud_operations");
    let rt = Runtime::new().unwrap();

    // 設置測試數據庫
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_crud.db");
    let db = rt.block_on(async {
        DatabaseManager::new(db_path.to_str().unwrap())
            .await
            .unwrap()
    });
    let db = Arc::new(db);

    // Prompt CRUD 性能
    group.bench_function("create_prompt_sync", |b| {
        let direct_db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let title = format!("基準測試標題 {}", counter);
            let content = format!(
                "基準測試內容 {} - 這是一個相對長的測試內容，用於模擬實際使用情況",
                counter
            );
            black_box(direct_db.create_prompt(&title, &content).unwrap())
        });
    });

    group.bench_function("create_prompt_async", |b| {
        let db = db.clone();
        let mut counter = 0;
        b.to_async(&rt).iter(|| {
            counter += 1;
            let db = db.clone();
            let title = format!("異步基準測試標題 {}", counter);
            let content = format!("異步基準測試內容 {} - 模擬實際使用情況的長內容", counter);
            async move { black_box(db.create_prompt_async(title, content).await.unwrap()) }
        });
    });

    // 批量創建性能
    for batch_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_create_prompts", batch_size),
            batch_size,
            |b, &batch_size| {
                let db = db.clone();
                b.to_async(&rt).iter(|| {
                    let db = db.clone();
                    async move {
                        for i in 0..batch_size {
                            let title = format!("批量標題 {}", i);
                            let content = format!("批量內容 {} - 測試批量插入性能", i);
                            black_box(db.create_prompt_async(title, content).await.unwrap());
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

// 基準測試：並發操作性能
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_concurrent.db");
    let db = rt.block_on(async {
        DatabaseManager::new(db_path.to_str().unwrap())
            .await
            .unwrap()
    });
    let db = Arc::new(db);

    for concurrency in [1, 2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_writes", concurrency),
            concurrency,
            |b, &concurrency| {
                let db = db.clone();
                b.to_async(&rt).iter(|| {
                    let db = db.clone();
                    async move {
                        let mut handles = Vec::new();

                        for i in 0..concurrency {
                            let db = db.clone();
                            let handle = tokio::spawn(async move {
                                let title = format!("並發測試 {}", i);
                                let content = format!("並發內容 {} - 測試並發寫入性能", i);
                                db.create_prompt_async(title, content).await.unwrap()
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            black_box(handle.await.unwrap());
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

// 基準測試：Schedule 操作性能
fn benchmark_schedule_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("schedule_operations");
    let rt = Runtime::new().unwrap();

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_schedule.db");
    let db = rt.block_on(async {
        DatabaseManager::new(db_path.to_str().unwrap())
            .await
            .unwrap()
    });
    let db = Arc::new(db);

    // 預先創建一些 prompts
    let prompt_ids: Vec<i64> = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..100 {
            let title = format!("排程測試 Prompt {}", i);
            let content = format!("排程測試內容 {}", i);
            let id = db.create_prompt_async(title, content).await.unwrap();
            ids.push(id);
        }
        ids
    });

    group.bench_function("create_schedule", |b| {
        let db = db.clone();
        let mut counter = 0;
        b.to_async(&rt).iter(|| {
            counter += 1;
            let db = db.clone();
            let prompt_id = prompt_ids[counter % prompt_ids.len()];
            async move {
                let schedule_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
                black_box(
                    db.create_schedule_async(
                        prompt_id,
                        schedule_time,
                        Some("0 */6 * * *".to_string()),
                    )
                    .await
                    .unwrap(),
                )
            }
        });
    });

    group.bench_function("get_pending_schedules", |b| {
        let db = db.clone();
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            async move { black_box(db.get_pending_schedules_async().await.unwrap()) }
        });
    });

    group.finish();
}

// 基準測試：Token 統計性能
fn benchmark_token_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_operations");
    let rt = Runtime::new().unwrap();

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_tokens.db");
    let db = rt.block_on(async {
        DatabaseManager::new(db_path.to_str().unwrap())
            .await
            .unwrap()
    });
    let db = Arc::new(db);

    group.bench_function("update_token_stats", |b| {
        let db = db.clone();
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            async move {
                black_box(
                    db.update_token_usage_stats_async(1000, 500, 0.05)
                        .await
                        .unwrap(),
                )
            }
        });
    });

    group.bench_function("get_token_stats", |b| {
        let db = db.clone();
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            async move { black_box(db.get_token_usage_stats_async().await.unwrap()) }
        });
    });

    group.finish();
}

// 基準測試：健康檢查和統計性能
fn benchmark_system_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_operations");
    let rt = Runtime::new().unwrap();

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_system.db");
    let db = rt.block_on(async {
        DatabaseManager::new(db_path.to_str().unwrap())
            .await
            .unwrap()
    });
    let db = Arc::new(db);

    group.bench_function("health_check", |b| {
        let db = db.clone();
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            async move { black_box(db.health_check().await.unwrap()) }
        });
    });

    group.bench_function("get_stats", |b| {
        let db = db.clone();
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            async move { black_box(db.get_stats().await.unwrap()) }
        });
    });

    group.finish();
}

criterion_group!(
    database_benches,
    benchmark_database_initialization,
    benchmark_crud_operations,
    benchmark_concurrent_operations,
    benchmark_schedule_operations,
    benchmark_token_operations,
    benchmark_system_operations
);

criterion_main!(database_benches);
