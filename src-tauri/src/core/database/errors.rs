// 统一的数据库错误处理
use thiserror::Error;

/// 统一的数据库错误类型
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("连接失败: {0}")]
    Connection(#[from] rusqlite::Error),

    #[error("数据不存在: {table} id={id}")]
    NotFound { table: String, id: i64 },

    #[error("验证失败: {message}")]
    Validation { message: String },

    #[error("并发冲突: {operation}")]
    Concurrency { operation: String },

    #[error("迁移失败: {version} -> {target_version}: {reason}")]
    Migration {
        version: u32,
        target_version: u32,
        reason: String,
    },

    #[error("事务失败: {reason}")]
    Transaction { reason: String },

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("异步任务失败: {0}")]
    AsyncTask(#[from] tokio::task::JoinError),

    #[error("配置错误: {parameter} = {value}")]
    Configuration { parameter: String, value: String },

    #[error("资源不足: {resource}")]
    ResourceExhaustion { resource: String },

    #[error("权限不足: {operation}")]
    Permission { operation: String },

    #[error("内部错误: {message}")]
    Internal { message: String },
}

/// 数据库操作结果类型
pub type DatabaseResult<T> = Result<T, DatabaseError>;

impl DatabaseError {
    /// 创建不存在错误
    pub fn not_found(table: &str, id: i64) -> Self {
        Self::NotFound {
            table: table.to_string(),
            id,
        }
    }

    /// 创建验证错误
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// 创建并发错误
    pub fn concurrency<S: Into<String>>(operation: S) -> Self {
        Self::Concurrency {
            operation: operation.into(),
        }
    }

    /// 创建事务错误
    pub fn transaction<S: Into<String>>(reason: S) -> Self {
        Self::Transaction {
            reason: reason.into(),
        }
    }

    /// 创建内部错误
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// 检查是否为连接错误
    pub fn is_connection_error(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// 检查是否为致命错误（需要重新连接）
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::Connection(e) => {
                matches!(
                    e,
                    rusqlite::Error::SqliteFailure(_, _)
                        | rusqlite::Error::SqliteSingleThreadedMode
                )
            }
            Self::ResourceExhaustion { .. } => true,
            Self::Internal { .. } => true,
            _ => false,
        }
    }

    /// 获取错误的严重性等级
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::NotFound { .. } => ErrorSeverity::Info,
            Self::Validation { .. } => ErrorSeverity::Warning,
            Self::Concurrency { .. } => ErrorSeverity::Warning,
            Self::Connection(_) => ErrorSeverity::Error,
            Self::Migration { .. } => ErrorSeverity::Critical,
            Self::Transaction { .. } => ErrorSeverity::Error,
            Self::ResourceExhaustion { .. } => ErrorSeverity::Critical,
            Self::Permission { .. } => ErrorSeverity::Error,
            Self::Internal { .. } => ErrorSeverity::Critical,
            _ => ErrorSeverity::Warning,
        }
    }
}

/// 错误严重性等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// 信息性错误（如数据不存在）
    Info,
    /// 警告级错误（可以恢复）
    Warning,
    /// 错误级（需要处理但不致命）
    Error,
    /// 严重错误（可能需要重启或管理员干预）
    Critical,
}

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub table: Option<String>,
    pub record_id: Option<i64>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new<S: Into<String>>(operation: S) -> Self {
        Self {
            operation: operation.into(),
            table: None,
            record_id: None,
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_table<S: Into<String>>(mut self, table: S) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn with_record_id(mut self, id: i64) -> Self {
        self.record_id = Some(id);
        self
    }

    pub fn with_info<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.additional_info.insert(key.into(), value.into());
        self
    }
}

/// 错误追踪扩展 trait
pub trait DatabaseErrorExt<T> {
    fn with_context(self, context: ErrorContext) -> DatabaseResult<T>;
    fn map_not_found(self, table: &str, id: i64) -> DatabaseResult<T>;
    fn map_validation<S: Into<String>>(self, message: S) -> DatabaseResult<T>;
}

impl<T> DatabaseErrorExt<T> for DatabaseResult<T> {
    fn with_context(self, context: ErrorContext) -> DatabaseResult<T> {
        self.map_err(|e| {
            log::error!(
                "数据库错误 [{}]: {} (table: {:?}, id: {:?})",
                context.operation,
                e,
                context.table,
                context.record_id
            );
            e
        })
    }

    fn map_not_found(self, table: &str, id: i64) -> DatabaseResult<T> {
        self.map_err(|e| match e {
            DatabaseError::Connection(rusqlite::Error::QueryReturnedNoRows) => {
                DatabaseError::not_found(table, id)
            }
            _ => e,
        })
    }

    fn map_validation<S: Into<String>>(self, message: S) -> DatabaseResult<T> {
        self.map_err(|_| DatabaseError::validation(message))
    }
}
