// 📝 Claude Night Pilot - 企業級結構化日誌系統
// 基於Context7 tracing最佳實踐
// 創建時間: 2025-08-17T05:20:00+00:00

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer, Registry,
};

/// 企業級日誌配置
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// 日誌級別
    pub level: Level,

    /// 日誌格式
    pub format: LogFormat,

    /// 日誌輸出目標
    pub targets: Vec<LogTarget>,

    /// 是否包含檔案位置信息
    pub include_location: bool,

    /// 是否包含執行緒ID
    pub include_thread_id: bool,

    /// 是否啟用顏色輸出
    pub enable_colors: bool,
}

/// 日誌格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// 結構化JSON格式（生產環境）
    Json,

    /// 緊湊格式（開發環境）
    Compact,

    /// 完整格式（調試用）
    Full,

    /// 自定義格式
    Custom(String),
}

/// 日誌輸出目標
#[derive(Debug, Clone)]
pub enum LogTarget {
    /// 標準輸出
    Stdout,

    /// 標準錯誤
    Stderr,

    /// 檔案輸出
    File(PathBuf),

    /// 系統日誌
    Syslog,

    /// 遠程日誌服務
    Remote(String),
}

impl LogConfig {
    /// 創建新的日誌配置
    pub fn new() -> Self {
        LogConfig {
            level: Level::INFO,
            format: LogFormat::Compact,
            targets: vec![LogTarget::Stdout],
            include_location: false,
            include_thread_id: false,
            enable_colors: true,
        }
    }

    /// 開發環境配置
    pub fn development() -> Self {
        LogConfig {
            level: Level::DEBUG,
            format: LogFormat::Full,
            targets: vec![LogTarget::Stdout],
            include_location: true,
            include_thread_id: true,
            enable_colors: true,
        }
    }

    /// 生產環境配置
    pub fn production() -> Self {
        LogConfig {
            level: Level::INFO,
            format: LogFormat::Json,
            targets: vec![
                LogTarget::Stdout,
                LogTarget::File(PathBuf::from("logs/claude-night-pilot.log")),
            ],
            include_location: false,
            include_thread_id: false,
            enable_colors: false,
        }
    }

    /// 測試環境配置
    pub fn testing() -> Self {
        LogConfig {
            level: Level::WARN,
            format: LogFormat::Compact,
            targets: vec![LogTarget::Stdout],
            include_location: false,
            include_thread_id: false,
            enable_colors: false,
        }
    }

    /// 設置日誌級別
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// 設置日誌格式
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// 添加日誌輸出目標
    pub fn add_target(mut self, target: LogTarget) -> Self {
        self.targets.push(target);
        self
    }

    /// 啟用檔案位置信息
    pub fn with_location(mut self) -> Self {
        self.include_location = true;
        self
    }

    /// 啟用執行緒ID
    pub fn with_thread_id(mut self) -> Self {
        self.include_thread_id = true;
        self
    }
}

/// 初始化日誌系統
pub fn init_logging(config: &LogConfig) -> Result<()> {
    // 創建環境過濾器
    let env_filter = EnvFilter::new(format!("claude_night_pilot={}", config.level))
        .add_directive("tokio=info".parse()?)
        .add_directive("rusqlite=warn".parse()?)
        .add_directive("tauri=info".parse()?);

    // 創建基礎訂閱器
    let registry = Registry::default().with(env_filter);

    // 根據目標配置輸出
    let has_stdout = config
        .targets
        .iter()
        .any(|t| matches!(t, LogTarget::Stdout));
    let has_file = config
        .targets
        .iter()
        .any(|t| matches!(t, LogTarget::File(_)));

    if has_stdout {
        // 配置控制台輸出
        let console_layer = match config.format {
            LogFormat::Json => fmt::layer()
                .json()
                .with_timer(UtcTime::rfc_3339())
                .with_thread_ids(config.include_thread_id)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .boxed(),
            LogFormat::Compact => fmt::layer()
                .compact()
                .with_timer(UtcTime::rfc_3339())
                .with_thread_ids(config.include_thread_id)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_ansi(config.enable_colors)
                .boxed(),
            LogFormat::Full => fmt::layer()
                .with_timer(UtcTime::rfc_3339())
                .with_thread_ids(config.include_thread_id)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_ansi(config.enable_colors)
                .boxed(),
            LogFormat::Custom(_) => {
                // TODO: 實現自定義格式
                fmt::layer()
                    .compact()
                    .with_timer(UtcTime::rfc_3339())
                    .boxed()
            }
        };

        registry.with(console_layer).init();
    } else {
        registry.init();
    }

    // 檔案輸出（如果需要）
    if has_file {
        // TODO: 實現檔案輸出配置
        // 這需要額外的檔案 appender 配置
    }

    info!("📝 企業級日誌系統初始化完成");
    info!(
        level = format!("{:?}", config.level),
        format = format!("{:?}", config.format),
        targets_count = config.targets.len(),
        "日誌配置詳情"
    );

    Ok(())
}

