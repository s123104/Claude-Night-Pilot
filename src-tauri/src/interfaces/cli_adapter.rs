// CLI適配器 - CLI介面的統一入口
use anyhow::Result;
use crate::services::{PromptService, JobService, SyncService};
use crate::services::health_service::HealthService;
// use crate::interfaces::shared_types::*; // 暫時無用
use crate::state::AppStateManager;
use std::sync::Arc;

pub struct CLIAdapter {
    prompt_service: Arc<PromptService>,
    job_service: Arc<JobService>,
    sync_service: Arc<SyncService>,
    health_service: Arc<HealthService>,
    state_manager: Arc<AppStateManager>,
}

impl CLIAdapter {
    pub async fn new() -> Result<Self> {
        let prompt_service = Arc::new(PromptService::new().await?);
        let job_service = Arc::new(JobService::new().await?);
        let sync_service = Arc::new(SyncService::new());
        let health_service = Arc::new(HealthService::new());
        let state_manager = Arc::new(AppStateManager::new());

        Ok(Self {
            prompt_service,
            job_service,
            sync_service,
            health_service,
            state_manager,
        })
    }

    /// CLI專用的Prompt操作 - 簡化版本，適合命令行輸出
    pub async fn cli_list_prompts(&self, format: &str) -> Result<String> {
        match self.prompt_service.list_prompts().await {
            Ok(prompts) => {
                match format {
                    "json" => {
                        Ok(serde_json::to_string_pretty(&prompts)?)
                    }
                    "table" => {
                        if prompts.is_empty() {
                            Ok("無 Prompt 資料".to_string())
                        } else {
                            let mut output = String::new();
                            output.push_str("ID  | 標題                    | 建立時間\n");
                            output.push_str("----+------------------------+--------------------\n");
                            
                            for prompt in prompts {
                                let title = if prompt.title.len() > 20 {
                                    format!("{}...", &prompt.title[0..17])
                                } else {
                                    format!("{:<20}", prompt.title)
                                };
                                
                                let created = chrono::DateTime::parse_from_rfc3339(&prompt.created_at)
                                    .map(|dt| dt.format("%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|_| "未知".to_string());
                                
                                output.push_str(&format!("{:<3} | {} | {}\n", prompt.id, title, created));
                            }
                            
                            Ok(output)
                        }
                    }
                    _ => {
                        // 預設簡潔格式
                        if prompts.is_empty() {
                            Ok("無 Prompt 資料".to_string())
                        } else {
                            let mut output = String::new();
                            for prompt in prompts {
                                output.push_str(&format!("- #{}: {} ({})\n", 
                                    prompt.id, 
                                    prompt.title,
                                    prompt.created_at
                                ));
                            }
                            Ok(output)
                        }
                    }
                }
            }
            Err(e) => Err(anyhow::anyhow!("列出 Prompts 失敗: {}", e))
        }
    }

    pub async fn cli_create_prompt(
        &self,
        title: String,
        content: String,
        tags: Option<String>,
    ) -> Result<String> {
        let request = crate::services::prompt_service::CreatePromptRequest {
            title: title.clone(),
            content,
            tags,
        };

        match self.prompt_service.create_prompt(request).await {
            Ok(id) => {
                // 觸發CLI同步通知
                self.notify_cli_change("prompt_created", id, &title).await?;
                Ok(format!("✅ 成功建立 Prompt: {} (ID: {})", title, id))
            }
            Err(e) => Err(anyhow::anyhow!("建立 Prompt 失敗: {}", e))
        }
    }

    pub async fn cli_delete_prompt(&self, id: i64) -> Result<String> {
        // 先檢查是否存在
        if let Ok(Some(prompt)) = self.prompt_service.get_prompt(id).await {
            match self.prompt_service.delete_prompt(id).await {
                Ok(_) => {
                    self.notify_cli_change("prompt_deleted", id, &prompt.title).await?;
                    Ok(format!("✅ 成功刪除 Prompt: {} (ID: {})", prompt.title, id))
                }
                Err(e) => Err(anyhow::anyhow!("刪除 Prompt 失敗: {}", e))
            }
        } else {
            Err(anyhow::anyhow!("Prompt {} 不存在", id))
        }
    }

    /// CLI專用的Job操作
    pub async fn cli_list_jobs(&self, format: &str) -> Result<String> {
        match self.job_service.list_jobs().await {
            Ok(jobs) => {
                match format {
                    "json" => {
                        Ok(serde_json::to_string_pretty(&jobs)?)
                    }
                    "table" => {
                        if jobs.is_empty() {
                            Ok("無排程任務".to_string())
                        } else {
                            let mut output = String::new();
                            output.push_str("ID  | 名稱                | 狀態    | Cron 表達式\n");
                            output.push_str("----+--------------------+---------+------------------\n");
                            
                            for job in jobs {
                                let name = if job.name.len() > 16 {
                                    format!("{}...", &job.name[0..13])
                                } else {
                                    format!("{:<16}", job.name)
                                };
                                
                                let status_display = match job.status.as_str() {
                                    "active" => "運行中",
                                    "paused" => "已暫停",
                                    "error" => "錯誤",
                                    _ => &job.status,
                                };
                                
                                output.push_str(&format!("{:<3} | {} | {:<7} | {}\n", 
                                    job.id, 
                                    name, 
                                    status_display,
                                    job.cron_expression
                                ));
                            }
                            
                            Ok(output)
                        }
                    }
                    _ => {
                        // 預設簡潔格式
                        if jobs.is_empty() {
                            Ok("無排程任務".to_string())
                        } else {
                            let mut output = String::new();
                            for job in jobs {
                                output.push_str(&format!("- ID: {}  名稱: {}  狀態: {}  Cron: {}\n",
                                    job.id, job.name, job.status, job.cron_expression));
                            }
                            Ok(output)
                        }
                    }
                }
            }
            Err(e) => Err(anyhow::anyhow!("列出排程任務失敗: {}", e))
        }
    }

    pub async fn cli_create_job(
        &self,
        prompt_id: i64,
        name: String,
        cron_expression: String,
        description: Option<String>,
    ) -> Result<String> {
        let request = crate::services::job_service::CreateJobRequest {
            prompt_id,
            name: name.clone(),
            cron_expression: cron_expression.clone(),
            description,
        };

        match self.job_service.create_job(request).await {
            Ok(id) => {
                self.notify_cli_change("job_created", id, &name).await?;
                Ok(format!("✅ 成功建立排程任務: {} (ID: {}, Cron: {})", name, id, cron_expression))
            }
            Err(e) => Err(anyhow::anyhow!("建立排程任務失敗: {}", e))
        }
    }

    pub async fn cli_delete_job(&self, id: i64) -> Result<String> {
        if let Ok(Some(job)) = self.job_service.get_job(id).await {
            match self.job_service.delete_job(id).await {
                Ok(_) => {
                    self.notify_cli_change("job_deleted", id, &job.name).await?;
                    Ok(format!("✅ 成功刪除排程任務: {} (ID: {})", job.name, id))
                }
                Err(e) => Err(anyhow::anyhow!("刪除排程任務失敗: {}", e))
            }
        } else {
            Err(anyhow::anyhow!("排程任務 {} 不存在", id))
        }
    }

    /// CLI專用的同步操作
    pub async fn cli_get_sync_status(&self, format: &str) -> Result<String> {
        match self.sync_service.get_sync_status().await {
            Ok(status) => {
                match format {
                    "json" => Ok(serde_json::to_string_pretty(&status)?),
                    _ => {
                        let health_icon = match status.sync_health.as_str() {
                            "healthy" => "✅",
                            "syncing" => "🔄",
                            "conflicts" => "⚠️",
                            "overloaded" => "🚨",
                            "error" => "❌",
                            _ => "❓",
                        };
                        
                        Ok(format!(
                            "{} 同步狀態: {}\n待處理變更: {}\n衝突數量: {}\n性能影響: {:.1}%\n最後同步: {}",
                            health_icon,
                            status.sync_health,
                            status.pending_changes,
                            status.sync_conflicts,
                            status.performance_impact,
                            status.last_sync_timestamp
                        ))
                    }
                }
            }
            Err(e) => Err(anyhow::anyhow!("獲取同步狀態失敗: {}", e))
        }
    }

    pub async fn cli_trigger_sync(&self) -> Result<String> {
        match self.sync_service.trigger_sync().await {
            Ok(sync_id) => Ok(format!("✅ 同步已觸發: {}", sync_id)),
            Err(e) => Err(anyhow::anyhow!("觸發同步失敗: {}", e))
        }
    }

    /// CLI專用的健康檢查
    pub async fn cli_health_check(&self, format: &str, quick: bool) -> Result<String> {
        if quick {
            match self.health_service.quick_health_check().await {
                Ok(health) => {
                    match format {
                        "json" => Ok(serde_json::to_string_pretty(&health)?),
                        _ => {
                            let status_icon = if health.is_healthy { "✅" } else { "❌" };
                            Ok(format!("{} 系統狀態: {} - {}", 
                                status_icon, health.status, health.message))
                        }
                    }
                }
                Err(e) => Err(anyhow::anyhow!("快速健康檢查失敗: {}", e))
            }
        } else {
            match self.health_service.comprehensive_health_check().await {
                Ok(health) => {
                    match format {
                        "json" => Ok(serde_json::to_string_pretty(&health)?),
                        _ => {
                            self.format_comprehensive_health(&health)
                        }
                    }
                }
                Err(e) => Err(anyhow::anyhow!("綜合健康檢查失敗: {}", e))
            }
        }
    }

    /// 格式化綜合健康報告
    fn format_comprehensive_health(&self, health: &crate::services::health_service::HealthStatus) -> Result<String> {
        let status_icon = match health.overall_status.as_str() {
            "healthy" => "✅",
            "degraded" => "⚠️",
            "unhealthy" => "❌",
            _ => "❓",
        };

        let mut output = format!(
            "{} 系統健康報告\n{}\n",
            status_icon,
            "=".repeat(40)
        );

        output.push_str(&format!("總體狀態: {}\n", health.overall_status));
        output.push_str(&format!("Claude CLI: {}\n", if health.claude_cli_available { "可用" } else { "不可用" }));
        output.push_str(&format!("資料庫: {}\n", if health.database_connected { "連接正常" } else { "連接異常" }));
        output.push_str(&format!("冷卻檢測: {}\n", if health.cooldown_detection_working { "正常" } else { "異常" }));
        
        output.push_str(&format!("\n🔧 效能指標\n{}\n", "-".repeat(20)));
        output.push_str(&format!("記憶體使用: {:.1} MB\n", health.performance_metrics.memory_usage_mb));
        output.push_str(&format!("CPU 使用率: {:.1}%\n", health.performance_metrics.cpu_usage_percent));
        output.push_str(&format!("活躍任務: {}\n", health.performance_metrics.jobs_active));
        output.push_str(&format!("成功率: {:.1}%\n", health.performance_metrics.success_rate_percent));
        
        output.push_str(&format!("\n📋 系統資訊\n{}\n", "-".repeat(20)));
        output.push_str(&format!("版本: {}\n", health.version));
        output.push_str(&format!("平台: {}\n", health.system_info.platform));
        output.push_str(&format!("運行時間: {} 秒\n", health.uptime_seconds));
        output.push_str(&format!("最後檢查: {}\n", health.last_check));

        Ok(output)
    }

    /// CLI同步變更通知
    async fn notify_cli_change(&self, change_type: &str, id: i64, name: &str) -> Result<()> {
        // 記錄CLI操作，用於同步
        tracing::info!("CLI操作: {} - ID: {}, 名稱: {}", change_type, id, name);
        
        // 觸發同步事件 (簡化實現)
        if let Ok(_) = self.sync_service.get_sync_status().await {
            // 實際實現中可以發送事件到事件匯流排
        }
        
        Ok(())
    }

    /// 獲取CLI適配器統計信息
    pub async fn get_cli_statistics(&self) -> Result<serde_json::Value> {
        let state_stats = self.state_manager.get_state_statistics().await?;
        
        Ok(serde_json::json!({
            "adapter_type": "cli",
            "services_initialized": true,
            "state_manager": state_stats,
            "last_activity": chrono::Utc::now().to_rfc3339(),
        }))
    }
}

// 全局CLI適配器實例
use std::sync::OnceLock;
use tokio::sync::Mutex;

static CLI_ADAPTER: OnceLock<Mutex<Option<CLIAdapter>>> = OnceLock::new();

impl CLIAdapter {
    pub async fn global() -> Result<Arc<CLIAdapter>> {
        let adapter_mutex = CLI_ADAPTER.get_or_init(|| Mutex::new(None));
        let mut adapter_guard = adapter_mutex.lock().await;
        
        if adapter_guard.is_none() {
            *adapter_guard = Some(CLIAdapter::new().await?);
        }
        
        // 這裡需要返回Arc<CLIAdapter>，但我們存儲的是Option<CLIAdapter>
        // 實際實現中可能需要調整架構
        match adapter_guard.as_ref() {
            Some(_adapter) => {
                // 臨時解決方案：創建新實例
                Ok(Arc::new(CLIAdapter::new().await?))
            }
            None => Err(anyhow::anyhow!("CLI適配器初始化失敗"))
        }
    }
}