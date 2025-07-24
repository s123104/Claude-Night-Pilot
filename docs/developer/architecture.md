# Claude Night Pilot - 系統架構文檔

**最後更新**: 2025-07-24T23:23:43+08:00  
**架構版本**: v1.0.0  
**技術棧**: Tauri 2 + Rust + SQLite + htmx  
**參考標準**: [Context7:tauri-apps/tauri-docs:2025-07-24T23:23:43+08:00]

## 🏗️ 整體架構概覽

Claude Night Pilot 採用現代化的跨平台桌面應用架構，基於 Tauri 2 框架實現前後端分離，並整合四大核心模組提供完整的 Claude CLI 管理解決方案。

### 架構特點
- **跨平台支援**: macOS, Windows, Linux
- **記憶體安全**: Rust 後端確保系統穩定性
- **輕量高效**: 原生性能，低資源佔用
- **模組化設計**: 高內聚低耦合的組件架構
- **企業級安全**: 多層安全檢查與審計機制

## 🔧 技術棧詳細

### 前端層 (Frontend)
```
┌─────────────────────────────────────┐
│            Web UI Layer             │
├─────────────────────────────────────┤
│ • htmx - 現代化互動框架              │
│ • HTML5/CSS3 - 標準化介面           │
│ • Material Design 3.0 - 設計系統    │
│ • JavaScript ES6+ - 客戶端邏輯      │
└─────────────────────────────────────┘
```

### 通訊層 (IPC Communication)
```
┌─────────────────────────────────────┐
│         Tauri Command Layer         │
├─────────────────────────────────────┤
│ • @tauri-apps/api - 前端 API        │
│ • JSON-RPC - 結構化通訊協議         │
│ • Event System - 事件驅動通知       │
│ • Capability System - 權限控制      │
└─────────────────────────────────────┘
```

### 後端層 (Backend)
```
┌─────────────────────────────────────┐
│           Rust Core Layer           │
├─────────────────────────────────────┤
│ • Tauri 2 Runtime - 應用框架        │
│ • Tokio - 異步運行時                │
│ • SQLx - 型別安全資料庫操作          │
│ • Serde - 序列化/反序列化           │
│ • Chrono - 時間處理與時區支援        │
└─────────────────────────────────────┘
```

## 🏢 四大核心模組架構

### 模組整合關係圖

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Smart Scheduler │◄──►│ Adaptive Monitor │◄──►│ Usage Tracker   │
│   (智能排程)      │    │   (自適應監控)    │    │  (使用量追蹤)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
          │                        │                        │
          │                        │                        │
          ▼                        ▼                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Executor (安全執行系統)                        │
│              ├─ ExecutionOptions                                │
│              ├─ SecurityCheck                                   │
│              ├─ AuditLog                                        │
│              └─ ClaudeExecutor                                  │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SQLite Database                            │
│              ├─ usage_records                                   │
│              ├─ execution_audits                                │
│              ├─ scheduled_tasks                                 │
│              └─ monitoring_events                               │
└─────────────────────────────────────────────────────────────────┘
```

### 1. Usage Tracker (使用量追蹤模組)
**檔案**: `src-tauri/src/usage_tracker.rs` (~400 行)

**職責**:
- Claude 使用量即時監控
- 多命令回退機制 (ccusage → npx → bunx)
- 智能快取與歷史記錄
- 使用量預測與分析

**核心 API**:
```rust
#[tauri::command] get_usage_status() -> UsageInfo
#[tauri::command] get_usage_history(hours: u32) -> Vec<UsageRecord>
#[tauri::command] invalidate_usage_cache()
```

### 2. Executor (安全執行系統)
**檔案**: `src-tauri/src/executor.rs` (~500 行)

**職責**:
- Claude CLI 安全執行
- 多層安全檢查機制
- 完整操作審計記錄
- 智能重試與錯誤處理

**核心 API**:
```rust
#[tauri::command] execute_claude_with_options(prompt, options) -> String
#[tauri::command] execute_claude_safe(prompt) -> String
#[tauri::command] validate_execution_options(options) -> SecurityCheckResult
```

### 3. Adaptive Monitor (自適應監控系統)
**檔案**: `src-tauri/src/adaptive_monitor.rs` (~600 行)

**職責**:
- 六層動態監控頻率調整
- 事件驅動狀態通知
- 資源使用優化
- 監控數據統計分析

**核心 API**:
```rust
#[tauri::command] get_monitoring_status() -> MonitoringStatus
#[tauri::command] trigger_monitoring_check()
#[tauri::command] update_monitoring_config(config)
```

### 4. Smart Scheduler (智能排程系統)
**檔案**: `src-tauri/src/smart_scheduler.rs` (~550 行)

**職責**:
- 時區感知的智能排程
- 5小時塊保護機制
- 工作時間最佳化
- 排程決策引擎

**核心 API**:
```rust
#[tauri::command] make_scheduling_decision(...) -> SchedulingDecision
#[tauri::command] schedule_smart_task(...) -> String
#[tauri::command] get_scheduled_tasks() -> Vec<ScheduledTask>
```

## 🗄️ 資料庫架構

### SQLite 資料庫設計

```sql
-- 使用量記錄表
CREATE TABLE usage_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    total_usage_usd REAL,
    remaining_time_minutes INTEGER,
    status TEXT,
    metadata TEXT  -- JSON格式的額外資訊
);

