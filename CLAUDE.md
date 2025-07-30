# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Development Reference

### Essential Commands (Run Before Any Code Changes)
```bash
# Parallel execution for fastest validation
npm run lint:check && npm typecheck && npm test
```

### Core Development Workflow
```bash
# Development (GUI + CLI)
npm run tauri dev          # Full desktop application
npm run dev:frontend       # Frontend-only development
npm run cli -- [args]      # CLI tool development

# Quality Assurance
npm run commitlint         # Validate commit messages
npm test                   # Full E2E test suite
npm run test:ui            # Interactive test debugging

# Production Building
npm run tauri build        # Desktop application
npm run cli:build          # CLI binary
npm run cli:install        # Install CLI globally
```

## Project Overview

Claude Night Pilot (夜間自動打工仔) is a modern automation tool for Claude CLI users, featuring:

- **Local-only execution** - Complete privacy protection, no cloud dependencies
- **Dual-mode operation** - Both GUI (Tauri desktop app) and CLI interfaces  
- **Ultra-lightweight** - Single executable < 10MB, startup time < 3s
- **Modern tech stack** - Tauri 2.0 + Rust backend + htmx frontend + SQLite database

## Development Commands

### Basic Development
```bash
# Install dependencies
npm install

# Start development mode (GUI + backend)
npm run tauri dev

# Start frontend only (for testing HTML/CSS/JS)
npm run dev:frontend

# Build application for production
npm run tauri build
```

### CLI Tool Development
```bash
# Run CLI tool directly during development
npm run cli -- [args]
# Example: npm run cli -- prompt list

# Build CLI binary (release mode)
npm run cli:build

# Install CLI globally
npm run cli:install
```

### Testing
```bash
# Run full test suite (Playwright E2E tests)
npm test

# Run tests with UI for debugging
npm run test:ui

# Run tests in headed mode (see browser)
npm run test:headed

# Run tests in debug mode
npm run test:debug
```

### Commit Message Standards
```bash
# Lint commit messages
npm run commitlint

# Check code before commit (runs automatically via Git hooks)
npm run lint:check

# Test commit message validation
echo "feat(core): add new feature" | npx commitlint
```

**Conventional Commit Format**:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Allowed Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Project Scopes**: `core`, `gui`, `cli`, `db`, `scheduler`, `executor`, `security`, `test`, `docs`, `deps`, `config`, `ci`, `release`

### Backend Development
```bash
# Inside src-tauri directory:
cd src-tauri

# Check Rust code
cargo check

# Run Rust tests
cargo test

# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy
```

## Architecture Overview

### Technology Stack
- **Frontend**: Material Design 3.0 + htmx + advanced JavaScript (class-based state management)
- **Backend**: Rust with Tauri 2.0 framework + enhanced security features
- **Database**: SQLite with sqlx for type-safe queries + usage tracking extensions
- **Scheduling**: tokio-cron-scheduler for background jobs + adaptive monitoring
- **Testing**: Playwright for E2E testing with comprehensive Chinese UI test coverage
- **CLI**: Full-featured command-line interface with colored output and subcommands
- **Integration**: Claude Code executor with stream-json parsing + usage detection

### Key Directories
```
├── src/                    # Frontend assets
│   ├── index.html         # Main GUI interface
│   ├── main.js            # JavaScript logic
│   ├── styles.css         # Custom styles
│   └── css/pico.min.css   # CSS framework
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Main application logic & Tauri commands
│   │   ├── db.rs          # Database models and operations
│   │   ├── executor.rs    # Claude CLI execution wrapper with stream-json parsing
│   │   ├── scheduler.rs   # Cron job scheduler
│   │   ├── usage_tracker.rs # Usage tracking (integrated ccusage functionality)
│   │   ├── claude_executor.rs # Enhanced Claude integration with @ symbol support
│   │   └── bin/cnp.rs     # CLI binary entry point
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── tests/                 # E2E test suite with Chinese UI coverage
├── docs/                  # Documentation
└── research-projects/     # Integration analysis and implementation guides
```

### Core Components

