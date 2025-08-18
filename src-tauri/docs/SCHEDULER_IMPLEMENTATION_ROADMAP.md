# 🚀 Claude Night Pilot 排程器實施路線圖

**文檔版本**: v1.0.0-implementation  
**創建時間**: 2025-08-17T17:15:00+00:00  
**基於**: SCHEDULER_COMPREHENSIVE_REFACTORING_PLAN.md 完整審計

---

## 🎯 實施優先級矩陣

| 任務 | 影響度 | 急迫性 | 實施難度 | 優先級 | 預估時間 |
|------|--------|--------|----------|--------|----------|
| 🔴 資料庫路徑統一 | 高 | 高 | 中 | P0 | 0.5天 |
| 🟡 統一排程器核心 | 高 | 中 | 高 | P1 | 1.5天 |
| 🟢 Enterprise功能整合 | 中 | 低 | 中 | P2 | 1天 |
| 🟣 E2E測試更新 | 中 | 中 | 低 | P3 | 0.5天 |
| 🔵 性能優化 | 低 | 低 | 中 | P4 | 0.5天 |

---

## 📅 詳細實施計劃

### **Day 1 Morning: P0 - 緊急資料庫修復**

#### 🔴 統一資料庫路徑 (0.5天)
```bash
# 1. 備份現有資料庫
cp claude-pilot.db claude-pilot-backup-$(date +%Y%m%d).db
cp claude-night-pilot.db claude-night-pilot-backup-$(date +%Y%m%d).db

# 2. 建立遷移腳本
touch src-tauri/migrations/urgent_path_unification.sql
```

**實施步驟**:
```rust
// src-tauri/src/database/migration.rs (新檔案)
use anyhow::Result;
use std::path::Path;

pub struct DatabasePathMigrator;

impl DatabasePathMigrator {
    pub async fn execute_emergency_unification() -> Result<()> {
        const LEGACY_PATH: &str = "claude-pilot.db";
        const UNIFIED_PATH: &str = "claude-night-pilot.db";
        
        // 1. 檢查舊資料庫是否存在
        if Path::new(LEGACY_PATH).exists() {
            println!("🔄 發現舊資料庫，開始遷移...");
            
            // 2. 備份現有資料
            Self::backup_existing_data().await?;
            
            // 3. 合併資料到統一路徑
            Self::merge_databases(LEGACY_PATH, UNIFIED_PATH).await?;
            
            // 4. 驗證資料完整性
            Self::verify_migration().await?;
            
            println!("✅ 資料庫路徑統一完成");
        }
        
        Ok(())
    }
}
```

### **Day 1 Afternoon: P1 - 核心架構重構**

#### 🟡 建立統一排程器 (1.5天)

**Step 1: 核心模組建立**
```rust
// src-tauri/src/scheduler/unified_scheduler.rs (新檔案)
use crate::models::job::{Job, JobStatus, JobExecutionResult};
use crate::scheduler::real_time_executor::{RealTimeExecutor, ExecutionStats};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 統一企業級排程器
/// 
/// 整合了所有現有排程器的最佳功能：
/// - RealTimeExecutor: 企業級執行引擎
/// - JobScheduler: 回調機制
/// - SimpleJobManager: 相容性API
/// - Context7最佳實踐: 錯誤處理與監控
pub struct UnifiedScheduler {
    /// 核心執行器 (基於 RealTimeExecutor)
    core_executor: Arc<RealTimeExecutor>,
    
    /// 企業級監控
    metrics_collector: Arc<MetricsCollector>,
    
    /// 使用量追蹤 (基於 ccusage 模式)
    usage_tracker: Arc<UsageTracker>,
    
    /// 任務關係管理 (基於 vibe-kanban 模式)
    task_hierarchy: Arc<RwLock<TaskHierarchy>>,
}

impl UnifiedScheduler {
    /// 建立新的統一排程器實例
    /// 
    /// 整合 Context7 最佳實踐的初始化流程
    pub async fn new() -> Result<Self> {
        // Context7 最佳實踐: 詳細的初始化錯誤處理
        let core_executor = Arc::new(
            RealTimeExecutor::new()
                .await
                .context("Failed to initialize core RealTimeExecutor")?
        );
        
        let metrics_collector = Arc::new(MetricsCollector::new().await?);
        let usage_tracker = Arc::new(UsageTracker::new().await?);
        let task_hierarchy = Arc::new(RwLock::new(TaskHierarchy::new()));
        
        Ok(Self {
            core_executor,
            metrics_collector,
            usage_tracker,
            task_hierarchy,
        })
    }
    
    /// 啟動統一排程器
    /// 
    /// 按順序啟動所有子系統
    pub async fn start(&self) -> Result<()> {
        tracing::info!("🚀 Starting UnifiedScheduler...");
        
        // 1. 啟動核心執行器
        self.core_executor.start().await
            .context("Failed to start core executor")?;
            
        // 2. 啟動監控系統
        self.metrics_collector.start().await
            .context("Failed to start metrics collector")?;
            
        // 3. 啟動使用量追蹤
        self.usage_tracker.start().await
            .context("Failed to start usage tracker")?;
            
        tracing::info!("✅ UnifiedScheduler started successfully");
        Ok(())
    }
}
```

