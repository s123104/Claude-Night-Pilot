// 進程編排系統 - 基於 Vibe-Kanban 最佳實踐的進程管理
use crate::core::retry::{RetryConfig, RetryOrchestrator};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHandle {
    pub id: Uuid,
    pub process_type: ProcessType,
    pub status: ProcessStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: ProcessMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessType {
    ClaudeExecution {
        prompt: String,
        options: ExecutionOptions,
    },
    SetupScript {
        script_path: String,
        args: Vec<String>,
    },
    CleanupScript {
        script_path: String,
        cleanup_type: CleanupType,
    },
    DevServer {
        port: u16,
        command: String,
    },
    DatabaseMigration {
        migration_type: String,
    },
    BackgroundTask {
        task_name: String,
        interval: Option<std::time::Duration>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Created,
    WaitingForPrerequisites,
    Running,
    Completed,
    Failed { error: String },
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupType {
    Temporary,
    Cache,
    Logs,
    Database,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetadata {
    pub job_id: Option<Uuid>,
    pub parent_process: Option<Uuid>,
    pub dependencies: Vec<Uuid>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub timeout: Option<std::time::Duration>,
    pub retry_config: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    pub skip_permissions: bool,
    pub output_format: String,
    pub timeout_seconds: Option<u64>,
    pub dry_run: bool,
    pub working_directory: Option<String>,
    pub allowed_operations: Vec<String>,
    pub safety_check: bool,
    pub max_retries: u32,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            skip_permissions: false,
            output_format: "json".to_string(),
            timeout_seconds: Some(300),
            dry_run: false,
            working_directory: None,
            allowed_operations: vec![
                "read".to_string(),
                "write".to_string(),
                "compile".to_string(),
            ],
            safety_check: true,
            max_retries: 3,
        }
    }
}

pub struct ProcessOrchestrator {
    active_processes: HashMap<Uuid, ProcessHandle>,
    _process_dependencies: HashMap<Uuid, Vec<Uuid>>,
    completion_callbacks: HashMap<Uuid, Box<dyn Fn(&ProcessHandle) + Send + Sync>>,
    _retry_orchestrator: RetryOrchestrator,
}

impl std::fmt::Debug for ProcessOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessOrchestrator")
            .field("active_processes", &self.active_processes)
            .field("_process_dependencies", &self._process_dependencies)
            .field(
                "completion_callbacks_count",
                &self.completion_callbacks.len(),
            )
            .field("_retry_orchestrator", &self._retry_orchestrator)
            .finish()
    }
}

