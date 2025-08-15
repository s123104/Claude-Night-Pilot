# Enhanced Claude Code Prompt for session-3-architecture-refactoring

## Project Context

**Project**: Claude Night Pilot
**Type**: Tauri Desktop Application + CLI Tool
**Tech Stack**: Rust (backend), JavaScript (frontend), SQLite (database)
**Goal**: Professional, maintainable, enterprise-grade automation tool
**Current Phase**: Analysis and refactoring for production readiness

**Key Directories**:
- src-tauri/: Rust backend code
- src/: Frontend JavaScript/HTML/CSS
- tests/: E2E and integration tests
- scripts/: Development and build scripts
- docs/: Documentation

**Architecture Reference**: research-projects/vibe-kanban (similar Rust + web architecture)


## Your Specific Task
# Session 3: 架構重構分析

## 🎯 任務目標

評估 Claude Night Pilot 的當前架構設計，參考 Vibe-Kanban 的最佳實踐，提出模組化改進方案，建立高度可維護的架構。

## 📋 具體任務

### 1. 當前架構分析

- 分析專案的模組結構和依賴關係
- 評估代碼的耦合度和內聚性
- 識別架構異味和設計問題
- 對比 Vibe-Kanban 的架構模式

### 2. 模組化評估

- 分析各模組的職責邊界
- 檢查模組間的介面設計
- 評估依賴注入和控制反轉的使用
- 識別可以抽象的共同模式

### 3. 設計模式應用

- 評估當前使用的設計模式
- 建議適合的設計模式應用
- 分析 Repository、Service、Factory 等模式的使用
- 提出改進的架構模式

### 4. 可維護性改進

- 分析代碼的可測試性
- 評估配置管理和環境隔離
- 檢查錯誤處理和日誌記錄
- 建議改進的開發工作流程

## 🔧 分析工具

### 架構分析

```bash
# Rust 模組依賴分析
cargo modules generate tree --with-types

# JavaScript 模組分析
npx madge --circular src/
npx madge --image architecture.svg src/

# 代碼複雜度分析
cargo clippy -- -W clippy::cognitive_complexity
```

### 設計品質評估

```bash
# 代碼品質指標
tokei . --exclude node_modules --exclude target
cargo audit
npm audit

# 測試覆蓋率
cargo tarpaulin --out Html
```

### 架構比較分析

```bash
# 與 Vibe-Kanban 架構對比
diff -r src/ research-projects/vibe-kanban/backend/src/ --brief
```

## 📊 輸出格式

### 架構分析報告

```json
{
  "architecture_analysis": {
    "timestamp": "2025-08-14T03:00:00Z",
    "current_structure": {
      "modules": {
        "core": {
          "path": "src-tauri/src/core/",
          "responsibilities": ["database", "business_logic"],
          "coupling": "medium",
          "cohesion": "high"
        },
        "interfaces": {
          "path": "src-tauri/src/interfaces/",
          "responsibilities": ["api", "cli"],
          "coupling": "low",
          "cohesion": "medium"
        }
      },
      "dependencies": {
        "circular_dependencies": [],
        "tight_coupling": [
          {
            "modules": ["executor", "database"],
            "reason": "Direct database access in executor"
          }
        ]
      }
    },
    "vibe_kanban_comparison": {
      "similarities": [
        "Rust backend with async runtime",
        "SQLite database with migrations"
      ],
      "differences": [
        "Vibe-Kanban uses Axum, we use Tauri",
        "Different frontend approaches"
      ],
      "adoptable_patterns": [
        "Executor pattern for agent management",
        "Type-safe API with shared types",
        "Workspace-based monorepo structure"
      ]
    }
  }
}
```

### 重構建議

```yaml
refactoring_recommendations:
  high_priority:
    - title: "Implement Repository Pattern"
      description: "Abstract database access behind repository interfaces"
      impact: "high"
      effort: "medium"
      files: ["src-tauri/src/core/database/"]

    - title: "Extract Service Layer"
      description: "Separate business logic from API handlers"
      impact: "high"
      effort: "high"
      files: ["src-tauri/src/lib.rs"]

  medium_priority:
    - title: "Implement Dependency Injection"
      description: "Use DI container for better testability"
      impact: "medium"
      effort: "high"

    - title: "Standardize Error Handling"
      description: "Implement consistent error types and handling"
      impact: "medium"
      effort: "medium"
```

## 🏗️ 參考架構模式 (基於 Vibe-Kanban)

### 1. 分層架構

```
┌─────────────────┐
│   Presentation  │ ← Tauri Commands / CLI
├─────────────────┤
│    Service      │ ← Business Logic
├─────────────────┤
│   Repository    │ ← Data Access
├─────────────────┤
│    Database     │ ← SQLite
└─────────────────┘
```

### 2. 模組組織

```
src-tauri/src/
├── api/           # API 端點和處理器
├── services/      # 業務邏輯服務
├── repositories/  # 資料存取層
├── models/        # 資料模型
├── executors/     # Claude 執行器 (參考 Vibe-Kanban)
├── shared/        # 共享類型和工具
└── config/        # 配置管理
```

### 3. 依賴注入模式

```rust
// 參考 Vibe-Kanban 的 DI 模式
pub struct AppState {
    pub db: Arc<Database>,
    pub claude_service: Arc<ClaudeService>,
    pub scheduler_service: Arc<SchedulerService>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let db = Arc::new(Database::new()?);
        let claude_service = Arc::new(ClaudeService::new(db.clone()));
        let scheduler_service = Arc::new(SchedulerService::new(db.clone()));

        Ok(Self {
            db,
            claude_service,
            scheduler_service,
        })
    }
}
```

## 🚀 執行步驟

1. **現狀分析**: 深入分析當前架構結構
2. **模式識別**: 識別現有的設計模式和反模式
3. **對比研究**: 與 Vibe-Kanban 架構進行詳細對比
4. **改進設計**: 設計新的架構方案
5. **影響評估**: 評估重構的影響和風險
6. **實施計劃**: 制定分階段的重構計劃
7. **驗證測試**: 設計架構改進的驗證方法

## 📝 預期成果

- **架構分析報告**: 詳細的當前架構評估
- **重構路線圖**: 分階段的架構改進計劃
- **設計文檔**: 新架構的詳細設計
- **實施指南**: 具體的重構實施步驟
- **測試策略**: 架構改進的驗證方法


## Additional Instructions
- Focus on actionable, implementable recommendations
- Provide confidence scores (0-1) for each recommendation
- Include risk assessment for proposed changes
- Generate machine-readable JSON outputs where specified
- Consider the parallel execution context - other sessions are running simultaneously

## Output Structure
Please structure your analysis results as follows:
1. Executive Summary (human-readable)
2. Detailed Analysis (structured data)
3. Recommendations (prioritized list)
4. Implementation Plan (step-by-step)
5. Risk Assessment (potential issues and mitigations)

Begin your analysis now.