**Step 2: 相容性API層**
```rust
// src-tauri/src/scheduler/compatibility.rs (新檔案)
/// 為現有代碼提供相容性API
/// 
/// 確保重構期間向後相容
pub struct CompatibilityLayer {
    unified_scheduler: Arc<UnifiedScheduler>,
}

impl CompatibilityLayer {
    /// SimpleJobManager 相容API
    pub async fn simple_job_manager_add_job(&self, job: &Job) -> Result<String> {
        // 轉發到統一排程器
        self.unified_scheduler.add_job(job).await
    }
    
    /// JobScheduler 相容API  
    pub async fn job_scheduler_schedule(&self, job: Job) -> Result<()> {
        // 轉換格式並轉發
        let job_id = self.unified_scheduler.add_job(&job).await?;
        tracing::info!("Job scheduled via compatibility layer: {}", job_id);
        Ok(())
    }
}
```

### **Day 2: P1 繼續 + P2 開始**

#### 🟡 完成統一排程器整合

**Step 3: 資料模型統一**
```rust
// src-tauri/src/models/unified_job.rs (新檔案)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 統一的任務模型
/// 
/// 整合所有現有任務結構的最佳功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedJob {
    // 基本屬性 (來自 models/job.rs)
    pub id: Uuid,
    pub name: String, 
    pub prompt_id: String,
    pub cron_expression: String,
    pub status: JobStatus,
    pub job_type: JobType,
    
    // 企業級功能 (來自 RealTimeExecutor)
    pub execution_stats: ExecutionStats,
    pub retry_config: RetryConfig,
    
    // vibe-kanban 模式整合
    pub parent_job_id: Option<Uuid>,
    pub execution_processes: Vec<JobExecutionProcess>,
    
    // ccusage 使用量追蹤
    pub usage_data: UsageData,
    
    // 時間戳
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 任務執行流程 (基於 vibe-kanban ExecutionProcess)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionProcess {
    pub id: Uuid,
    pub job_id: Uuid,
    pub process_type: ProcessType,
    pub status: ProcessStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessType {
    Setup,        // 前置準備
    Execution,    // 主要執行  
    Cleanup,      // 後續清理
    Validation,   // 結果驗證
}
```

#### 🟢 Enterprise 功能整合 (1天)

**Step 4: 使用量追蹤整合**
```rust
// src-tauri/src/tracking/usage_tracker.rs (新檔案)
use crate::models::usage::{UsageData, CostMode};
use anyhow::Result;

/// 基於 ccusage 模式的使用量追蹤器
pub struct UsageTracker {
    database: Arc<SqlitePool>,
    cost_calculator: CostCalculator,
}

impl UsageTracker {
    pub async fn track_execution(
        &self,
        job_id: &str,
        result: &JobExecutionResult
    ) -> Result<()> {
        let usage_data = UsageData {
            job_id: job_id.to_string(),
            session_id: result.session_id.clone(),
            tokens_input: result.tokens_input,
            tokens_output: result.tokens_output,
            cost_usd: result.calculate_cost(CostMode::Auto)?,
            model_name: result.model_name.clone(),
            timestamp: result.start_time,
        };
        
        // 儲存到統一資料庫
        self.store_usage_data(&usage_data).await?;
        
        // 更新即時統計
        self.update_realtime_metrics(&usage_data).await?;
        
        Ok(())
    }
}
```

### **Day 3: P2 完成 + P3 開始**

#### 🟢 完成 Enterprise 功能

**Step 5: 監控與告警系統**
```rust
// src-tauri/src/monitoring/metrics_collector.rs (新檔案)
use std::collections::HashMap;
use tokio::time::{interval, Duration};

/// 企業級監控收集器
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, MetricValue>>>,
    alert_manager: AlertManager,
}

impl MetricsCollector {
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // 收集系統指標
            self.collect_system_metrics().await?;
            
            // 收集任務指標  
            self.collect_job_metrics().await?;
            
            // 檢查告警條件
            self.check_alerts().await?;
        }
    }
}
```

#### 🟣 E2E測試更新 (0.5天)

**Step 6: 測試套件更新**
```rust
// src-tauri/tests/integration/unified_scheduler_tests.rs (新檔案)
#[cfg(test)]
mod unified_scheduler_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_scheduler_full_lifecycle() -> Result<()> {
        // 測試統一排程器完整生命週期
        let scheduler = UnifiedScheduler::new().await?;
        
        // 1. 啟動測試
        scheduler.start().await?;
        
        // 2. 任務管理測試
        let job = create_test_unified_job();
        let job_id = scheduler.add_job(&job).await?;
        
        // 3. 階層式任務測試 (vibe-kanban 模式)
        let child_job = create_child_job(&job_id);
        scheduler.add_child_job(&child_job).await?;
        
        // 4. 使用量追蹤測試
        let usage = scheduler.get_usage_stats(&job_id).await?;
        assert!(usage.total_executions >= 0);
        
        // 5. 清理測試
        scheduler.remove_job(&job_id).await?;
        scheduler.stop().await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_backward_compatibility() -> Result<()> {
        // 測試向後相容性
        let scheduler = UnifiedScheduler::new().await?;
        let compat_layer = CompatibilityLayer::new(scheduler);
        
        // 測試 SimpleJobManager API
        let simple_result = compat_layer
            .simple_job_manager_add_job(&create_legacy_job())
            .await;
        assert!(simple_result.is_ok());
        
        // 測試 JobScheduler API  
        let scheduler_result = compat_layer
            .job_scheduler_schedule(create_legacy_job())
            .await;
        assert!(scheduler_result.is_ok());
        
        Ok(())
    }
}
```

