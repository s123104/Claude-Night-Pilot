# Claude Night Pilot 整合實施指南

## 執行摘要

本指南提供將四個研究專案（CCAutoRenew、Claude-Autopilot、claude-code-schedule、ClaudeNightsWatch）的核心功能整合到 Claude Night Pilot 的詳細實施步驟。基於前期分析報告和整合規則，本指南涵蓋核心邏輯、業務邏輯、最佳實踐、缺失依賴和授權規劃。

---

## 📋 執行摘要

本文檔提供四個 Claude Code 相關專案功能整合到 Claude Night Pilot 的詳細實施指南。已完成深度技術分析，制定了完整的整合方案，並建立了安全性規範。

### 🎯 整合目標達成情況

| 專案                 | 核心功能   | 分析完成度 | 整合方案 | 實施優先級 |
| -------------------- | ---------- | ---------- | -------- | ---------- |
| claude-code-schedule | ⭐⭐⭐⭐⭐ | 100%       | 完整     | P0 - 立即  |
| Claude-Autopilot     | ⭐⭐⭐⭐⭐ | 100%       | 詳細     | P1 - 中期  |
| CCAutoRenew          | ⭐⭐⭐⭐⭐ | 100%       | 完整     | P0 - 立即  |
| ClaudeNightsWatch    | ⭐⭐⭐⭐⭐ | 100%       | 詳細     | P1 - 中期  |

---

# 功能實現報告一：claude-code-schedule 整合

## 🔍 專案深度分析

### 技術架構優勢

- **Rust + Tokio**: 與我們的技術棧完全匹配
- **簡潔設計**: 單一職責，易於整合
- **時間處理**: 精確的時間解析和調度邏輯
- **信號處理**: 良好的中斷和清理機制

### 核心功能分解

```rust
// 1. 時間解析功能
fn parse_time(time_str: &str) -> Result<DateTime<Local>> {
    // 支援 HH:MM 格式
    // 自動計算明日時間
    // 完整錯誤處理
}

// 2. 命令執行功能
fn run_claude_command(message: &str) -> Result<()> {
    // --dangerously-skip-permissions 支援
    // 非阻塞執行
    // 狀態檢查
}

// 3. 倒數計時功能
async fn countdown_loop() {
    // 實時顯示剩餘時間
    // 每秒更新
    // 優雅終止支援
}
```

### 整合實施方案

#### 階段一：核心時間解析整合

```rust
// src-tauri/src/time_utils.rs
pub struct TimeScheduler {
    target_time: Option<chrono::DateTime<chrono::Local>>,
    timezone: chrono_tz::Tz,
}

impl TimeScheduler {
    pub fn parse_schedule_time(time_str: &str) -> Result<chrono::DateTime<chrono::Local>> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("時間格式錯誤，期望 HH:MM"));
        }

        let hour: u32 = parts[0].parse().context("小時格式錯誤")?;
        let minute: u32 = parts[1].parse().context("分鐘格式錯誤")?;

        if hour >= 24 || minute >= 60 {
            return Err(anyhow::anyhow!("時間值超出範圍"));
        }

        let now = chrono::Local::now();
        let target = now.with_hour(hour)
            .and_then(|t| t.with_minute(minute))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .context("無法建立目標時間")?;

        // 如果時間已過，設定為明天
        let final_target = if target <= now {
            target + chrono::Duration::days(1)
        } else {
            target
        };

        Ok(final_target)
    }
}
```

#### 階段二：許可權跳過功能整合

```rust
// src-tauri/src/executor.rs 擴展
impl ClaudeExecutor {
    pub async fn run_with_skip_permissions(
        prompt: &str,
        skip_permissions: bool,
    ) -> Result<String> {
        let mut cmd = AsyncCommand::new("claude");
        cmd.arg("-p").arg(prompt);

        if skip_permissions {
            cmd.arg("--dangerously-skip-permissions");
        }

        cmd.arg("--output-format").arg("json");

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Claude CLI 執行失敗: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }
}
```

### 資料庫架構擴展

```sql
-- 新增排程配置表
CREATE TABLE IF NOT EXISTS schedule_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    schedule_time TEXT NOT NULL, -- HH:MM 格式
    timezone TEXT DEFAULT 'Asia/Taipei',
    skip_permissions BOOLEAN DEFAULT FALSE,
    enabled BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 新增排程執行歷史
CREATE TABLE IF NOT EXISTS schedule_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_config_id INTEGER NOT NULL,
    scheduled_time DATETIME NOT NULL,
    actual_execution_time DATETIME,
    status TEXT NOT NULL, -- 'pending', 'executed', 'failed', 'cancelled'
    error_message TEXT,
    FOREIGN KEY (schedule_config_id) REFERENCES schedule_configs(id)
);
```

