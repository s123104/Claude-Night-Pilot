// 共享服務層模組
pub mod prompt_service;
pub mod job_service;
pub mod health_service;
pub mod sync_service;

#[cfg(test)]
pub mod tests;

// 重新導出服務以便統一使用
pub use prompt_service::PromptService;
pub use job_service::JobService; 
pub use health_service::HealthService;
pub use sync_service::SyncService;

// Tauri命令包裝器 - 提供給GUI調用
pub use prompt_service::{
    prompt_service_list_prompts,
    prompt_service_create_prompt,
    prompt_service_delete_prompt,
};

pub use job_service::{
    job_service_list_jobs,
    job_service_create_job,
    job_service_delete_job,
};

pub use sync_service::{
    sync_service_get_status,
    sync_service_trigger_sync,
};