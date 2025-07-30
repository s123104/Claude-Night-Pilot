# Research Projects Analysis & Implementation Report

## 排程功能核心邏輯分析

### 1. CCAutoRenew - Bash 時間追蹤系統

**核心架構**：
```bash
# 時間檢測邏輯
get_minutes_until_reset() {
    local ccusage_cmd=$(get_ccusage_cmd)
    local output=$($ccusage_cmd blocks 2>/dev/null | grep -i "time remaining")
    
    # 解析時間格式：3h 45m
    if [[ "$output" =~ ([0-9]+)h[[:space:]]*([0-9]+)m ]]; then
        hours=${BASH_REMATCH[1]}
        minutes=${BASH_REMATCH[2]}
        echo $((hours * 60 + minutes))
    fi
}

# 智慧頻率調整
check_frequency() {
    local minutes_remaining=$1
    if [ "$minutes_remaining" -gt 30 ]; then
        sleep 600  # 10分鐘
    elif [ "$minutes_remaining" -gt 5 ]; then
        sleep 120  # 2分鐘
    else
        sleep 30   # 30秒
    fi
}
```

**Rust 轉換邏輯**：
```rust
// src-tauri/src/smart_scheduler.rs
use std::time::Duration;
use tokio::time::sleep;
use regex::Regex;

pub struct SmartScheduler {
    ccusage_command: String,
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    adaptive_intervals: Vec<(u32, Duration)>, // (分鐘閾值, 檢查間隔)
}

impl SmartScheduler {
    pub fn new() -> Self {
        Self {
            ccusage_command: Self::detect_ccusage_command(),
            last_check: None,
            adaptive_intervals: vec![
                (30, Duration::from_secs(600)), // >30分鐘: 10分鐘檢查
                (5, Duration::from_secs(120)),  // 5-30分鐘: 2分鐘檢查
                (0, Duration::from_secs(30)),   // <5分鐘: 30秒檢查
            ],
        }
    }

    pub async fn get_minutes_until_reset(&self) -> Result<u32> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&format!("{} blocks", self.ccusage_command))
            .output()
            .await?;

        let stdout = String::from_utf8(output.stdout)?;
        self.parse_time_remaining(&stdout)
    }

    fn parse_time_remaining(&self, output: &str) -> Result<u32> {
        let re = Regex::new(r"(\d+)h\s*(\d+)m")?;
        if let Some(caps) = re.captures(output) {
            let hours: u32 = caps[1].parse()?;
            let minutes: u32 = caps[2].parse()?;
            return Ok(hours * 60 + minutes);
        }
        
        let re_min = Regex::new(r"(\d+)m")?;
        if let Some(caps) = re_min.captures(output) {
            return Ok(caps[1].parse()?);
        }
        
        Err(anyhow::anyhow!("無法解析剩餘時間"))
    }

    pub async fn adaptive_sleep(&self, minutes_remaining: u32) {
        for (threshold, duration) in &self.adaptive_intervals {
            if minutes_remaining > *threshold {
                sleep(*duration).await;
                return;
            }
        }
        sleep(Duration::from_secs(30)).await; // 預設
    }
}
```

### 2. Claude-Autopilot - TypeScript 排程系統

**核心架構**：
```typescript
// src/services/scheduler/index.ts
function startScheduledSession(callback: () => void): void {
    const scheduledTime = parseTime(config.session.scheduledStartTime);
    const now = new Date();
    const scheduledDate = new Date(now);
    
    scheduledDate.setHours(scheduledTime.hours, scheduledTime.minutes, 0, 0);
    
    // 如果已過期，安排到明天
    if (scheduledDate <= now) {
        scheduledDate.setDate(scheduledDate.getDate() + 1);
    }
    
    const msUntilStart = scheduledDate.getTime() - now.getTime();
    
    scheduledTimer = setTimeout(() => {
        callback();
        startScheduledSession(callback); // 重新安排下一天
    }, msUntilStart);
}
```

