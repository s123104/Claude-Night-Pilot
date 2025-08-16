# Job 管理功能開發 Spec

## 📋 項目概述

Claude Night Pilot Job 管理功能將提供企業級的任務調度、執行和監控系統，支援 Claude CLI 的自動化工作流程。

## 🎯 功能目標

### 主要目標
- **可靠的任務調度**: 支援 Cron 表達式、一次性任務、間隔執行
- **智能重試機制**: 指數退避、自定義重試策略、失敗處理
- **實時監控**: 任務狀態追蹤、性能指標、健康檢查
- **高可用性**: 冷卻檢測、資源限制、併發控制
- **易用性**: GUI/CLI 雙介面、豐富的配置選項

### 技術目標
- **性能**: 單任務執行延遲 <100ms，支援 1000+ 併發任務
- **可靠性**: 99.9% 成功率，自動故障恢復
- **擴展性**: 模組化架構，支援外部通知、資源監控

## 🏗️ 系統架構

### 核心組件

#### 1. Job Engine (`JobEngine`)
```rust
pub struct JobEngine {
    scheduler: Arc<JobScheduler>,
    executor: Arc<ClaudeExecutor>,
    monitor: Arc<TaskMonitor>,
    retry_manager: Arc<RetryManager>,
    notification_service: Arc<NotificationService>,
}
```

#### 2. 調度器 (`JobScheduler`)
- **Cron 調度**: 基於 `tokio-cron-scheduler`
- **即時調度**: 立即執行和延遲執行
- **優先級隊列**: 支援 1-10 優先級
- **並發控制**: 基於信號量的併發限制

#### 3. 執行器 (`TaskExecutor`)
- **Claude CLI 集成**: 封裝現有的 ClaudeExecutor
- **會話管理**: 支援長期會話和 Git Worktree
- **流式輸出**: 實時日誌和進度回饋
- **資源監控**: 記憶體、CPU 使用率追蹤

#### 4. 重試管理 (`RetryManager`)
- **策略模式**: Fixed, ExponentialBackoff, Linear, Custom
- **智能退避**: 基於錯誤類型的動態調整
- **冷卻感知**: 與 Claude API 限制集成

#### 5. 通知服務 (`NotificationService`)
- **多通道支援**: System, Email, Webhook, Log
- **事件觸發**: 成功、失敗、開始、完成
- **自定義模板**: 支援變數替換

### 數據模型

#### Job 實體
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub prompt_id: String,
    pub cron_expression: String,
    pub status: JobStatus,
    pub job_type: JobType,
    pub priority: u8,
    pub execution_options: JobExecutionOptions,
    pub retry_config: RetryConfig,
    pub notification_config: Option<NotificationConfig>,
    // 執行統計
    pub execution_count: u64,
    pub failure_count: u64,
    pub avg_execution_time_ms: Option<u64>,
    pub success_rate: Option<f64>,
    // 時間管理
    pub next_run_time: Option<DateTime<Utc>>,
    pub last_run_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### 執行結果
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct JobExecution {
    pub id: String,
    pub job_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<u64>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub resource_usage: Option<ResourceUsage>,
}
```

## 🔧 API 設計

### REST API

#### Job 管理
```http
POST   /api/jobs                    # 創建任務
GET    /api/jobs                    # 列出任務
GET    /api/jobs/:id                # 獲取任務詳情
PUT    /api/jobs/:id                # 更新任務
DELETE /api/jobs/:id                # 刪除任務
POST   /api/jobs/:id/execute        # 手動執行
POST   /api/jobs/:id/pause          # 暫停任務
POST   /api/jobs/:id/resume         # 恢復任務
```

#### 執行管理
```http
GET    /api/jobs/:id/executions     # 執行歷史
GET    /api/executions/:id          # 執行詳情
POST   /api/executions/:id/cancel   # 取消執行
GET    /api/executions/:id/logs     # 執行日誌
```

#### 監控和統計
```http
GET    /api/jobs/stats              # 任務統計
GET    /api/jobs/health             # 健康檢查
GET    /api/jobs/metrics            # 性能指標
```

### WebSocket API
```typescript
// 實時任務狀態更新
interface TaskStatusUpdate {
  jobId: string;
  status: JobStatus;
  progress?: number;
  message?: string;
  timestamp: string;
}

// 執行日誌流
interface ExecutionLogStream {
  executionId: string;
  logLevel: 'info' | 'warn' | 'error';
  message: string;
  timestamp: string;
}
```

### CLI 命令

#### Job 管理
```bash
# 創建任務
cnp job create "Daily Report" --prompt-id 1 --cron "0 9 * * *" 

# 列出任務
cnp job list --status active --format table

# 執行任務
cnp job execute <job-id> --wait

# 暫停/恢復
cnp job pause <job-id>
cnp job resume <job-id>

# 查看執行歷史
cnp job history <job-id> --last 10

# 實時監控
cnp job monitor <job-id> --follow
```

#### 批量操作
```bash
# 批量創建
cnp job batch-create --file jobs.yaml

# 批量執行
cnp job batch-execute --tag "daily-tasks"

# 批量操作
cnp job batch-pause --status running
```

## 📊 配置系統

### Job 配置
```yaml
# job-config.yaml
name: "代碼品質檢查"
prompt_id: "code-quality-check"
schedule:
  type: "cron"
  expression: "0 2 * * *"
  timezone: "Asia/Taipei"

execution:
  timeout_seconds: 1800
  max_parallel: 1
  working_directory: "/project/src"
  environment:
    NODE_ENV: "production"
    
