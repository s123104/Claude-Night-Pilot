use claude_night_pilot_lib::simple_db::SimpleDatabase;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use std::thread;
use tempfile::tempdir;

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

    group.finish();
}

// 基準測試：基本 CRUD 操作性能
fn benchmark_crud_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("crud_operations");

    // 設置測試數據庫
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_crud.db");
    let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

    group.bench_function("create_prompt_sync", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let title = format!("基準測試標題 {}", counter);
            let content = format!(
                "基準測試內容 {} - 這是一個相對長的測試內容，用於模擬實際使用情況",
                counter
            );
            black_box(db.create_prompt(&title, &content).unwrap())
        });
    });

    // 批量創建性能
    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_create_prompts", batch_size),
            batch_size,
            |b, &batch_size| {
                let temp_dir = tempdir().unwrap();
                let db_path = temp_dir.path().join("bench_batch.db");
                let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

                b.iter(|| {
                    for i in 0..batch_size {
                        let title = format!("批量標題 {}", i);
                        let content = format!("批量內容 {} - 測試批量插入性能", i);
                        black_box(db.create_prompt(&title, &content).unwrap());
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

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_schedule.db");
    let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

    // 預先創建一些 prompts
    let mut prompt_ids = Vec::new();
    for i in 0..10 {
        let title = format!("排程測試 Prompt {}", i);
        let content = format!("排程測試內容 {}", i);
        let id = db.create_prompt(&title, &content).unwrap();
        prompt_ids.push(id);
    }

    group.bench_function("create_schedule", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let prompt_id = prompt_ids[counter % prompt_ids.len()];
            let schedule_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            black_box(
                db.create_schedule(prompt_id, &schedule_time, Some("0 */6 * * *"))
                    .unwrap(),
            )
        });
    });

    group.bench_function("get_pending_schedules", |b| {
        b.iter(|| black_box(db.get_pending_schedules().unwrap()));
    });

    group.finish();
}

// 基準測試：Token 統計性能
fn benchmark_token_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_operations");

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_tokens.db");
    let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

    group.bench_function("update_token_stats", |b| {
        b.iter(|| black_box(db.update_token_usage_stats(1000, 500, 0.05).unwrap()));
    });

    group.bench_function("get_token_stats", |b| {
        b.iter(|| black_box(db.get_token_usage_stats().unwrap()));
    });

    group.finish();
}

// 基準測試：序列操作性能 (模擬並發場景)
fn benchmark_sequential_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_operations");

    for operation_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("sequential_writes", operation_count),
            operation_count,
            |b, &operation_count| {
                b.iter(|| {
                    let temp_dir = tempdir().unwrap();
                    let db_path = temp_dir.path().join("bench_sequential.db");
                    let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

                    let mut results = Vec::new();

                    for i in 0..operation_count {
                        let title = format!("序列測試 {}", i);
                        let content = format!("序列內容 {} - 測試序列寫入性能", i);
                        let id = db.create_prompt(&title, &content).unwrap();
                        results.push(id);
                    }

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

// 基準測試：記錄執行結果的性能
fn benchmark_execution_results(c: &mut Criterion) {
    let mut group = c.benchmark_group("execution_results");

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench_results.db");
    let db = SimpleDatabase::new(db_path.to_str().unwrap()).unwrap();

    // 預先創建一個 schedule
    let prompt_id = db.create_prompt("測試 Prompt", "測試內容").unwrap();
    let schedule_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let schedule_id = db
        .create_schedule(prompt_id, &schedule_time, Some("0 */6 * * *"))
        .unwrap();

    group.bench_function("record_execution_result", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let content = format!("執行結果 {} - 模擬實際執行輸出", counter);
            let status = "completed";
            let token_usage = Some(1000 + counter);
            let cost_usd = Some(0.05);
            let execution_time_ms = 2500;

            black_box(
                db.record_execution_result(
                    schedule_id,
                    &content,
                    status,
                    token_usage,
                    cost_usd,
                    execution_time_ms,
                )
                .unwrap(),
            )
        });
    });

    group.finish();
}

criterion_group!(
    simple_benches,
    benchmark_database_initialization,
    benchmark_crud_operations,
    benchmark_schedule_operations,
    benchmark_token_operations,
    benchmark_sequential_operations,
    benchmark_execution_results
);

criterion_main!(simple_benches);