**Rust 轉換邏輯**：
```rust
// src-tauri/src/session_scheduler.rs
use chrono::{DateTime, Local, Timelike, Duration};
use tokio::time::{sleep, Duration as TokioDuration};

pub struct SessionScheduler {
    scheduled_time: Option<(u32, u32)>, // (小時, 分鐘)
    timer_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SessionScheduler {
    pub fn new() -> Self {
        Self {
            scheduled_time: None,
            timer_handle: None,
        }
    }

    pub async fn schedule_session<F>(&mut self, time_str: &str, callback: F) -> Result<()>
    where
        F: Fn() + Send + 'static + Copy,
    {
        let (hours, minutes) = self.parse_time(time_str)?;
        self.scheduled_time = Some((hours, minutes));
        
        let next_execution = self.calculate_next_execution(hours, minutes);
        let duration_until = next_execution.signed_duration_since(Local::now());
        
        if let Ok(tokio_duration) = duration_until.to_std() {
            // 取消現有計時器
            if let Some(handle) = self.timer_handle.take() {
                handle.abort();
            }
            
            let handle = tokio::spawn(async move {
                sleep(tokio_duration).await;
                callback();
                
                // 自動安排下一天
                let mut scheduler = SessionScheduler::new();
                let _ = scheduler.schedule_session(&format!("{:02}:{:02}", hours, minutes), callback).await;
            });
            
            self.timer_handle = Some(handle);
        }
        
        Ok(())
    }

    fn parse_time(&self, time_str: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("無效的時間格式"));
        }
        
        let hours: u32 = parts[0].parse()?;
        let minutes: u32 = parts[1].parse()?;
        
        if hours >= 24 || minutes >= 60 {
            return Err(anyhow::anyhow!("無效的時間值"));
        }
        
        Ok((hours, minutes))
    }

    fn calculate_next_execution(&self, hours: u32, minutes: u32) -> DateTime<Local> {
        let now = Local::now();
        let mut target = now
            .with_hour(hours)
            .and_then(|t| t.with_minute(minutes))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap();
        
        // 如果已過期，安排到明天
        if target <= now {
            target = target + Duration::days(1);
        }
        
        target
    }
}
```

### 3. Vibe-Kanban - Rust 進程管理系統

**核心架構** (已是 Rust)：
```rust
// backend/src/services/process_service.rs
impl ProcessService {
    pub async fn auto_setup_and_execute(
        pool: &SqlitePool,
        app_state: &AppState,
        attempt_id: Uuid,
        task_id: Uuid,
        project_id: Uuid,
        operation: &str,
        operation_params: Option<serde_json::Value>,
    ) -> Result<(), TaskAttemptError> {
        let setup_completed = TaskAttempt::is_setup_completed(pool, attempt_id).await?;
        let project = Project::find_by_id(pool, project_id).await?;
        
        if Self::should_run_setup_script(&project) && !setup_completed {
            Self::execute_setup_with_delegation(
                pool, app_state, attempt_id, task_id, project_id,
                operation, operation_params
            ).await
        } else {
            match operation {
                "dev_server" => Self::start_dev_server(/* ... */).await,
                "coding_agent" => Self::start_coding_agent(/* ... */).await,
                _ => Err(TaskAttemptError::UnsupportedOperation(operation.to_string()))
            }
        }
    }
}
```

**最佳實踐提取**：
```rust
// src-tauri/src/process_orchestrator.rs
use uuid::Uuid;
use sqlx::SqlitePool;

pub struct ProcessOrchestrator {
    pool: SqlitePool,
    active_processes: std::collections::HashMap<Uuid, ProcessHandle>,
}

pub struct ProcessHandle {
    pub id: Uuid,
    pub process_type: ProcessType,
    pub status: ProcessStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub handle: Option<tokio::process::Child>,
}

#[derive(Debug, Clone)]
pub enum ProcessType {
    ClaudeExecution,
    SetupScript,
    CleanupScript,
    DevServer,
}

impl ProcessOrchestrator {
    pub async fn execute_with_prerequisites<F, Fut>(
        &mut self,
        job_id: Uuid,
        prerequisites: Vec<ProcessType>,
        main_operation: F,
    ) -> Result<ProcessHandle>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send,
    {
        // 檢查並執行前置條件
        for prerequisite in prerequisites {
            if !self.is_prerequisite_completed(job_id, &prerequisite).await? {
                self.execute_prerequisite(job_id, prerequisite).await?;
            }
        }
        
        // 執行主操作
        let process_id = Uuid::new_v4();
        let handle = tokio::spawn(async move {
            main_operation().await
        });
        
        let process_handle = ProcessHandle {
            id: process_id,
            process_type: ProcessType::ClaudeExecution,
            status: ProcessStatus::Running,
            created_at: chrono::Utc::now(),
            handle: None,
        };
        
        self.active_processes.insert(process_id, process_handle.clone());
        Ok(process_handle)
    }
}
```

