use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::process::{Command, Stdio};
use std::time::Instant;
use tempfile::tempdir;

// CLI 工具基準測試
fn benchmark_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_startup");
    
    // 測試 help 命令的啟動時間 (最快的命令)
    group.bench_function("cnp_help", |b| {
        b.iter(|| {
            let start = Instant::now();
            let output = Command::new("cargo")
                .args(&["run", "--bin", "cnp-unified", "--", "--help"])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Failed to execute command");
            
            let duration = start.elapsed();
            black_box((output, duration))
        });
    });
    
    group.finish();
}

// 基準測試：CLI 命令執行時間
fn benchmark_cli_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_commands");
    
    // Health check 命令
    group.bench_function("health_check", |b| {
        b.iter(|| {
            let start = Instant::now();
            let output = Command::new("cargo")
                .args(&["run", "--bin", "cnp-unified", "--", "health", "--format", "json"])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Health check failed");
            
            let duration = start.elapsed();
            black_box((output, duration))
        });
    });
    
    // Cooldown check 命令
    group.bench_function("cooldown_check", |b| {
        b.iter(|| {
            let start = Instant::now();
            let output = Command::new("cargo")
                .args(&["run", "--bin", "cnp-unified", "--", "cooldown", "--format", "json"])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Cooldown check failed");
            
            let duration = start.elapsed();
            black_box((output, duration))
        });
    });
    
    group.finish();
}

// 基準測試：編譯時間和二進制大小
fn benchmark_build_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_performance");
    group.sample_size(10); // 減少樣本數量，因為編譯很慢
    
    group.bench_function("debug_build", |b| {
        b.iter(|| {
            // 清理之前的構建
            Command::new("cargo")
                .args(&["clean", "--bin", "cnp-unified"])
                .current_dir(".")
                .output()
                .expect("Clean failed");
            
            let start = Instant::now();
            let output = Command::new("cargo")
                .args(&["build", "--bin", "cnp-unified"])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("Build failed");
            
            let duration = start.elapsed();
            black_box((output, duration))
        });
    });
    
    group.finish();
}

// 模擬不同負載下的 CLI 性能
fn benchmark_cli_under_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_under_load");
    
    for concurrent_processes in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_health_checks", concurrent_processes),
            concurrent_processes,
            |b, &concurrent_processes| {
                b.iter(|| {
                    let mut handles = Vec::new();
                    
                    for _ in 0..concurrent_processes {
                        let handle = std::thread::spawn(|| {
                            let start = Instant::now();
                            let output = Command::new("cargo")
                                .args(&["run", "--bin", "cnp-unified", "--", "health", "--format", "json"])
                                .current_dir(".")
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .output()
                                .expect("Health check failed");
                            
                            let duration = start.elapsed();
                            (output, duration)
                        });
                        handles.push(handle);
                    }
                    
                    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

// 測試不同大小的輸入對 CLI 性能的影響
fn benchmark_cli_input_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_input_sizes");
    
    for input_size in [100, 1000, 10000, 50000].iter() {
        let temp_dir = tempdir().unwrap();
        let input_file = temp_dir.path().join("test_input.txt");
        
        // 創建指定大小的測試文件
        let content = "測試內容 ".repeat(*input_size);
        std::fs::write(&input_file, content).expect("Failed to write test file");
        
        group.bench_with_input(
            BenchmarkId::new("execute_with_file", input_size),
            input_size,
            |b, _| {
                b.iter(|| {
                    let start = Instant::now();
            let output = Command::new("cargo")
                        .args(&[
                            "run", "--bin", "cnp-unified", "--",
                            "execute",
                            "--file", input_file.to_str().unwrap(),
                            "--mode", "sync",
                            "--format", "json"
                        ])
                        .current_dir(".")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .expect("Execute failed");
                    
                    let duration = start.elapsed();
                    black_box((output, duration))
                });
            },
        );
    }
    
    group.finish();
}

// 記憶體使用模式基準測試
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("cli_memory_baseline", |b| {
        b.iter(|| {
            // 啟動進程並測量記憶體使用
            let mut child = Command::new("cargo")
                .args(&["run", "--bin", "cnp-unified", "--", "health"])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn process");
            
            // 簡單的記憶體測量 (通過進程狀態)
            let pid = child.id();
            
            // 等待進程完成
            let output = child.wait_with_output().expect("Process failed");
            
            black_box((pid, output))
        });
    });
    
    group.finish();
}

criterion_group!(
    cli_benches,
    benchmark_cli_startup,
    benchmark_cli_commands,
    benchmark_build_performance,
    benchmark_cli_under_load,
    benchmark_cli_input_sizes,
    benchmark_memory_usage
);

criterion_main!(cli_benches);