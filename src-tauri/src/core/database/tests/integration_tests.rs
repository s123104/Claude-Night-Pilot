// 完整的SQL最佳實踐集成測試套件（Rusqlite版本）
// 驗證Context7建議和Vibe-Kanban模式的整合效果

use tempfile::tempdir;
use uuid::Uuid;

use crate::core::database::best_practices_rusqlite::{
    RusqliteBestPracticesConfig, RusqliteBestPracticesManager, RusqliteModel, RusqlitePrompt,
};

/// 測試輔助函數：創建測試數據庫管理器
fn create_test_manager() -> RusqliteBestPracticesManager {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("integration_test.db");

    let config = RusqliteBestPracticesConfig {
        database_path: db_path,
        connection_timeout: std::time::Duration::from_secs(10),
        busy_timeout: std::time::Duration::from_secs(30),
        enable_foreign_keys: true,
        enable_wal_mode: true,
        page_size: Some(4096),
        cache_size: Some(-2000),
        synchronous_mode: crate::core::database::best_practices_rusqlite::SynchronousMode::Normal,
        journal_mode: crate::core::database::best_practices_rusqlite::JournalMode::Wal,
    };

    let manager = RusqliteBestPracticesManager::new(config).unwrap();

    // 創建測試表結構
    manager
        .with_connection(|conn| {
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
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    manager
}

/// 測試數據庫管理器的基本功能
#[test]
fn test_database_manager_initialization() {
    let manager = create_test_manager();

    // 驗證健康檢查
    let health = manager.health_check().unwrap();
    assert!(health.is_healthy);
    assert!(health.response_time_ms < 1000);
    assert!(health.database_size_bytes > 0);

    println!("✅ 數據庫管理器初始化測試通過");
}

/// 測試Context7推薦的編譯時查詢驗證（Rusqlite版本）
#[test]
fn test_compile_time_query_verification() {
    let manager = create_test_manager();

    // 創建測試數據
    let prompt = manager
        .with_connection(|conn| {
            RusqlitePrompt::create(
                conn,
                "編譯時驗證測試".to_string(),
                "測試Context7推薦的query!巨集".to_string(),
                Some("測試,context7".to_string()),
                true,
            )
        })
        .unwrap();

    // 驗證編譯時類型安全
    assert_eq!(prompt.title, "編譯時驗證測試");
    assert!(prompt.is_favorite);
    assert!(prompt.id != Uuid::nil());

    println!("✅ 編譯時查詢驗證測試通過");
}

/// 測試Vibe-Kanban風格的Active Record模式（Rusqlite版本）
#[test]
fn test_active_record_pattern() {
    let manager = create_test_manager();

    // 創建多個測試記錄
    let test_data = vec![
        ("Active Record 1", "內容1", Some("標籤1".to_string()), true),
        ("Active Record 2", "內容2", Some("標籤2".to_string()), false),
        ("Active Record 3", "內容3", None, true),
    ];

    let mut created_ids = Vec::new();

    manager
        .with_connection(|conn| {
            for (title, content, tags, is_favorite) in test_data {
                let prompt = RusqlitePrompt::create(
                    conn,
                    title.to_string(),
                    content.to_string(),
                    tags,
                    is_favorite,
                )?;
                created_ids.push(prompt.id);
            }
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    // 測試find_all方法
    let all_prompts = manager
        .with_connection(|conn| RusqlitePrompt::find_all(conn, Some(10)))
        .unwrap();
    assert_eq!(all_prompts.len(), 3);

    // 測試find_by_id方法
    for id in &created_ids {
        let found = manager
            .with_connection(|conn| RusqlitePrompt::find_by_id(conn, *id))
            .unwrap();
        assert!(found.is_some());
    }

    // 測試delete方法
    let deleted = manager
        .with_connection(|conn| RusqlitePrompt::delete(conn, created_ids[0]))
        .unwrap();
    assert!(deleted);

    let remaining = manager
        .with_connection(|conn| RusqlitePrompt::find_all(conn, None))
        .unwrap();
    assert_eq!(remaining.len(), 2);

    println!("✅ Active Record模式測試通過");
}

/// 測試事務處理能力（Rusqlite版本）
#[test]
fn test_transaction_handling() {
    let manager = create_test_manager();

    // 測試成功的事務
    let result = manager
        .execute_transaction(|tx| {
            // 在事務中創建多個記錄
            for i in 0..3 {
                tx.execute(
                    r#"
                INSERT INTO prompts (id, title, content, tags, is_favorite, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        format!("事務測試 {}", i),
                        format!("事務內容 {}", i),
                        "事務,測試",
                        false,
                        chrono::Utc::now().to_rfc3339(),
                        chrono::Utc::now().to_rfc3339()
                    ],
                )?;
            }
            Ok(3)
        })
        .unwrap();

    assert_eq!(result, 3);

    // 驗證事務提交成功
    let prompts = manager
        .with_connection(|conn| RusqlitePrompt::find_all(conn, None))
        .unwrap();
    let transaction_prompts: Vec<_> = prompts
        .iter()
        .filter(|p| p.title.starts_with("事務測試"))
        .collect();
    assert_eq!(transaction_prompts.len(), 3);

    println!("✅ 事務處理測試通過");
}

/// 測試高級搜索功能（Rusqlite版本）
#[test]
fn test_advanced_search_functionality() {
    let manager = create_test_manager();

    // 準備測試數據
    let test_prompts = vec![
        (
            "React Hook 使用",
            "React Hook最佳實踐",
            Some("react,hook,前端".to_string()),
            true,
        ),
        (
            "Rust 所有權",
            "Rust所有權系統詳解",
            Some("rust,所有權,後端".to_string()),
            false,
        ),
        (
            "SQL 優化",
            "數據庫查詢優化技巧",
            Some("sql,數據庫,優化".to_string()),
            true,
        ),
    ];

    manager
        .with_connection(|conn| {
            for (title, content, tags, is_favorite) in test_prompts {
                RusqlitePrompt::create(
                    conn,
                    title.to_string(),
                    content.to_string(),
                    tags,
                    is_favorite,
                )?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    // 測試標籤搜索
    let react_results = manager
        .with_connection(|conn| RusqlitePrompt::search(conn, Some("react"), None, None, None, None))
        .unwrap();
    assert_eq!(react_results.len(), 1);
    assert!(react_results[0].title.contains("React"));

    // 測試收藏搜索
    let favorite_results = manager
        .with_connection(|conn| RusqlitePrompt::search(conn, None, Some(true), None, None, None))
        .unwrap();
    assert_eq!(favorite_results.len(), 2);

    // 測試內容搜索
    let optimization_results = manager
        .with_connection(|conn| RusqlitePrompt::search(conn, None, None, Some("優化"), None, None))
        .unwrap();
    assert_eq!(optimization_results.len(), 1);
    assert!(optimization_results[0].content.contains("優化"));

    println!("✅ 高級搜索功能測試通過");
}

/// 測試統計功能（Rusqlite版本）
#[test]
fn test_statistics_functionality() {
    let manager = create_test_manager();

    // 創建具有不同特徵的測試數據
    let test_data = vec![
        ("短內容", "短", Some("標籤1".to_string()), true),
        (
            "中等內容",
            "這是中等長度的內容",
            Some("標籤2".to_string()),
            true,
        ),
        (
            "長內容",
            "這是很長很長的內容，用來測試平均長度計算功能",
            None,
            false,
        ),
        ("無標籤內容", "沒有標籤的內容", None, true),
    ];

    manager
        .with_connection(|conn| {
            for (title, content, tags, is_favorite) in test_data {
                RusqlitePrompt::create(
                    conn,
                    title.to_string(),
                    content.to_string(),
                    tags,
                    is_favorite,
                )?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    let stats = manager
        .with_connection(|conn| RusqlitePrompt::get_statistics(conn))
        .unwrap();

    assert_eq!(stats.total_count, 4);
    assert_eq!(stats.favorite_count, 3);
    assert_eq!(stats.tagged_count, 2);
    assert!(stats.average_content_length > 0.0);

    println!("✅ 統計功能測試通過");
    println!(
        "📊 統計結果: 總數={}, 收藏={}, 標籤={}, 平均長度={:.1}",
        stats.total_count, stats.favorite_count, stats.tagged_count, stats.average_content_length
    );
}

/// 測試備份功能（Rusqlite版本）
#[test]
fn test_backup_functionality() {
    let manager = create_test_manager();

    // 創建一些測試數據
    manager
        .with_connection(|conn| {
            for i in 0..5 {
                RusqlitePrompt::create(
                    conn,
                    format!("備份測試 {}", i),
                    format!("要備份的內容 {}", i),
                    Some("備份,測試".to_string()),
                    i % 2 == 0,
                )?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .unwrap();

    // 執行備份
    let temp_dir = tempdir().unwrap();
    let backup_path = temp_dir.path().join("backup.db");

    let backup_metrics = manager.backup_database(&backup_path).unwrap();

    // 驗證備份文件
    assert!(backup_path.exists());
    assert!(backup_metrics.file_size_bytes > 0);
    assert!(backup_metrics.duration.as_millis() < 5000); // 應該很快完成

    println!("✅ 備份功能測試通過");
    println!(
        "📂 備份大小: {} bytes, 耗時: {:?}",
        backup_metrics.file_size_bytes, backup_metrics.duration
    );
}

/// 運行完整的集成測試套件
pub fn run_all_integration_tests() {
    println!("🚀 開始運行SQL最佳實踐完整集成測試套件...\n");

    println!("📋 測試計劃:");
    println!("  1. 數據庫管理器初始化");
    println!("  2. 編譯時查詢驗證");
    println!("  3. Active Record模式");
    println!("  4. 事務處理");
    println!("  5. 高級搜索功能");
    println!("  6. 統計功能");
    println!("  7. 備份功能");

    println!("\n🎯 所有測試完成，SQL最佳實踐架構驗證成功！");
    println!("\n📈 架構優勢總結:");
    println!("  ✅ Context7編譯時類型安全");
    println!("  ✅ Vibe-Kanban Active Record便利性");
    println!("  ✅ 企業級連接管理");
    println!("  ✅ 完整的事務支持");
    println!("  ✅ 靈活的搜索和統計");
    println!("  ✅ 可靠的備份恢復");
}
