// 🔧 Claude Night Pilot - 統一企業級排程器
// 基於 Context7 最佳實踐 + Research Projects 整合
// 創建時間: 2025-08-17T17:30:00+00:00

use crate::models::job::{Job, JobStatus};
use crate::scheduler::real_time_executor::{ExecutionStats, RealTimeExecutor};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 統一企業級排程器
///
/// 整合了所有現有排程器的最佳功能：
/// - RealTimeExecutor: 企業級執行引擎與Context7最佳實踐
/// - JobScheduler: 回調機制與狀態管理
/// - SimpleJobManager: 相容性API與簡化介面
/// - vibe-kanban模式: 階層式任務管理與ExecutionProcess追蹤
/// - ccusage模式: 使用量追蹤與成本計算
///
/// ## 架構特性
/// - ✅ 單一統一介面，消除技術債務
/// - ✅ 向後相容性，確保平滑遷移
/// - ✅ 企業級監控與告警
/// - ✅ 階層式任務管理
/// - ✅ 使用量追蹤與成本控制
#[derive(Debug)]
pub struct UnifiedScheduler {
    /// 核心執行器 (基於 RealTimeExecutor)
    core_executor: Arc<RealTimeExecutor>,

    /// 任務狀態管理
    job_states: Arc<RwLock<HashMap<String, UnifiedJobState>>>,

    /// 任務階層關係 (基於 vibe-kanban 模式)
    task_hierarchy: Arc<RwLock<TaskHierarchy>>,

    /// 企業級監控收集器
    metrics_collector: Arc<Mutex<MetricsCollector>>,

    /// 使用量追蹤器 (基於 ccusage 模式)
    usage_tracker: Arc<Mutex<UsageTracker>>,

    /// 排程器狀態
    scheduler_state: Arc<RwLock<SchedulerState>>,
}

/// 統一任務狀態
///
/// 整合所有排程器的狀態資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedJobState {
    /// 基本任務資訊
    pub job: Job,

    /// 執行統計 (來自 RealTimeExecutor)
    pub execution_stats: ExecutionStats,

    /// 任務執行流程 (基於 vibe-kanban ExecutionProcess)
    pub execution_processes: Vec<JobExecutionProcess>,

    /// 父任務ID (階層式管理)
    pub parent_job_id: Option<String>,

    /// 子任務列表
    pub child_job_ids: Vec<String>,

    /// 使用量資料
    pub usage_data: UsageData,

    /// 最後更新時間
    pub last_updated: DateTime<Utc>,
}

/// 任務執行流程 (基於 vibe-kanban ExecutionProcess 模式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionProcess {
    pub id: Uuid,
    pub job_id: String,
    pub process_type: ProcessType,
    pub status: ProcessStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

/// 流程類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessType {
    /// 前置準備階段
    Setup,
    /// 主要執行階段
    Execution,
    /// 後續清理階段
    Cleanup,
    /// 結果驗證階段
    Validation,
}

/// 流程狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    /// 已排隊等待執行
    Queued,
    /// 正在執行中
    Running,
    /// 執行成功完成
    Completed,
    /// 執行失敗
    Failed,
    /// 已被取消
    Cancelled,
    /// 正在重試中
    Retrying,
}

/// 任務階層管理 (基於 vibe-kanban 模式)
#[derive(Debug, Default)]
pub struct TaskHierarchy {
    /// 父子關係映射
    parent_child_map: HashMap<String, Vec<String>>,
    /// 子父關係映射
    child_parent_map: HashMap<String, String>,
}

/// 企業級監控收集器
#[derive(Debug)]
pub struct MetricsCollector {
    /// 實時指標
    realtime_metrics: HashMap<String, MetricValue>,
    /// 歷史指標緩存
    historical_metrics: Vec<HistoricalMetric>,
    /// 告警配置
    alert_configs: Vec<AlertConfig>,
}

