# 🔧 Claude Night Pilot 排程系統診斷與修復完整規劃

## 📊 **問題診斷總結**

### **已識別的根本問題**

1. **資料庫不一致問題** ❌

   - `cnp-unified.rs` 使用 `claude-pilot.db`
   - 其他組件使用 `claude-night-pilot.db`
   - **缺少 `jobs` 表** - 任務保存失敗的根本原因

2. **tokio-cron-scheduler 整合失敗** ❌

   - "Failed to create cron job" 警告
   - `JobScheduler` 未正確初始化或配置
   - 缺少實際的任務執行回調邏輯

3. **Prompt 執行邏輯缺失** ❌

   - `execute_job_logic` 僅為 TODO 佔位符
   - 無實際 Claude API 調用或任務執行
   - 缺少 prompt 內容檢索機制

4. **監控與診斷機制不足** ⚠️
   - 缺少排程器健康檢查
   - 任務執行狀態追蹤不完整
   - 錯誤診斷資訊不足

## 🎯 **分階段修復策略**

### **Phase 1: 資料庫 Schema 修復與統一**

**優先級**: 🔴 Critical
**預計時間**: 30 分鐘

#### **1.1 統一資料庫路徑**

- 修復 `cnp-unified.rs` 中的路徑不一致
- 確保所有組件使用 `claude-night-pilot.db`

#### **1.2 建立完整的 Jobs 表 Schema**

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    status TEXT NOT NULL,
    job_type TEXT NOT NULL,
    priority INTEGER DEFAULT 5,
    execution_options TEXT,
    retry_config TEXT,
    notification_config TEXT,
    next_run_time TEXT,
    last_run_time TEXT,
    execution_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    tags TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT,
    FOREIGN KEY (prompt_id) REFERENCES prompts(id)
);
```

#### **1.3 建立任務執行結果表**

```sql
CREATE TABLE IF NOT EXISTS job_executions (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    status TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_ms INTEGER,
    output TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id),
    FOREIGN KEY (prompt_id) REFERENCES prompts(id)
);
```

### **Phase 2: tokio-cron-scheduler 深度整合修復**

**優先級**: 🔴 Critical  
**預計時間**: 45 分鐘

#### **2.1 基於 Context7 最佳實踐重構 RealTimeExecutor**

- 修復 `JobScheduler` 初始化問題
- 實現正確的 Tokio Runtime 整合
- 加強錯誤處理與診斷

#### **2.2 實現實際的任務執行邏輯**

```rust
async fn execute_job_logic(job_id: &str, prompt_id: &str) -> Result<JobExecutionResult> {
    // 1. 從資料庫檢索 prompt 內容
    // 2. 解析 prompt 類型（Claude API / 本地執行）
    // 3. 執行實際任務
    // 4. 記錄執行結果
    // 5. 更新任務狀態
}
```

#### **2.3 增強 Cron 任務回調機制**

- 實現詳細的執行追蹤
- 加入重試機制
- 錯誤恢復策略

### **Phase 3: Prompt 執行引擎實現**

**優先級**: 🟡 High
**預計時間**: 60 分鐘

#### **3.1 Prompt 類型分類系統**

```rust
#[derive(Debug, Clone)]
pub enum PromptExecutionType {
    ClaudeApi {
        model: String,
        temperature: f32,
        max_tokens: u32,
    },
    LocalScript {
        interpreter: String,
        args: Vec<String>,
    },
    SystemCommand {
        command: String,
        working_dir: Option<PathBuf>,
    },
    HttpRequest {
        url: String,
        method: String,
        headers: HashMap<String, String>,
    },
}
```

#### **3.2 執行引擎實現**

- Claude API 整合 (使用 `claude-cli`)
- 本地腳本執行
- 系統命令執行
- HTTP 請求處理

#### **3.3 結果處理與儲存**

- 結構化執行結果
- 錯誤追蹤與分析
- 效能指標收集

### **Phase 4: 企業級監控與診斷**

**優先級**: 🟡 High
**預計時間**: 40 分鐘

#### **4.1 排程器健康檢查**

```rust
#[derive(Debug, Clone)]
pub struct SchedulerHealthStatus {
    pub is_running: bool,
    pub active_jobs_count: usize,
    pub total_executions: u64,
    pub success_rate: f64,
    pub average_execution_time: Duration,
    pub last_check: DateTime<Utc>,
    pub errors: Vec<SchedulerError>,
}
```

#### **4.2 實時監控儀表板**

- 任務執行狀態顯示
- 成功/失敗率統計
- 執行時間分析
- 資源使用監控

#### **4.3 告警與通知系統**

- 任務執行失敗告警
- 排程器離線檢測
- 效能異常通知
- 自動恢復機制

### **Phase 5: E2E 測試框架建立**

**優先級**: 🟢 Medium
**預計時間**: 50 分鐘

#### **5.1 單元測試**

```rust
#[tokio::test]
async fn test_job_creation_and_scheduling() {
    // 測試任務創建
    // 測試排程器註冊
    // 測試任務執行
    // 測試結果儲存
}