/// 日誌宏擴展
#[macro_export]
macro_rules! log_operation {
    ($level:expr, $operation:expr, $result:expr) => {
        match $result {
            Ok(ref value) => {
                tracing::event!(
                    $level,
                    operation = $operation,
                    success = true,
                    "操作成功"
                );
            },
            Err(ref error) => {
                tracing::event!(
                    $level,
                    operation = $operation,
                    success = false,
                    error = %error,
                    "操作失敗"
                );
            }
        }
    };
}

/// 企業級日誌工具
pub struct LoggingUtils;

impl LoggingUtils {
    /// 記錄系統啟動
    pub fn log_system_startup(component: &str, version: &str) {
        info!(
            component = component,
            version = version,
            pid = std::process::id(),
            timestamp = chrono::Utc::now().to_rfc3339(),
            "🚀 系統組件啟動"
        );
    }

    /// 記錄系統關閉
    pub fn log_system_shutdown(component: &str, uptime: std::time::Duration) {
        info!(
            component = component,
            uptime_seconds = uptime.as_secs(),
            timestamp = chrono::Utc::now().to_rfc3339(),
            "🛑 系統組件關閉"
        );
    }

    /// 記錄效能指標
    pub fn log_performance_metrics(
        operation: &str,
        duration: std::time::Duration,
        throughput: Option<f64>,
        memory_usage: Option<u64>,
    ) {
        let mut fields = vec![
            ("operation", operation.to_string()),
            ("duration_ms", duration.as_millis().to_string()),
        ];

        if let Some(tp) = throughput {
            fields.push(("throughput_ops_sec", tp.to_string()));
        }

        if let Some(mem) = memory_usage {
            fields.push(("memory_usage_bytes", mem.to_string()));
        }

        info!(
            operation = operation,
            duration_ms = duration.as_millis(),
            throughput = throughput,
            memory_usage = memory_usage,
            "📊 效能指標"
        );
    }

    /// 記錄安全事件
    pub fn log_security_event(
        event_type: &str,
        severity: &str,
        source: Option<&str>,
        details: Option<&str>,
    ) {
        tracing::warn!(
            event_type = event_type,
            severity = severity,
            source = source,
            details = details,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "🔒 安全事件"
        );
    }

    /// 記錄業務事件
    pub fn log_business_event(event_name: &str, user_id: Option<&str>, context: Option<&str>) {
        info!(
            event = event_name,
            user_id = user_id,
            context = context,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "💼 業務事件"
        );
    }
}

/// 日誌中間件（用於HTTP請求等）
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    /// 記錄請求開始
    pub fn log_request_start(request_id: &str, method: &str, path: &str) {
        info!(
            request_id = request_id,
            method = method,
            path = path,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "📨 請求開始"
        );
    }

    /// 記錄請求完成
    pub fn log_request_end(
        request_id: &str,
        status_code: u16,
        duration: std::time::Duration,
        response_size: Option<usize>,
    ) {
        info!(
            request_id = request_id,
            status_code = status_code,
            duration_ms = duration.as_millis(),
            response_size = response_size,
            timestamp = chrono::Utc::now().to_rfc3339(),
            "📨 請求完成"
        );
    }
}

/// 日誌級別工具
pub struct LogLevel;

impl LogLevel {
    /// 從字符串解析日誌級別
    pub fn from_str(level: &str) -> Result<Level> {
        match level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" | "warning" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => Err(anyhow::anyhow!("無效的日誌級別: {}", level)),
        }
    }

    /// 檢查是否啟用特定級別
    pub fn is_enabled(level: Level) -> bool {
        match level {
            Level::ERROR => tracing::enabled!(tracing::Level::ERROR),
            Level::WARN => tracing::enabled!(tracing::Level::WARN),
            Level::INFO => tracing::enabled!(tracing::Level::INFO),
            Level::DEBUG => tracing::enabled!(tracing::Level::DEBUG),
            Level::TRACE => tracing::enabled!(tracing::Level::TRACE),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}