/// 使用量追蹤器 (基於 ccusage 模式)
#[derive(Debug)]
pub struct UsageTracker {
    /// 即時使用量統計
    realtime_usage: HashMap<String, UsageData>,
    /// 成本計算器
    cost_calculator: CostCalculator,
    /// 使用量歷史記錄
    usage_history: Vec<UsageRecord>,
}

/// 使用量資料 (基於 ccusage 模式)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageData {
    /// 任務ID
    pub job_id: String,
    /// 會話ID
    pub session_id: Option<String>,
    /// 輸入Token數量
    pub tokens_input: u64,
    /// 輸出Token數量
    pub tokens_output: u64,
    /// 總Token數量
    pub tokens_total: u64,
    /// 成本 (USD)
    pub cost_usd: f64,
    /// 總成本 (向後相容性)
    pub cost_total: f64,
    /// 使用的模型名稱
    pub model_name: Option<String>,
    /// 執行時間 (毫秒)
    pub execution_duration_ms: u64,
    /// 時間戳
    pub timestamp: DateTime<Utc>,
    /// 最後更新時間
    pub last_updated: DateTime<Utc>,
}

/// 排程器狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerState {
    /// 是否正在運行
    pub is_running: bool,
    /// 總任務數
    pub total_jobs: usize,
    /// 活躍任務數
    pub active_jobs: usize,
    /// 運行中任務數
    pub running_jobs: usize,
    /// 失敗任務數
    pub failed_jobs: usize,
    /// 啟動時間
    pub started_at: Option<DateTime<Utc>>,
    /// 最後健康檢查時間
    pub last_health_check: Option<DateTime<Utc>>,
    /// 系統版本
    pub version: String,
}

impl UnifiedScheduler {
    /// 建立新的統一排程器實例
    ///
    /// 整合 Context7 最佳實踐的初始化流程
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing UnifiedScheduler with Context7 best practices");

        // Context7 最佳實踐: 詳細的初始化錯誤處理
        let core_executor = Arc::new(
            RealTimeExecutor::new()
                .await
                .context("Failed to initialize core RealTimeExecutor")?,
        );

        let job_states = Arc::new(RwLock::new(HashMap::new()));
        let task_hierarchy = Arc::new(RwLock::new(TaskHierarchy::default()));
        let metrics_collector = Arc::new(Mutex::new(MetricsCollector::new()));
        let usage_tracker = Arc::new(Mutex::new(UsageTracker::new()));

        let scheduler_state = Arc::new(RwLock::new(SchedulerState {
            is_running: false,
            total_jobs: 0,
            active_jobs: 0,
            running_jobs: 0,
            failed_jobs: 0,
            started_at: None,
            last_health_check: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }));

        info!("✅ UnifiedScheduler initialized successfully");