impl ProcessOrchestrator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            active_processes: HashMap::new(),
            _process_dependencies: HashMap::new(),
            completion_callbacks: HashMap::new(),
            _retry_orchestrator: RetryOrchestrator::with_smart_defaults()?,
        })
    }

    /// 執行帶前置條件的進程 - 基於 Vibe-Kanban 的自動設置邏輯
    pub async fn execute_with_prerequisites<F, Fut>(
        &mut self,
        process_type: ProcessType,
        prerequisites: Vec<ProcessType>,
        metadata: ProcessMetadata,
        main_operation: F,
    ) -> Result<ProcessHandle>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<String>> + Send,
    {
        let process_id = Uuid::new_v4();

        // 創建進程句柄
        let mut process_handle = ProcessHandle {
            id: process_id,
            process_type: process_type.clone(),
            status: ProcessStatus::Created,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            metadata,
        };

        // 檢查並執行前置條件
        let prerequisite_ids = self.execute_prerequisites(prerequisites).await?;
        process_handle.metadata.dependencies = prerequisite_ids.clone();

        if !prerequisite_ids.is_empty() {
            process_handle.status = ProcessStatus::WaitingForPrerequisites;
            self.active_processes
                .insert(process_id, process_handle.clone());

            // 等待前置條件完成
            self.wait_for_prerequisites(&prerequisite_ids).await?;
        }

        // 更新狀態並執行主操作
        process_handle.status = ProcessStatus::Running;
        process_handle.started_at = Some(chrono::Utc::now());
        self.active_processes
            .insert(process_id, process_handle.clone());

        // 執行主操作
        let execution_result = main_operation().await;

        // 更新最終狀態
        match execution_result {
            Ok(_) => {
                process_handle.status = ProcessStatus::Completed;
                process_handle.completed_at = Some(chrono::Utc::now());

                tracing::info!("進程 {} 執行成功: {:?}", process_id, process_type);
            }
            Err(e) => {
                process_handle.status = ProcessStatus::Failed {
                    error: e.to_string(),
                };
                process_handle.completed_at = Some(chrono::Utc::now());

                tracing::error!("進程 {} 執行失敗: {:?} - {}", process_id, process_type, e);
            }
        }

        self.active_processes
            .insert(process_id, process_handle.clone());

        // 觸發完成回調
        if let Some(callback) = self.completion_callbacks.get(&process_id) {
            callback(&process_handle);
        }

        Ok(process_handle)
    }

    /// 執行前置條件進程
    async fn execute_prerequisites(
        &mut self,
        prerequisites: Vec<ProcessType>,
    ) -> Result<Vec<Uuid>> {
        let mut prerequisite_ids = Vec::new();

        for prereq in prerequisites {
            match prereq {
                ProcessType::SetupScript { script_path, args } => {
                    let prereq_id = self.execute_setup_script(&script_path, &args).await?;
                    prerequisite_ids.push(prereq_id);
                }
                ProcessType::DatabaseMigration { migration_type } => {
                    let prereq_id = self.execute_database_migration(&migration_type).await?;
                    prerequisite_ids.push(prereq_id);
                }
                ProcessType::CleanupScript {
                    script_path,
                    cleanup_type,
                } => {
                    let prereq_id = self
                        .execute_cleanup_script(&script_path, &cleanup_type)
                        .await?;
                    prerequisite_ids.push(prereq_id);
                }
                _ => {
                    tracing::warn!("不支援的前置條件類型: {:?}", prereq);
                }
            }
        }

        Ok(prerequisite_ids)
    }

    /// 執行設置腳本
    async fn execute_setup_script(&mut self, script_path: &str, args: &[String]) -> Result<Uuid> {
        let process_id = Uuid::new_v4();

        let mut process_handle = ProcessHandle {
            id: process_id,
            process_type: ProcessType::SetupScript {
                script_path: script_path.to_string(),
                args: args.to_vec(),
            },
            status: ProcessStatus::Running,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            metadata: ProcessMetadata {
                job_id: None,
                parent_process: None,
                dependencies: Vec::new(),
                environment: std::env::vars().collect(),
                working_directory: None,
                timeout: Some(std::time::Duration::from_secs(300)), // 5分鐘超時
                retry_config: None,
            },
        };

        self.active_processes
            .insert(process_id, process_handle.clone());

        // 執行腳本
        let result = self
            .run_script_with_timeout(script_path, args, process_handle.metadata.timeout)
            .await;

        match result {
            Ok(_) => {
                process_handle.status = ProcessStatus::Completed;
                tracing::info!("設置腳本執行成功: {}", script_path);
            }
            Err(e) => {
                process_handle.status = ProcessStatus::Failed {
                    error: e.to_string(),
                };
                tracing::error!("設置腳本執行失敗: {} - {}", script_path, e);
            }
        }

        process_handle.completed_at = Some(chrono::Utc::now());
        self.active_processes.insert(process_id, process_handle);

        Ok(process_id)
    }

    /// 執行資料庫遷移
    async fn execute_database_migration(&mut self, migration_type: &str) -> Result<Uuid> {
        let process_id = Uuid::new_v4();

        let mut process_handle = ProcessHandle {
            id: process_id,
            process_type: ProcessType::DatabaseMigration {
                migration_type: migration_type.to_string(),
            },
            status: ProcessStatus::Running,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            metadata: ProcessMetadata {
                job_id: None,
                parent_process: None,
                dependencies: Vec::new(),
                environment: HashMap::new(),
                working_directory: None,
                timeout: Some(std::time::Duration::from_secs(120)),
                retry_config: None,
            },
        };

        self.active_processes
            .insert(process_id, process_handle.clone());

        // 執行資料庫遷移邏輯
        let result = self.run_database_migration(migration_type).await;

        match result {
            Ok(_) => {
                process_handle.status = ProcessStatus::Completed;
                tracing::info!("資料庫遷移執行成功: {}", migration_type);
            }
            Err(e) => {
                process_handle.status = ProcessStatus::Failed {
                    error: e.to_string(),
                };
                tracing::error!("資料庫遷移執行失敗: {} - {}", migration_type, e);
            }
        }

        process_handle.completed_at = Some(chrono::Utc::now());
        self.active_processes.insert(process_id, process_handle);

        Ok(process_id)
    }

    /// 執行清理腳本
    async fn execute_cleanup_script(
        &mut self,
        script_path: &str,
        cleanup_type: &CleanupType,
    ) -> Result<Uuid> {
        let process_id = Uuid::new_v4();

        let mut process_handle = ProcessHandle {
            id: process_id,
            process_type: ProcessType::CleanupScript {
                script_path: script_path.to_string(),
                cleanup_type: cleanup_type.clone(),
            },
            status: ProcessStatus::Running,
            created_at: chrono::Utc::now(),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            metadata: ProcessMetadata {
                job_id: None,
                parent_process: None,
                dependencies: Vec::new(),
                environment: HashMap::new(),
                working_directory: None,
                timeout: Some(std::time::Duration::from_secs(180)),
                retry_config: None,
            },
        };

        self.active_processes
            .insert(process_id, process_handle.clone());

        // 執行清理腳本
        let result = self.run_cleanup_script(script_path, cleanup_type).await;

        match result {
            Ok(_) => {
                process_handle.status = ProcessStatus::Completed;
                tracing::info!("清理腳本執行成功: {} ({:?})", script_path, cleanup_type);
            }
            Err(e) => {
                process_handle.status = ProcessStatus::Failed {
                    error: e.to_string(),
                };
                tracing::error!(
                    "清理腳本執行失敗: {} ({:?}) - {}",
                    script_path,
                    cleanup_type,
                    e
                );
            }
        }

        process_handle.completed_at = Some(chrono::Utc::now());
        self.active_processes.insert(process_id, process_handle);

        Ok(process_id)
    }

    /// 等待前置條件完成
    async fn wait_for_prerequisites(&self, prerequisite_ids: &[Uuid]) -> Result<()> {
        let mut completed_count = 0;
        let total_count = prerequisite_ids.len();

        while completed_count < total_count {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            completed_count = 0;
            for &prereq_id in prerequisite_ids {
                if let Some(process) = self.active_processes.get(&prereq_id) {
                    match process.status {
                        ProcessStatus::Completed => completed_count += 1,
                        ProcessStatus::Failed { ref error } => {
                            return Err(anyhow::anyhow!(
                                "前置條件 {} 執行失敗: {}",
                                prereq_id,
                                error
                            ));
                        }
                        ProcessStatus::Cancelled => {
                            return Err(anyhow::anyhow!("前置條件 {} 被取消", prereq_id));
                        }
                        ProcessStatus::Timeout => {
                            return Err(anyhow::anyhow!("前置條件 {} 執行超時", prereq_id));
                        }
                        _ => {} // 仍在執行中
                    }
                }
            }
        }

        tracing::info!("所有前置條件已完成 ({} 個)", total_count);
        Ok(())
    }

    /// 帶超時的腳本執行
    async fn run_script_with_timeout(
        &self,
        script_path: &str,
        args: &[String],
        timeout: Option<std::time::Duration>,
    ) -> Result<String> {
        let timeout_duration = timeout.unwrap_or(std::time::Duration::from_secs(300));

        let execution_future = async {
            let mut cmd = AsyncCommand::new(script_path);
            cmd.args(args);

            let output = cmd.output().await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("腳本執行失敗: {}", stderr));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        };

        match tokio::time::timeout(timeout_duration, execution_future).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!(
                "腳本執行超時，超過 {} 秒",
                timeout_duration.as_secs()
            )),
        }
    }

    /// 執行資料庫遷移
    async fn run_database_migration(&self, migration_type: &str) -> Result<()> {
        match migration_type {
            "init" => {
                tracing::info!("執行資料庫初始化遷移");
                // 這裡可以調用現有的資料庫初始化邏輯
                // 例如：crate::db::Database::initialize().await?;
                Ok(())
            }
            "upgrade" => {
                tracing::info!("執行資料庫升級遷移");
                // 資料庫升級邏輯
                Ok(())
            }
            "cleanup" => {
                tracing::info!("執行資料庫清理遷移");
                // 資料庫清理邏輯
                Ok(())
            }
            _ => Err(anyhow::anyhow!("不支援的遷移類型: {}", migration_type)),
        }
    }

    /// 執行清理腳本
    async fn run_cleanup_script(
        &self,
        script_path: &str,
        cleanup_type: &CleanupType,
    ) -> Result<()> {
        tracing::info!("執行清理腳本: {} ({:?})", script_path, cleanup_type);

        match cleanup_type {
            CleanupType::Temporary => {
                // 清理臨時檔案
                self.cleanup_temporary_files().await?;
            }
            CleanupType::Cache => {
                // 清理快取
                self.cleanup_cache().await?;
            }
            CleanupType::Logs => {
                // 清理日誌檔案
                self.cleanup_logs().await?;
            }
            CleanupType::Database => {
                // 清理資料庫
                self.cleanup_database().await?;
            }
            CleanupType::Complete => {
                // 完整清理
                self.cleanup_temporary_files().await?;
                self.cleanup_cache().await?;
                self.cleanup_logs().await?;
            }
        }

        Ok(())
    }

    /// 清理臨時檔案
    async fn cleanup_temporary_files(&self) -> Result<()> {
        tracing::info!("清理臨時檔案");
        // 實際的臨時檔案清理邏輯
        Ok(())
    }

    /// 清理快取
    async fn cleanup_cache(&self) -> Result<()> {
        tracing::info!("清理快取");
        // 實際的快取清理邏輯
        Ok(())
    }

    /// 清理日誌檔案
    async fn cleanup_logs(&self) -> Result<()> {
        tracing::info!("清理日誌檔案");
        // 實際的日誌清理邏輯
        Ok(())
    }

    /// 清理資料庫
    async fn cleanup_database(&self) -> Result<()> {
        tracing::info!("清理資料庫");
        // 實際的資料庫清理邏輯
        Ok(())
    }

    /// 取消進程
    pub async fn cancel_process(&mut self, process_id: Uuid) -> Result<()> {
        if let Some(process) = self.active_processes.get_mut(&process_id) {
            match process.status {
                ProcessStatus::Running | ProcessStatus::WaitingForPrerequisites => {
                    process.status = ProcessStatus::Cancelled;
                    process.completed_at = Some(chrono::Utc::now());

                    tracing::info!("進程 {} 已取消", process_id);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!(
                    "進程 {} 無法取消，當前狀態: {:?}",
                    process_id,
                    process.status
                )),
            }
        } else {
            Err(anyhow::anyhow!("進程 {} 不存在", process_id))
        }
    }

    /// 獲取進程狀態
    pub fn get_process_status(&self, process_id: Uuid) -> Option<&ProcessHandle> {
        self.active_processes.get(&process_id)
    }

    /// 列出所有活躍進程
    pub fn list_active_processes(&self) -> Vec<&ProcessHandle> {
        self.active_processes
            .values()
            .filter(|process| {
                matches!(
                    process.status,
                    ProcessStatus::Running | ProcessStatus::WaitingForPrerequisites
                )
            })
            .collect()
    }

    /// 清理已完成的進程
    pub fn cleanup_completed_processes(&mut self) {
        let completed_ids: Vec<Uuid> = self
            .active_processes
            .iter()
            .filter(|(_, process)| {
                matches!(
                    process.status,
                    ProcessStatus::Completed
                        | ProcessStatus::Failed { .. }
                        | ProcessStatus::Cancelled
                )
            })
            .map(|(id, _)| *id)
            .collect();

        let count = completed_ids.len();
        for id in completed_ids {
            self.active_processes.remove(&id);
            self.completion_callbacks.remove(&id);
        }

        tracing::info!("已清理 {} 個已完成的進程", count);
    }

    /// 註冊完成回調
    pub fn register_completion_callback<F>(&mut self, process_id: Uuid, callback: F)
    where
        F: Fn(&ProcessHandle) + Send + Sync + 'static,
    {
        self.completion_callbacks
            .insert(process_id, Box::new(callback));
    }

    /// 獲取進程統計信息
    pub fn get_process_stats(&self) -> ProcessStats {
        let mut stats = ProcessStats::default();

        for process in self.active_processes.values() {
            match process.status {
                ProcessStatus::Created => stats.created += 1,
                ProcessStatus::WaitingForPrerequisites => stats.waiting += 1,
                ProcessStatus::Running => stats.running += 1,
                ProcessStatus::Completed => stats.completed += 1,
                ProcessStatus::Failed { .. } => stats.failed += 1,
                ProcessStatus::Cancelled => stats.cancelled += 1,
                ProcessStatus::Timeout => stats.timeout += 1,
            }
        }

        stats.total = self.active_processes.len() as u32;
        stats
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcessStats {
    pub total: u32,
    pub created: u32,
    pub waiting: u32,
    pub running: u32,
    pub completed: u32,
    pub failed: u32,
    pub cancelled: u32,
    pub timeout: u32,
}

impl Default for ProcessMetadata {
    fn default() -> Self {
        Self {
            job_id: None,
            parent_process: None,
            dependencies: Vec::new(),
            environment: HashMap::new(),
            working_directory: None,
            timeout: Some(std::time::Duration::from_secs(300)),
            retry_config: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_orchestrator_creation() {
        let orchestrator = ProcessOrchestrator::new();
        assert!(orchestrator.is_ok());
    }

    #[tokio::test]
    async fn test_process_handle_creation() {
        let process_handle = ProcessHandle {
            id: Uuid::new_v4(),
            process_type: ProcessType::ClaudeExecution {
                prompt: "test".to_string(),
                options: ExecutionOptions::default(),
            },
            status: ProcessStatus::Created,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            metadata: ProcessMetadata::default(),
        };

        assert_eq!(process_handle.status, ProcessStatus::Created);
        assert!(process_handle.started_at.is_none());
        assert!(process_handle.completed_at.is_none());
    }

    #[test]
    fn test_process_stats_default() {
        let stats = ProcessStats::default();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.running, 0);
        assert_eq!(stats.completed, 0);
    }

    #[test]
    fn test_execution_options_default() {
        let options = ExecutionOptions::default();
        assert!(!options.skip_permissions);
        assert!(options.safety_check);
        assert_eq!(options.output_format, "json");
        assert_eq!(options.max_retries, 3);
    }
}