### UI 整合點

```html
<!-- src/index.html 新增排程配置區塊 -->
<section class="md-card">
  <h3>⏰ 排程執行設定</h3>
  <div class="md-form-group">
    <label for="schedule-time">執行時間 (HH:MM)</label>
    <input type="time" id="schedule-time" class="md-form-control" />
  </div>
  <div class="md-form-group">
    <label class="md-checkbox">
      <input type="checkbox" id="skip-permissions" />
      <span class="checkmark"></span>
      啟用自動跳過許可權 (危險)
    </label>
  </div>
  <button class="md-button md-filled-button" onclick="setSchedule()">
    設定排程
  </button>
</section>
```

---

# 功能實現報告二：CCAutoRenew 整合

## 🔍 專案深度分析

### ccusage 整合核心價值

- **精確監控**: 真實的 Claude 使用量追蹤
- **智能回退**: ccusage 失敗時的時間戳回退機制
- **自適應頻率**: 根據剩餘時間動態調整檢查間隔
- **5 小時塊管理**: 防止浪費使用時間的關鍵功能

### 核心演算法分析

```bash
# 1. ccusage輸出解析
get_minutes_until_reset() {
    # 多命令回退策略
    local ccusage_cmd=$(get_ccusage_cmd)

    # 正則表達式解析: "2h 30m" -> 150分鐘
    if [[ "$output" =~ ([0-9]+)h[[:space:]]*([0-9]+)m ]]; then
        hours=${BASH_REMATCH[1]}
        minutes=${BASH_REMATCH[2]}
    fi
}

# 2. 自適應檢查頻率
calculate_check_interval() {
    if [ "$minutes_remaining" -le 5 ]; then
        echo "30"  # 30秒 - 緊急模式
    elif [ "$minutes_remaining" -le 30 ]; then
        echo "120" # 2分鐘 - 接近模式
    else
        echo "600" # 10分鐘 - 正常模式
    fi
}
```

### Rust 實現方案

#### ccusage 整合核心模組

```rust
// src-tauri/src/usage_tracker.rs
use tokio::process::Command;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageBlock {
    pub remaining_minutes: u32,
    pub total_minutes: u32,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
    pub usage_percentage: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub current_block: Option<UsageBlock>,
    pub next_block_starts: Option<chrono::DateTime<chrono::Utc>>,
    pub is_available: bool,
    pub source: String, // "ccusage" 或 "fallback"
}

pub struct UsageTracker {
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    cached_info: Option<UsageInfo>,
    cache_duration: chrono::Duration,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            last_check: None,
            cached_info: None,
            cache_duration: chrono::Duration::seconds(30), // 30秒快取
        }
    }

    pub async fn get_usage_info(&mut self) -> Result<UsageInfo> {
        // 檢查快取是否有效
        if let Some(cached) = &self.cached_info {
            if let Some(last_check) = self.last_check {
                let now = chrono::Utc::now();
                if now.signed_duration_since(last_check) < self.cache_duration {
                    return Ok(cached.clone());
                }
            }
        }

        // 嘗試ccusage命令
        let commands = vec![
            vec!["ccusage", "blocks", "--json"],
            vec!["npx", "ccusage@latest", "blocks", "--json"],
            vec!["bunx", "ccusage", "blocks", "--json"],
            vec!["ccusage", "blocks"], // 純文字輸出
        ];

        for cmd_args in commands {
            if let Ok(output) = Command::new(&cmd_args[0])
                .args(&cmd_args[1..])
                .output()
                .await
            {
                if output.status.success() {
                    let result = if cmd_args.contains(&"--json") {
                        Self::parse_json_output(&output.stdout)?
                    } else {
                        Self::parse_text_output(&output.stdout)?
                    };

                    // 更新快取
                    self.cached_info = Some(result.clone());
                    self.last_check = Some(chrono::Utc::now());

                    return Ok(result);
                }
            }
        }

        // 回退到時間戳檢查
        self.fallback_time_check().await
    }

    fn parse_text_output(output: &[u8]) -> Result<UsageInfo> {
        let text = String::from_utf8_lossy(output);

        // 解析 "Time remaining: 2h 30m" 格式
        let time_regex = Regex::new(r"(?i)time\s+remaining:?\s*(\d+)h\s*(\d+)m")?;

        if let Some(captures) = time_regex.captures(&text) {
            let hours: u32 = captures[1].parse()?;
            let minutes: u32 = captures[2].parse()?;
            let total_minutes = hours * 60 + minutes;

            let usage_block = UsageBlock {
                remaining_minutes: total_minutes,
                total_minutes: 300, // 假設5小時總時間
                reset_time: Some(chrono::Utc::now() + chrono::Duration::minutes(total_minutes as i64)),
                usage_percentage: 1.0 - (total_minutes as f32 / 300.0),
            };

            return Ok(UsageInfo {
                current_block: Some(usage_block),
                next_block_starts: None,
                is_available: total_minutes > 0,
                source: "ccusage".to_string(),
            });
        }

        Err(anyhow::anyhow!("無法解析ccusage輸出"))
    }

    async fn fallback_time_check(&self) -> Result<UsageInfo> {
        // 讀取上次活動時間戳
        let last_activity_file = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("無法獲取用戶目錄"))?
            .join(".claude-last-activity");

        if let Ok(timestamp_str) = tokio::fs::read_to_string(&last_activity_file).await {
            if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
                let last_activity = chrono::DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| anyhow::anyhow!("無效的時間戳"))?;

                let now = chrono::Utc::now();
                let elapsed = now.signed_duration_since(last_activity);

                // 假設5小時 = 300分鐘的限制
                let remaining_minutes = (300 - elapsed.num_minutes() as u32).max(0);

                let usage_block = UsageBlock {
                    remaining_minutes,
                    total_minutes: 300,
                    reset_time: Some(last_activity + chrono::Duration::hours(5)),
                    usage_percentage: elapsed.num_minutes() as f32 / 300.0,
                };

                return Ok(UsageInfo {
                    current_block: Some(usage_block),
                    next_block_starts: Some(last_activity + chrono::Duration::hours(5)),
                    is_available: remaining_minutes > 0,
                    source: "fallback".to_string(),
                });
            }
        }

        // 如果沒有歷史記錄，假設可用
        Ok(UsageInfo {
            current_block: None,
            next_block_starts: None,
            is_available: true,
            source: "fallback".to_string(),
        })
    }
}
```