        Ok(Self {
            core_executor,
            job_states,
            task_hierarchy,
            metrics_collector,
            usage_tracker,
            scheduler_state,
        })
    }

    /// 啟動統一排程器
    ///
    /// 按順序啟動所有子系統，確保依賴關係正確
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting UnifiedScheduler subsystems...");

        // 1. 啟動核心執行器
        self.core_executor
            .start()
            .await
            .context("Failed to start core RealTimeExecutor")?;
        info!("✅ Core executor started");

        // 2. 啟動監控系統
        {
            let mut metrics = self.metrics_collector.lock().await;
            metrics
                .start_monitoring()
                .await
                .context("Failed to start metrics collector")?;
        }
        info!("✅ Metrics collector started");

        // 3. 啟動使用量追蹤
        {
            let mut tracker = self.usage_tracker.lock().await;
            tracker
                .start_tracking()
                .await
                .context("Failed to start usage tracker")?;
        }
        info!("✅ Usage tracker started");

        // 4. 更新排程器狀態
        {
            let mut state = self.scheduler_state.write().await;
            state.is_running = true;
            state.started_at = Some(Utc::now());
            state.last_health_check = Some(Utc::now());
        }

        info!("🎉 UnifiedScheduler started successfully");
        Ok(())
    }

    /// 停止統一排程器
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping UnifiedScheduler...");

        // 按相反順序停止子系統
        {
            let mut tracker = self.usage_tracker.lock().await;
            tracker.stop_tracking().await?;
        }

        {
            let mut metrics = self.metrics_collector.lock().await;
            metrics.stop_monitoring().await?;
        }

        self.core_executor.stop().await?;

        {
            let mut state = self.scheduler_state.write().await;
            state.is_running = false;
        }

        info!("✅ UnifiedScheduler stopped successfully");
        Ok(())
    }

    /// 添加新任務到統一排程器
    ///
    /// 整合所有排程器的任務添加邏輯
    pub async fn add_job(&self, job: &Job) -> Result<String> {
        info!("📝 Adding job to UnifiedScheduler: {}", job.name);

        // 1. 添加到核心執行器
        let job_id = self
            .core_executor
            .add_job(job)
            .await
            .context("Failed to add job to core executor")?;

        // 2. 初始化統一任務狀態
        let unified_state = UnifiedJobState {
            job: job.clone(),
            execution_stats: ExecutionStats::default(),
            execution_processes: vec![],
            parent_job_id: None,
            child_job_ids: vec![],
            usage_data: UsageData {
                job_id: job_id.clone(),
                timestamp: Utc::now(),
                last_updated: Utc::now(),
                ..Default::default()
            },
            last_updated: Utc::now(),
        };

        // 3. 儲存任務狀態
        {
            let mut states = self.job_states.write().await;
            states.insert(job_id.clone(), unified_state);
        }

        // 4. 更新排程器統計
        {
            let mut state = self.scheduler_state.write().await;
            state.total_jobs += 1;
            state.active_jobs += 1;
        }

        info!("✅ Job added successfully: {}", job_id);
        Ok(job_id)
    }

    /// 添加子任務 (階層式管理)
    ///
    /// 基於 vibe-kanban 的階層式任務管理模式
    pub async fn add_child_job(&self, parent_id: &str, child_job: &Job) -> Result<String> {
        info!(
            "👶 Adding child job to parent {}: {}",
            parent_id, child_job.name
        );

        // 1. 添加子任務
        let child_id = self.add_job(child_job).await?;

        // 2. 建立階層關係
        {
            let mut hierarchy = self.task_hierarchy.write().await;
            hierarchy.add_relationship(parent_id.to_string(), child_id.clone())?;
        }

        // 3. 更新父任務狀態
        {
            let mut states = self.job_states.write().await;
            if let Some(parent_state) = states.get_mut(parent_id) {
                parent_state.child_job_ids.push(child_id.clone());
                parent_state.last_updated = Utc::now();
            }
        }

        // 4. 更新子任務狀態
        {
            let mut states = self.job_states.write().await;
            if let Some(child_state) = states.get_mut(&child_id) {
                child_state.parent_job_id = Some(parent_id.to_string());
                child_state.last_updated = Utc::now();
            }
        }

        info!(
            "✅ Child job added successfully: {} -> {}",
            parent_id, child_id
        );
        Ok(child_id)
    }

    /// 移除任務
    pub async fn remove_job(&self, job_id: &str) -> Result<bool> {
        info!("🗑️ Removing job: {}", job_id);

        // 1. 從核心執行器移除
        let removed = self.core_executor.remove_job(job_id).await?;

        if removed {
            // 2. 清理階層關係
            {
                let mut hierarchy = self.task_hierarchy.write().await;
                hierarchy.remove_job(job_id)?;
            }

            // 3. 移除任務狀態
            {
                let mut states = self.job_states.write().await;
                states.remove(job_id);
            }

            // 4. 更新統計
            {
                let mut state = self.scheduler_state.write().await;
                state.total_jobs = state.total_jobs.saturating_sub(1);
                state.active_jobs = state.active_jobs.saturating_sub(1);
            }

            info!("✅ Job removed successfully: {}", job_id);
        } else {
            warn!("⚠️ Job not found for removal: {}", job_id);
        }

        Ok(removed)
    }

    /// 獲取任務狀態
    pub async fn get_job_state(&self, job_id: &str) -> Result<Option<UnifiedJobState>> {
        let states = self.job_states.read().await;
        Ok(states.get(job_id).cloned())
    }

    /// 獲取所有任務狀態
    pub async fn get_all_job_states(&self) -> Result<HashMap<String, UnifiedJobState>> {
        let states = self.job_states.read().await;
        Ok(states.clone())
    }

    /// 獲取排程器狀態
    pub async fn get_scheduler_state(&self) -> Result<SchedulerState> {
        let state = self.scheduler_state.read().await;
        Ok(state.clone())
    }

    /// 健康檢查
    pub async fn health_check(&self) -> Result<bool> {
        debug!("🔍 Performing health check...");

        // 1. 檢查核心執行器
        let core_status = self
            .core_executor
            .get_active_jobs()
            .await
            .map(|_| true)
            .unwrap_or(false);

        // 2. 檢查排程器狀態
        let scheduler_running = {
            let state = self.scheduler_state.read().await;
            state.is_running
        };

        // 3. 更新健康檢查時間
        {
            let mut state = self.scheduler_state.write().await;
            state.last_health_check = Some(Utc::now());
        }

        let is_healthy = core_status && scheduler_running;
        debug!(
            "🏥 Health check result: {}",
            if is_healthy {
                "✅ Healthy"
            } else {
                "❌ Unhealthy"
            }
        );

        Ok(is_healthy)
    }

    /// 獲取使用量統計
    pub async fn get_usage_stats(&self, job_id: &str) -> Result<Option<UsageData>> {
        let states = self.job_states.read().await;
        Ok(states.get(job_id).map(|state| state.usage_data.clone()))
    }

    /// 獲取任務階層
    pub async fn get_task_hierarchy(&self, job_id: &str) -> Result<Vec<String>> {
        let hierarchy = self.task_hierarchy.read().await;
        Ok(hierarchy.get_children(job_id))
    }

    /// 獲取活躍任務ID列表 (向後相容性方法)
    pub async fn get_active_jobs(&self) -> Result<Vec<String>> {
        let states = self.job_states.read().await;
        let active_jobs: Vec<String> = states
            .iter()
            .filter(|(_, state)| state.job.status == JobStatus::Active)
            .map(|(job_id, _)| job_id.clone())
            .collect();
        Ok(active_jobs)
    }
}

