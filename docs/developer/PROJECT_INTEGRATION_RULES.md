# Claude Night Pilot - 專案整合規則與實施指南

**文檔版本**: v1.0.0  
**建立時間**: 2025-07-24T00:55:47+08:00  
**維護者**: Claude Night Pilot Team  
**適用範圍**: 整合四個Claude Code相關專案的功能  

---

## 📋 整合專案概覽

本文檔定義了從以下四個專案整合關鍵功能到Claude Night Pilot的完整規則：

| 專案 | 主要整合功能 | 整合優先級 | 實施階段 |
|------|-------------|-----------|---------|
| claude-code-schedule | 定時執行、許可權跳過 | P0 | 階段一 |
| Claude-Autopilot | 佇列管理、睡眠防護、配置系統 | P1 | 階段二 |
| CCAutoRenew | ccusage整合、自適應監控 | P0 | 階段一 |
| ClaudeNightsWatch | 任務模板、安全規則 | P1 | 階段二 |

---

## 🎯 核心整合原則

### 1. 架構一致性原則
- **保持Rust + Tauri架構**: 所有新功能必須用Rust實現
- **SQLite優先**: 數據持久化統一使用SQLite
- **API設計一致**: 新增的Tauri command遵循現有模式
- **錯誤處理統一**: 使用anyhow::Result<T>模式

### 2. 安全性原則
- **--dangerously-skip-permissions**: 僅在明確授權下使用
- **安全規則系統**: 必須實施安全約束檢查
- **權限最小化**: 僅賦予必要的系統權限
- **審計日誌**: 所有危險操作必須記錄

### 3. 漸進式整合原則
- **MVP優先**: 先實現核心功能，再添加進階特性
- **向後相容**: 不破壞現有功能
- **可配置性**: 新功能提供開關選項
- **測試覆蓋**: 新功能必須有對應測試

---

## 🏗️ 實施階段規劃

### 階段一：核心功能整合 (即刻執行)

#### 1.1 ccusage整合 (來自CCAutoRenew)

**目標**: 精確監控Claude使用量和冷卻時間

**技術實現**:
```rust
// src-tauri/src/usage.rs
use tokio::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageBlock {
    pub remaining_minutes: u32,
    pub total_minutes: u32,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub current_block: Option<UsageBlock>,
    pub next_block_starts: Option<chrono::DateTime<chrono::Utc>>,
    pub is_available: bool,
}

pub struct UsageTracker;

impl UsageTracker {
    pub async fn get_usage_info() -> anyhow::Result<UsageInfo> {
        // 嘗試多種ccusage調用方式
        let commands = vec![
            vec!["ccusage", "blocks", "--json"],
            vec!["npx", "ccusage@latest", "blocks", "--json"],
            vec!["bunx", "ccusage", "blocks", "--json"],
        ];
        
        for cmd in commands {
            if let Ok(output) = Command::new(&cmd[0])
                .args(&cmd[1..])
                .output()
                .await 
            {
                if output.status.success() {
                    return Self::parse_ccusage_output(&output.stdout);
                }
            }
        }
        
        // 回退到時間戳檢查
        Self::fallback_time_check().await
    }
    
    fn parse_ccusage_output(output: &[u8]) -> anyhow::Result<UsageInfo> {
        let text = String::from_utf8_lossy(output);
        
        // 解析ccusage JSON輸出
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            // 處理JSON格式回應
            return Self::parse_json_response(json);
        }
        
        // 解析文本格式回應
        Self::parse_text_response(&text)
    }
}
```

**資料庫擴展**:
```sql
-- 新增usage_tracking表
CREATE TABLE IF NOT EXISTS usage_tracking (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    remaining_minutes INTEGER,
    total_minutes INTEGER,
    reset_time DATETIME,
    source TEXT -- 'ccusage' 或 'fallback'
);
```

#### 1.2 排程開始時間功能 (來自CCAutoRenew)

**目標**: 防止浪費5小時使用塊，支援指定開始時間