#### Database Layer (`src-tauri/src/db.rs`)
- **Models**: `Prompt`, `Job`, `JobResult` structs with SQLite mapping
- **Operations**: CRUD operations for prompts, jobs, and results
- **Schema**: Automatic table creation with foreign key relationships

#### Execution Layer (`src-tauri/src/executor.rs`)
- **ClaudeExecutor**: Wrapper for Claude CLI with error handling and stream-json parsing
- **ExecutionOptions**: Comprehensive execution configuration including security options
- **SecurityCheckResult**: Multi-level risk assessment (Low/Medium/High/Critical)
- **ExecutionAudit**: Detailed audit logging with SHA256 prompt hashing
- **Cooldown Management**: Parses Claude API rate limit responses
- **Mock Support**: Debug-mode mock executor for development
- **Stream Processing**: Real-time parsing of Claude's stream-json output format
- **Usage Tracking**: Integrated token and cost monitoring from JSON logs

#### Scheduling Layer (`src-tauri/src/scheduler.rs`)  
- **TaskScheduler**: Cron-based job scheduling with tokio
- **Job Management**: Automatic retry logic and status tracking
- **Error Recovery**: Handles API cooldowns and execution failures

#### GUI Layer (`src/`)
- **Material Design 3.0**: Modern design system with theme management (dark/light/auto)
- **Advanced JavaScript**: Class-based architecture with AppState and MaterialThemeManager
- **htmx Integration**: Dynamic UI updates without complex JavaScript frameworks
- **Tauri Commands**: Frontend-backend communication via IPC
- **Responsive Design**: Mobile-friendly interface with CSS custom properties
- **Ripple Effects**: Material Design interaction animations

#### CLI Tool (`src-tauri/src/bin/cnp.rs`)
- **Command Structure**: Clap-based argument parsing with subcommands
- **Database Integration**: Direct SQLite access for all operations
- **Colored Output**: Professional terminal output with status indicators
- **Subcommands**:
  - `init`: Initialize database
  - `prompt`: Manage prompts (list, create, delete, show)
  - `job`: Manage scheduled jobs (list, create, cancel)
  - `run`: Execute prompts directly or schedule them
  - `status`: System status and health checks
  - `cooldown`: Check Claude CLI cooldown status
  - `results`: View execution results and logs

#### Advanced Modules
- **usage_tracker.rs**: Integrated ccusage functionality for Claude usage monitoring
- **claude_executor.rs**: Enhanced Claude Code integration with @ symbol file reference support
- **adaptive_monitor.rs**: Intelligent monitoring frequency adjustment
- **smart_scheduler.rs**: Enhanced scheduling with adaptive logic
- **prompt_parser.rs**: Advanced prompt parsing with @ symbol detection and file resolution

## Development Patterns

### Core Development Workflow
Always run these commands in parallel after making changes:
```bash
npm run lint:check && npm typecheck && npm test
```

### Tauri Command Pattern
1. Define the command function in `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
async fn your_command(param: String) -> Result<String, String> {
    // Implementation with proper error handling
    Ok("result".to_string())
}
```

2. Register in the `invoke_handler!` macro:
```rust
.invoke_handler(tauri::generate_handler![
    your_command,
    // ... other commands
])
```

3. Call from frontend JavaScript:
```javascript  
const result = await invoke('your_command', { param: 'value' });
```

### Claude Integration Pattern
Enhanced Claude Code integration with stream processing:
```rust
// Enhanced executor with @ symbol support
let claude_executor = ClaudeExecutor::new()
    .with_file_reference_support(true)
    .with_usage_tracking(true)
    .with_stream_processing(true);

// Parse prompts with @ symbol file references
let parsed_prompt = PromptParser::parse(&prompt)
    .resolve_file_references(&working_dir)
    .validate_permissions();
```

### Database Operations
Use the existing `Database` struct methods for type-safe operations:
```rust
// In Tauri commands
let db = Database::new("sqlite:claude-pilot.db").await?;
let prompts = db.list_prompts().await?;
```

