# Gemini Agent Onboarding Guide

This document provides a comprehensive guide for the Gemini agent to understand and contribute to the Claude Night Pilot project.

## 1. Core Project Objective

**Claude Night Pilot (夜間自動打工仔)** is an enterprise-grade, local-first automation platform for Claude Code. It provides a dual-mode interface (a Tauri-based GUI and a comprehensive CLI) to manage and automate interactions with the Claude AI.

**Core Principles:**
- **Local-First:** Complete privacy protection with no cloud dependencies. All operations run locally.
- **Dual-Mode Operation:** A Tauri-based GUI for visual interaction and a powerful CLI for automation and scripting.
- **Performance-Focused:** The architecture includes a performance-optimized CLI with an ~11.7ms startup time.
- **Developer-Centric:** Built with a modern tech stack and tools to enhance the developer workflow, including session management with Git worktree integration.

## 2. Technology Stack

-   **Backend:** Rust, Tauri 2.0
-   **Frontend:** Vanilla JavaScript, htmx, Material Design 3.0
-   **Database:** SQLite (via `sqlx` for type-safe queries)
-   **CLI Framework:** `clap`
-   **Testing:** Playwright (E2E), Cargo test (Rust unit/integration)
-   **Scheduling:** `tokio-cron-scheduler` for background jobs

## 3. Project Structure

```
.
├── src/
│   ├── main.js             // Main frontend logic, state management
│   └── index.html          // GUI entry point
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         // Tauri application entry point
│   │   ├── lib.rs          // Core application logic and Tauri commands
│   │   ├── unified_interface.rs // Unified Claude interface system
│   │   ├── database_manager.rs  // Enhanced database management
│   │   ├── executor.rs     // Claude CLI execution wrapper
│   │   ├── claude_session_manager.rs // Session management with Git worktree integration
│   │   └── bin/
│   │       ├── cnp-optimized.rs // Performance-optimized CLI (~11.7ms startup)
│   │       └── cnp-unified.rs   // Full-featured CLI with session management
│   └── Cargo.toml
├── tests/
│   └── e2e/                // Playwright E2E tests
└── scripts/
    └── ...                 // Helper scripts for development
```

## 4. Key Commands (Development)

The project uses `npm` scripts to orchestrate development, testing, and build tasks.

| Command | Description |
|---|---|
| `npm run dev` | Starts the Tauri GUI application in development mode. |
| `npm run cli:unified -- [args]` | Runs the full-featured CLI for development. |
| `npm run cli:optimized -- [args]` | Runs the performance-optimized CLI for development. |
| `npm test` | Executes the full Playwright E2E test suite. |
| `npm run test:rust` | Runs all Rust unit and integration tests. |
| `npm run lint:check` | Lints JavaScript and Rust code. |
| `npm run build` | Builds the production-ready desktop application. |
| `npm run cli:build` | Builds the production-ready CLI binaries. |

**Example Usage:**
```bash
# Start the GUI application
npm run dev

# Run the full-featured CLI to list prompts
npm run cli:unified -- prompt list

# Run the fast CLI for a quick health check
npm run cli:optimized -- health --fast
```

## 5. Development Workflow

### Setup
1.  Install Node.js 18+, Rust 1.76+, and the Claude Code CLI.
2.  Clone the repository.
3.  Install dependencies: `npm install`.

### Running Locally
-   **GUI:** `npm run dev`
-   **CLI (Full):** `npm run cli:unified -- <COMMAND>`
-   **CLI (Optimized):** `npm run cli:optimized -- <COMMAND>`

### Testing
-   Run all tests: `npm run test:all`
-   Run Playwright E2E tests: `npm test`
-   Run Rust tests: `cd src-tauri && cargo test`

### Code Style
-   **JavaScript:** ESLint enforces the style. Run `npm run lint` to check and fix.
-   **Rust:** `rustfmt` is used for formatting. Run `npm run format` or `cargo fmt`.
-   `clippy` is used for linting. Run `cargo clippy -- -D warnings`.

### Commits
-   The project follows the **Conventional Commits** specification.
-   A `commitlint` configuration is in place to enforce this.
-   **Format:** `<type>(<scope>): <subject>`
-   **Scopes:** `core`, `gui`, `cli`, `db`, `scheduler`, `executor`, `security`, `test`, `docs`, `deps`, `config`, `ci`, `release`

## 6. Core Logic Explained

### Dual CLI Architecture
The project features two distinct CLI binaries to serve different needs:
1.  **`cnp-optimized`**: Built for speed (~11.7ms startup). It handles frequent, simple tasks like status checks and quick executions.
2.  **`cnp-unified`**: A full-featured CLI that mirrors the GUI's capabilities, including complex session and worktree management.

### Session Management (`claude_session_manager.rs`)
-   Integrates with Git to create isolated **worktrees** for each development session.
-   This allows for context-switching without disrupting other ongoing work.
-   Sessions are persisted in the SQLite database, allowing users to resume work later.

### Database Layer (`database_manager.rs`)
-   Uses `sqlx` for compile-time checked, asynchronous SQL queries against a local SQLite database.
-   Manages prompts, jobs, results, usage data, and session information.
-   Migrations are handled via `.sql` files in `src-tauri/migrations/`.

### Execution Layer (`executor.rs`)
-   Wraps the `@anthropic-ai/claude-code` CLI.
-   Parses `stream-json` output for real-time feedback.
-   Includes features for security checks, cooldown detection, and usage tracking.
-   Supports `@` file referencing (`@/path/to/file`) for passing context to Claude.

## 7. How to Contribute

1.  **Pick an issue** or propose a new feature.
2.  **Create a new branch:** `git checkout -b feat/my-new-feature`.
3.  **Write your code.** Add or update tests in `tests/` or within the relevant Rust modules.
4.  **Run checks:** `npm run format` and `npm run lint:check`.
5.  **Run tests:** `npm run test:all`.
6.  **Commit your changes** using the Conventional Commits format.
7.  **Push to your branch** and open a Pull Request with a clear description of the changes.