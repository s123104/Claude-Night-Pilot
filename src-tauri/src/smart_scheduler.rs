// src-tauri/src/smart_scheduler.rs
// 智能排程系統 - 時區感知、5小時塊保護、智能延遲排程
// 基於claude-code-schedule與ClaudeNightsWatch最佳實踐 [2025-07-24T00:55:47+08:00]

use anyhow::Result;
use chrono::{DateTime, Duration, TimeZone, Utc, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{JobScheduler, Job};

use crate::adaptive_monitor::AdaptiveMonitor;
use crate::usage_tracker::{UsageTracker, UsageInfo};
use crate::executor::{ClaudeExecutor, ExecutionOptions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    pub min_execution_minutes: u32,    // 最小執行時間 (預設: 30分鐘)
    pub buffer_minutes: u32,           // 安全緩衝時間 (預設: 10分鐘)
    pub max_delay_hours: u32,          // 最大延遲時間 (預設: 24小時)
    pub prefer_full_blocks: bool,      // 優先使用完整5小時塊
    pub timezone: String,              // 時區設定 (預設: "Asia/Taipei")
    pub working_hours_start: u32,      // 工作時間開始 (預設: 9)
    pub working_hours_end: u32,        // 工作時間結束 (預設: 18)
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            min_execution_minutes: 30,
            buffer_minutes: 10,
            max_delay_hours: 24,
            prefer_full_blocks: true,
            timezone: "Asia/Taipei".to_string(),
            working_hours_start: 9,
            working_hours_end: 18,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    pub should_execute: bool,
    pub recommended_start_time: Option<DateTime<Utc>>,
    pub reason: String,
    pub estimated_duration_minutes: Option<u32>,
    pub risk_level: SchedulingRisk,
    pub delay_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingRisk {
    Low,      // 充足時間，安全執行
    Medium,   // 時間稍緊，但可執行
    High,     // 時間不足，建議延遲
    Critical, // 絕對不執行，會浪費塊
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub prompt: String,
    pub execution_options: ExecutionOptions,
    pub scheduled_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub estimated_duration_minutes: u32,
    pub timezone: String,
    pub retry_count: u32,
    pub max_retries: u32,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Scheduled,  // 已排程
    Executing,  // 執行中
    Completed,  // 已完成
    Failed,     // 執行失敗
    Delayed,    // 已延遲
    Cancelled,  // 已取消
}

pub struct SmartScheduler {
    config: SchedulingConfig,
    timezone: Tz,
    monitor: Arc<Mutex<AdaptiveMonitor>>,
    usage_tracker: Arc<Mutex<UsageTracker>>,
    scheduler: JobScheduler,
    scheduled_tasks: Arc<Mutex<Vec<ScheduledTask>>>,
}

impl SmartScheduler {
    pub async fn new(
        monitor: Arc<Mutex<AdaptiveMonitor>>,
        usage_tracker: Arc<Mutex<UsageTracker>>,
        config: Option<SchedulingConfig>,
    ) -> Result<Self> {
        let config = config.unwrap_or_default();
        let timezone: Tz = config.timezone.parse()
            .map_err(|_| anyhow::anyhow!("無效的時區: {}", config.timezone))?;

        let scheduler = JobScheduler::new().await?;

        Ok(Self {
            config,
            timezone,
            monitor,
            usage_tracker,
            scheduler,
            scheduled_tasks: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// 智能排程決策 - 核心邏輯
    /// [最佳實踐:claude-code-schedule:智能時間計算:2025-07-24T00:55:47+08:00]
    pub async fn make_scheduling_decision(
        &self,
        _prompt: &str,
        estimated_duration_minutes: u32,
        preferred_time: Option<DateTime<Utc>>,
    ) -> Result<SchedulingDecision> {
        // 1. 獲取當前使用量狀態
        let usage_info = {
            let mut tracker = self.usage_tracker.lock().await;
            tracker.get_usage_info().await?
        };

        // 2. 獲取監控狀態
        let _monitoring_status = {
            let monitor = self.monitor.lock().await;
            monitor.get_status()
        };

        // 3. 檢查基本可用性
        if !usage_info.is_available {
            return Ok(SchedulingDecision {
                should_execute: false,
                recommended_start_time: self.calculate_next_available_time(&usage_info).await,
                reason: "Claude當前不可用".to_string(),
                estimated_duration_minutes: Some(estimated_duration_minutes),
                risk_level: SchedulingRisk::Critical,
                delay_reason: Some("等待下一個5小時塊".to_string()),
            });
        }

        // 4. 計算剩餘時間
        let remaining_minutes = usage_info.current_block
            .as_ref()
            .map(|block| block.remaining_minutes)
            .unwrap_or(0);

        // 5. 智能時間分析
        let time_analysis = self.analyze_time_sufficiency(
            remaining_minutes,
            estimated_duration_minutes,
            preferred_time,
        );

        // 6. 生成排程決策
        self.generate_decision(time_analysis, &usage_info, estimated_duration_minutes).await
    }

    /// 時間充足性分析
    /// [最佳實踐:ClaudeNightsWatch:5小時塊保護:2025-07-24T00:55:47+08:00]
    fn analyze_time_sufficiency(
        &self,
        remaining_minutes: u32,
        estimated_duration_minutes: u32,
        preferred_time: Option<DateTime<Utc>>,
    ) -> TimeSufficiencyAnalysis {
        let total_required = estimated_duration_minutes + self.config.buffer_minutes;

        // 檢查是否有足夠時間
        let has_sufficient_time = remaining_minutes >= total_required;

        // 檢查是否在工作時間
        let is_working_hours = self.is_within_working_hours(preferred_time.unwrap_or_else(Utc::now));

        // 計算時間緊迫程度
        let urgency = if remaining_minutes == 0 {
            TimeUrgency::Critical
        } else if remaining_minutes < self.config.min_execution_minutes {
            TimeUrgency::High
        } else if remaining_minutes < total_required {
            TimeUrgency::Medium
        } else {
            TimeUrgency::Low
        };

        // 評估是否會浪費5小時塊
        let would_waste_block = !has_sufficient_time && remaining_minutes > 0;

        TimeSufficiencyAnalysis {
            remaining_minutes,
            required_minutes: total_required,
            has_sufficient_time,
            is_working_hours,
            urgency,
            would_waste_block,
            efficiency_score: self.calculate_efficiency_score(remaining_minutes, total_required),
        }
    }

    /// 生成最終排程決策
    async fn generate_decision(
        &self,
        analysis: TimeSufficiencyAnalysis,
        usage_info: &UsageInfo,
        estimated_duration_minutes: u32,
    ) -> Result<SchedulingDecision> {
        if analysis.has_sufficient_time && analysis.is_working_hours {
            // 理想情況：時間充足且在工作時間
            Ok(SchedulingDecision {
                should_execute: true,
                recommended_start_time: Some(Utc::now()),
                reason: format!(
                    "時間充足（剩餘{}分鐘，需要{}分鐘），立即執行",
                    analysis.remaining_minutes,
                    analysis.required_minutes
                ),
                estimated_duration_minutes: Some(estimated_duration_minutes),
                risk_level: SchedulingRisk::Low,
                delay_reason: None,
            })
        } else if analysis.would_waste_block {
            // 會浪費5小時塊，建議延遲
            let next_time = self.calculate_next_available_time(usage_info).await;
            Ok(SchedulingDecision {
                should_execute: false,
                recommended_start_time: next_time,
                reason: format!(
                    "時間不足會浪費5小時塊（剩餘{}分鐘，需要{}分鐘）",
                    analysis.remaining_minutes,
                    analysis.required_minutes
                ),
                estimated_duration_minutes: Some(estimated_duration_minutes),
                risk_level: SchedulingRisk::High,
                delay_reason: Some("等待下一個完整5小時塊".to_string()),
            })
        } else if !analysis.is_working_hours {
            // 非工作時間，建議延遲到工作時間
            let next_working_time = self.calculate_next_working_hours();
            Ok(SchedulingDecision {
                should_execute: false,
                recommended_start_time: Some(next_working_time),
                reason: "當前非工作時間，建議延遲到工作時間執行".to_string(),
                estimated_duration_minutes: Some(estimated_duration_minutes),
                risk_level: SchedulingRisk::Medium,
                delay_reason: Some("等待工作時間".to_string()),
            })
        } else {
            // 其他情況，謹慎延遲
            let next_time = self.calculate_next_available_time(usage_info).await;
            Ok(SchedulingDecision {
                should_execute: false,
                recommended_start_time: next_time,
                reason: "綜合考慮時間與風險，建議延遲執行".to_string(),
                estimated_duration_minutes: Some(estimated_duration_minutes),
                risk_level: SchedulingRisk::Medium,
                delay_reason: Some("優化執行時機".to_string()),
            })
        }
    }

    /// 計算下一個可用時間
    async fn calculate_next_available_time(&self, usage_info: &UsageInfo) -> Option<DateTime<Utc>> {
        if let Some(next_block_time) = usage_info.next_block_starts {
            // 如果有明確的下一個塊開始時間
            Some(next_block_time)
        } else {
            // 估算下一個5小時塊（通常在當地時間凌晨1點重置）
            let local_now = self.timezone.from_utc_datetime(&Utc::now().naive_utc());
            let tomorrow = local_now.date_naive().succ_opt()?.and_hms_opt(1, 0, 0)?;
            let next_reset = self.timezone.from_local_datetime(&tomorrow).single()?;
            Some(next_reset.with_timezone(&Utc))
        }
    }

    /// 計算下一個工作時間
    fn calculate_next_working_hours(&self) -> DateTime<Utc> {
        let local_now = self.timezone.from_utc_datetime(&Utc::now().naive_utc());
        let today = local_now.date_naive();
        
        // 嘗試今天的工作時間開始
        if let Some(work_start) = today.and_hms_opt(self.config.working_hours_start, 0, 0) {
            if let Some(work_start_utc) = self.timezone.from_local_datetime(&work_start).single() {
                let work_start_utc = work_start_utc.with_timezone(&Utc);
                if work_start_utc > Utc::now() {
                    return work_start_utc;
                }
            }
        }
        
        // 明天的工作時間開始
        if let Some(tomorrow) = today.succ_opt() {
            if let Some(work_start) = tomorrow.and_hms_opt(self.config.working_hours_start, 0, 0) {
                if let Some(work_start_utc) = self.timezone.from_local_datetime(&work_start).single() {
                    return work_start_utc.with_timezone(&Utc);
                }
            }
        }
        
        // 預設延遲24小時
        Utc::now() + Duration::hours(24)
    }

    /// 檢查是否在工作時間內
    fn is_within_working_hours(&self, time: DateTime<Utc>) -> bool {
        let local_time = self.timezone.from_utc_datetime(&time.naive_utc());
        let hour = local_time.hour();
        
        hour >= self.config.working_hours_start && hour < self.config.working_hours_end
    }

    /// 計算執行效率分數 (0.0-1.0)
    fn calculate_efficiency_score(&self, remaining_minutes: u32, required_minutes: u32) -> f32 {
        if remaining_minutes == 0 {
            return 0.0;
        }
        
        let usage_ratio = required_minutes as f32 / remaining_minutes as f32;
        
        if usage_ratio <= 0.8 {
            1.0 // 理想使用率
        } else if usage_ratio <= 1.0 {
            0.8 // 緊湊但可接受
        } else {
            0.0 // 超出容量
        }
    }

    /// 安排任務執行
    pub async fn schedule_task(
        &self,
        prompt: String,
        execution_options: ExecutionOptions,
        estimated_duration_minutes: u32,
        preferred_time: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let decision = self.make_scheduling_decision(
            &prompt,
            estimated_duration_minutes,
            preferred_time,
        ).await?;

        let task_id = format!("task_{}", Utc::now().timestamp());
        
        let task = ScheduledTask {
            id: task_id.clone(),
            prompt: prompt.clone(),
            execution_options: execution_options.clone(),
            scheduled_at: decision.recommended_start_time.unwrap_or_else(Utc::now),
            created_at: Utc::now(),
            estimated_duration_minutes,
            timezone: self.config.timezone.clone(),
            retry_count: 0,
            max_retries: 3,
            status: TaskStatus::Scheduled,
        };

        if decision.should_execute {
            // 立即執行
            self.execute_task_now(task).await?;
        } else {
            // 延遲執行
            self.schedule_task_for_later(task).await?;
        }

        Ok(task_id)
    }

    /// 立即執行任務
    async fn execute_task_now(&self, mut task: ScheduledTask) -> Result<()> {
        task.status = TaskStatus::Executing;
        
        println!("🚀 立即執行任務: {}", task.id);
        
        match ClaudeExecutor::run_with_options(&task.prompt, task.execution_options.clone()).await {
            Ok(result) => {
                task.status = TaskStatus::Completed;
                println!("✅ 任務執行成功: {}", task.id);
                println!("📝 執行結果: {}", result.chars().take(100).collect::<String>());
            }
            Err(e) => {
                task.status = TaskStatus::Failed;
                eprintln!("❌ 任務執行失敗: {} - {}", task.id, e);
                
                // 重試邏輯
                if task.retry_count < task.max_retries {
                    task.retry_count += 1;
                    task.status = TaskStatus::Scheduled;
                    println!("🔄 任務重試: {} (第{}次)", task.id, task.retry_count);
                }
            }
        }

        // 更新任務狀態
        let mut tasks = self.scheduled_tasks.lock().await;
        tasks.push(task);
        
        Ok(())
    }

    /// 安排延遲執行
    async fn schedule_task_for_later(&self, task: ScheduledTask) -> Result<()> {
        println!("⏰ 安排延遲執行: {} 於 {}", task.id, task.scheduled_at);
        
        let scheduler = &self.scheduler;
        let task_clone = task.clone();
        let tasks = Arc::clone(&self.scheduled_tasks);
        
        // 計算延遲持續時間
        let now = Utc::now();
        let delay_duration = if task.scheduled_at > now {
            let duration_secs = (task.scheduled_at - now).num_seconds();
            std::time::Duration::from_secs(duration_secs.max(0) as u64)
        } else {
            std::time::Duration::from_secs(0)
        };
        
        // 創建 Tokio cron job
        let job = Job::new_one_shot_async(delay_duration, move |_uuid, _l| {
            let task = task_clone.clone();
            let tasks = Arc::clone(&tasks);
            
            Box::pin(async move {
                println!("🕐 延遲任務觸發: {}", task.id);
                
                match ClaudeExecutor::run_with_options(&task.prompt, task.execution_options.clone()).await {
                    Ok(_result) => {
                        println!("✅ 延遲任務執行成功: {}", task.id);
                        
                        let mut tasks_list = tasks.lock().await;
                        if let Some(stored_task) = tasks_list.iter_mut().find(|t| t.id == task.id) {
                            stored_task.status = TaskStatus::Completed;
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ 延遲任務執行失敗: {} - {}", task.id, e);
                        
                        let mut tasks_list = tasks.lock().await;
                        if let Some(stored_task) = tasks_list.iter_mut().find(|t| t.id == task.id) {
                            stored_task.status = TaskStatus::Failed;
                        }
                    }
                }
            })
        })?;

        scheduler.add(job).await?;

        // 保存任務到列表
        let mut tasks = self.scheduled_tasks.lock().await;
        tasks.push(task);
        
        Ok(())
    }

    /// 啟動排程器
    pub async fn start(&self) -> Result<()> {
        self.scheduler.start().await?;
        println!("🚀 智能排程器已啟動");
        Ok(())
    }

    /// 停止排程器
    pub async fn stop(&mut self) -> Result<()> {
        self.scheduler.shutdown().await?;
        println!("⏹️ 智能排程器已停止");
        Ok(())
    }

    /// 獲取所有排程任務
    pub async fn get_scheduled_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.scheduled_tasks.lock().await;
        tasks.clone()
    }

    /// 取消任務
    pub async fn cancel_task(&self, task_id: &str) -> Result<bool> {
        let mut tasks = self.scheduled_tasks.lock().await;
        
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            if matches!(task.status, TaskStatus::Scheduled) {
                task.status = TaskStatus::Cancelled;
                println!("❌ 任務已取消: {}", task_id);
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

// 輔助結構體
#[derive(Debug)]
struct TimeSufficiencyAnalysis {
    remaining_minutes: u32,
    required_minutes: u32,
    has_sufficient_time: bool,
    is_working_hours: bool,
    urgency: TimeUrgency,
    would_waste_block: bool,
    efficiency_score: f32,
}

#[derive(Debug)]
enum TimeUrgency {
    Low,      // 充足時間
    Medium,   // 時間緊迫
    High,     // 時間不足
    Critical, // 無可用時間
}

// Tauri命令介面 [最佳實踐:tauri-docs:2025-07-24T00:55:47+08:00]
#[tauri::command]
pub async fn make_scheduling_decision(
    prompt: String,
    estimated_duration_minutes: u32,
    preferred_time: Option<String>,
    state: tauri::State<'_, Arc<Mutex<SmartScheduler>>>
) -> Result<SchedulingDecision, String> {
    let scheduler = state.lock().await;
    
    let preferred_time = if let Some(time_str) = preferred_time {
        DateTime::parse_from_rfc3339(&time_str)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    } else {
        None
    };
    
    scheduler.make_scheduling_decision(&prompt, estimated_duration_minutes, preferred_time)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn schedule_smart_task(
    prompt: String,
    execution_options: ExecutionOptions,
    estimated_duration_minutes: u32,
    preferred_time: Option<String>,
    state: tauri::State<'_, Arc<Mutex<SmartScheduler>>>
) -> Result<String, String> {
    let scheduler = state.lock().await;
    
    let preferred_time = if let Some(time_str) = preferred_time {
        DateTime::parse_from_rfc3339(&time_str)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    } else {
        None
    };
    
    scheduler.schedule_task(prompt, execution_options, estimated_duration_minutes, preferred_time)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_scheduled_tasks(
    state: tauri::State<'_, Arc<Mutex<SmartScheduler>>>
) -> Result<Vec<ScheduledTask>, String> {
    let scheduler = state.lock().await;
    Ok(scheduler.get_scheduled_tasks().await)
}

#[tauri::command]

// 測試輔助函數
#[cfg(test)]
fn calculate_efficiency_score_static(remaining_minutes: u32, required_minutes: u32) -> f32 {
    if remaining_minutes == 0 {
        return 0.0;
    }
    
    let usage_ratio = required_minutes as f32 / remaining_minutes as f32;
    
    if usage_ratio <= 0.8 {
        1.0 // 理想使用率
    } else if usage_ratio <= 1.0 {
        0.8 // 緊湊但可接受
    } else {
        0.0 // 超出容量
    }
}

#[cfg(test)]
fn is_within_working_hours_static(time: DateTime<Utc>, timezone: &Tz, config: &SchedulingConfig) -> bool {
    let local_time = timezone.from_utc_datetime(&time.naive_utc());
    let hour = local_time.hour();
    
    hour >= config.working_hours_start && hour < config.working_hours_end
}
pub async fn cancel_scheduled_task(
    task_id: String,
    state: tauri::State<'_, Arc<Mutex<SmartScheduler>>>
) -> Result<bool, String> {
    let scheduler = state.lock().await;
    scheduler.cancel_task(&task_id).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_efficiency_calculation() {
        let _config = SchedulingConfig::default();
        
        // 測試理想使用率 (80%)
        assert_eq!(calculate_efficiency_score_static(100, 80), 1.0);
        
        // 測試緊湊使用率 (100%)
        assert_eq!(calculate_efficiency_score_static(100, 100), 0.8);
        
        // 測試超出容量 (120%)
        assert_eq!(calculate_efficiency_score_static(100, 120), 0.0);
    }

    #[test]
    fn test_working_hours_check() {
        let _config = SchedulingConfig::default();
        let timezone: Tz = "Asia/Taipei".parse().unwrap();
        
        // 測試工作時間 (14:00 台北時間)
        let work_time = Utc::now()
            .with_hour(6).unwrap()  // 14:00 台北 = 06:00 UTC
            .with_minute(0).unwrap();
        assert!(is_within_working_hours_static(work_time, &timezone, &config));

        // 測試非工作時間 (22:00 台北時間)
        let non_work_time = Utc::now()
            .with_hour(14).unwrap()  // 22:00 台北 = 14:00 UTC
            .with_minute(0).unwrap();
        assert!(!is_within_working_hours_static(non_work_time, &timezone, &config));
    }
} 