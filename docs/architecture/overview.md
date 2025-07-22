# 🏗️ Claude Night Pilot - 系統架構概覽

> **文件版本**: v2.0.0  
> **最後更新**: 2025-07-23T03:14:08+08:00  
> **負責人**: 系統架構師

## 📋 目錄

1. [系統概述](#系統概述)
2. [架構設計原則](#架構設計原則)
3. [系統架構圖](#系統架構圖)
4. [技術棧架構](#技術棧架構)
5. [資料流設計](#資料流設計)
6. [模組設計](#模組設計)
7. [部署架構](#部署架構)

---

## 🎯 系統概述

### 專案定位

Claude Night Pilot 是一個現代化的 Claude CLI 自動化工具，專為需要大量使用 Claude 的開發者和創作者設計。系統採用 **極簡架構** 與 **高效能設計**，確保在提供強大功能的同時保持輕量化。

### 核心架構特性

| 特性         | 說明             | 技術實現             |
| ------------ | ---------------- | -------------------- |
| **單體應用** | 避免微服務複雜性 | Tauri 2.0 單一二進位 |
| **本地優先** | 零雲端依賴       | SQLite + 本地存儲    |
| **跨平台**   | 支援主流作業系統 | Rust + WebView       |
| **輕量化**   | 極小資源占用     | < 10MB 檔案大小      |

---

## 🎨 架構設計原則

### SOLID 原則應用

#### 1. 單一職責原則 (SRP)

```rust
// ✅ 每個模組只負責一項核心功能
pub mod prompt_manager;    // 只處理 Prompt CRUD
pub mod claude_executor;   // 只處理 Claude CLI 執行
pub mod scheduler;         // 只處理排程管理
pub mod database;          // 只處理資料持久化
```

#### 2. 開放封閉原則 (OCP)

```rust
// ✅ 透過 trait 實現擴展性
pub trait ExecutorInterface {
    async fn execute(&self, prompt: &str) -> Result<String>;
}

pub struct ClaudeExecutor;
pub struct MockExecutor;  // 測試用實作

impl ExecutorInterface for ClaudeExecutor { /* ... */ }
impl ExecutorInterface for MockExecutor { /* ... */ }
```

### DRY 原則 (Don't Repeat Yourself)

```rust
// ✅ 共享組件設計
pub mod common {
    pub mod error_handling;
    pub mod validation;
    pub mod logging;
    pub mod config;
}
```

### KISS 原則 (Keep It Simple, Stupid)

- **前端**: 純 HTML + htmx，避免複雜 SPA 框架
- **狀態管理**: 直接使用 Tauri IPC，無需複雜狀態庫
- **資料庫**: SQLite 單檔，無需複雜 ORM

---

## 🏛️ 系統架構圖

### 高層架構視圖

```mermaid
graph TB
    subgraph "使用者介面層 (Presentation Layer)"
        GUI[GUI - WebView]
        CLI[CLI - 命令列工具]
    end

    subgraph "應用服務層 (Application Layer)"
        TAURI[Tauri 2.0 框架]
        IPC[IPC 通訊層]
        ROUTER[路由與命令分發]
    end

    subgraph "業務邏輯層 (Business Layer)"
        PM[Prompt Manager]
        EX[Claude Executor]
        SC[Scheduler]
        CM[Cooldown Manager]
    end

    subgraph "資料存取層 (Data Access Layer)"
        DB[SQLite 資料庫]
        FS[檔案系統]
        CACHE[記憶體快取]
    end

    subgraph "外部整合層 (External Integration)"
        CLAUDE[Claude CLI]
        OS[作業系統 API]
    end

    GUI --> IPC
    CLI --> ROUTER
    IPC --> ROUTER
    ROUTER --> PM
    ROUTER --> EX
    ROUTER --> SC
    ROUTER --> CM

    PM --> DB
    EX --> CLAUDE
    EX --> CACHE
    SC --> DB
    CM --> CLAUDE

    CLAUDE --> OS
    DB --> FS
```

### 資料流程圖

```mermaid
sequenceDiagram
    participant U as 使用者
    participant GUI as GUI介面
    participant IPC as Tauri IPC
    participant PM as Prompt Manager
    participant DB as SQLite
    participant EX as Claude Executor
    participant CLI as Claude CLI

    U->>GUI: 建立新 Prompt
    GUI->>IPC: create_prompt(title, content)
    IPC->>PM: 驗證與處理
    PM->>DB: 存儲 Prompt
    DB-->>PM: 返回 ID
    PM-->>IPC: 成功回應
    IPC-->>GUI: 更新介面
    GUI-->>U: 顯示成功訊息

    U->>GUI: 執行 Prompt
    GUI->>IPC: execute_prompt(id, mode)
    IPC->>EX: 執行請求
    EX->>CLI: 呼叫 claude CLI
    CLI-->>EX: 返回結果
    EX->>DB: 存儲結果
    EX-->>IPC: 執行完成
    IPC-->>GUI: 更新狀態
    GUI-->>U: 顯示結果
```

---

## 🛠️ 技術棧架構

### 分層技術選擇

```mermaid
graph LR
    subgraph "前端技術棧"
        HTML[HTML5]
        CSS[現代 CSS]
        HTMX[htmx 1.9+]
        JS[Vanilla JavaScript]
    end

    subgraph "應用框架"
        TAURI[Tauri 2.0]
        RUST[Rust 1.76+]
        TOKIO[Tokio 異步運行時]
    end

    subgraph "資料層"
        SQLITE[SQLite 3.35+]
        SQLX[sqlx ORM]
        MIGRATION[Migration 系統]
    end

    subgraph "工具鏈"
        CARGO[Cargo 包管理]
        NPM[npm 前端管理]
        PLAYWRIGHT[Playwright 測試]
    end

    HTML --> TAURI
    CSS --> TAURI
    HTMX --> TAURI
    JS --> TAURI

    TAURI --> RUST
    RUST --> TOKIO
    RUST --> SQLX

    SQLX --> SQLITE
    SQLITE --> MIGRATION
```

### 依賴關係圖

```mermaid
graph TD
    APP[主應用程式]

    APP --> TAURI[tauri 2.0]
    APP --> SERDE[serde]
    APP --> TOKIO[tokio]
    APP --> SQLX[sqlx]
    APP --> ANYHOW[anyhow]
    APP --> CHRONO[chrono]
    APP --> REGEX[regex]
    APP --> CLAP[clap]
    APP --> COLORED[colored]

    TAURI --> PLUGINS[tauri-plugins]
    PLUGINS --> SQL_PLUGIN[tauri-plugin-sql]
    PLUGINS --> SHELL_PLUGIN[tauri-plugin-shell]
    PLUGINS --> STORE_PLUGIN[tauri-plugin-store]
    PLUGINS --> NOTIFICATION_PLUGIN[tauri-plugin-notification]

    SQLX --> SQLITE[sqlite]
    TOKIO --> CRON[tokio-cron-scheduler]
```

---

## 📊 資料流設計

### 資料庫架構

```mermaid
erDiagram
    PROMPTS ||--o{ JOBS : "has"
    JOBS ||--o{ RESULTS : "produces"

    PROMPTS {
        int id PK
        string title
        text content
        string tags
        datetime created_at
        datetime updated_at
    }

    JOBS {
        int id PK
        int prompt_id FK
        string cron_expr
        string mode
        string status
        int eta_unix
        datetime last_run_at
        datetime created_at
    }

    RESULTS {
        int id PK
        int job_id FK
        text content
        int duration_ms
        datetime created_at
    }
```

### 狀態管理

```mermaid
stateDiagram-v2
    [*] --> Idle: 應用啟動

    Idle --> Creating: 建立 Prompt
    Creating --> Idle: 建立完成
    Creating --> Error: 建立失敗

    Idle --> Executing: 執行 Prompt
    Executing --> Running: Claude CLI 執行中
    Running --> Completed: 執行成功
    Running --> Failed: 執行失敗
    Completed --> Idle: 結果儲存
    Failed --> Idle: 錯誤處理

    Idle --> Scheduling: 建立排程
    Scheduling --> Scheduled: 排程啟動
    Scheduled --> Running: 時間觸發

    Error --> Idle: 錯誤恢復
    Failed --> Idle: 失敗恢復
```

---

## 🧩 模組設計

### 核心模組架構

#### 1. Prompt Manager 模組

```rust
// src/prompt_manager.rs
pub struct PromptManager {
    db_pool: SqlitePool,
}

impl PromptManager {
    // CRUD 操作
    pub async fn create(&self, prompt: CreatePromptRequest) -> Result<i64>;
    pub async fn list(&self) -> Result<Vec<Prompt>>;
    pub async fn get(&self, id: i64) -> Result<Option<Prompt>>;
    pub async fn update(&self, id: i64, prompt: UpdatePromptRequest) -> Result<()>;
    pub async fn delete(&self, id: i64) -> Result<()>;

    // 搜尋與過濾
    pub async fn search(&self, query: &str) -> Result<Vec<Prompt>>;
    pub async fn filter_by_tags(&self, tags: &[String]) -> Result<Vec<Prompt>>;
}
```

#### 2. Claude Executor 模組

```rust
// src/claude_executor.rs
pub struct ClaudeExecutor {
    cooldown_manager: Arc<CooldownManager>,
}

impl ClaudeExecutor {
    pub async fn execute(&self, prompt: &str) -> Result<ExecutionResult>;
    pub async fn execute_with_timeout(&self, prompt: &str, timeout: Duration) -> Result<ExecutionResult>;

    // Claude Code 語法支援
    pub async fn execute_with_files(&self, prompt: &str, files: &[PathBuf]) -> Result<ExecutionResult>;
    pub async fn execute_with_context(&self, prompt: &str, context: &ExecutionContext) -> Result<ExecutionResult>;
}
```

#### 3. Scheduler 模組

```rust
// src/scheduler.rs
pub struct Scheduler {
    job_scheduler: JobScheduler,
    db_pool: SqlitePool,
    executor: Arc<ClaudeExecutor>,
}

impl Scheduler {
    pub async fn create_job(&self, job: CreateJobRequest) -> Result<i64>;
    pub async fn pause_job(&self, job_id: i64) -> Result<()>;
    pub async fn resume_job(&self, job_id: i64) -> Result<()>;
    pub async fn delete_job(&self, job_id: i64) -> Result<()>;
    pub async fn list_jobs(&self) -> Result<Vec<Job>>;
}
```

### 模組間通訊設計

```mermaid
graph LR
    subgraph "IPC 層"
        COMMANDS[Tauri Commands]
        EVENTS[Tauri Events]
    end

    subgraph "業務邏輯層"
        PM[Prompt Manager]
        EX[Claude Executor]
        SC[Scheduler]
        CM[Cooldown Manager]
    end

    COMMANDS --> PM
    COMMANDS --> EX
    COMMANDS --> SC
    COMMANDS --> CM

    PM --> EVENTS
    EX --> EVENTS
    SC --> EVENTS
    CM --> EVENTS

    EX <--> CM
    SC --> EX
    SC <--> PM
```

---

## 🚀 部署架構

### 建置流程架構

```mermaid
graph TD
    SOURCE[原始碼]

    subgraph "建置階段"
        DEPS[依賴安裝]
        LINT[程式碼檢查]
        TEST[測試執行]
        BUILD[應用建置]
    end

    subgraph "打包階段"
        BUNDLE[Tauri 打包]
        SIGN[程式碼簽名]
        OPTIMIZE[最佳化]
    end

    subgraph "發布階段"
        RELEASE[GitHub Release]
        DISTRIBUTE[分發]
    end

    SOURCE --> DEPS
    DEPS --> LINT
    LINT --> TEST
    TEST --> BUILD
    BUILD --> BUNDLE
    BUNDLE --> SIGN
    SIGN --> OPTIMIZE
    OPTIMIZE --> RELEASE
    RELEASE --> DISTRIBUTE
```

### 跨平台部署

```mermaid
graph TB
    subgraph "開發環境"
        DEV[開發機器]
        SOURCE[原始碼倉庫]
    end

    subgraph "CI/CD 環境"
        GITHUB[GitHub Actions]
        WINDOWS[Windows Runner]
        MACOS[macOS Runner]
        LINUX[Linux Runner]
    end

    subgraph "產出物"
        EXE[Windows .exe]
        DMG[macOS .dmg]
        APPIMAGE[Linux .AppImage]
        DEB[Linux .deb]
    end

    subgraph "分發管道"
        GITHUB_RELEASES[GitHub Releases]
        DIRECT[直接下載]
    end

    DEV --> SOURCE
    SOURCE --> GITHUB

    GITHUB --> WINDOWS
    GITHUB --> MACOS
    GITHUB --> LINUX

    WINDOWS --> EXE
    MACOS --> DMG
    LINUX --> APPIMAGE
    LINUX --> DEB

    EXE --> GITHUB_RELEASES
    DMG --> GITHUB_RELEASES
    APPIMAGE --> GITHUB_RELEASES
    DEB --> GITHUB_RELEASES

    GITHUB_RELEASES --> DIRECT
```

---

## 📈 效能架構

### 效能設計策略

| 層級       | 最佳化策略 | 實施方式                 |
| ---------- | ---------- | ------------------------ |
| **應用層** | 啟動最佳化 | Lazy loading、漸進式載入 |
| **資料層** | 查詢最佳化 | 索引設計、預編譯語句     |
| **記憶體** | 記憶體管理 | 物件池、及時釋放         |
| **I/O**    | 非同步處理 | Tokio 異步運行時         |
| **UI**     | 渲染最佳化 | 虛擬化、防抖動           |

### 快取架構

```mermaid
graph LR
    subgraph "快取層級"
        L1[L1: 記憶體快取]
        L2[L2: SQLite 查詢快取]
        L3[L3: 檔案系統快取]
    end

    subgraph "快取策略"
        LRU[LRU 淘汰]
        TTL[TTL 過期]
        INVALIDATE[主動失效]
    end

    APP[應用程式] --> L1
    L1 --> L2
    L2 --> L3

    L1 --> LRU
    L2 --> TTL
    L3 --> INVALIDATE
```

---

## 🔒 安全架構

### 安全設計原則

```mermaid
graph TD
    subgraph "輸入驗證"
        SANITIZE[輸入清理]
        VALIDATE[格式驗證]
        ESCAPE[特殊字符轉義]
    end

    subgraph "資料保護"
        ENCRYPT[敏感資料加密]
        SECURE_STORE[Tauri Secure Store]
        LOCAL_ONLY[本地存儲優先]
    end

    subgraph "執行安全"
        SANDBOX[進程沙箱]
        PRIVILEGE[最小權限原則]
        ISOLATION[模組隔離]
    end

    USER_INPUT[使用者輸入] --> SANITIZE
    SANITIZE --> VALIDATE
    VALIDATE --> ESCAPE

    SENSITIVE_DATA[敏感資料] --> ENCRYPT
    ENCRYPT --> SECURE_STORE
    SECURE_STORE --> LOCAL_ONLY

    EXTERNAL_EXEC[外部執行] --> SANDBOX
    SANDBOX --> PRIVILEGE
    PRIVILEGE --> ISOLATION
```

---

## 📚 參考資料

### 架構決策記錄 (ADR)

1. [ADR-001: 選擇 Tauri 作為桌面應用框架](../decisions/ADR-001-tauri-framework.md)
2. [ADR-002: 採用 SQLite 作為資料庫](../decisions/ADR-002-sqlite-database.md)
3. [ADR-003: 使用 htmx 進行前端互動](../decisions/ADR-003-htmx-frontend.md)

### 相關文檔

- [資料庫設計](database-schema.md)
- [API 設計](api-design.md)
- [安全性設計](security-design.md)
- [效能基準](performance-benchmarks.md)

---

**文檔維護者**: 系統架構師  
**審查頻率**: 每季度  
**下次審查**: 2025-10-23
