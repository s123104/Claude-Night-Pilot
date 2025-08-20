-- 0003_create_unified_jobs.sql
-- 創建統一任務表格 - 基於 Context7 最佳實踐和企業級需求
-- 遷移日期: 2025-08-20
-- 目標: 統一所有任務管理到單一企業級表格

-- 創建統一任務表格
CREATE TABLE IF NOT EXISTS unified_jobs (
    -- 基礎欄位
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    job_type TEXT NOT NULL DEFAULT 'scheduled',
    priority INTEGER DEFAULT 5,
    
    -- 企業級執行統計
    execution_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    last_run_time TEXT,
    next_run_time TEXT,
    last_success_time TEXT,
    last_failure_time TEXT,
    
    -- 階層式任務管理 (vibe-kanban 模式)
    parent_job_id TEXT,
    
    -- 執行選項與配置 (JSON 格式)
    execution_options TEXT DEFAULT '{}',
    retry_config TEXT DEFAULT '{"max_retries": 3, "backoff_strategy": "exponential"}',
    notification_config TEXT DEFAULT '{}',
    timeout_config TEXT DEFAULT '{"execution_timeout_seconds": 3600}',
    
    -- 標籤與元數據
    tags TEXT DEFAULT '[]',
    metadata TEXT DEFAULT '{}',
    
    -- 審計與追蹤
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT DEFAULT 'system',
    updated_by TEXT DEFAULT 'system',
    version INTEGER DEFAULT 1,
    
    -- 外鍵約束
    FOREIGN KEY (parent_job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 創建性能優化索引
CREATE INDEX IF NOT EXISTS idx_unified_jobs_status ON unified_jobs(status);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_created_at ON unified_jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_parent ON unified_jobs(parent_job_id);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_next_run ON unified_jobs(next_run_time);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_job_type ON unified_jobs(job_type);

-- 創建複合索引用於常見查詢模式
CREATE INDEX IF NOT EXISTS idx_unified_jobs_status_type ON unified_jobs(status, job_type);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_priority_next_run ON unified_jobs(priority DESC, next_run_time ASC);

-- 創建任務執行過程表 (支援 vibe-kanban ExecutionProcess 模式)
CREATE TABLE IF NOT EXISTS job_execution_processes (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    process_type TEXT NOT NULL, -- 'setup', 'execution', 'cleanup', 'validation'
    status TEXT NOT NULL,       -- 'queued', 'running', 'completed', 'failed', 'cancelled', 'retrying'
    start_time TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time TEXT,
    output TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 為執行過程表創建索引
CREATE INDEX IF NOT EXISTS idx_job_execution_processes_job_id ON job_execution_processes(job_id);
CREATE INDEX IF NOT EXISTS idx_job_execution_processes_status ON job_execution_processes(status);
CREATE INDEX IF NOT EXISTS idx_job_execution_processes_start_time ON job_execution_processes(start_time);

-- 創建任務執行結果表
CREATE TABLE IF NOT EXISTS job_execution_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    execution_start_time TEXT NOT NULL,
    execution_end_time TEXT,
    status TEXT NOT NULL,       -- 'success', 'failed', 'timeout', 'cancelled'
    output TEXT,
    error_message TEXT,
    execution_duration_ms INTEGER,
    tokens_used INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 為執行結果表創建索引
CREATE INDEX IF NOT EXISTS idx_job_execution_results_job_id ON job_execution_results(job_id);
CREATE INDEX IF NOT EXISTS idx_job_execution_results_status ON job_execution_results(status);
CREATE INDEX IF NOT EXISTS idx_job_execution_results_start_time ON job_execution_results(execution_start_time);

-- 創建使用量追蹤表 (基於 ccusage 模式)
CREATE TABLE IF NOT EXISTS usage_tracking (
    id TEXT PRIMARY KEY,
    job_id TEXT,
    session_id TEXT,
    model_name TEXT,
    tokens_input INTEGER DEFAULT 0,
    tokens_output INTEGER DEFAULT 0,
    tokens_total INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0.0,
    execution_duration_ms INTEGER DEFAULT 0,
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT DEFAULT '{}',
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE SET NULL
);

-- 為使用量追蹤表創建索引
CREATE INDEX IF NOT EXISTS idx_usage_tracking_job_id ON usage_tracking(job_id);
CREATE INDEX IF NOT EXISTS idx_usage_tracking_session_id ON usage_tracking(session_id);
CREATE INDEX IF NOT EXISTS idx_usage_tracking_timestamp ON usage_tracking(timestamp);

-- 創建系統配置表
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    category TEXT DEFAULT 'general',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 插入預設系統配置
INSERT OR IGNORE INTO system_config (key, value, description, category) VALUES 
('scheduler_enabled', 'true', '排程器是否啟用', 'scheduler'),
('max_concurrent_jobs', '5', '最大並行任務數', 'scheduler'),
('default_job_timeout', '3600', '預設任務超時時間(秒)', 'scheduler'),
('cleanup_retention_days', '30', '清理保留天數', 'maintenance'),
('enable_usage_tracking', 'true', '是否啟用使用量追蹤', 'monitoring'),
('version', '0.2.0', '資料庫架構版本', 'system');

-- 啟用外鍵約束 (SQLite 特定)
PRAGMA foreign_keys = ON;

-- 啟用 WAL 模式以提升併發性能 (基於 Context7 最佳實踐)
PRAGMA journal_mode = WAL;

-- 設置合理的 cache size
PRAGMA cache_size = -2000; -- 2MB cache

-- 啟用查詢優化
PRAGMA optimize;