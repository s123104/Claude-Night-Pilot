// 介面適配器模組 - GUI和CLI介面統一
pub mod cli_adapter;
pub mod shared_types;
pub mod tauri_adapter;

pub use cli_adapter::CLIAdapter;
pub use shared_types::*;
pub use tauri_adapter::TauriAdapter;