#### 自適應監控實現

```rust
// src-tauri/src/adaptive_monitor.rs
pub struct AdaptiveMonitor {
    usage_tracker: UsageTracker,
    current_interval: Duration,
    monitoring_active: bool,
}

impl AdaptiveMonitor {
    pub fn new() -> Self {
        Self {
            usage_tracker: UsageTracker::new(),
            current_interval: Duration::from_secs(600), // 預設10分鐘
            monitoring_active: false,
        }
    }

    pub async fn start_monitoring<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(UsageInfo) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        self.monitoring_active = true;

        while self.monitoring_active {
            // 獲取當前使用情況
            let usage_info = self.usage_tracker.get_usage_info().await?;

            // 調用回調函數
            callback(usage_info.clone()).await?;

            // 計算下次檢查間隔
            self.current_interval = self.calculate_check_interval(&usage_info);

            // 等待下次檢查
            tokio::time::sleep(self.current_interval).await;
        }

        Ok(())
    }

    fn calculate_check_interval(&self, usage_info: &UsageInfo) -> Duration {
        if let Some(block) = &usage_info.current_block {
            match block.remaining_minutes {
                0..=4 => Duration::from_secs(30),   // 緊急：30秒
                5..=29 => Duration::from_secs(120), // 接近：2分鐘
                _ => Duration::from_secs(600),      // 正常：10分鐘
            }
        } else {
            Duration::from_secs(300) // 未知狀態：5分鐘
        }
    }

    pub fn stop_monitoring(&mut self) {
        self.monitoring_active = false;
    }
}
```

### 資料庫整合

