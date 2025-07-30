# Integration Plan: Vibe-Kanban Claude Connection + ccusage Usage Detection

## Overview

This document outlines the integration of advanced Claude Code connectivity from **vibe-kanban** and comprehensive usage tracking from **ccusage** into Claude Night Pilot.

## Phase 1: Enhanced Claude Executor with @ Symbol Support

### Core Components to Integrate

#### 1. Stream-JSON Processing Engine
**Source**: `research-projects/vibe-kanban/backend/src/executors/claude.rs`

**Key Features**:
- Real-time parsing of Claude's `--output-format=stream-json`
- Normalized conversation logs with metadata preservation
- Tool usage tracking and action type classification
- Session ID extraction and resume capability

**Implementation Strategy**:
```rust
// New file: src-tauri/src/claude_executor.rs
pub struct EnhancedClaudeExecutor {
    stream_processor: StreamJsonProcessor,
    file_resolver: FileReferenceResolver,
    usage_tracker: UsageTracker,
    session_manager: SessionManager,
}

impl EnhancedClaudeExecutor {
    pub async fn execute_with_file_support(&self, prompt: &str) -> Result<ExecutionResult> {
        // 1. Parse @ symbols in prompt
        let resolved_prompt = self.file_resolver.resolve_references(prompt)?;
        
        // 2. Execute with stream processing
        let process = self.spawn_claude_process(&resolved_prompt).await?;
        
        // 3. Process streaming JSON output
        let conversation = self.stream_processor.normalize_logs(&process.stdout).await?;
        
        // 4. Track usage metrics
        self.usage_tracker.record_execution(&conversation).await?;
        
        Ok(ExecutionResult { conversation, metrics: usage_metrics })
    }
}
```

#### 2. File Reference Resolver
**New Component**: Advanced @ symbol parsing and resolution

**Features**:
- Support for `@file.md`, `@folder/`, `@*.ts` patterns
- Security validation and permission checking
- Working directory context awareness
- Relative path normalization

**Implementation**:
```rust
// New file: src-tauri/src/file_resolver.rs
pub struct FileReferenceResolver {
    working_dir: PathBuf,
    allowed_patterns: Vec<String>,
    security_validator: SecurityValidator,
}

impl FileReferenceResolver {
    pub fn resolve_references(&self, prompt: &str) -> Result<String> {
        let mut resolved = prompt.to_string();
        
        // Find all @ references using regex
        let references = self.extract_file_references(&resolved)?;
        
        for reference in references {
            match reference {
                FileReference::Single(path) => {
                    let content = self.read_file_safely(&path)?;
                    resolved = resolved.replace(&reference.original, &content);
                },
                FileReference::Directory(path) => {
                    let listing = self.list_directory_safely(&path)?;
                    resolved = resolved.replace(&reference.original, &listing);
                },
                FileReference::Glob(pattern) => {
                    let matches = self.glob_files_safely(&pattern)?;
                    resolved = resolved.replace(&reference.original, &matches);
                }
            }
        }
        
        Ok(resolved)
    }
}
```

### Database Schema Extensions

#### New Tables for Enhanced Functionality
```sql
-- Enhanced execution tracking
CREATE TABLE execution_sessions (
    id TEXT PRIMARY KEY,
    job_id INTEGER REFERENCES jobs(id),
    session_id TEXT NOT NULL,
    conversation_log TEXT,
    token_usage_input INTEGER,
    token_usage_output INTEGER,
    model_name TEXT,
    cost_usd REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- File reference tracking
CREATE TABLE file_references (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_session_id TEXT REFERENCES execution_sessions(id),
    reference_type TEXT NOT NULL, -- 'file', 'directory', 'glob'
    original_reference TEXT NOT NULL,
    resolved_path TEXT NOT NULL,
    file_size INTEGER,
    last_modified DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Usage analytics
CREATE TABLE usage_analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    session_count INTEGER DEFAULT 0,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cost_usd REAL DEFAULT 0.0,
    model_breakdown TEXT, -- JSON object
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Phase 2: Comprehensive Usage Tracking Integration

### ccusage Functionality Integration
**Source**: `research-projects/ccusage/src/data-loader.ts` and related modules

#### 1. Multi-Directory Data Loading
**Features**:
- Support both `~/.claude/projects/` and `~/.config/claude/projects/`
- Automatic JSONL file discovery and parsing
- Data aggregation across multiple Claude installations

**Implementation**:
```rust
// Enhanced file: src-tauri/src/usage_tracker.rs
pub struct UsageTracker {
    claude_data_dirs: Vec<PathBuf>,
    pricing_fetcher: PricingFetcher,
    session_analyzer: SessionAnalyzer,
}