**實現方案**:
```rust
// src-tauri/src/scheduler.rs 擴展
#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduledStart {
    pub enabled: bool,
    pub start_time: Option<chrono::NaiveTime>, // HH:MM格式
    pub start_datetime: Option<chrono::DateTime<chrono::Local>>, // 具體日期時間
    pub timezone: String,
}

impl TaskScheduler {
    pub async fn set_scheduled_start(&mut self, schedule: ScheduledStart) -> Result<()> {
        // 儲存排程開始時間到資料庫
        self.db.save_scheduled_start(&schedule).await?;
        
        if schedule.enabled {
            self.setup_start_time_monitor(schedule).await?;
        }
        
        Ok(())
    }
    
    async fn setup_start_time_monitor(&self, schedule: ScheduledStart) -> Result<()> {
        let target_time = self.calculate_target_time(schedule)?;
        let now = chrono::Local::now();
        
        if target_time > now {
            let delay = target_time.signed_duration_since(now);
            
            // 使用Tokio timer等待到指定時間
            tokio::spawn(async move {
                tokio::time::sleep(delay.to_std().unwrap()).await;
                // 開始監控和執行任務
                self.start_monitoring().await;
            });
        }
        
        Ok(())
    }
}
```

#### 1.3 --dangerously-skip-permissions支援

**目標**: 支援自動跳過許可權確認，提供安全開關

**實現方案**:
```rust
// src-tauri/src/executor.rs 擴展
impl ClaudeExecutor {
    pub async fn run_with_options(
        prompt: &str, 
        options: ExecutionOptions
    ) -> Result<String> {
        let mut cmd = AsyncCommand::new("claude");
        cmd.arg("-p").arg(prompt);
        
        if options.skip_permissions {
            cmd.arg("--dangerously-skip-permissions");
        }
        
        if options.output_format == "json" {
            cmd.arg("--output-format").arg("json");
        }
        
        let output = cmd.output().await?;
        // 處理結果...
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    pub skip_permissions: bool,
    pub output_format: String,
    pub timeout_seconds: Option<u64>,
    pub dry_run: bool,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            skip_permissions: false, // 默認安全模式
            output_format: "json".to_string(),
            timeout_seconds: Some(300),
            dry_run: false,
        }
    }
}
```

### 階段二：進階功能整合 (中期目標)

#### 2.1 任務模板系統 (來自ClaudeNightsWatch)

**目標**: 支援Markdown格式的任務定義和安全規則

**實現方案**:
```rust
// src-tauri/src/templates.rs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub content: String,          // 主要任務內容
    pub safety_rules: Option<String>, // 安全規則
    pub tags: Option<String>,
    pub skip_permissions: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl TaskTemplate {
    pub fn prepare_full_prompt(&self) -> String {
        let mut prompt = String::new();
        
        // 添加安全規則（如果存在）
        if let Some(rules) = &self.safety_rules {
            prompt.push_str("🛡️ IMPORTANT SAFETY RULES TO FOLLOW:\n\n");
            prompt.push_str(rules);
            prompt.push_str("\n\n---END OF SAFETY RULES---\n\n");
        }
        
        // 添加任務內容
        prompt.push_str("📋 TASK TO EXECUTE:\n\n");
        prompt.push_str(&self.content);
        prompt.push_str("\n\n---END OF TASK---\n\n");
        
        // 添加執行指引
        prompt.push_str("🚀 EXECUTION INSTRUCTIONS:\n");
        prompt.push_str("1. Read and understand the safety rules above\n");
        prompt.push_str("2. Create a step-by-step action plan for the task\n");
        prompt.push_str("3. Execute each step carefully\n");
        prompt.push_str("4. Provide detailed feedback on each step\n");
        prompt.push_str("5. Summarize the results when complete\n");
        
        prompt
    }
    
    pub fn validate_safety_rules(&self) -> Result<Vec<String>, Vec<String>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        if let Some(rules) = &self.safety_rules {
            // 檢查是否包含危險操作警告
            if !rules.to_lowercase().contains("delete") && 
               !rules.to_lowercase().contains("remove") {
                warnings.push("安全規則未明確禁止刪除操作".to_string());
            }
            
            // 檢查是否定義了工作目錄限制
            if !rules.to_lowercase().contains("directory") &&
               !rules.to_lowercase().contains("path") {
                warnings.push("安全規則未定義工作目錄限制".to_string());
            }
        } else {
            errors.push("缺少安全規則定義".to_string());
        }
        
        if errors.is_empty() {
            Ok(warnings)
        } else {
            Err(errors)
        }
    }
}
```