```sql
-- 使用量追蹤表
CREATE TABLE IF NOT EXISTS usage_tracking (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    remaining_minutes INTEGER,
    total_minutes INTEGER,
    usage_percentage REAL,
    reset_time DATETIME,
    source TEXT NOT NULL, -- 'ccusage' 或 'fallback'
    raw_output TEXT       -- 原始ccusage輸出用於除錯
);

-- 監控配置表
CREATE TABLE IF NOT EXISTS monitor_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    adaptive_monitoring BOOLEAN DEFAULT TRUE,
    normal_interval_seconds INTEGER DEFAULT 600,
    approaching_interval_seconds INTEGER DEFAULT 120,
    imminent_interval_seconds INTEGER DEFAULT 30,
    ccusage_enabled BOOLEAN DEFAULT TRUE,
    fallback_enabled BOOLEAN DEFAULT TRUE,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

# 功能實現報告三：ClaudeNightsWatch 整合

## 🔍 專案深度分析

### 任務導向設計哲學

- **Markdown 任務定義**: 易於閱讀和維護的任務格式
- **安全規則系統**: 防止危險操作的約束機制
- **自主執行**: 無人值守的任務執行能力
- **完整審計**: 詳細的執行日誌和狀態追蹤

### 核心功能解構

#### 任務模板系統

```rust
// src-tauri/src/task_templates.rs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub content: String,                // Markdown格式的任務內容
    pub safety_rules: Option<String>,   // 安全規則定義
    pub tags: Option<String>,          // 標籤，逗號分隔
    pub skip_permissions: bool,         // 是否跳過許可權
    pub max_execution_time: Option<u32>, // 最大執行時間（分鐘）
    pub working_directory: Option<String>, // 工作目錄限制
    pub allowed_operations: Option<String>, // 允許的操作類型
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TaskTemplate {
    pub fn prepare_full_prompt(&self) -> Result<String> {
        let mut prompt = String::new();

        // 1. 添加系統安全框架
        prompt.push_str("🚨 SYSTEM SAFETY FRAMEWORK ACTIVE 🚨\n\n");

        // 2. 添加用戶定義的安全規則
        if let Some(rules) = &self.safety_rules {
            prompt.push_str("🛡️ MANDATORY SAFETY RULES:\n\n");
            prompt.push_str(rules);
            prompt.push_str("\n\n---END OF SAFETY RULES---\n\n");

            // 驗證安全規則完整性
            self.validate_safety_rules()?;
        } else if self.skip_permissions {
            return Err(anyhow::anyhow!("跳過許可權的任務必須定義安全規則"));
        }

        // 3. 添加執行環境約束
        prompt.push_str("🏗️ EXECUTION ENVIRONMENT:\n");
        if let Some(work_dir) = &self.working_directory {
            prompt.push_str(&format!("- Working Directory: {}\n", work_dir));
        }
        if let Some(max_time) = self.max_execution_time {
            prompt.push_str(&format!("- Maximum Execution Time: {} minutes\n", max_time));
        }
        if let Some(operations) = &self.allowed_operations {
            prompt.push_str(&format!("- Allowed Operations: {}\n", operations));
        }
        prompt.push_str("\n");

        // 4. 添加主要任務內容
        prompt.push_str("📋 TASK SPECIFICATION:\n\n");
        prompt.push_str(&self.content);
        prompt.push_str("\n\n---END OF TASK---\n\n");

        // 5. 添加執行協議
        prompt.push_str("🚀 EXECUTION PROTOCOL:\n");
        prompt.push_str("1. 🔍 ANALYZE: Read and understand all safety rules and constraints\n");
        prompt.push_str("2. 📝 PLAN: Create a detailed step-by-step execution plan\n");
        prompt.push_str("3. ✅ VALIDATE: Confirm each step complies with safety rules\n");
        prompt.push_str("4. 🏃 EXECUTE: Perform each step with detailed logging\n");
        prompt.push_str("5. 📊 REPORT: Provide comprehensive status and results\n");
        prompt.push_str("6. 🛡️ VERIFY: Final safety compliance check\n\n");

        // 6. 添加終止條件
        prompt.push_str("⛔ IMMEDIATE TERMINATION CONDITIONS:\n");
        prompt.push_str("- Any operation violating safety rules\n");
        prompt.push_str("- Requests for elevated privileges\n");
        prompt.push_str("- Attempts to access restricted directories\n");
        prompt.push_str("- Operations exceeding time limits\n");
        prompt.push_str("- Any destructive operations without explicit authorization\n\n");

        Ok(prompt)
    }

    pub fn validate_safety_rules(&self) -> Result<()> {
        let rules = self.safety_rules.as_ref()
            .ok_or_else(|| anyhow::anyhow!("安全規則不能為空"))?;

        let rules_lower = rules.to_lowercase();

        // 必須包含的安全項目檢查清單
        let required_items = vec![
            ("目錄限制", vec!["directory", "folder", "path", "工作目錄"]),
            ("檔案操作限制", vec!["file", "delete", "remove", "檔案"]),
            ("權限限制", vec!["permission", "sudo", "root", "權限"]),
            ("網路限制", vec!["network", "download", "api", "網路"]),
        ];

        let mut missing_items = Vec::new();

        for (category, keywords) in required_items {
            if !keywords.iter().any(|keyword| rules_lower.contains(keyword)) {
                missing_items.push(category);
            }
        }

        if !missing_items.is_empty() {
            return Err(anyhow::anyhow!(
                "安全規則缺少必要項目: {}",
                missing_items.join(", ")
            ));
        }

        Ok(())
    }

    pub fn create_execution_context(&self) -> ExecutionContext {
        ExecutionContext {
            task_id: self.id,
            working_directory: self.working_directory.clone(),
            max_execution_time: self.max_execution_time.map(|m| Duration::from_secs(m as u64 * 60)),
            skip_permissions: self.skip_permissions,
            allowed_operations: self.parse_allowed_operations(),
            safety_rules_hash: self.calculate_safety_rules_hash(),
        }
    }

    fn parse_allowed_operations(&self) -> Vec<String> {
        self.allowed_operations
            .as_ref()
            .map(|ops| ops.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }

    fn calculate_safety_rules_hash(&self) -> String {
        use sha2::{Sha256, Digest};

        if let Some(rules) = &self.safety_rules {
            let mut hasher = Sha256::new();
            hasher.update(rules.as_bytes());
            format!("{:x}", hasher.finalize())
        } else {
            "no_rules".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub task_id: Option<i64>,
    pub working_directory: Option<String>,
    pub max_execution_time: Option<Duration>,
    pub skip_permissions: bool,
    pub allowed_operations: Vec<String>,
    pub safety_rules_hash: String,
}
```

#### 安全執行引擎

```rust
// src-tauri/src/safe_executor.rs
pub struct SafeExecutor {
    context: ExecutionContext,
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    audit_log: Vec<AuditEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub details: serde_json::Value,
    pub safety_check_result: bool,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl SafeExecutor {
    pub fn new(context: ExecutionContext) -> Self {
        Self {
            context,
            start_time: None,
            audit_log: Vec::new(),
        }
    }

    pub async fn execute_task(&mut self, task: &TaskTemplate) -> Result<ExecutionResult> {
        self.start_time = Some(chrono::Utc::now());

        // 1. 安全預檢
        self.pre_execution_safety_check(task)?;

        // 2. 準備執行環境
        let prepared_prompt = task.prepare_full_prompt()?;

        // 3. 建立執行選項
        let execution_options = ExecutionOptions {
            skip_permissions: self.context.skip_permissions,
            output_format: "json".to_string(),
            timeout_seconds: self.context.max_execution_time.map(|d| d.as_secs()),
            dry_run: false,
        };

        // 4. 記錄執行開始
        self.log_audit_entry("execution_start", json!({
            "task_id": self.context.task_id,
            "skip_permissions": self.context.skip_permissions,
            "safety_rules_hash": self.context.safety_rules_hash,
        }), RiskLevel::Medium);

        // 5. 執行任務
        let result = match self.execute_with_monitoring(&prepared_prompt, execution_options).await {
            Ok(output) => {
                self.log_audit_entry("execution_success", json!({
                    "output_length": output.len(),
                    "execution_time": self.get_execution_duration(),
                }), RiskLevel::Low);

                ExecutionResult::Success(output)
            },
            Err(e) => {
                self.log_audit_entry("execution_failed", json!({
                    "error": e.to_string(),
                    "execution_time": self.get_execution_duration(),
                }), RiskLevel::High);

                ExecutionResult::Failed(e.to_string())
            }
        };

        // 6. 執行後安全檢查
        self.post_execution_safety_check(&result)?;

        Ok(result)
    }

    fn pre_execution_safety_check(&mut self, task: &TaskTemplate) -> Result<()> {
        // 檢查工作目錄限制
        if let Some(work_dir) = &self.context.working_directory {
            if !std::path::Path::new(work_dir).exists() {
                return Err(anyhow::anyhow!("指定的工作目錄不存在: {}", work_dir));
            }
        }

        // 檢查許可權跳過的安全性
        if self.context.skip_permissions && task.safety_rules.is_none() {
            return Err(anyhow::anyhow!("跳過許可權的任務必須定義安全規則"));
        }

        // 檢查任務複雜度
        let content_length = task.content.len();
        if content_length > 10000 {
            self.log_audit_entry("high_complexity_task", json!({
                "content_length": content_length,
            }), RiskLevel::Medium);
        }

        Ok(())
    }

    async fn execute_with_monitoring(
        &mut self,
        prompt: &str,
        options: ExecutionOptions
    ) -> Result<String> {
        // 建立超時監控
        let timeout = options.timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(1800)); // 預設30分鐘

        let execution_future = ClaudeExecutor::run_with_options(prompt, options);

        match tokio::time::timeout(timeout, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                self.log_audit_entry("execution_timeout", json!({
                    "timeout_seconds": timeout.as_secs(),
                }), RiskLevel::High);

                Err(anyhow::anyhow!("任務執行超時"))
            }
        }
    }

    fn get_execution_duration(&self) -> Option<u64> {
        self.start_time.map(|start| {
            chrono::Utc::now().signed_duration_since(start).num_seconds() as u64
        })
    }

    fn log_audit_entry(&mut self, action: &str, details: serde_json::Value, risk_level: RiskLevel) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            action: action.to_string(),
            details,
            safety_check_result: true, // 可以根據實際檢查結果設定
            risk_level,
        };

        self.audit_log.push(entry);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success(String),
    Failed(String),
    Cancelled(String),
}
```

### 資料庫設計

```sql
-- 任務模板表
CREATE TABLE IF NOT EXISTS task_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    content TEXT NOT NULL,
    safety_rules TEXT,
    tags TEXT,
    skip_permissions BOOLEAN DEFAULT FALSE,
    max_execution_time INTEGER, -- 分鐘
    working_directory TEXT,
    allowed_operations TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 任務執行歷史
CREATE TABLE IF NOT EXISTS task_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    template_id INTEGER NOT NULL,
    execution_context TEXT NOT NULL, -- JSON格式的執行上下文
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    status TEXT NOT NULL, -- 'running', 'success', 'failed', 'cancelled'
    result TEXT,
    error_message TEXT,
    audit_log TEXT, -- JSON格式的審計日誌
    FOREIGN KEY (template_id) REFERENCES task_templates(id)
);