### Execution Options and Security
The `ExecutionOptions` struct provides comprehensive configuration:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    pub skip_permissions: bool,           // Skip permission confirmations
    pub output_format: String,           // "json" or "text"
    pub timeout_seconds: Option<u64>,    // Execution timeout
    pub dry_run: bool,                   // Test mode
    pub working_directory: Option<String>, // Directory restrictions
    pub allowed_operations: Vec<String>, // Permitted operation types
    pub safety_check: bool,              // Enable security validation
    pub max_retries: u32,                // Maximum retry attempts
}
```

Security checks include risk assessment with `RiskLevel` enum (Low/Medium/High/Critical) and detailed audit logging via `ExecutionAudit` struct.

### Usage Tracking Integration
Integrated ccusage functionality for comprehensive monitoring:
```rust
// Usage tracking with real-time monitoring
let usage_tracker = UsageTracker::new()
    .with_claude_data_directories(vec![
        "~/.claude/projects/",
        "~/.config/claude/projects/"
    ])
    .with_live_monitoring(true)
    .with_cost_calculation(true);

// Track execution with detailed metrics
usage_tracker.track_execution(&execution_id, &prompt, &result)
    .with_model_info(&model_info)
    .with_token_counts(&token_usage)
    .with_session_context(&session_id);
```

### Error Handling
Follow the established pattern:
- Use `anyhow::Result` for internal errors
- Convert to `Result<T, String>` for Tauri commands
- Include context with `.context()` for debugging
- Use `Result.try()` for functional error handling (ccusage pattern)

### Frontend Architecture Patterns
The JavaScript uses a class-based architecture:
```javascript
// State management
class AppState {
  constructor() {
    this.theme = localStorage.getItem("theme") || "auto";
    this.currentTab = "prompts";
    // Reactive state updates via custom events
  }
}

// Material Design theme system
class MaterialThemeManager {
  applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme);
    // Ripple effects and theme transitions
  }
}
```

### Testing Approach
- **E2E Tests**: Primary testing strategy using Playwright with Chinese UI text
- **Test Structure**: Located in `tests/` directory with comprehensive spec files
- **Test Coverage**: GUI interactions, CLI integration, Material Design components
- **Test Commands**: 
  - `npm test`: Full test suite
  - `npm run test:ui`: Interactive test debugging
  - `npm run test:headed`: Visual browser testing
- **Unit Tests**: For Rust utility functions and parsing logic
- **Integration Tests**: For database operations and CLI integration
- **Manual Testing**: GUI workflows and CLI commands

## Key Features Implementation

### Enhanced Prompt Management
- Create, edit, delete prompts with advanced tagging system
- **@ Symbol Support**: Full Claude Code file reference syntax (`@file.md`, `@folder/`, `@*.ts`)
- **File Resolution**: Automatic file path resolution and permission validation
- **Template System**: Advanced templating with variable substitution
- **Bulk Operations**: Multi-prompt operations with batch processing

### Advanced Job Scheduling  
- **Cron Expressions**: Full cron support with human-readable descriptions
- **Real-time Execution**: Live status updates with WebSocket streaming
- **Intelligent Retry**: Exponential backoff with cooldown awareness
- **Status Pipeline**: pending → queued → running → completed/failed/cancelled
- **Resource Management**: Memory and CPU usage monitoring during execution

### Smart Cooldown Management
- **Rate Limit Detection**: Advanced parsing of Claude API responses
- **Predictive Countdown**: Real-time cooldown estimation and display
- **Adaptive Scheduling**: Automatic job rescheduling based on rate limits
- **Usage Optimization**: Intelligent batching to minimize API calls

### Comprehensive Usage Tracking
- **Real-time Monitoring**: Live token usage and cost tracking
- **Multi-Directory Support**: Both `~/.claude/` and `~/.config/claude/` paths
- **Cost Calculation**: Accurate pricing with LiteLLM integration
- **Session Analytics**: Per-session usage breakdown and optimization insights
- **Export Capabilities**: JSON/CSV export for usage analysis

### Dual Interface Architecture
- **GUI**: Tauri desktop app with Material Design 3.0 and htmx
- **CLI**: Full-featured Rust binary (`cnp`) with colored output and subcommands
- **API Integration**: RESTful endpoints with WebSocket streaming
- **Cross-Platform**: Windows, macOS, Linux support with native performance

## Common Tasks

### Adding a New Database Table
1. Update SQL schema in `src-tauri/migrations/0001_init.sql`
2. Add corresponding Rust struct in `db.rs` with `#[derive(FromRow)]`
3. Implement CRUD methods in `Database` impl block
4. Add Tauri commands if GUI access needed