#### 2.2 睡眠防護機制 (來自Claude-Autopilot)

**目標**: 防止長時間任務執行時系統進入睡眠模式

**實現方案**:
```rust
// src-tauri/src/system.rs
use std::process::Command;

pub struct SleepPrevention {
    is_active: bool,
    method: SleepPreventionMethod,
    process_handle: Option<std::process::Child>,
}

#[derive(Debug, Clone)]
pub enum SleepPreventionMethod {
    Caffeinate,    // macOS
    PowerShell,    // Windows
    SystemdInhibit, // Linux
    Auto,          // 自動檢測
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
                let child = Command::new("caffeinate")
                    .arg("-d") // 防止顯示器睡眠
                    .arg("-i") // 防止系統閒置睡眠
                    .spawn()?;
                self.process_handle = Some(child);
            },
            SleepPreventionMethod::PowerShell => {
                // Windows PowerShell實現
                let child = Command::new("powershell")
                    .args(["-Command", "Add-Type -AssemblyName System.Windows.Forms; while($true) { [System.Windows.Forms.Cursor]::Position = [System.Windows.Forms.Cursor]::Position; Start-Sleep 30 }"])
                    .spawn()?;
                self.process_handle = Some(child);
            },
            SleepPreventionMethod::SystemdInhibit => {
                // Linux systemd-inhibit實現
                let child = Command::new("systemd-inhibit")
                    .args(["--what=sleep:idle", "--who=claude-night-pilot", "--why=Long running Claude tasks", "sleep", "infinity"])
                    .spawn()?;
                self.process_handle = Some(child);
            },
            _ => return Err(anyhow::anyhow!("不支援的睡眠防護方法")),
        }
        
        self.is_active = true;
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process_handle.take() {
            child.kill()?;
            child.wait()?;
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
    }
}
```

#### 2.3 自適應監控頻率 (來自CCAutoRenew)

**目標**: 根據剩餘時間動態調整檢查頻率

**實現方案**:
```rust
// src-tauri/src/adaptive_monitor.rs
pub struct AdaptiveMonitor {
    normal_interval: Duration,     // 10分鐘
    approaching_interval: Duration, // 2分鐘
    imminent_interval: Duration,   // 30秒
}

impl AdaptiveMonitor {
    pub fn new() -> Self {
        Self {
            normal_interval: Duration::from_secs(600),      // 10分鐘
            approaching_interval: Duration::from_secs(120), // 2分鐘
            imminent_interval: Duration::from_secs(30),     // 30秒
        }
    }
    
    pub fn get_check_interval(&self, minutes_remaining: u32) -> Duration {
        match minutes_remaining {
            0..=4 => self.imminent_interval,
            5..=29 => self.approaching_interval,
            _ => self.normal_interval,
        }
    }
    
    pub async fn start_monitoring<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(u32) -> BoxFuture<'static, Result<()>> + Send + 'static,
    {
        loop {
            // 獲取剩餘時間
            let usage_info = UsageTracker::get_usage_info().await?;
            let minutes_remaining = usage_info.current_block
                .map(|b| b.remaining_minutes)
                .unwrap_or(300); // 默認5小時
            
            // 調用回調函數
            callback(minutes_remaining).await?;
            
            // 計算下次檢查間隔
            let interval = self.get_check_interval(minutes_remaining);
            
            tokio::time::sleep(interval).await;
        }
    }
}
```