## 冷卻檢測邏輯分析

### 1. Claude-Autopilot 冷卻檢測

**TypeScript 實現**：
```typescript
// src/services/usage/index.ts
export function isCurrentUsageLimit(output: string): boolean {
    const usageLimitPattern = /(Claude\s+)?usage\s+limit\s+reached.*?reset\s+at\s+(\d{1,2}[:\d]*(?:\s*[APM]{2})?(?:\s*\([^)]+\))?)/gi;
    const matches = [];
    let match;
    
    while ((match = usageLimitPattern.exec(output)) !== null) {
        matches.push({
            fullMatch: match[0],
            resetTime: match[2],
            index: match.index
        });
    }
    
    if (matches.length === 0) return false;
    
    const lastMatch = matches[matches.length - 1];
    const resetTime = lastMatch.resetTime;
    const resetDate = parseResetTime(resetTime, new Date());
    
    if (!resetDate) return true;
    
    const timeDiffMs = resetDate.getTime() - new Date().getTime();
    const timeDiffHours = timeDiffMs / (1000 * 60 * 60);
    
    return timeDiffMs > 0 && timeDiffHours <= 6;
}

function parseResetTime(resetTime: string, referenceTime: Date): Date | null {
    const cleanTime = resetTime.replace(/\s*\([^)]+\)/, '').trim();
    const ampmMatch = cleanTime.match(/(am|pm)$/i);
    const ampm = ampmMatch ? ampmMatch[0] : null;
    const timePartOnly = cleanTime.replace(/(am|pm)$/i, '').trim();
    
    let hours: number, minutes: number;
    
    if (timePartOnly.includes(':')) {
        const [hoursStr, minutesStr] = timePartOnly.split(':');
        hours = parseInt(hoursStr.replace(/[^\d]/g, ''));
        minutes = parseInt(minutesStr.replace(/[^\d]/g, ''));
    } else {
        hours = parseInt(timePartOnly.replace(/[^\d]/g, ''));
        minutes = 0;
    }
    
    // AM/PM 轉換
    if (ampm) {
        const isPM = /pm/i.test(ampm);
        if (isPM && hours !== 12) {
            hours += 12;
        } else if (!isPM && hours === 12) {
            hours = 0;
        }
    }
    
    const resetDate = new Date(referenceTime);
    resetDate.setHours(hours, minutes, 0, 0);
    
    if (resetDate.getTime() <= referenceTime.getTime()) {
        resetDate.setDate(resetDate.getDate() + 1);
    }
    
    return resetDate;
}
```