### **Day 4: P3 完成 + P4 + 最終整合**

#### 🟣 完成測試整合

**Step 7: Playwright E2E 更新**
```javascript
// tests/integration/unified-scheduler.spec.js (新檔案)
import { test, expect } from "@playwright/test";

test.describe("統一排程器 E2E 測試", () => {
  test("測試新統一排程器介面", async ({ page }) => {
    await page.goto("http://localhost:8080");
    
    // 驗證新的統一介面
    await expect(page.locator("#unified-scheduler-status")).toBeVisible();
    
    // 測試階層式任務建立
    await page.click('button:has-text("建立階層任務")');
    await page.fill("#parent-task-name", "主任務");
    await page.fill("#child-task-name", "子任務");
    
    await page.click('button:has-text("建立任務階層")');
    await expect(page.locator("text=任務階層建立成功")).toBeVisible();
    
    // 驗證任務關係顯示
    await expect(page.locator(".task-hierarchy-view")).toBeVisible();
  });
  
  test("測試使用量追蹤介面", async ({ page }) => {
    await page.goto("http://localhost:8080");
    
    // 檢查使用量儀表板
    await page.click('nav a:has-text("使用量統計")');
    await expect(page.locator("#usage-dashboard")).toBeVisible();
    
    // 驗證統計資料顯示
    await expect(page.locator(".token-usage-chart")).toBeVisible();
    await expect(page.locator(".cost-breakdown")).toBeVisible();
  });
});
```

#### 🔵 性能優化 (0.5天)

**Step 8: 最終優化**
```rust
// src-tauri/src/scheduler/optimization.rs (新檔案)
/// 性能優化模組
pub struct PerformanceOptimizer;

impl PerformanceOptimizer {
    /// 記憶體使用優化
    pub async fn optimize_memory_usage() -> Result<()> {
        // 1. 清理過期的執行記錄
        // 2. 優化資料庫連接池
        // 3. 實施智能緩存策略
        Ok(())
    }
    
    /// 並發性能優化
    pub async fn optimize_concurrency() -> Result<()> {
        // 1. 調整tokio執行緒池大小
        // 2. 優化鎖爭用
        // 3. 實施背壓機制
        Ok(())
    }
}
```

---

## 📊 實施檢核清單

### **Day 1 檢核點**
- [ ] 資料庫路徑統一完成
- [ ] 資料遷移驗證通過
- [ ] 統一排程器核心模組建立
- [ ] 基本功能測試通過

### **Day 2 檢核點** 
- [ ] 資料模型統一完成
- [ ] 相容性API層實作完成
- [ ] Enterprise功能整合開始
- [ ] 使用量追蹤系統就緒

### **Day 3 檢核點**
- [ ] 監控系統整合完成
- [ ] E2E測試套件更新
- [ ] 向後相容性驗證通過
- [ ] 性能基準測試執行

### **Day 4 檢核點**
- [ ] 最終整合測試通過
- [ ] 性能優化完成
- [ ] 文檔更新完成
- [ ] 部署就緒確認

---

## 🚨 風險管控

### **高風險項目**
1. **資料庫遷移** - 備份策略 + 段階式遷移
2. **向後相容性** - 完整回歸測試 + 相容性層

### **中風險項目**  
1. **性能影響** - 基準測試 + 漸進式部署
2. **功能整合** - 模組化設計 + 獨立測試

### **風險緩解措施**
```bash
# 1. 自動備份腳本
./scripts/backup-before-migration.sh

# 2. 回滾計劃
./scripts/rollback-to-previous-version.sh  

# 3. 健康檢查
./scripts/health-check-unified-scheduler.sh

# 4. 漸進式部署
./scripts/canary-deployment.sh
```

---

## 📈 成功指標

### **技術指標**
- ✅ 啟動時間 < 50ms (目標: <25ms)  
- ✅ 記憶體使用 降低 40%
- ✅ 代碼覆蓋率 > 90%
- ✅ 零停機時間遷移

### **業務指標**
- ✅ 向後相容性 100%
- ✅ 功能完整性 100%  
- ✅ 用戶體驗改善 25%
- ✅ 開發效率提升 30%

---

**實施狀態**: 🟡 準備就緒  
**下一步**: 執行 Day 1 資料庫路徑統一  
**負責團隊**: Backend Team + QA Team  
**預計完成**: 4個工作天