-- 添加使用量追蹤和監控相關表
-- 創建時間: 2025-07-24T00:55:47+08:00

-- usage_records 表 - 記錄Claude使用量歷史
CREATE TABLE usage_records (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp           DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  remaining_minutes   INTEGER NOT NULL,
  total_minutes       INTEGER NOT NULL,
  usage_percentage    REAL NOT NULL,
  source              TEXT NOT NULL,     -- "ccusage", "fallback", etc.
  raw_output          TEXT,              -- 原始ccusage輸出（可選）
  created_at          DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- execution_audits 表 - 記錄執行審計日誌
CREATE TABLE execution_audits (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp           DATETIME NOT NULL,
  prompt_hash         TEXT NOT NULL,     -- SHA256雜湊
  execution_options   TEXT NOT NULL,     -- JSON格式的執行選項
  security_check      TEXT NOT NULL,     -- JSON格式的安全檢查結果
  execution_start     DATETIME,
  execution_end       DATETIME,
  result              TEXT NOT NULL,     -- Success, Failed, Cancelled, etc.
  output_length       INTEGER,
  error_message       TEXT,
  created_at          DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- scheduled_tasks 表 - 記錄排程任務
CREATE TABLE scheduled_tasks (
  id                         TEXT PRIMARY KEY,  -- task_id
  prompt                     TEXT NOT NULL,
  execution_options          TEXT NOT NULL,     -- JSON格式
  scheduled_at               DATETIME NOT NULL,
  created_at                 DATETIME NOT NULL,
  estimated_duration_minutes INTEGER NOT NULL,
  timezone                   TEXT NOT NULL,
  retry_count                INTEGER DEFAULT 0,
  max_retries                INTEGER DEFAULT 3,
  status                     TEXT NOT NULL,     -- Scheduled, Executing, etc.
  updated_at                 DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- monitoring_events 表 - 記錄監控事件
CREATE TABLE monitoring_events (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp           DATETIME NOT NULL,
  event_type          TEXT NOT NULL,     -- ModeChanged, StatusUpdated, etc.
  mode                TEXT NOT NULL,     -- Normal, Approaching, etc.
  usage_info          TEXT,              -- JSON格式的使用量資訊
  message             TEXT NOT NULL,
  created_at          DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 建立索引以提升查詢效能
CREATE INDEX idx_usage_records_timestamp ON usage_records(timestamp);
CREATE INDEX idx_usage_records_source ON usage_records(source);
CREATE INDEX idx_execution_audits_timestamp ON execution_audits(timestamp);
CREATE INDEX idx_execution_audits_prompt_hash ON execution_audits(prompt_hash);
CREATE INDEX idx_scheduled_tasks_status ON scheduled_tasks(status);
CREATE INDEX idx_scheduled_tasks_scheduled_at ON scheduled_tasks(scheduled_at);
CREATE INDEX idx_monitoring_events_timestamp ON monitoring_events(timestamp);
CREATE INDEX idx_monitoring_events_event_type ON monitoring_events(event_type); 