-- 執行審計表
CREATE TABLE execution_audits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    prompt_hash TEXT,  -- SHA256雜湊
    execution_options TEXT,  -- JSON格式
    risk_level TEXT,  -- Low/Medium/High/Critical
    result_summary TEXT,
    duration_ms INTEGER
);

-- 排程任務表
CREATE TABLE scheduled_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_name TEXT,
    prompt TEXT,
    schedule_expression TEXT,  -- Cron格式
    next_execution DATETIME,
    status TEXT,  -- pending/running/completed/failed
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 監控事件表
CREATE TABLE monitoring_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    event_type TEXT,  -- status_change/threshold_crossed/error
    old_value TEXT,
    new_value TEXT,
    metadata TEXT  -- JSON格式
);
```

## 🔐 安全架構

### 多層安全檢查機制

```
┌─────────────────────────────────────┐
│           Security Layers           │
├─────────────────────────────────────┤
│ 1. Environment Authorization       │
│ 2. Working Directory Validation    │
│ 3. Dangerous Mode Detection        │
│ 4. Command Safety Verification     │
│ 5. Execution Audit Logging         │
│ 6. Result Sanitization            │
└─────────────────────────────────────┘
```

### 權限管理
- **最小權限原則**: 預設拒絕所有危險操作
- **明確授權**: 透過 `--dangerously-skip-permissions` 明確授權
- **完整審計**: 所有操作記錄 SHA256 雜湊與風險等級
- **即時監控**: 危險操作即時警告與攔截

## 📡 事件驅動架構

### 事件流程圖

```
Frontend Event ──► Tauri Command ──► Rust Handler
       │                                     │
       │◄────── JSON Response ◄──────────────┤
       │                                     │
       │           Background Tasks          │
       │◄────── Event Emission ◄─────── Tokio Spawn
```

### 主要事件類型
- **usage_update**: 使用量變更通知
- **monitoring_status_change**: 監控狀態變更
- **task_scheduled**: 任務排程通知
- **security_alert**: 安全警告事件
- **system_ready**: 系統就緒通知

## 🚀 部署架構

### 目錄結構
```
tauri-app/
├── src/                      # 前端源碼
│   ├── index.html           # 主頁面
│   ├── main.js             # 應用邏輯
│   └── styles.css          # 樣式文件
├── src-tauri/              # 後端源碼
│   ├── src/
│   │   ├── main.rs         # 應用入口
│   │   ├── lib.rs          # 模組匯出
│   │   ├── usage_tracker.rs
│   │   ├── executor.rs
│   │   ├── adaptive_monitor.rs
│   │   └── smart_scheduler.rs
│   ├── Cargo.toml          # Rust 依賴
│   ├── tauri.conf.json     # Tauri 配置
│   └── capabilities/       # 權限定義
├── package.json            # Node.js 配置
└── README.md              # 專案說明
```

### 建置流程
1. **前端建置**: 編譯 HTML/CSS/JS 資源
2. **後端編譯**: Rust 代碼編譯為原生二進制檔案
3. **資源打包**: 整合前後端為單一可執行檔案
4. **平台打包**: 生成平台特定的安裝包

## 📈 性能架構

### 性能指標
- **啟動時間**: < 300ms (Release 版本)
- **記憶體使用**: < 50MB (系統閒置)
- **API 響應**: < 50ms (本地 SQL 查詢)
- **GUI 互動**: < 100ms (用戶操作響應)

### 優化策略
- **智能快取**: 30秒使用量快取避免重複查詢
- **異步處理**: Tokio 非同步運行時最大化並發
- **資源池化**: 資料庫連接池管理
- **事件驅動**: 避免輪詢式的資源浪費

## 🔄 擴展架構

### 模組化設計原則
- **高內聚**: 每個模組職責單一且完整
- **低耦合**: 模組間透過定義良好的介面通訊
- **可插拔**: 支援模組的動態載入與卸載
- **可配置**: 所有行為透過配置檔案調整

### 未來擴展方向
- **插件系統**: 第三方模組整合機制
- **分散式架構**: 支援多實例協作
- **雲端整合**: 數據同步與備份機制
- **AI 輔助**: 智能化決策與建議系統

---

**維護者**: Claude Night Pilot 開發團隊  
**技術標準**: 基於 Tauri 官方文檔最佳實踐  
**更新週期**: 隨技術棧版本更新同步維護 