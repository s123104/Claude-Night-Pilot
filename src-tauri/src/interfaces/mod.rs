// 介面適配器模組 - GUI和CLI介面統一
pub mod tauri_adapter;
pub mod cli_adapter;
pub mod shared_types;

pub use tauri_adapter::TauriAdapter;
pub use cli_adapter::CLIAdapter;
pub use shared_types::*;