### Modifying the GUI
1. Edit HTML structure in `src/index.html`
2. Update JavaScript logic in `src/main.js`
3. Modify styles in `src/styles.css`
4. Test with `npm run tauri dev`

### CLI Command Development
1. Modify `src-tauri/src/bin/cnp.rs` for argument parsing
2. Add business logic using existing database/executor modules
3. Test with `npm run cli -- your-command`
4. Build release binary with `npm run cli:build`

## Configuration Files

### `src-tauri/tauri.conf.json`
- Tauri application configuration
- CLI argument definitions
- Window settings and security policies

### `src-tauri/Cargo.toml`
- Rust dependencies and optimization settings
- Binary targets (main app + CLI)
- Release profile optimizations for small binary size

### `package.json`
- npm scripts for development workflow
- Playwright test configuration
- Frontend dependencies (minimal)

## Database Schema

### Core Tables
- **prompts**: User-created prompt templates
- **jobs**: Scheduled or manual execution jobs  
- **results**: Execution results and output storage

### Extended Tables (New Features)
- **usage_tracking**: Claude usage monitoring and analytics
- **execution_audit**: Security audit logs with risk assessment
- **system_config**: Application configuration storage

### Migration System
Database migrations are located in `src-tauri/migrations/`:
- `0001_init.sql`: Initial schema creation
- `0002_usage_tracking.sql`: Usage tracking extensions

### Relationships
- jobs.prompt_id → prompts.id (ON DELETE CASCADE)
- results.job_id → jobs.id (ON DELETE CASCADE)
- All audit tables maintain referential integrity

## Claude CLI Integration

Enhanced Claude Code integration with advanced stream processing and usage tracking:

### Core Integration Points
- **Stream Execution**: `npx @anthropic-ai/claude-code@latest -p --output-format=stream-json`
- **Session Management**: Support for `--resume=session_id` continuation
- **Permission Handling**: `--dangerously-skip-permissions` for automation
- **Real-time Parsing**: Stream-json format processing with normalized conversation logs
- **Status Monitoring**: `claude doctor --json` for health checks
- **Usage Detection**: Automatic token and cost tracking from execution logs

### Advanced Features
- **@ Symbol Processing**: Full file reference resolution (`@file.md`, `@folder/`, `@*.ts`)
- **Working Directory Management**: Git worktree integration for isolated execution
- **Session Persistence**: Resume interrupted conversations with full context
- **Error Recovery**: Intelligent retry with exponential backoff
- **Rate Limit Awareness**: Proactive cooldown detection and management

### Stream-JSON Processing
Real-time parsing of Claude's output format:
```rust
// Process streaming JSON responses
match json_line.get("type") {
    "assistant" => handle_assistant_message(&content),
    "user" => handle_user_message(&content), 
    "system" => handle_system_message(&content),
    "tool_use" => handle_tool_execution(&tool_name, &input),
    _ => handle_unknown_message(&content)
}
```

## Performance Targets

- **Binary Size**: < 10MB final executable
- **Memory Usage**: < 150MB during normal operation  
- **Startup Time**: < 3 seconds from launch to UI ready
- **UI Response**: < 100ms for user interactions
- **Database Queries**: < 50ms for typical operations

## Security Considerations

- **Input Validation**: All user inputs sanitized and validated
- **SQL Injection**: Use parameterized queries exclusively
- **File Access**: Restricted to application data directory
- **API Keys**: Use Tauri secure storage for sensitive data
- **Process Execution**: Limited to Claude CLI with validated arguments
- **Enhanced Security**: Multi-level risk assessment and audit logging
- **Execution Constraints**: Working directory restrictions and operation whitelisting

## Current Development State