impl UsageTracker {
    pub async fn load_usage_data(&self) -> Result<Vec<UsageEntry>> {
        let mut all_entries = Vec::new();
        
        for dir in &self.claude_data_dirs {
            let entries = self.load_from_directory(dir).await?;
            all_entries.extend(entries);
        }
        
        // Sort by timestamp and deduplicate
        all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(self.deduplicate_entries(all_entries))
    }
    
    pub async fn get_daily_report(&self, date: &str) -> Result<DailyUsageReport> {
        let entries = self.load_usage_data().await?;
        let filtered = entries.into_iter()
            .filter(|entry| entry.date_string() == date)
            .collect();
            
        Ok(DailyUsageReport::from_entries(filtered))
    }
    
    pub async fn get_live_monitoring(&self) -> Result<LiveUsageStats> {
        // Real-time monitoring of active sessions
        let active_sessions = self.detect_active_sessions().await?;
        let current_usage = self.calculate_current_usage(&active_sessions).await?;
        
        Ok(LiveUsageStats {
            active_sessions: active_sessions.len(),
            current_tokens: current_usage.tokens,
            projected_cost: current_usage.projected_cost,
            rate_limit_status: self.check_rate_limits().await?,
        })
    }
}
```

#### 2. Cost Calculation Engine
**Features**:
- Integration with LiteLLM pricing database
- Support for multiple cost calculation modes
- Cache token tracking and cost optimization

**Implementation**:
```rust
// New file: src-tauri/src/cost_calculator.rs
pub struct CostCalculator {
    pricing_data: HashMap<String, ModelPricing>,
    cost_mode: CostMode,
}

impl CostCalculator {
    pub async fn calculate_cost(&self, usage: &TokenUsage) -> Result<f64> {
        match self.cost_mode {
            CostMode::Auto => {
                if let Some(precalculated) = usage.cost_usd {
                    Ok(precalculated)
                } else {
                    self.calculate_from_tokens(usage).await
                }
            },
            CostMode::Calculate => self.calculate_from_tokens(usage).await,
            CostMode::Display => Ok(usage.cost_usd.unwrap_or(0.0))
        }
    }
    
    async fn calculate_from_tokens(&self, usage: &TokenUsage) -> Result<f64> {
        let pricing = self.pricing_data.get(&usage.model_name)
            .ok_or_else(|| anyhow!("No pricing data for model: {}", usage.model_name))?;
            
        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * pricing.input_cost_per_million;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * pricing.output_cost_per_million;
        
        // Add cache token costs if available
        let cache_cost = if let Some(cache_tokens) = usage.cache_tokens {
            (cache_tokens as f64 / 1_000_000.0) * pricing.cache_cost_per_million.unwrap_or(0.0)
        } else {
            0.0
        };
        
        Ok(input_cost + output_cost + cache_cost)
    }
}
```

## Phase 3: Frontend Integration

### Enhanced GUI Components

#### 1. Usage Dashboard
**New Component**: Real-time usage monitoring dashboard

**Features**:
- Live token usage graphs
- Cost projections and budgets
- Session-based analytics
- Model usage breakdown

**Implementation**:
```javascript
// Enhanced file: src/main.js
class UsageDashboard {
    constructor() {
        this.usageChart = new Chart(ctx, usageChartConfig);
        this.updateInterval = null;
    }
    
    async startLiveMonitoring() {
        this.updateInterval = setInterval(async () => {
            const stats = await invoke('get_live_usage_stats');
            this.updateCharts(stats);
            this.updateCostProjections(stats);
        }, 5000); // Update every 5 seconds
    }
    
    async updateCharts(stats) {
        this.usageChart.data.datasets[0].data = stats.token_history;
        this.usageChart.update('none'); // No animation for live updates
        
        // Update cost display
        document.getElementById('current-cost').textContent = 
            `$${stats.projected_cost.toFixed(4)}`;
    }
}
```

#### 2. Advanced Prompt Editor
**Enhanced Component**: Prompt editor with @ symbol support

**Features**:
- Syntax highlighting for @ references
- Auto-completion for file paths
- File preview in tooltips
- Validation warnings for missing files

**Implementation**:
```javascript
// New component: src/components/PromptEditor.js
class PromptEditor {
    constructor(container) {
        this.editor = this.initializeEditor(container);
        this.fileResolver = new FileReferenceResolver();
    }
    