**Rust 轉換與增強**：
```rust
// src-tauri/src/cooldown_detector.rs
use regex::Regex;
use chrono::{DateTime, Local, NaiveTime, TimeZone};

pub struct CooldownDetector {
    usage_limit_regex: Regex,
    time_parsing_regex: Regex,
}

impl CooldownDetector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            usage_limit_regex: Regex::new(
                r"(?i)(Claude\s+)?usage\s+limit\s+reached.*?reset\s+at\s+(\d{1,2}[:\d]*(?:\s*[APM]{2})?(?:\s*\([^)]+\))?)"
            )?,
            time_parsing_regex: Regex::new(r"(\d{1,2})(?::(\d{2}))?\s*(am|pm)?")?,
        })
    }

    pub fn detect_cooldown(&self, claude_output: &str) -> Option<CooldownInfo> {
        let matches: Vec<_> = self.usage_limit_regex
            .captures_iter(claude_output)
            .collect();
        
        if matches.is_empty() {
            return None;
        }
        
        // 取最後一個匹配（最新的錯誤訊息）
        let last_match = matches.last()?;
        let reset_time_str = last_match.get(2)?.as_str();
        
        let reset_time = self.parse_reset_time(reset_time_str)?;
        let now = Local::now();
        
        if reset_time <= now {
            return Some(CooldownInfo {
                is_cooling: false,
                seconds_remaining: 0,
                next_available_time: None,
                reset_time: Some(reset_time),
                original_message: last_match.get(0)?.as_str().to_string(),
            });
        }
        
        let duration = reset_time.signed_duration_since(now);
        let seconds_remaining = duration.num_seconds().max(0) as u64;
        
        Some(CooldownInfo {
            is_cooling: true,
            seconds_remaining,
            next_available_time: Some(reset_time.into()),
            reset_time: Some(reset_time),
            original_message: last_match.get(0)?.as_str().to_string(),
        })
    }

    fn parse_reset_time(&self, reset_time_str: &str) -> Option<DateTime<Local>> {
        // 清理時區資訊
        let clean_time = reset_time_str
            .replace(char::is_whitespace, " ")
            .trim()
            .to_lowercase();
        
        let caps = self.time_parsing_regex.captures(&clean_time)?;
        
        let hours: u32 = caps.get(1)?.as_str().parse().ok()?;
        let minutes: u32 = caps.get(2)
            .map(|m| m.as_str().parse().unwrap_or(0))
            .unwrap_or(0);
        let ampm = caps.get(3).map(|m| m.as_str());
        
        let mut final_hours = hours;
        
        // AM/PM 處理
        if let Some(period) = ampm {
            match period {
                "pm" if hours != 12 => final_hours += 12,
                "am" if hours == 12 => final_hours = 0,
                _ => {}
            }
        }
        
        if final_hours >= 24 || minutes >= 60 {
            return None;
        }
        
        let now = Local::now();
        let naive_time = NaiveTime::from_hms_opt(final_hours, minutes, 0)?;
        let today = now.date_naive();
        
        // 嘗試今天
        let mut target = today.and_time(naive_time);
        let mut target_dt = Local.from_local_datetime(&target).single()?;
        
        // 如果已過期，移到明天
        if target_dt <= now {
            target = target + chrono::Duration::days(1);
            target_dt = Local.from_local_datetime(&target).single()?;
        }
        
        Some(target_dt)
    }

    pub fn extract_cooldown_seconds(&self, error_message: &str) -> Option<u64> {
        // 直接從錯誤訊息提取秒數的模式
        let patterns = [
            r"cooldown[:\s]+(\d+)s",
            r"wait\s+(\d+)\s+seconds",
            r"retry\s+in\s+(\d+)\s+seconds",
            r"rate\s+limit.*?(\d+)\s+seconds",
        ];
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(error_message) {
                    if let Ok(seconds) = caps[1].parse::<u64>() {
                        return Some(seconds);
                    }
                }
            }
        }
        
        None
    }
}

#[derive(Debug, Clone)]
pub struct CooldownInfo {
    pub is_cooling: bool,
    pub seconds_remaining: u64,
    pub next_available_time: Option<std::time::SystemTime>,
    pub reset_time: Option<DateTime<Local>>,
    pub original_message: String,
}
```

### 2. 當前專案冷卻檢測增強

**整合現有邏輯**：
```rust
// src-tauri/src/enhanced_executor.rs
use crate::cooldown_detector::{CooldownDetector, CooldownInfo};
use crate::smart_scheduler::SmartScheduler;

pub struct EnhancedClaudeExecutor {
    cooldown_detector: CooldownDetector,
    scheduler: SmartScheduler,
    retry_strategy: RetryStrategy,
}

impl EnhancedClaudeExecutor {
    pub async fn execute_with_auto_retry(
        &self,
        prompt: &str,
        options: &ExecutionOptions,
    ) -> Result<ClaudeResponse> {
        let mut attempts = 0;
        let max_attempts = options.max_retries;
        
        loop {
            match self.execute_claude_command(prompt, options).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    
                    // 檢測是否為冷卻錯誤
                    if let Some(cooldown) = self.cooldown_detector.detect_cooldown(&e.to_string()) {
                        if cooldown.is_cooling && attempts <= max_attempts {
                            tracing::info!(
                                "檢測到冷卻，將等待 {} 秒後重試 (嘗試 {}/{})",
                                cooldown.seconds_remaining,
                                attempts,
                                max_attempts
                            );
                            
                            // 智慧等待
                            self.smart_wait(cooldown.seconds_remaining).await;
                            continue;
                        }
                    }
                    
                    // 其他錯誤或超過重試次數
                    if attempts >= max_attempts {
                        return Err(e);
                    }
                    
                    // 指數退避
                    let delay = self.retry_strategy.calculate_delay(attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn smart_wait(&self, seconds: u64) {
        if seconds > 300 { // 超過5分鐘
            // 使用ccusage進行更精確的追蹤
            self.scheduler.adaptive_wait_with_ccusage().await;
        } else {
            // 直接等待
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        }
    }
}

pub struct RetryStrategy {
    base_delay: std::time::Duration,
    max_delay: std::time::Duration,
    multiplier: f64,
}

impl RetryStrategy {
    pub fn exponential_backoff() -> Self {
        Self {
            base_delay: std::time::Duration::from_millis(1000),
            max_delay: std::time::Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
    
    pub fn calculate_delay(&self, attempt: u32) -> std::time::Duration {
        let delay_ms = (self.base_delay.as_millis() as f64 
            * self.multiplier.powi(attempt as i32 - 1)) as u64;
        std::time::Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as u64))
    }
}
```