### Implemented Features
- ✅ Material Design 3.0 frontend with theme management
- ✅ Enhanced security features with risk assessment
- ✅ Usage tracking system with database extensions
- ✅ Full-featured CLI tool with colored output
- ✅ Comprehensive Playwright test suite
- ✅ Advanced monitoring and scheduling modules

### Mock Mode
Currently, most Tauri commands return mock data for development. The actual Claude CLI integration and database operations are scaffolded but may need activation for production use.

### Research Integration
The project includes extensive research from multiple Claude automation projects (CCAutoRenew, Claude-Autopilot, claude-code-schedule, ClaudeNightsWatch, ccusage) with detailed integration documentation in the `docs/` directory.

## 自動委派 (Automatic Delegation)

Claude Code 根據以下情境智慧召喚 Sub-Agent：

- **code-reviewer**: 偵測 `git commit`, `git push`, 或 Pull Request 時，檢查程式碼品質、安全性、可維護性  
- **test-runner**: Pre-Push、CI Pipeline 或 Merge Request 時，執行單元/整合/E2E 測試並生成覆蓋率  
- **error-debugger**: 測試失敗、Build Crash 或 uncaught exception 時，定位根因並生成 Hotfix  
- **doc-writer**: 功能合併至 `main`、公開 API 變動或 release 標籤時，更新 README、CHANGELOG、ADR

### 智慧偵測機制

- **語言檢測**: 自動識別 Node.js (JavaScript/TypeScript)、Rust 技術棧
- **優先級排序**: error-debugger > code-reviewer > test-runner > doc-writer
- **動態觸發**: 根據專案結構和檔案變更動態調整觸發條件

### Sub-Agent 專案適應

**Node.js/JavaScript 專案**:
- code-reviewer: 檢查 ESLint 規則、TypeScript 類型安全
- test-runner: 執行 `npm test`, Jest/Playwright 測試套件
- error-debugger: 分析 Node.js stack trace、依賴衝突

**Rust 專案**:
- code-reviewer: 檢查 Clippy 建議、Cargo 依賴管理
- test-runner: 執行 `cargo test`, 生成測試覆蓋率
- error-debugger: 分析 Rust 編譯錯誤、記憶體安全問題

**Tauri 混合專案** (本專案類型):
- 前端 JavaScript 和後端 Rust 雙重檢測
- 同時支援 E2E Playwright 測試和 Rust 單元測試
- 跨語言依賴分析和安全檢查

## AI-Powered Commit Message Generation

### Token-Efficient Diff Processing
為避免 LLM token 超限，專案支援多種 diff 限制策略：

```bash
# 限制 Git diff 上下文為前後 100 行
git config diff.contextLines 100

# 使用 AI commit 工具時的配置
export DIFF_CONTEXT_LINES=100

# 在 .aicommitsrc 中配置
echo '{"diffContextLines": 100}' > .aicommitsrc
```

### Claude Code Integration
```bash
# 在 ~/.claude/settings.json 中配置（未來版本）
{
  "diffContextLines": 100,
  "commitPromptTemplate": "Generate commit message for: {diff}"
}

# 使用 claude code commit 命令
claude code commit --diff-lines=100
```

### AI Commit Tools Configuration
**支援的工具與配置**：
- `aicommits`: `.aicommitsrc` 設置 `diffContextLines`
- `cz-ai`: `package.json` 中配置 `cz-ai.diff_context_lines`
- `opencommit`: 環境變數 `OCO_DIFF_LINES=100`
- VS Code 擴充：`settings.json` 配置 `maxDiffSize`

### Git Hook Integration
專案已配置 Husky Git hooks：
- **pre-commit**: 運行 ESLint 檢查
- **commit-msg**: 驗證 commit 訊息格式
- 自動觸發 commitlint 規則驗證
- 支援 AI 生成的 commit 訊息後處理
## Important Development Notes

- The frontend uses Chinese text extensively - all UI tests are in Chinese
- All mock responses include Chinese content for authentic testing
- The CLI tool (`cnp`) provides a complete command-line interface to all functionality
- Security features are designed with production use in mind
- Database schema supports future enterprise features like audit compliance
- **Commit messages must follow Conventional Commits standard**
- **AI commit generation tools should limit diff context to 100 lines per direction**