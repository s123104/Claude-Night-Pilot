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
npm run cli -- [args]      # Legacy CLI tool development

# Enhanced CLI Development (Dual Architecture)
./target/debug/cnp-optimized [command]  # Performance-optimized CLI (11.7ms startup)
./target/debug/cnp-unified [command]     # Full-featured CLI with session management

# Quality Assurance
npm run commitlint         # Validate commit messages
npm test                   # Full E2E test suite
npm run test:ui            # Interactive test debugging

# Production Building
npm run tauri build        # Desktop application
npm run cli:build          # CLI binary
npm run cli:install        # Install CLI globally
cargo build --release      # Build optimized CLI binaries
```

## Project Overview

Claude Night Pilot (夜間自動打工仔) is a modern automation tool for Claude CLI users, featuring:

- **Local-only execution** - Complete privacy protection, no cloud dependencies
- **Dual-mode operation** - Both GUI (Tauri desktop app) and CLI interfaces  
- **Ultra-lightweight** - Single executable < 10MB, CLI startup time 11.7ms (88% faster than target)
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
- **CLI**: Dual-architecture CLI system - `cnp-optimized` (11.7ms startup) and `cnp-unified` (full features with session management)
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
│   │   ├── unified_interface.rs # Unified Claude interface system
│   │   ├── database_manager.rs # Enhanced database management
│   │   ├── executor.rs    # Claude CLI execution wrapper with stream-json parsing
│   │   ├── claude_session_manager.rs # Session management with Git worktree integration
│   │   ├── worktree_manager.rs # Git worktree management and cleanup
│   │   ├── agents_registry.rs # Agent coordination and registry
│   │   ├── enhanced_executor.rs # Advanced execution with cooldown detection
│   │   └── bin/
│   │       ├── cnp-optimized.rs # Performance-optimized CLI (11.7ms startup)
│   │       └── cnp-unified.rs   # Full-featured CLI with session management
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── tests/                 # E2E test suite with Chinese UI coverage
├── docs/
│   └── tmp/               # Enhanced CLI documentation and testing reports
└── research-projects/     # Integration analysis and vibe-kanban patterns
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

#### Dual CLI Architecture

##### Performance CLI (`src-tauri/src/bin/cnp-optimized.rs`)
- **Ultra-Fast Startup**: 11.7ms startup time (88% faster than 100ms target)
- **Core Commands**: `execute`, `status`, `health`, `benchmark`
- **Optimized for**: Frequent usage, script integration, performance-critical workflows
- **Output Formats**: JSON, text, pretty formatting

##### Full-Featured CLI (`src-tauri/src/bin/cnp-unified.rs`)
- **Complete Feature Set**: All functionality with GUI consistency
- **Session Management**: Create, resume, execute within Claude sessions
- **Git Worktree Integration**: Isolated development environments
- **Enhanced Subcommands**:
  - `session`: Create, resume, list, execute, pause, complete Claude sessions
  - `worktree`: Create, list, cleanup Git worktrees
  - `prompt`: Full CRUD operations (list, create, delete, show, update)
  - `job`: Complete job management (list, create, update, delete, show)
  - `run`/`execute`: Execute prompts with session support
  - `batch`: Concurrent prompt execution
  - `status`: System status and health checks
  - `results`: Enhanced results viewing with filtering

#### Advanced Modules
- **claude_session_manager.rs**: Comprehensive session management with Git worktree integration
- **worktree_manager.rs**: Git worktree creation, management, and intelligent cleanup
- **unified_interface.rs**: Unified Claude interface with enhanced error handling
- **database_manager.rs**: Advanced database operations with connection pooling
- **agents_registry.rs**: Agent coordination and specialized task delegation
- **enhanced_executor.rs**: Advanced execution with cooldown detection and retry logic
- **claude_cooldown_detector.rs**: Intelligent Claude API rate limit detection

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

### Enhanced Session Management with Git Worktree Integration
- **Session Creation**: Create Claude sessions with automatic Git worktree setup
- **Branch Isolation**: Each session gets its own Git branch and working directory
- **Session Persistence**: Resume sessions across terminal sessions with full context
- **Worktree Cleanup**: Intelligent cleanup of worktrees and metadata when sessions complete
- **Cross-Platform Support**: Windows/WSL compatibility with path normalization

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
- **Parallel Detection**: Concurrent cooldown checking with `tokio::join!` for <50ms response
- **Rate Limit Parsing**: Advanced parsing of Claude API responses and error codes
- **Predictive Countdown**: Real-time cooldown estimation and display
- **Adaptive Scheduling**: Automatic job rescheduling based on rate limits
- **Usage Optimization**: Intelligent batching to minimize API calls

### Comprehensive Usage Tracking
- **Real-time Monitoring**: Live token usage and cost tracking
- **Multi-Directory Support**: Both `~/.claude/` and `~/.config/claude/` paths
- **Cost Calculation**: Accurate pricing with LiteLLM integration
- **Session Analytics**: Per-session usage breakdown and optimization insights
- **Export Capabilities**: JSON/CSV export for usage analysis

### Triple Interface Architecture
- **GUI**: Tauri desktop app with Material Design 3.0 and htmx
- **Performance CLI**: `cnp-optimized` - Ultra-fast CLI (11.7ms startup) for frequent operations
- **Full CLI**: `cnp-unified` - Complete feature set with session management and Git worktree integration
- **API Integration**: RESTful endpoints with WebSocket streaming
- **Cross-Platform**: Windows, macOS, Linux support with native performance
- **Session Management**: Claude session persistence with worktree isolation
- **Git Integration**: Automatic branch creation and cleanup for development workflows

## Common Tasks

### Adding a New Database Table
1. Update SQL schema in `src-tauri/migrations/0001_init.sql`
2. Add corresponding Rust struct in database modules with `#[derive(FromRow)]`
3. Implement CRUD methods in `DatabaseManager` impl block
4. Add Tauri commands if GUI access needed
5. Update both CLI binaries if needed