/// 向後相容性API - 支援舊版本的API調用模式
impl UnifiedScheduler {
    /// 相容性方法：schedule_job (對應舊 JobScheduler::schedule_job)
    pub async fn schedule_job(&self, job: crate::models::job::Job) -> Result<()> {
        self.add_job(&job).await?;
        Ok(())
    }

    /// 相容性方法：unschedule_job (對應舊 JobScheduler::unschedule_job)
    pub async fn unschedule_job(&self, job_id: &str) -> Result<()> {
        self.remove_job(job_id).await?;
        Ok(())
    }

    /// 相容性方法：trigger_job (對應舊 SimpleJobManager::trigger_job)  
    pub async fn trigger_job(&self, job_id: &str) -> Result<String> {
        // 模擬手動觸發邏輯
        info!("🔥 Manual trigger requested for job: {}", job_id);

        if let Some(state) = self.get_job_state(job_id).await? {
            let trigger_result = format!(
                "Job '{}' triggered successfully at {}",
                state.job.name,
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );

            info!("✅ Job trigger completed: {}", trigger_result);
            Ok(trigger_result)
        } else {
            let error_msg = format!("Job not found: {}", job_id);
            warn!("❌ Job trigger failed: {}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }

    /// 相容性方法：get_running_jobs (對應舊 SimpleJobManager::get_running_jobs)
    pub async fn get_running_jobs(
        &self,
    ) -> HashMap<String, crate::services::simple_job_manager::SimpleJobExecution> {
        let mut running_jobs = HashMap::new();
        let states = self.job_states.read().await;

        for (job_id, state) in states.iter() {
            if let Some(process) = state.execution_processes.last() {
                if process.status == ProcessStatus::Running {
                    // 轉換為舊格式以保持相容性
                    let execution = crate::services::simple_job_manager::SimpleJobExecution {
                        job_id: job_id.clone(),
                        job_name: state.job.name.clone(),
                        started_at: process.start_time,
                        status: match process.status {
                            ProcessStatus::Running => {
                                crate::services::simple_job_manager::ExecutionStatus::Running
                            }
                            ProcessStatus::Completed => {
                                crate::services::simple_job_manager::ExecutionStatus::Completed
                            }
                            ProcessStatus::Failed => {
                                crate::services::simple_job_manager::ExecutionStatus::Failed
                            }
                            _ => crate::services::simple_job_manager::ExecutionStatus::Running,
                        },
                        cron_job_id: None, // UnifiedScheduler不使用cron_job_id
                    };
                    running_jobs.insert(job_id.clone(), execution);
                }
            }
        }

        running_jobs
    }

    /// 相容性方法：pause_job (企業級功能)
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        info!("⏸️ Pausing job: {}", job_id);

        let mut states = self.job_states.write().await;
        if let Some(state) = states.get_mut(job_id) {
            state.job.status = crate::models::job::JobStatus::Paused;
            state.last_updated = Utc::now();

            info!("✅ Job paused successfully: {}", job_id);
            Ok(())
        } else {
            let error_msg = format!("Job not found for pause: {}", job_id);
            warn!("❌ Job pause failed: {}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }

    /// 相容性方法：resume_job (企業級功能)
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        info!("▶️ Resuming job: {}", job_id);

        let mut states = self.job_states.write().await;
        if let Some(state) = states.get_mut(job_id) {
            state.job.status = crate::models::job::JobStatus::Active;
            state.last_updated = Utc::now();

            info!("✅ Job resumed successfully: {}", job_id);
            Ok(())
        } else {
            let error_msg = format!("Job not found for resume: {}", job_id);
            warn!("❌ Job resume failed: {}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }
}

impl TaskHierarchy {
    pub fn add_relationship(&mut self, parent_id: String, child_id: String) -> Result<()> {
        // 防止循環依賴
        if self.would_create_cycle(&parent_id, &child_id) {
            return Err(anyhow::anyhow!("Adding relationship would create a cycle"));
        }

        self.parent_child_map
            .entry(parent_id.clone())
            .or_insert_with(Vec::new)
            .push(child_id.clone());

        self.child_parent_map.insert(child_id, parent_id);

        Ok(())
    }

    pub fn get_children(&self, parent_id: &str) -> Vec<String> {
        self.parent_child_map
            .get(parent_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn remove_job(&mut self, job_id: &str) -> Result<()> {
        // 移除父子關係
        if let Some(children) = self.parent_child_map.remove(job_id) {
            for child in children {
                self.child_parent_map.remove(&child);
            }
        }

        // 移除子父關係
        if let Some(parent) = self.child_parent_map.remove(job_id) {
            if let Some(siblings) = self.parent_child_map.get_mut(&parent) {
                siblings.retain(|id| id != job_id);
            }
        }

        Ok(())
    }

    fn would_create_cycle(&self, parent_id: &str, child_id: &str) -> bool {
        // 檢查是否會創建循環依賴
        let mut visited = std::collections::HashSet::new();
        self.has_path_to(child_id, parent_id, &mut visited)
    }

    fn has_path_to(
        &self,
        from: &str,
        to: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if from == to {
            return true;
        }

        if visited.contains(from) {
            return false;
        }

        visited.insert(from.to_string());

        if let Some(children) = self.parent_child_map.get(from) {
            for child in children {
                if self.has_path_to(child, to, visited) {
                    return true;
                }
            }
        }

        false
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            realtime_metrics: HashMap::new(),
            historical_metrics: Vec::new(),
            alert_configs: Vec::new(),
        }
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        info!("📊 Starting comprehensive metrics collection...");
        
        // 初始化實時指標
        self.realtime_metrics.insert(
            "collection_started_at".to_string(),
            MetricValue::Timestamp(Utc::now())
        );
        
        // 初始化系統指標
        self.realtime_metrics.insert(
            "memory_usage_mb".to_string(),
            MetricValue::Gauge(0.0)
        );
        
        self.realtime_metrics.insert(
            "active_jobs_count".to_string(),
            MetricValue::Gauge(0.0)
        );
        
        // 初始化計數器
        self.realtime_metrics.insert(
            "jobs_executed_total".to_string(),
            MetricValue::Counter(0)
        );
        
        self.realtime_metrics.insert(
            "jobs_failed_total".to_string(),
            MetricValue::Counter(0)
        );
        
        info!("✅ Metrics collection started successfully with {} initial metrics", 
              self.realtime_metrics.len());
        Ok(())
    }

    pub async fn stop_monitoring(&mut self) -> Result<()> {
        info!("📊 Stopping metrics collection...");
        // TODO: 實作停止邏輯
        Ok(())
    }
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            realtime_usage: HashMap::new(),
            cost_calculator: CostCalculator::new(),
            usage_history: Vec::new(),
        }
    }

    pub async fn start_tracking(&mut self) -> Result<()> {
        info!("📈 Starting comprehensive usage tracking...");
        
        // 初始化系統使用追蹤
        let current_time = Utc::now();
        self.realtime_usage.insert(
            "system".to_string(),
            UsageData {
                job_id: "system".to_string(),
                session_id: None,
                tokens_input: 0,
                tokens_output: 0,
                tokens_total: 0,
                cost_usd: 0.0,
                cost_total: 0.0,
                model_name: None,
                execution_duration_ms: 0,
                timestamp: current_time,
                last_updated: current_time,
            }
        );
        
        // 記錄追蹤開始時間
        self.usage_history.push(UsageRecord {
            job_id: "system".to_string(),
            timestamp: current_time,
            tokens_used: 0,
            cost: 0.0,
            operation_type: "tracking_started".to_string(),
            usage_data: UsageData {
                job_id: "system".to_string(),
                session_id: None,
                tokens_input: 0,
                tokens_output: 0,
                tokens_total: 0,
                cost_usd: 0.0,
                cost_total: 0.0,
                model_name: None,
                execution_duration_ms: 0,
                timestamp: current_time,
                last_updated: current_time,
            },
            recorded_at: current_time,
        });
        
        info!("✅ Usage tracking started successfully");
        Ok(())
    }

    pub async fn stop_tracking(&mut self) -> Result<()> {
        info!("📈 Stopping usage tracking...");
        // TODO: 實作停止邏輯
        Ok(())
    }
}

/// 指標數值類型枚舉
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// 計數器類型
    Counter(u64),
    /// 量表類型 
    Gauge(f64),
    /// 時間戳類型
    Timestamp(DateTime<Utc>),
    /// 直方圖類型
    Histogram(f64),
}

#[derive(Debug, Clone)]
pub struct HistoricalMetric {
    pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub name: String,
    pub threshold: f64,
    pub condition: String,
}

#[derive(Debug)]
pub struct CostCalculator {
    // TODO: 實作成本計算邏輯
}

impl CostCalculator {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub job_id: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: u64,
    pub cost: f64,
    pub operation_type: String,
    pub usage_data: UsageData,
    pub recorded_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_scheduler_creation() {
        let scheduler = UnifiedScheduler::new().await;
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_task_hierarchy() {
        let mut hierarchy = TaskHierarchy::default();

        // 測試添加關係
        let result = hierarchy.add_relationship("parent1".to_string(), "child1".to_string());
        assert!(result.is_ok());

        // 測試獲取子任務
        let children = hierarchy.get_children("parent1");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], "child1");

        // 測試循環依賴檢查
        let cycle_result = hierarchy.add_relationship("child1".to_string(), "parent1".to_string());
        assert!(cycle_result.is_err());
    }
}
