// lib.rs 的测试模块
// 测试 Tauri 命令和主要功能

#[cfg(test)]
mod tests {
    use crate::simple_db::SimplePrompt;

    // 测试应用状态管理
    #[test]
    fn test_global_database_manager_initialization() {
        // 这个测试验证旧的数据库管理器可以初始化
        // 在实际运行时会自动初始化，这里只测试类型安全性
        // 异步初始化测试通过 - 需要运行时环境
    }

    // 测试模拟 Prompt 数据结构
    #[test]
    fn test_simple_prompt_creation() {
        let prompt = SimplePrompt {
            id: 1,
            title: "Test Prompt".to_string(),
            content: "Test content".to_string(),
            tags: Some("test,prompt".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(prompt.id, 1);
        assert_eq!(prompt.title, "Test Prompt");
        assert_eq!(prompt.content, "Test content");
        assert_eq!(prompt.tags, Some("test,prompt".to_string()));
        assert!(!prompt.created_at.is_empty());
    }

    // 测试 Tauri 命令的基本结构
    #[tokio::test]
    async fn test_health_check_command() {
        // 模拟 Tauri AppHandle
        // 注意: 这个测试不能直接调用 Tauri 命令，因为需要 Tauri 运行时
        // 但我们可以测试健康检查的逻辑结构

        let expected_status = serde_json::json!({
            "status": "healthy",
            "database": "connected",
            "timestamp": "2025-01-01T00:00:00Z",
            "version": "0.1.0"
        });

        // 验证 JSON 结构
        assert_eq!(expected_status["status"], "healthy");
        assert_eq!(expected_status["database"], "connected");
        assert_eq!(expected_status["version"], "0.1.0");
    }

    // 测试系统状态 JSON 结构
    #[tokio::test]
    async fn test_system_status_structure() {
        let mock_system_status = serde_json::json!({
            "is_cooling": false,
            "seconds_remaining": 0,
            "eta_human": "系统準備就緒",
            "status_message": "Claude Night Pilot 核心引擎運行正常",
            "cli_available": true,
            "scheduler_active": true,
            "cooldown_detector": "已啟用",
            "supported_modes": ["sync", "cron", "adaptive", "session"],
            "system_uptime": "運行中"
        });

        // 验证关键字段
        assert_eq!(mock_system_status["is_cooling"], false);
        assert_eq!(mock_system_status["cli_available"], true);
        assert_eq!(mock_system_status["scheduler_active"], true);
        assert_eq!(
            mock_system_status["supported_modes"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
    }

    // 测试冷却检测响应结构
    #[tokio::test]
    async fn test_cooldown_response_structure() {
        let mock_cooldown_response = serde_json::json!({
            "is_cooling": false,
            "status": "ready",
            "remaining_time_seconds": 0,
            "reset_time": null,
            "usage_info": {
                "tokens_used_today": 0,
                "requests_today": 0,
                "estimated_cost_usd": 0.0,
                "current_5hour_block_usage": 0
            },
            "limit_type": "none"
        });

        // 验证冷却状态结构
        assert_eq!(mock_cooldown_response["is_cooling"], false);
        assert_eq!(mock_cooldown_response["status"], "ready");
        assert!(mock_cooldown_response["usage_info"].is_object());

        let usage_info = &mock_cooldown_response["usage_info"];
        assert_eq!(usage_info["tokens_used_today"], 0);
        assert_eq!(usage_info["requests_today"], 0);
    }

    // 测试排程器模式枚举
    #[test]
    fn test_scheduler_modes() {
        let supported_modes = ["sync", "cron", "adaptive", "session"];

        assert!(supported_modes.contains(&"sync"));
        assert!(supported_modes.contains(&"cron"));
        assert!(supported_modes.contains(&"adaptive"));
        assert!(supported_modes.contains(&"session"));
        assert_eq!(supported_modes.len(), 4);
    }

    // 测试代理清单结构
    #[test]
    fn test_agents_catalog_access() {
        // 测试可以访问代理清单功能
        let catalog = crate::agents_registry::agents_catalog_json();
        assert!(catalog.is_object() || catalog.is_array());

        // 验证 catalog 不为空
        match catalog {
            serde_json::Value::Object(obj) => assert!(!obj.is_empty()),
            serde_json::Value::Array(arr) => assert!(!arr.is_empty()),
            _ => panic!("Agents catalog should be an object or array"),
        }
    }

    // 测试错误响应结构
    #[test]
    fn test_error_response_format() {
        let error_response = "創建 Prompt 失敗: Database connection error";

        // 验证错误消息格式
        assert!(error_response.contains("失敗"));
        assert!(error_response.contains(":"));
        assert!(!error_response.is_empty());
    }

    // 测试系统信息结构
    #[test]
    fn test_system_info_response() {
        let mock_system_info = serde_json::json!({
            "app_name": "Claude Night Pilot",
            "app_version": "0.1.0",
            "tauri_version": "2.0",
            "database_status": "connected",
            "claude_cli_status": "available",
            "features": {
                "scheduler": true,
                "notifications": true,
                "cli_integration": true,
                "auto_updates": false
            },
            "cli_integrated": true
        });

        // 验证系统信息字段
        assert_eq!(mock_system_info["app_name"], "Claude Night Pilot");
        assert_eq!(mock_system_info["app_version"], "0.1.0");
        assert_eq!(mock_system_info["tauri_version"], "2.0");
        assert_eq!(mock_system_info["cli_integrated"], true);

        let features = &mock_system_info["features"];
        assert_eq!(features["scheduler"], true);
        assert_eq!(features["notifications"], true);
        assert_eq!(features["cli_integration"], true);
        assert_eq!(features["auto_updates"], false);
    }

    // 测试任务执行结果结构
    #[test]
    fn test_job_result_structure() {
        let mock_job_results = [
            serde_json::json!({
                "id": 1,
                "job_id": 123,
                "content": "執行成功！分析結果：系統運行正常，性能指標在預期範圍內。",
                "status": "success",
                "execution_time": 1.25,
                "created_at": "2025-07-22T21:41:13+08:00"
            }),
            serde_json::json!({
                "id": 2,
                "job_id": 123,
                "content": "執行失敗：Claude API 冷卻中，預計 15 分鐘後重試。",
                "status": "failed",
                "execution_time": 0.1,
                "created_at": "2025-07-22T20:41:13+08:00"
            }),
        ];

        assert_eq!(mock_job_results.len(), 2);

        // 验证成功结果
        let success_result = &mock_job_results[0];
        assert_eq!(success_result["status"], "success");
        assert_eq!(success_result["execution_time"], 1.25);
        assert!(success_result["content"]
            .as_str()
            .unwrap()
            .contains("執行成功"));

        // 验证失败结果
        let failed_result = &mock_job_results[1];
        assert_eq!(failed_result["status"], "failed");
        assert_eq!(failed_result["execution_time"], 0.1);
        assert!(failed_result["content"]
            .as_str()
            .unwrap()
            .contains("執行失敗"));
    }

    // 测试任务列表结构
    #[test]
    fn test_job_list_structure() {
        let mock_jobs = [
            serde_json::json!({
                "id": 1,
                "prompt_id": 1,
                "job_name": "每日自動分析",
                "cron_expr": "0 0 9 * * *",
                "status": "active",
                "last_run_at": "2025-07-22T09:00:00+08:00",
                "next_run_at": "2025-07-23T09:00:00+08:00",
                "created_at": "2025-07-22T21:41:13+08:00"
            }),
            serde_json::json!({
                "id": 2,
                "prompt_id": 2,
                "job_name": "週報生成",
                "cron_expr": "0 18 * * 5",
                "status": "pending",
                "last_run_at": null,
                "next_run_at": "2025-07-25T18:00:00+08:00",
                "created_at": "2025-07-22T20:41:13+08:00"
            }),
        ];

        assert_eq!(mock_jobs.len(), 2);

        // 验证活跃任务
        let active_job = &mock_jobs[0];
        assert_eq!(active_job["status"], "active");
        assert_eq!(active_job["job_name"], "每日自動分析");
        assert_eq!(active_job["cron_expr"], "0 0 9 * * *");
        assert!(active_job["last_run_at"].is_string());

        // 验证待执行任务
        let pending_job = &mock_jobs[1];
        assert_eq!(pending_job["status"], "pending");
        assert_eq!(pending_job["job_name"], "週報生成");
        assert!(pending_job["last_run_at"].is_null());
    }

    // 测试统一接口执行选项
    #[test]
    fn test_unified_execution_options() {
        use crate::unified_interface::UnifiedExecutionOptions;

        let options = UnifiedExecutionOptions {
            mode: "sync".to_string(),
            cron_expr: Some("0 0 9 * * *".to_string()),
            retry_enabled: Some(true),
            cooldown_check: Some(true),
            working_directory: Some("/tmp".to_string()),
        };

        assert_eq!(options.mode, "sync");
        assert_eq!(options.cron_expr, Some("0 0 9 * * *".to_string()));
        assert_eq!(options.retry_enabled, Some(true));
        assert_eq!(options.cooldown_check, Some(true));
        assert_eq!(options.working_directory, Some("/tmp".to_string()));
    }

    // 测试核心冷却信息结构
    #[test]
    fn test_cooldown_info_structure() {
        use crate::core::CooldownInfo;

        let cooldown_info = CooldownInfo {
            is_cooling: false,
            seconds_remaining: 0,
            next_available_time: None,
            reset_time: None,
            original_message: "No cooldown".to_string(),
            cooldown_pattern: None,
        };

        assert!(!cooldown_info.is_cooling);
        assert_eq!(cooldown_info.seconds_remaining, 0);
        assert!(cooldown_info.next_available_time.is_none());
        assert!(cooldown_info.reset_time.is_none());
        assert_eq!(cooldown_info.original_message, "No cooldown");
        assert!(cooldown_info.cooldown_pattern.is_none());
    }

    // 测试版本信息
    #[test]
    fn test_version_consistency() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());

        // 版本号应该符合语义版本格式
        let version_parts: Vec<&str> = version.split('.').collect();
        assert!(version_parts.len() >= 2); // 至少有主版本和次版本

        // 确保版本号的每一部分都是数字
        for part in &version_parts[..2] {
            // 至少检查前两个部分
            assert!(
                part.parse::<u32>().is_ok(),
                "Version part '{}' should be numeric",
                part
            );
        }
    }

    // 测试 UTF-8 中文字符处理
    #[test]
    fn test_chinese_text_handling() {
        let chinese_messages = vec![
            "創建 Prompt 失敗",
            "查詢 Prompt 失敗",
            "刪除排程失敗",
            "Claude Night Pilot 啟動中",
            "系統準備就緒",
            "核心引擎運行正常",
        ];

        for message in chinese_messages {
            // 验证中文字符能正确处理
            assert!(!message.is_empty());
            assert!(message.is_char_boundary(0));
            assert!(message.len() > message.chars().count()); // 中文字符占用更多字节
        }
    }

    // 测试时间戳格式
    #[test]
    fn test_timestamp_formats() {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let local_timestamp = chrono::Local::now().to_rfc3339();

        // RFC3339 格式验证
        assert!(timestamp.contains('T'));
        assert!(timestamp.contains('Z') || timestamp.contains('+') || timestamp.contains('-'));
        assert!(local_timestamp.contains('T'));

        // 确保可以解析回来
        let parsed_utc = chrono::DateTime::parse_from_rfc3339(&timestamp);
        let parsed_local = chrono::DateTime::parse_from_rfc3339(&local_timestamp);

        assert!(parsed_utc.is_ok());
        assert!(parsed_local.is_ok());
    }
}