retry:
  max_attempts: 3
  strategy: "exponential_backoff"
  initial_interval: 60
  max_interval: 3600
  multiplier: 2.0

notifications:
  on_success: false
  on_failure: true
  channels:
    - type: "webhook"
      url: "https://hooks.slack.com/..."
    - type: "email"
      recipients: ["admin@company.com"]

resource_limits:
  max_memory_mb: 512
  max_cpu_percent: 50.0
```

### 全局配置
```toml
# claude-pilot.toml
[job_engine]
max_concurrent_jobs = 10
cleanup_retention_days = 30
health_check_interval = "30s"
metrics_enabled = true

[scheduler]
tick_interval = "1s"
timezone = "UTC"
max_missed_runs = 3

[executor]
default_timeout = "15m"
claude_cli_path = "npx @anthropic-ai/claude-code@latest"
session_timeout = "1h"

[notifications]
rate_limit_per_hour = 100
template_directory = "./templates"

[database]
cleanup_old_executions = true
max_execution_history = 1000
```

## 🚀 實施計劃

### Phase 1: 核心調度器 (Week 1-2)
- [ ] 實作 `JobScheduler` 基於 tokio-cron-scheduler
- [ ] 基本 Job CRUD 操作
- [ ] Cron 表達式解析和驗證
- [ ] 簡單的任務執行

### Phase 2: 執行引擎 (Week 3-4)
- [ ] 集成現有 ClaudeExecutor
- [ ] 重試機制和錯誤處理
- [ ] 並發控制和資源限制
- [ ] 執行狀態追蹤

### Phase 3: 監控和通知 (Week 5)
- [ ] 實時狀態更新 (WebSocket)
- [ ] 通知系統 (Email, Webhook)
- [ ] 性能指標收集
- [ ] 健康檢查端點

### Phase 4: CLI 增強 (Week 6)
- [ ] 完整 CLI 命令實作
- [ ] 批量操作支援
- [ ] 配置文件支援
- [ ] 互動式任務創建

### Phase 5: 高級功能 (Week 7-8)
- [ ] 任務依賴關係
- [ ] 條件執行
- [ ] 資源使用分析
- [ ] 任務模板系統

## 📋 BDD 測試策略

### Feature: Job 調度和執行
```gherkin
Feature: Job 調度和執行
  作為系統管理員
  我想要創建和管理定時任務
  以便自動化 Claude CLI 工作流程

  Scenario: 創建每日 Cron 任務
    Given 我有一個有效的 Prompt "代碼檢查"
    When 我創建一個 Job 使用 cron "0 9 * * *"
    Then 任務應該被保存到數據庫
    And 調度器應該在下次 9:00 AM 執行任務

  Scenario: 任務執行成功
    Given 我有一個活躍的 Job
    When 調度器觸發任務執行
    Then Claude CLI 應該被調用
    And 執行狀態應該被記錄
    And 成功完成後狀態應該是 "Completed"

  Scenario: 任務執行失敗並重試
    Given 我有一個配置了重試的 Job
    When 任務執行失敗
    Then 系統應該根據重試策略安排重試
    And 重試次數應該被記錄
    And 達到最大重試次數後標記為 "Failed"
```

### Feature: 冷卻檢測和恢復
```gherkin
Feature: Claude API 冷卻檢測
  作為系統
  我需要檢測 Claude API 冷卻狀態
  以便智能調整任務調度

  Scenario: 檢測 API 冷卻
    Given Claude CLI 返回冷卻錯誤
    When 系統檢測到冷卻狀態
    Then 所有任務應該被暫停
    And 系統應該定期檢查冷卻狀態
    And 冷卻結束後恢復任務調度

  Scenario: 冷卻期間的任務排隊
    Given 系統處於冷卻狀態
    When 有新任務需要執行
    Then 任務應該被添加到延遲隊列
    And 冷卻結束後按優先級執行
```

## 🔍 質量保證

### 測試覆蓋率目標
- **單元測試**: 90%+ 代碼覆蓋率
- **集成測試**: 所有 API 端點
- **端到端測試**: 關鍵用戶流程
- **性能測試**: 1000+ 併發任務

### 監控指標
- **可用性**: 99.9% uptime
- **性能**: 平均響應時間 <100ms
- **可靠性**: 任務成功率 >95%
- **資源使用**: CPU <80%, 記憶體 <500MB

### 故障處理
- **自動恢復**: 服務重啟、連接重連
- **降級策略**: 禁用非關鍵功能
- **警報系統**: 關鍵錯誤立即通知
- **備份機制**: 定期數據備份

## 📚 文檔和培訓

### 技術文檔
- API 參考文檔
- 配置指南
- 故障排除手冊
- 架構設計文檔

### 用戶文檔
- 快速開始指南
- 最佳實踐
- 常見用例示例
- FAQ 和疑難解答

## 🔮 未來擴展

### 高級功能
- **任務依賴**: DAG 工作流程
- **條件執行**: 基於結果的條件分支
- **分佈式調度**: 多實例集群支援
- **插件系統**: 自定義執行器

### 集成擴展
- **監控集成**: Prometheus, Grafana
- **日誌集成**: ELK Stack, Fluentd
- **雲平台**: AWS Lambda, Google Cloud Functions
- **CI/CD 集成**: GitHub Actions, GitLab CI

---

**文檔版本**: v1.0  
**創建日期**: 2025-08-16  
**最後更新**: 2025-08-16  
**負責人**: Claude Night Pilot Team