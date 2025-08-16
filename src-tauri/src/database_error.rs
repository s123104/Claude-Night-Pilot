// 數據庫最佳實踐：結構化錯誤處理
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] rusqlite::Error),

    #[error("Async task error: {0}")]
    Task(#[from] tokio::task::JoinError),

    #[error("Database is not initialized")]
    NotInitialized,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> String {
        err.to_string()
    }
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