### 階段三：UI/UX增強 (長期目標)

#### 3.1 配置管理系統增強

**目標**: 參考Claude-Autopilot的配置驗證系統

**實現方案**:
```rust
// src-tauri/src/config.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedConfig {
    // 現有配置...
    
    // 新增整合功能配置
    pub usage_tracking: UsageTrackingConfig,
    pub sleep_prevention: SleepPreventionConfig,
    pub task_templates: TaskTemplateConfig,
    pub safety: SafetyConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageTrackingConfig {
    pub enabled: bool,
    pub ccusage_command: String,
    pub fallback_to_timestamp: bool,
    pub check_interval_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub allow_skip_permissions: bool,
    pub require_safety_rules: bool,
    pub max_task_execution_time: u64,
    pub allowed_operations: Vec<String>,
    pub forbidden_patterns: Vec<String>,
}

impl EnhancedConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // 驗證使用量追蹤配置
        if self.usage_tracking.enabled && self.usage_tracking.ccusage_command.is_empty() {
            errors.push("ccusage命令路徑不能為空".to_string());
        }
        
        // 驗證安全配置
        if self.safety.allow_skip_permissions && !self.safety.require_safety_rules {
            errors.push("啟用跳過許可權時必須要求安全規則".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## 🔒 安全性規則

### 1. --dangerously-skip-permissions使用規範

#### 何時允許使用：
- ✅ 用戶明確啟用該功能
- ✅ 任務包含安全規則定義
- ✅ 在隔離環境中執行
- ✅ 有完整的操作日誌

#### 何時禁止使用：
- ❌ 生產環境的關鍵系統
- ❌ 包含敏感數據的環境
- ❌ 缺少安全規則的任務
- ❌ 用戶未明確授權

#### 實施檢查：
```rust
pub fn validate_skip_permissions_request(
    user_config: &SafetyConfig,
    task: &TaskTemplate,
    execution_context: &ExecutionContext,
) -> Result<(), String> {
    // 檢查用戶配置
    if !user_config.allow_skip_permissions {
        return Err("用戶未啟用跳過許可權功能".to_string());
    }
    
    // 檢查任務安全規則
    if user_config.require_safety_rules && task.safety_rules.is_none() {
        return Err("任務缺少必要的安全規則".to_string());
    }
    
    // 檢查執行環境
    if execution_context.is_production && !execution_context.is_isolated {
        return Err("生產環境必須在隔離環境中執行".to_string());
    }
    
    Ok(())
}
```

### 2. 任務安全規則範本

#### 標準安全規則模板：
```markdown
# 任務安全規則

## 🛡️ 基本安全約束
- 僅在專案目錄內操作（/path/to/project）
- 禁止刪除任何檔案，除非明確指定
- 禁止修改系統設定檔
- 禁止執行需要root權限的命令

## 📁 檔案操作限制
- 允許讀取：專案目錄下的所有檔案
- 允許寫入：src/, docs/, tests/ 目錄
- 禁止寫入：.git/, node_modules/, target/ 目錄
- 備份重要檔案：package.json, Cargo.toml, tsconfig.json

## 🌐 網路操作限制
- 允許：npm install, cargo build, git pull/push
- 禁止：下載未知檔案、訪問外部API
- 代理設定：使用專案配置的代理

