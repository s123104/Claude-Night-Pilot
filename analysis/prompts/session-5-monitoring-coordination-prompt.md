# Enhanced Claude Code Prompt for session-5-monitoring-coordination

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
# Session 5: 監控與協調

## 🎯 任務目標

監控其他四個 Claude Code sessions 的執行進度，協調各工作流程，整合分析結果，生成統一的專案改進報告和實施計劃。

## 📋 具體任務

### 1. 進度監控

- 實時監控各 session 的執行狀態
- 追蹤任務完成進度和時間估算
- 識別阻塞問題和資源衝突
- 維護整體專案時程表

### 2. 結果協調

- 收集各 session 的分析結果
- 識別結果間的衝突和重疊
- 協調不同建議的優先級
- 整合成統一的改進方案

### 3. 風險管理

- 評估各項改進的風險和影響
- 識別相互依賴的改進項目
- 制定風險緩解策略
- 建立回滾計劃

### 4. 實施規劃

- 制定分階段的實施計劃
- 分配資源和時間估算
- 建立里程碑和檢查點
- 設計驗證和測試策略

## 🔧 監控工具

### 進度追蹤

```bash
# 監控各 session 的狀態檔案
watch -n 30 'find analysis/logs/ -name "*.status" -exec cat {} \;'

# 檢查各 session 的輸出檔案
ls -la analysis/reports/session-*/

# 監控系統資源使用
htop
iostat -x 1
```

### 結果聚合

```bash
# 合併各 session 的 JSON 報告
jq -s 'add' analysis/reports/session-*/report.json > analysis/reports/consolidated-report.json

# 生成統計摘要
python3 analysis/tools/aggregate-results.py
```

### 衝突檢測

```bash
# 檢測檔案修改衝突
git status --porcelain
git diff --name-only

# 分析建議衝突
python3 analysis/tools/conflict-detector.py
```

## 📊 監控儀表板

### 實時狀態顯示

```json
{
  "monitoring_dashboard": {
    "timestamp": "2025-08-14T03:00:00Z",
    "sessions": {
      "session-1-file-analysis": {
        "status": "running",
        "progress": 75,
        "eta": "2025-08-14T03:15:00Z",
        "last_update": "2025-08-14T03:00:00Z",
        "current_task": "Analyzing duplicate code"
      },
      "session-2-cli-analysis": {
        "status": "completed",
        "progress": 100,
        "completion_time": "2025-08-14T02:45:00Z",
        "output_files": [
          "analysis/reports/session-2/cli-commands.json",
          "analysis/reports/session-2/bdd-scenarios.yaml"
        ]
      },
      "session-3-architecture": {
        "status": "running",
        "progress": 60,
        "eta": "2025-08-14T03:30:00Z",
        "current_task": "Comparing with Vibe-Kanban architecture"
      },
      "session-4-technical-debt": {
        "status": "pending",
        "progress": 0,
        "waiting_for": ["session-1-file-analysis"]
      }
    },
    "overall_progress": 58,
    "estimated_completion": "2025-08-14T04:00:00Z"
  }
}
```

### 衝突和依賴分析

```yaml
conflicts_and_dependencies:
  conflicts:
    - type: "file_modification"
      sessions: ["session-1", "session-3"]
      files: ["src-tauri/src/lib.rs"]
      resolution: "Merge changes sequentially"

    - type: "recommendation_conflict"
      issue: "Directory structure"
      session_1_suggests: "Move utils to src/lib/"
      session_3_suggests: "Create src/shared/ directory"
      resolution: "Adopt session-3 suggestion (better modularity)"

  dependencies:
    - prerequisite: "session-1-file-analysis"
      dependent: "session-4-technical-debt"
      reason: "Need file analysis before debt cleanup"

    - prerequisite: "session-2-cli-analysis"
      dependent: "session-3-architecture"
      reason: "CLI structure affects architecture decisions"
```

## 🎯 整合策略

### 1. 結果優先級矩陣

```
           Impact
         High | Medium | Low
    High  P1  |   P2   | P3
Effort Medium P2  |   P3   | P4
    Low   P3  |   P4   | P5

P1: 立即實施 (高影響, 低/中等工作量)
P2: 計劃實施 (高影響, 高工作量 或 中等影響, 低工作量)
P3: 考慮實施 (其他組合)
P4: 延後實施 (低影響)
P5: 不實施 (低影響, 高工作量)
```

### 2. 實施階段規劃

```yaml
implementation_phases:
  phase_1_foundation:
    duration: "1-2 weeks"
    focus: "Critical fixes and cleanup"
    tasks:
      - "Remove obsolete files (Session 1)"
      - "Fix security vulnerabilities (Session 4)"
      - "Implement basic CLI tests (Session 2)"

  phase_2_structure:
    duration: "2-3 weeks"
    focus: "Architecture improvements"
    tasks:
      - "Implement repository pattern (Session 3)"
      - "Refactor module structure (Session 3)"
      - "Add comprehensive CLI documentation (Session 2)"

  phase_3_quality:
    duration: "1-2 weeks"
    focus: "Quality and performance"
    tasks:
      - "Improve test coverage (Session 4)"
      - "Performance optimization (Session 4)"
      - "Complete BDD test suite (Session 2)"

  phase_4_polish:
    duration: "1 week"
    focus: "Final improvements"
    tasks:
      - "Documentation updates"
      - "Final cleanup and validation"
      - "Release preparation"
```

## 🚀 執行步驟

1. **初始化監控**: 設置監控系統和狀態追蹤
2. **啟動協調**: 開始監控其他 sessions
3. **進度追蹤**: 持續監控和狀態更新
4. **結果收集**: 收集各 session 的輸出
5. **衝突解決**: 識別和解決衝突
6. **結果整合**: 整合所有分析結果
7. **計劃制定**: 制定統一的實施計劃
8. **報告生成**: 生成最終的專案改進報告

## 📝 預期成果

- **監控儀表板**: 實時的專案進度監控
- **整合報告**: 統一的專案分析和改進報告
- **實施計劃**: 詳細的分階段實施計劃
- **風險評估**: 完整的風險分析和緩解策略
- **協調機制**: 可重複使用的多 session 協調流程


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