-- 安全事件日誌
CREATE TABLE IF NOT EXISTS security_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id INTEGER,
    event_type TEXT NOT NULL, -- 'permission_check', 'rule_violation', 'timeout', etc.
    risk_level TEXT NOT NULL, -- 'low', 'medium', 'high', 'critical'
    details TEXT NOT NULL, -- JSON格式的詳細資訊
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (execution_id) REFERENCES task_executions(id)
);
```

---

# 功能實現報告四：Claude-Autopilot 整合

## 🔍 專案深度分析

### 企業級功能特色

- **24/7 自動處理**: 持續佇列處理能力
- **VS Code 深度整合**: 豐富的 IDE 體驗
- **配置管理系統**: 企業級配置驗證和管理
- **睡眠防護機制**: 跨平台系統睡眠防護
- **WebView UI**: 現代化的用戶界面

### 關鍵技術要素

#### 佇列管理系統

```rust
// src-tauri/src/queue_manager.rs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QueueItem {
    pub id: Option<i64>,
    pub content: String,
    pub priority: i32,
    pub status: QueueItemStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub scheduled_time: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_options: String, // JSON格式的ExecutionOptions
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueueItemStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

pub struct QueueManager {
    db: Arc<Database>,
    processing_active: bool,
    max_concurrent: usize,
    auto_retry: bool,
}