## 最佳實踐模式識別

### 1. 時間管理模式

**適應性頻率調整** (從 CCAutoRenew):
- 30分鐘以上：10分鐘檢查一次
- 5-30分鐘：2分鐘檢查一次  
- 5分鐘以下：30秒檢查一次

**實現**：
```rust
pub struct AdaptiveTimer {
    intervals: Vec<(Duration, Duration)>, // (剩餘時間閾值, 檢查間隔)
}
```

### 2. 錯誤處理模式

**多層次錯誤檢測** (從 Claude-Autopilot):
- 正則表達式模式匹配
- 時間解析與驗證
- 6小時窗口檢測

**實現**：
```rust
pub enum CooldownPattern {
    UsageLimitReached { reset_time: String },
    RateLimitExceeded { seconds: u64 },
    ApiQuotaExhausted { next_reset: DateTime<Local> },
}
```

### 3. 進程編排模式

**前置條件檢查** (從 Vibe-Kanban):
- 自動設置腳本執行
- 依賴關係管理
- 狀態持久化

**實現**：
```rust
pub trait ProcessDependency {
    async fn is_satisfied(&self) -> bool;
    async fn satisfy(&self) -> Result<()>;
}
```

### 4. 會話管理模式

**計劃模式排程** (從 Claude-code-schedule):
- 簡潔的時間解析
- 次日自動排程
- Ctrl+C 優雅退出

**實現**：
```rust
pub struct GracefulScheduler {
    shutdown_signal: tokio::sync::watch::Receiver<bool>,
    current_task: Option<tokio::task::JoinHandle<()>>,
}
```

## 模組化重構建議

### 1. 核心模組架構

```rust
// src-tauri/src/
├── core/
│   ├── scheduler.rs          // 統一排程介面
│   ├── cooldown.rs          // 冷卻檢測與管理
│   ├── retry.rs             // 重試策略
│   └── process.rs           // 進程編排
├── executors/
│   ├── claude.rs            // Claude 執行器
│   ├── enhanced.rs          // 增強執行器
│   └── mock.rs              // 測試用模擬器
├── scheduling/
│   ├── cron.rs              // Cron 排程
│   ├── adaptive.rs          // 適應性排程
│   └── session.rs           // 會話排程
├── detection/
│   ├── cooldown.rs          // 冷卻檢測
│   ├── usage.rs             // 使用量檢測
│   └── patterns.rs          // 模式匹配
└── utils/
    ├── time.rs              // 時間工具
    ├── process.rs           // 進程工具
    └── config.rs            // 配置管理
```

### 2. 統一介面設計

```rust
// src-tauri/src/core/scheduler.rs
#[async_trait]
pub trait Scheduler: Send + Sync {
    type Config;
    type Handle;
    
    async fn schedule(&mut self, config: Self::Config) -> Result<Self::Handle>;
    async fn cancel(&mut self, handle: Self::Handle) -> Result<()>;
    async fn reschedule(&mut self, handle: Self::Handle, config: Self::Config) -> Result<Self::Handle>;
    fn is_running(&self, handle: &Self::Handle) -> bool;
}

// 具體實現
pub struct CronScheduler {
    jobs: HashMap<Uuid, CronJob>,
}

pub struct AdaptiveScheduler {
    timers: HashMap<Uuid, AdaptiveTimer>,
}

pub struct SessionScheduler {
    sessions: HashMap<Uuid, ScheduledSession>,
}
```