#[tokio::test]
async fn test_cron_expression_validation() {
    // 測試各種 cron 表達式
    // 測試錯誤處理
}

#[tokio::test]
async fn test_prompt_execution_types() {
    // 測試不同類型的 prompt 執行
    // 測試錯誤恢復
}
```

#### **5.2 整合測試**

- 資料庫操作測試
- 排程器生命週期測試
- 任務執行端到端測試
- 併發安全性測試

#### **5.3 效能測試**

- 大量任務調度測試
- 記憶體洩漏檢測
- 並發執行壓力測試
- 長期運行穩定性測試

## 🚀 **實施順序與里程碑**

### **里程碑 1: 核心修復 (75 分鐘)**

- ✅ 資料庫 Schema 統一
- ✅ tokio-cron-scheduler 整合修復
- ✅ 基礎 prompt 執行邏輯

### **里程碑 2: 功能完善 (100 分鐘)**

- ✅ 完整的執行引擎
- ✅ 監控與診斷系統
- ✅ 錯誤處理與恢復

### **里程碑 3: 品質保證 (130 分鐘)**

- ✅ 完整的測試覆蓋
- ✅ 效能優化
- ✅ 文檔與部署指南

## 📈 **成功指標**

### **功能指標**

- ✅ 14 個排程任務能正確執行
- ✅ 排程器註冊警告完全消除
- ✅ Prompt 內容正確檢索和執行
- ✅ 任務執行結果正確儲存

### **品質指標**

- 🎯 任務執行成功率 ≥ 99%
- 🎯 平均任務響應時間 ≤ 500ms
- 🎯 系統可用性 ≥ 99.9%
- 🎯 測試覆蓋率 ≥ 85%

### **維護性指標**

- 🔧 技術債務比例 ≤ 10%
- 🔧 代碼複雜度 ≤ 8 (循環複雜度)
- 🔧 MTTR (平均恢復時間) ≤ 5 分鐘
- 🔧 錯誤診斷時間 ≤ 1 分鐘

## 🛠️ **所需技術棧升級**

### **Context7 整合**

- 最新 Tokio 最佳實踐
- Rusqlite 連接池優化
- Tracing 結構化日誌
- Serde 序列化最佳化

### **新增依賴**

```toml
# 需要添加到 Cargo.toml
reqwest = { version = "0.11", features = ["json", "stream"] }
clap = { version = "4.0", features = ["derive"] }
which = "4.4"
tempfile = "3.8"
criterion = { version = "0.5", features = ["html_reports"] }
```

---

**📅 執行開始時間**: 2025-08-17T22:30:00+08:00  
**🎯 預期完成時間**: 2025-08-17T25:00:00+08:00  
**👨‍💻 負責人**: Claude Night Pilot AI Assistant  
**📋 追蹤方式**: TODO 列表 + 實時進度更新