### Adding Session Management Features
1. Extend `ClaudeSessionManager` in `claude_session_manager.rs`
2. Add worktree operations in `worktree_manager.rs` if needed
3. Update CLI commands in `cnp-unified.rs`
4. Test with both session creation and resumption workflows

### Performance Optimization
1. Profile with `cargo build --release` and benchmark
2. Use `which::which()` for binary existence checks
3. Implement parallel operations with `tokio::join!`
4. Minimize initialization overhead for frequently-used commands

### Modifying the GUI
1. Edit HTML structure in `src/index.html`
2. Update JavaScript logic in `src/main.js`
3. Modify styles in `src/styles.css`
4. Test with `npm run tauri dev`

### CLI Command Development

#### Performance CLI (cnp-optimized)
1. Modify `src-tauri/src/bin/cnp-optimized.rs` for new commands
2. Focus on minimal dependencies and fast startup
3. Test with `./target/debug/cnp-optimized your-command`
4. Optimize for <100ms operations

#### Full-Featured CLI (cnp-unified)
1. Modify `src-tauri/src/bin/cnp-unified.rs` for comprehensive features
2. Add session management and worktree integration
3. Test with `./target/debug/cnp-unified your-command`
4. Build with `cargo build --release`

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
- **Git Worktree Integration**: Automatic branch creation and isolated working directories
- **Session Management**: Create, resume, pause, and complete Claude sessions with full context
- **Cross-Platform Worktree Support**: Windows/WSL compatibility with intelligent path handling
- **Session Persistence**: SQLite-backed session storage with metadata and token tracking
- **Intelligent Cleanup**: Safe worktree removal with Git reference cleanup
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

## Performance Targets & Results

### Dual CLI Performance Results

#### cnp-optimized (Performance CLI)
- **啟動時間**: 目標 100ms → **實際 11.7ms** ✅ (超越目標 88%)
- **健康檢查**: 目標 200ms → **快速模式 12ms** ✅ (超越目標 94%)
- **標準健康檢查**: 311ms (完整驗證適用)
- **狀態查詢**: <10ms (超快響應)

#### cnp-unified (Full-Featured CLI)
- **啟動時間**: ~50ms (包含完整功能初始化)
- **會話創建**: <2s (包含 Git worktree 設置)
- **會話恢復**: <500ms (快速上下文載入)
- **批量執行**: 3個並發任務支援

