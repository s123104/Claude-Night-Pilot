# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
│   │   ├── executor.rs    # Claude CLI execution wrapper
│   │   ├── scheduler.rs   # Cron job scheduler
│   │   ├── usage_tracker.rs # Usage tracking (new feature)
│   │   └── bin/cnp.rs     # CLI binary entry point
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── tests/                 # E2E test suite
└── docs/                  # Documentation
```

### Core Components

#### Database Layer (`src-tauri/src/db.rs`)
- **Models**: `Prompt`, `Job`, `JobResult` structs with SQLite mapping
- **Operations**: CRUD operations for prompts, jobs, and results
- **Schema**: Automatic table creation with foreign key relationships

#### Execution Layer (`src-tauri/src/executor.rs`)
- **ClaudeExecutor**: Wrapper for Claude CLI with error handling
- **ExecutionOptions**: Comprehensive execution configuration including security options
- **SecurityCheckResult**: Multi-level risk assessment (Low/Medium/High/Critical)
- **ExecutionAudit**: Detailed audit logging with SHA256 prompt hashing
- **Cooldown Management**: Parses Claude API rate limit responses
- **Mock Support**: Debug-mode mock executor for development

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
- **usage_tracker.rs**: New usage monitoring and tracking system
- **adaptive_monitor.rs**: Intelligent monitoring frequency adjustment
- **smart_scheduler.rs**: Enhanced scheduling with adaptive logic

## Development Patterns
1. Define the command function in `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
async fn your_command(param: String) -> Result<String, String> {
    // Implementation
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

### Error Handling
Follow the established pattern:
- Use `anyhow::Result` for internal errors
- Convert to `Result<T, String>` for Tauri commands
- Include context with `.context()` for debugging

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

### Prompt Management
- Create, edit, delete prompts with tags
- Support for Claude Code file reference syntax (`@file.md`)
- Bulk operations and template system

### Job Scheduling  
- Cron expression support for automated execution
- Manual execution with immediate feedback
- Status tracking: pending → running → done/error
- Automatic retry on API cooldowns

### Cooldown Detection
- Parses Claude CLI error messages for rate limit info
- Real-time countdown display in GUI
- Automatic job postponement during cooldowns

### Dual Interface
- **GUI**: Tauri desktop app with htmx-powered interface
- **CLI**: Rust binary (`cnp`) for command-line usage
- Shared backend logic and database

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

The application wraps the Claude CLI tool and expects it to be available in PATH. Key integration points:

- **Execution**: `claude -p "prompt" --output-format json`
- **Status Check**: `claude doctor --json` 
- **Error Parsing**: Rate limit detection and cooldown extraction
- **Mock Mode**: Debug-only mock responses for development

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

## Important Development Notes

- The frontend uses Chinese text extensively - all UI tests are in Chinese
- All mock responses include Chinese content for authentic testing
- The CLI tool (`cnp`) provides a complete command-line interface to all functionality
- Security features are designed with production use in mind
- Database schema supports future enterprise features like audit compliance