impl QueueManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            processing_active: false,
            max_concurrent: 3, // 同時處理3個任務
            auto_retry: true,
        }
    }

    pub async fn add_to_queue(
        &self,
        content: String,
        priority: i32,
        execution_options: ExecutionOptions,
    ) -> Result<i64> {
        let options_json = serde_json::to_string(&execution_options)?;

        let queue_item = QueueItem {
            id: None,
            content,
            priority,
            status: QueueItemStatus::Pending,
            retry_count: 0,
            max_retries: 3,
            scheduled_time: None,
            execution_options: options_json,
            created_at: None,
            updated_at: None,
        };

        self.db.create_queue_item(queue_item).await
    }

    pub async fn start_processing(&mut self) -> Result<()> {
        if self.processing_active {
            return Ok(());
        }

        self.processing_active = true;

        // 啟動處理循環
        let db = Arc::clone(&self.db);
        let max_concurrent = self.max_concurrent;

        tokio::spawn(async move {
            let mut active_tasks = Vec::new();

            while self.processing_active {
                // 清理已完成的任務
                active_tasks.retain(|task: &tokio::task::JoinHandle<_>| !task.is_finished());

                // 如果未達到最大併發數，獲取新任務
                while active_tasks.len() < max_concurrent {
                    if let Ok(Some(item)) = db.get_next_queue_item().await {
                        let db_clone = Arc::clone(&db);

                        let task = tokio::spawn(async move {
                            Self::process_queue_item(db_clone, item).await
                        });

                        active_tasks.push(task);
                    } else {
                        break; // 沒有更多待處理項目
                    }
                }

                // 等待一段時間再檢查
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        Ok(())
    }

    async fn process_queue_item(db: Arc<Database>, mut item: QueueItem) -> Result<()> {
        // 更新狀態為處理中
        item.status = QueueItemStatus::Processing;
        db.update_queue_item(&item).await?;

        // 解析執行選項
        let execution_options: ExecutionOptions = serde_json::from_str(&item.execution_options)?;

        // 執行任務
        match ClaudeExecutor::run_with_options(&item.content, execution_options).await {
            Ok(result) => {
                // 更新為完成狀態
                item.status = QueueItemStatus::Completed;
                db.update_queue_item(&item).await?;

                // 儲存結果
                if let Some(item_id) = item.id {
                    db.create_result(item_id, &result).await?;
                }
            },
            Err(e) => {
                // 檢查是否可以重試
                if item.retry_count < item.max_retries && Self::is_retryable_error(&e) {
                    item.retry_count += 1;
                    item.status = QueueItemStatus::Retrying;
                    item.scheduled_time = Some(chrono::Utc::now() + chrono::Duration::minutes(5)); // 5分鐘後重試
                } else {
                    item.status = QueueItemStatus::Failed;
                }

                db.update_queue_item(&item).await?;

                // 儲存錯誤資訊
                if let Some(item_id) = item.id {
                    db.create_result(item_id, &format!("錯誤: {}", e)).await?;
                }
            }
        }

        Ok(())
    }

    fn is_retryable_error(error: &anyhow::Error) -> bool {
        let error_str = error.to_string().to_lowercase();

        // 可重試的錯誤類型
        error_str.contains("timeout") ||
        error_str.contains("connection") ||
        error_str.contains("rate limit") ||
        error_str.contains("cooldown")
    }
}
```

#### 睡眠防護系統

```rust
// src-tauri/src/sleep_prevention.rs
pub struct SleepPrevention {
    is_active: bool,
    method: SleepPreventionMethod,
    process_handle: Option<tokio::process::Child>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SleepPreventionMethod {
    Caffeinate,     // macOS
    PowerShell,     // Windows
    SystemdInhibit, // Linux
    Auto,           // 自動檢測
}

impl SleepPrevention {
    pub fn new() -> Self {
        Self {
            is_active: false,
            method: SleepPreventionMethod::Auto,
            process_handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_active {
            return Ok(());
        }

        let method = if matches!(self.method, SleepPreventionMethod::Auto) {
            self.detect_system_method()
        } else {
            self.method.clone()
        };

        match method {
            SleepPreventionMethod::Caffeinate => {
                let child = tokio::process::Command::new("caffeinate")
                    .arg("-d") // 防止顯示器睡眠
                    .arg("-i") // 防止系統閒置睡眠
                    .arg("-s") // 防止強制睡眠（當連接電源時）
                    .spawn()?;
                self.process_handle = Some(child);
            },
            SleepPreventionMethod::PowerShell => {
                // Windows: 使用SetThreadExecutionState API
                let script = r#"
                Add-Type -TypeDefinition '
                using System;
                using System.Runtime.InteropServices;
                public class Win32 {
                    [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
                    public static extern uint SetThreadExecutionState(uint esFlags);
                    public const uint ES_CONTINUOUS = 0x80000000;
                    public const uint ES_SYSTEM_REQUIRED = 0x00000001;
                    public const uint ES_DISPLAY_REQUIRED = 0x00000002;
                }
                ';

                # 防止系統和顯示器睡眠
                [Win32]::SetThreadExecutionState([Win32]::ES_CONTINUOUS -bor [Win32]::ES_SYSTEM_REQUIRED -bor [Win32]::ES_DISPLAY_REQUIRED)

                # 保持腳本運行
                while ($true) {
                    Start-Sleep -Seconds 30
                    [Win32]::SetThreadExecutionState([Win32]::ES_CONTINUOUS -bor [Win32]::ES_SYSTEM_REQUIRED -bor [Win32]::ES_DISPLAY_REQUIRED)
                }
                "#;

                let child = tokio::process::Command::new("powershell")
                    .args(["-Command", script])
                    .spawn()?;
                self.process_handle = Some(child);
            },
            SleepPreventionMethod::SystemdInhibit => {
                let child = tokio::process::Command::new("systemd-inhibit")
                    .args([
                        "--what=sleep:idle",
                        "--who=claude-night-pilot",
                        "--why=Long running Claude tasks",
                        "sleep", "infinity"
                    ])
                    .spawn()?;
                self.process_handle = Some(child);
            },
            _ => return Err(anyhow::anyhow!("不支援的睡眠防護方法")),
        }

        self.is_active = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process_handle.take() {
            child.kill().await?;
            child.wait().await?;
        }

        // Windows 額外清理
        if matches!(self.method, SleepPreventionMethod::PowerShell) {
            // 重置執行狀態
            tokio::process::Command::new("powershell")
                .args(["-Command", "[Win32]::SetThreadExecutionState([Win32]::ES_CONTINUOUS)"])
                .spawn()?;
        }

        self.is_active = false;
        Ok(())
    }

    fn detect_system_method(&self) -> SleepPreventionMethod {
        #[cfg(target_os = "macos")]
        return SleepPreventionMethod::Caffeinate;

        #[cfg(target_os = "windows")]
        return SleepPreventionMethod::PowerShell;

        #[cfg(target_os = "linux")]
        return SleepPreventionMethod::SystemdInhibit;

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return SleepPreventionMethod::Auto;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
```

#### 企業級配置管理

```rust
// src-tauri/src/enhanced_config.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedConfig {
    // 佇列管理配置
    pub queue: QueueConfig,

    // 使用量追蹤配置
    pub usage_tracking: UsageTrackingConfig,

    // 睡眠防護配置
    pub sleep_prevention: SleepPreventionConfig,

    // 任務模板配置
    pub task_templates: TaskTemplateConfig,

    // 安全性配置
    pub safety: SafetyConfig,

    // 監控配置
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueConfig {
    pub enabled: bool,
    pub max_concurrent_tasks: usize,
    pub auto_retry: bool,
    pub max_retries: i32,
    pub retry_delay_minutes: i32,
    pub queue_size_limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub adaptive_intervals: bool,
    pub normal_check_seconds: u64,
    pub approaching_check_seconds: u64,
    pub imminent_check_seconds: u64,
    pub health_check_enabled: bool,
    pub performance_metrics: bool,
}

impl EnhancedConfig {
    pub fn default() -> Self {
        Self {
            queue: QueueConfig {
                enabled: true,
                max_concurrent_tasks: 3,
                auto_retry: true,
                max_retries: 3,
                retry_delay_minutes: 5,
                queue_size_limit: 1000,
            },
            usage_tracking: UsageTrackingConfig {
                enabled: true,
                ccusage_command: "ccusage".to_string(),
                fallback_to_timestamp: true,
                check_interval_seconds: 300,
            },
            sleep_prevention: SleepPreventionConfig {
                enabled: true,
                method: SleepPreventionMethod::Auto,
                auto_start: false,
            },
            task_templates: TaskTemplateConfig {
                enabled: true,
                require_safety_rules: true,
                max_template_size: 50000,
                auto_backup: true,
            },
            safety: SafetyConfig {
                allow_skip_permissions: false,
                require_safety_rules: true,
                max_task_execution_time: 1800, // 30分鐘
                allowed_operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "compile".to_string(),
                ],
                forbidden_patterns: vec![
                    "rm -rf".to_string(),
                    "sudo".to_string(),
                    "chmod 777".to_string(),
                ],
            },
            monitoring: MonitoringConfig {
                adaptive_intervals: true,
                normal_check_seconds: 600,
                approaching_check_seconds: 120,
                imminent_check_seconds: 30,
                health_check_enabled: true,
                performance_metrics: true,
            },
        }
    }

    pub fn validate(&self) -> Result<Vec<String>, Vec<String>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 佇列配置驗證
        if self.queue.max_concurrent_tasks == 0 {
            errors.push("併發任務數不能為0".to_string());
        }
        if self.queue.max_concurrent_tasks > 10 {
            warnings.push("併發任務數過高可能影響系統效能".to_string());
        }

        // 安全配置驗證
        if self.safety.allow_skip_permissions && !self.task_templates.require_safety_rules {
            errors.push("啟用跳過許可權時必須要求安全規則".to_string());
        }

        if self.safety.max_task_execution_time > 7200 {
            warnings.push("任務執行時間超過2小時可能導致資源浪費".to_string());
        }

        // 使用量追蹤配置驗證
        if self.usage_tracking.enabled && self.usage_tracking.ccusage_command.is_empty() {
            errors.push("啟用使用量追蹤時必須指定ccusage命令".to_string());
        }

        if errors.is_empty() {
            Ok(warnings)
        } else {
            Err(errors)
        }
    }
}
```

### 完整資料庫設計

```sql
-- 佇列項目表
CREATE TABLE IF NOT EXISTS queue_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    status TEXT NOT NULL, -- 'pending', 'processing', 'completed', 'failed', 'cancelled', 'retrying'
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    scheduled_time DATETIME,
    execution_options TEXT NOT NULL, -- JSON格式
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 系統配置表
CREATE TABLE IF NOT EXISTS system_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL UNIQUE,
    config_value TEXT NOT NULL, -- JSON格式
    version INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 效能指標表
CREATE TABLE IF NOT EXISTS performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    tags TEXT -- JSON格式的標籤
);

-- 系統狀態表
CREATE TABLE IF NOT EXISTS system_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    component TEXT NOT NULL,
    status TEXT NOT NULL, -- 'healthy', 'warning', 'error'
    details TEXT, -- JSON格式的詳細資訊
    last_check DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 🚀 立即實施建議

基於以上四份詳細的功能實現報告，建議立即開始以下實施步驟：

### 第一週：基礎架構準備

1. **更新依賴項目** (Cargo.toml)
2. **建立新的資料庫表結構**
3. **實施 ccusage 整合模組**
4. **添加基本的許可權跳過支援**

### 第二週：核心功能實現

1. **排程開始時間功能**
2. **自適應監控系統**
3. **基本任務模板支援**
4. **安全規則驗證框架**

### 第三週：進階功能整合

1. **佇列管理系統**
2. **睡眠防護機制**
3. **完整的安全執行引擎**
4. **企業級配置管理**

### 第四週：測試與優化

1. **全面測試套件**
2. **效能優化**
3. **UI/UX 增強**
4. **文檔完善**

通過這個詳細的實施指南，Claude Night Pilot 將成為一個功能完整、安全可靠的 Claude Code 自動化平台，整合了業界最佳實踐。