## ⚠️ 危險操作警告
如遇到以下操作請停止並詢問：
- 刪除超過10個檔案
- 修改權限設定
- 安裝新的系統依賴
- 修改防火牆規則
```

### 3. 執行監控與審計

#### 操作日誌格式：
```json
{
  "timestamp": "2025-07-24T00:55:47+08:00",
  "operation": "task_execution",
  "task_id": 123,
  "template_name": "code_review",
  "skip_permissions": true,
  "safety_rules_applied": true,
  "execution_context": {
    "working_directory": "/path/to/project",
    "user": "developer",
    "environment": "development"
  },
  "commands_executed": [
    "git status",
    "npm run lint",
    "npm test"
  ],
  "files_modified": [
    "src/main.rs",
    "docs/README.md"
  ],
  "result": "success",
  "duration_seconds": 45,
  "output_summary": "Completed linting and testing successfully"
}
```

---

## 📊 實施檢查清單

### 階段一檢查清單

- [ ] **ccusage整合**
  - [ ] 實現UsageTracker結構
  - [ ] 添加多重命令回退邏輯  
  - [ ] 建立usage_tracking資料表
  - [ ] 實現JSON和文本輸出解析
  - [ ] 添加Tauri命令接口

- [ ] **排程開始時間**
  - [ ] 擴展TaskScheduler支援排程開始
  - [ ] 實現時間計算邏輯
  - [ ] 添加配置選項
  - [ ] 建立UI控制界面
  - [ ] 實現時區支援

- [ ] **許可權跳過支援**
  - [ ] 擴展ExecutionOptions結構
  - [ ] 實現安全檢查機制
  - [ ] 添加用戶確認流程
  - [ ] 實施操作日誌記錄
  - [ ] 建立配置界面

### 階段二檢查清單

- [ ] **任務模板系統**
  - [ ] 建立TaskTemplate資料結構
  - [ ] 實現安全規則驗證
  - [ ] 建立模板管理界面
  - [ ] 實現模板匯入/匯出
  - [ ] 添加範例模板

- [ ] **睡眠防護機制**
  - [ ] 實現跨平台睡眠防護
  - [ ] 添加自動檢測邏輯
  - [ ] 實現生命週期管理
  - [ ] 建立狀態監控
  - [ ] 添加配置選項

- [ ] **自適應監控**
  - [ ] 實現AdaptiveMonitor結構
  - [ ] 整合到現有排程器
  - [ ] 添加監控狀態顯示
  - [ ] 實現頻率調整邏輯
  - [ ] 添加效能指標

### 測試檢查清單

- [ ] **單元測試**
  - [ ] ccusage解析邏輯測試
  - [ ] 時間計算功能測試
  - [ ] 安全規則驗證測試
  - [ ] 配置驗證測試

- [ ] **整合測試**
  - [ ] Claude CLI整合測試
  - [ ] 資料庫操作測試
  - [ ] 任務執行流程測試
  - [ ] 錯誤處理測試

- [ ] **E2E測試**
  - [ ] 完整任務執行測試
  - [ ] UI交互測試
  - [ ] 長時間運行測試
  - [ ] 多平台相容性測試

---

## 📈 成功指標

### 功能性指標
- ✅ ccusage整合成功率 > 95%
- ✅ 排程執行準確性 < 1分鐘誤差
- ✅ 任務執行成功率 > 90%
- ✅ 安全規則驗證覆蓋率 100%

### 效能指標
- ✅ 應用啟動時間 < 3秒
- ✅ 記憶體使用 < 200MB
- ✅ 監控檢查延遲 < 5秒
- ✅ 睡眠防護啟動時間 < 1秒

### 可用性指標
- ✅ 配置完成時間 < 5分鐘
- ✅ 任務模板建立時間 < 2分鐘
- ✅ 錯誤恢復時間 < 30秒
- ✅ 日誌查看響應時間 < 1秒

---

## 🔄 維護與更新策略

### 依賴更新策略
- **月度更新**: ccusage、Claude CLI相容性檢查
- **季度更新**: 安全規則範本審查
- **年度更新**: 整體架構評估

### 安全審查週期
- **每次發布前**: 安全規則驗證測試
- **季度**: 許可權使用模式分析
- **年度**: 完整安全審計

### 效能監控
- **即時**: 任務執行時間監控
- **每日**: 系統資源使用分析
- **每週**: 整合功能效能評估

---

**文檔結束**

此規則文檔將作為Claude Night Pilot專案整合外部功能的權威指南，所有開發決策都應參考此文檔中的原則和檢查清單。 