### 3. 配置驅動系統

```rust
// src-tauri/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    pub cron: CronConfig,
    pub adaptive: AdaptiveConfig,
    pub session: SessionConfig,
    pub cooldown: CooldownConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronConfig {
    pub default_timezone: String,
    pub max_concurrent_jobs: u32,
    pub job_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub intervals: Vec<(u64, u64)>, // (threshold_minutes, check_interval_seconds)
    pub ccusage_integration: bool,
    pub fallback_to_time_based: bool,
}
```

## 技術棧整合建議

### 1. Tauri 2.0 + Tokio 整合

基於 Context7 文檔，建議使用：
```rust
// Cargo.toml
[dependencies]
tauri = { version = "2.0", features = ["async-io"] }
tokio = { version = "1.0", features = ["full"] }
tokio-cron-scheduler = "0.10"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

### 2. 非同步任務管理

```rust
// src-tauri/src/lib.rs
use tauri::async_runtime;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            
            // 啟動排程器
            async_runtime::spawn(async move {
                let mut scheduler = EnhancedScheduler::new().await?;
                scheduler.start_daemon().await?;
                Ok::<(), anyhow::Error>(())
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            schedule_job,
            cancel_job,
            get_cooldown_status,
            execute_prompt_with_retry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. 前端整合

```javascript
// src/main.js
class SchedulerManager {
    constructor() {
        this.activeJobs = new Map();
        this.cooldownTimer = null;
    }
    
    async scheduleJob(jobConfig) {
        try {
            const jobId = await invoke('schedule_job', { config: jobConfig });
            this.activeJobs.set(jobId, jobConfig);
            this.startStatusMonitoring(jobId);
            return jobId;
        } catch (error) {
            console.error('排程失敗:', error);
            throw error;
        }
    }
    
    async checkCooldownStatus() {
        const status = await invoke('get_cooldown_status');
        if (status.is_cooling) {
            this.startCooldownCountdown(status.seconds_remaining);
        }
        return status;
    }
    
    startCooldownCountdown(seconds) {
        if (this.cooldownTimer) {
            clearInterval(this.cooldownTimer);
        }
        
        let remaining = seconds;
        this.updateCooldownDisplay(remaining);
        
        this.cooldownTimer = setInterval(() => {
            remaining--;
            this.updateCooldownDisplay(remaining);
            
            if (remaining <= 0) {
                clearInterval(this.cooldownTimer);
                this.cooldownTimer = null;
                this.onCooldownComplete();
            }
        }, 1000);
    }
}
```

## 實施時程建議

### Phase 1: 核心重構 (Week 1-2)
- [ ] 建立模組化架構
- [ ] 實現統一排程介面  
- [ ] 整合現有冷卻檢測邏輯
- [ ] 基本的適應性排程

### Phase 2: 增強功能 (Week 3-4)  
- [ ] 複雜錯誤模式檢測
- [ ] 指數退避重試策略
- [ ] ccusage 深度整合
- [ ] 進程編排系統

### Phase 3: 前端整合 (Week 5-6)
- [ ] 前端排程管理介面
- [ ] 即時狀態更新
- [ ] 冷卻倒計時顯示
- [ ] 任務進度追蹤

### Phase 4: 測試與優化 (Week 7-8)
- [ ] 完整測試套件
- [ ] 效能優化
- [ ] 錯誤處理完善
- [ ] 文檔完成

## 成功指標

### 功能性指標
- [ ] 100% 冷卻檢測準確率
- [ ] 自動重試成功率 >90%
- [ ] 排程任務準時執行率 >95%
- [ ] 錯誤恢復時間 <30 秒

### 效能指標  
- [ ] 冷卻檢測延遲 <100ms
- [ ] 排程精度誤差 <1 秒
- [ ] 記憶體使用量 <50MB
- [ ] CPU 使用率 <5% (閒置時)

### 可靠性指標
- [ ] 24/7 運行穩定性
- [ ] 意外重啟後狀態恢復
- [ ] 網路中斷時優雅降級
- [ ] 資料一致性保證

這個分析報告為您的專案提供了完整的重構路線圖，整合了所有研究專案的最佳實踐，並針對 Rust + Tauri 環境進行了優化。