### 應用程式性能
- **Binary Size**: < 10MB final executable
- **Memory Usage**: < 150MB during normal operation  
- **Startup Time**: < 3 seconds from launch to UI ready
- **UI Response**: < 100ms for user interactions
- **Database Queries**: < 50ms for typical operations

### 關鍵優化技術
1. **懶加載消除**: 移除 OnceCell 全局狀態，直接使用 UnifiedClaudeInterface 靜態方法
2. **命令行優先解析**: 使用 `Cli::parse()` 立即解析，避免不必要的初始化延遲
3. **並行健康檢查**: `tokio::join!` 並行執行 Claude CLI 檢查和冷卻檢測
4. **快速模式**: `which::which()` 檢查二進位檔案存在性，避免進程執行開銷
5. **選擇性初始化**: 僅在實際執行時才初始化完整介面

### 效能測試命令
```bash
# Performance CLI 測試
./target/debug/cnp-optimized benchmark --iterations 5
./target/debug/cnp-optimized health --fast --format json
./target/debug/cnp-optimized status

# Full-Featured CLI 測試
./target/debug/cnp-unified session list
./target/debug/cnp-unified worktree list
./target/debug/cnp-unified job list --format pretty

# 建置優化版本
cargo build --release
./target/release/cnp-optimized benchmark
./target/release/cnp-unified session create "測試會話"
```

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
## Enhanced Workflow Examples

### Session-Based Development Workflow
```bash
# 1. Create development session with Git worktree
cnp-unified session create "User Authentication Feature" \
  --description "Implementing OAuth2 login system" \
  --create-worktree \
  --branch "feature-oauth-login"

# 2. Work in isolated environment
cnp-unified session execute <session-uuid> \
  "Analyze current authentication patterns and create implementation plan"

# 3. Continue development in same session
cnp-unified session execute <session-uuid> \
  "Implement OAuth2 provider integration with proper error handling"

# 4. Complete and cleanup
cnp-unified session complete <session-uuid>  # Auto-cleanup worktree
```

### Performance-Critical Automation
```bash
# Use cnp-optimized for frequent operations
./target/release/cnp-optimized status  # <10ms
./target/release/cnp-optimized health --fast  # 12ms
./target/release/cnp-optimized execute -p "Quick analysis"  # Minimal overhead

# Use cnp-unified for complex workflows
./target/release/cnp-unified batch -f analysis-tasks.json --concurrent 3
```

### Git Worktree Management
```bash
# Create isolated worktrees for different features
cnp-unified worktree create feature-payments
cnp-unified worktree create hotfix-security

# List active worktrees
cnp-unified worktree list

# Clean up when done
cnp-unified worktree cleanup /path/to/feature-worktree
```

## SuperClaude Framework Integration

This project integrates with the SuperClaude framework patterns from `~/.claude/` directory:

### Framework Features
- **Task Management**: TodoWrite integration for complex workflows
- **Persona System**: Auto-activation based on development context
- **MCP Server Integration**: Context7 for documentation, Sequential for analysis
- **Token Efficiency**: Intelligent compression and optimization
- **Quality Gates**: 8-step validation cycle with evidence-based completion

### Framework Commands Support
```bash
# SuperClaude commands work seamlessly with CLI
cnp-unified execute -p "/analyze @src-tauri/src/claude_session_manager.rs --think-hard"
cnp-unified execute -p "/improve @src-tauri/src/ --focus performance --persona-architect"
```

## Important Development Notes

- The frontend uses Chinese text extensively - all UI tests are in Chinese
- All mock responses include Chinese content for authentic testing
- **Dual CLI Architecture**: `cnp-optimized` (11.7ms startup) vs `cnp-unified` (full features)
- **Session Management**: Git worktree integration provides isolated development environments
- **SuperClaude Integration**: Full compatibility with framework patterns and personas
- Security features are designed with production use in mind
- Database schema supports future enterprise features like audit compliance
- **Commit messages must follow Conventional Commits standard**
- **AI commit generation tools should limit diff context to 100 lines per direction**
- **Performance Testing**: Use `./target/debug/cnp-optimized benchmark` for performance validation