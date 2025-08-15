# Enhanced Claude Code Prompt for session-2-cli-analysis

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
# Session 2: CLI 指令分析

## 🎯 任務目標

全面分析 Claude Night Pilot 的 CLI 指令系統，建立完整的指令目錄，設計 BDD 測試場景，確保 CLI 功能的完整性和可靠性。

## 📋 具體任務

### 1. CLI 指令清單建立

- 掃描所有 CLI 二進制檔案 (`cnp-unified`, `cnp-optimized`)
- 分析 `clap` 定義的所有指令和子指令
- 記錄每個指令的參數、選項和用法
- 建立指令層級結構圖

### 2. 指令功能分析

- 測試每個指令的基本功能
- 驗證指令的輸入輸出格式
- 檢查錯誤處理和邊界條件
- 分析指令間的依賴關係

### 3. BDD 測試場景設計

- 為每個指令設計 Given-When-Then 場景
- 建立用戶故事和驗收標準
- 設計正面和負面測試用例
- 建立端到端測試流程

### 4. 文檔完整性檢查

- 檢查每個指令是否有對應文檔
- 驗證文檔與實際功能的一致性
- 識別缺失的使用範例
- 建議文檔改進方案

## 🔧 分析工具

### CLI 指令發現

```bash
# 列出所有可用指令
cargo run --bin cnp-unified -- --help
cargo run --bin cnp-optimized -- --help

# 遞歸獲取所有子指令
for cmd in $(cargo run --bin cnp-unified -- --help | grep -E "^\s+\w+" | awk '{print $1}'); do
  echo "=== $cmd ==="
  cargo run --bin cnp-unified -- $cmd --help
done
```

### 功能測試

```bash
# 基本功能測試
cargo run --bin cnp-unified -- status
cargo run --bin cnp-unified -- health --format json
cargo run --bin cnp-unified -- cooldown

# 錯誤處理測試
cargo run --bin cnp-unified -- invalid-command
cargo run --bin cnp-unified -- prompt create --invalid-option
```

### BDD 場景生成

```gherkin
Feature: CLI Status Command
  As a user
  I want to check the system status
  So that I can verify the application is working correctly

  Scenario: Check basic system status
    Given the Claude Night Pilot system is running
    When I execute "cnp status"
    Then I should see database connection status
    And I should see prompt count
    And I should see task count
    And the exit code should be 0
```

## 📊 輸出格式

### CLI 指令目錄

```json
{
  "cli_analysis": {
    "timestamp": "2025-08-14T03:00:00Z",
    "binaries": {
      "cnp-unified": {
        "version": "0.1.0",
        "commands": {
          "status": {
            "description": "Display system status summary",
            "options": [],
            "examples": ["cnp status"],
            "output_format": "text"
          },
          "health": {
            "description": "System health check",
            "options": [
              {
                "name": "--format",
                "type": "string",
                "default": "pretty",
                "values": ["json", "text", "pretty"]
              }
            ],
            "examples": ["cnp health", "cnp health --format json"]
          }
        }
      }
    }
  }
}
```

### BDD 測試規格

```yaml
test_scenarios:
  - feature: "CLI Basic Commands"
    scenarios:
      - name: "Help command displays usage"
        given: "The CLI tool is available"
        when: "I run 'cnp --help'"
        then:
          - "I should see usage information"
          - "I should see available commands"
          - "Exit code should be 0"

      - name: "Status command shows system info"
        given: "The system is initialized"
        when: "I run 'cnp status'"
        then:
          - "I should see database status"
          - "I should see prompt count"
          - "Exit code should be 0"
```

## 🚀 執行步驟

1. **指令發現**: 自動發現所有 CLI 指令和子指令
2. **功能測試**: 逐一測試每個指令的基本功能
3. **參數分析**: 分析每個指令的參數和選項
4. **場景設計**: 為每個指令設計 BDD 測試場景
5. **文檔檢查**: 驗證文檔完整性和準確性
6. **測試實施**: 實施自動化 BDD 測試
7. **報告生成**: 生成完整的 CLI 分析報告

## 📝 預期成果

- **CLI 指令目錄**: 完整的指令參考文檔
- **BDD 測試套件**: 全面的行為驅動測試
- **使用範例**: 每個指令的實用範例
- **測試自動化**: 可持續執行的測試腳本
- **文檔改進**: 文檔品質提升建議


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