    initializeEditor(container) {
        // Initialize code mirror or similar editor
        const editor = CodeMirror(container, {
            mode: 'claude-prompt',
            lineNumbers: true,
            autoCloseBrackets: true,
            extraKeys: {
                'Ctrl-Space': 'autocomplete'
            }
        });
        
        // Add @ symbol auto-completion
        editor.on('inputRead', (cm, change) => {
            if (change.text[0] === '@') {
                this.showFileCompletion(cm);
            }
        });
        
        return editor;
    }
    
    async showFileCompletion(editor) {
        const files = await invoke('get_available_files');
        const hints = files.map(file => ({
            text: file.path,
            displayText: file.name,
            className: 'file-hint'
        }));
        
        editor.showHint({
            hint: () => ({ list: hints, from: editor.getCursor(), to: editor.getCursor() })
        });
    }
}
```

## Phase 4: CLI Enhancement

### Enhanced CLI Commands

#### 1. Usage Analysis Commands
```bash
# New CLI commands for usage tracking
cnp usage daily                    # Daily usage report
cnp usage monthly                  # Monthly usage report  
cnp usage session                  # Session-based usage
cnp usage live                     # Live monitoring dashboard
cnp usage export --format json    # Export usage data

# Enhanced prompt commands with @ support
cnp prompt create --with-files     # Create prompt with file references
cnp prompt validate                # Validate @ references in prompts
cnp prompt preview                 # Preview resolved prompt content
```

#### 2. Implementation
```rust
// Enhanced file: src-tauri/src/bin/cnp.rs
#[derive(Subcommand)]
enum UsageCommands {
    Daily {
        #[arg(long)]
        since: Option<String>,
        #[arg(long)]
        until: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Monthly {
        #[arg(long)]
        json: bool,
    },
    Session {
        #[arg(long)]
        project: Option<String>,
    },
    Live {
        #[arg(long)]
        refresh_rate: Option<u64>,
    },
    Export {
        #[arg(long)]
        format: String, // json, csv
        #[arg(long)]
        output: Option<String>,
    },
}

async fn handle_usage_command(cmd: UsageCommands) -> Result<()> {
    let usage_tracker = UsageTracker::new().await?;
    
    match cmd {
        UsageCommands::Daily { since, until, json } => {
            let report = usage_tracker.get_daily_report(since, until).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_daily_table(&report);
            }
        },
        UsageCommands::Live { refresh_rate } => {
            start_live_monitoring(usage_tracker, refresh_rate.unwrap_or(5)).await?;
        },
        // ... other commands
    }
    
    Ok(())
}
```

## Implementation Timeline

### Phase 1 (Week 1-2): Core Integration
- [ ] Create `claude_executor.rs` with stream-json processing
- [ ] Implement `file_resolver.rs` for @ symbol support
- [ ] Update database schema with new tables
- [ ] Basic Tauri command integration

### Phase 2 (Week 3-4): Usage Tracking
- [ ] Integrate ccusage data loading functionality
- [ ] Implement cost calculation engine
- [ ] Create usage analytics dashboard
- [ ] Add live monitoring capabilities

### Phase 3 (Week 5-6): Frontend Enhancement
- [ ] Build usage dashboard component
- [ ] Enhance prompt editor with @ symbol support
- [ ] Add file reference validation UI
- [ ] Implement real-time updates

### Phase 4 (Week 7-8): CLI and Testing
- [ ] Expand CLI with usage commands
- [ ] Add comprehensive test coverage
- [ ] Performance optimization
- [ ] Documentation and user guides

## Technical Considerations

### Security
- File access validation and sandboxing
- Path traversal prevention
- Permission checking for file references
- Audit logging for all file access

### Performance
- Streaming JSON processing for large outputs
- Efficient file caching for repeated references
- Background usage data aggregation
- Optimized database queries with indexes

### Compatibility
- Support for both old (`~/.claude/`) and new (`~/.config/claude/`) Claude paths
- Backward compatibility with existing prompt format
- Graceful fallback when @ references fail

### Error Handling
- Graceful handling of missing files
- Network error recovery for usage data loading
- Clear error messages for invalid @ references
- Automatic retry for transient failures

## Success Metrics

### Functionality
- [ ] 100% compatibility with existing prompt workflows
- [ ] Support for all Claude Code @ reference patterns
- [ ] Real-time usage tracking with <5% overhead
- [ ] Cost calculation accuracy within 1% of actual billing

### Performance  
- [ ] @ symbol resolution in <100ms for typical files
- [ ] Stream processing with <200ms latency
- [ ] Usage dashboard updates in <1s
- [ ] CLI commands respond in <2s

### User Experience
- [ ] Intuitive @ symbol auto-completion
- [ ] Clear validation feedback for file references
- [ ] Comprehensive usage insights and analytics
- [ ] Seamless integration with existing workflows