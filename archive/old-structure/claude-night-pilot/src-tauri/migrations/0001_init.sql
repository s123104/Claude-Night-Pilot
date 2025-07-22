-- 初始化資料庫 schema
-- 創建時間: 2025-07-22T04:21:39+08:00

-- prompts 表
CREATE TABLE prompts (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  title     TEXT NOT NULL,
  content   TEXT NOT NULL,
  tags      TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- jobs 表
CREATE TABLE jobs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  prompt_id   INTEGER NOT NULL,
  cron_expr   TEXT NOT NULL,     -- '*' 即手動
  mode        TEXT NOT NULL,     -- 'sync' | 'async'
  status      TEXT NOT NULL DEFAULT 'pending',     -- 'pending' | 'running' | 'done' | 'error'
  eta_unix    INTEGER,           -- cooldown 倒數秒
  last_run_at DATETIME,
  FOREIGN KEY (prompt_id) REFERENCES prompts(id) ON DELETE CASCADE
);

-- results 表
CREATE TABLE results (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id    INTEGER NOT NULL,
  content   TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);

-- 建立索引以提升查詢效能
CREATE INDEX idx_prompts_created_at ON prompts(created_at);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_prompt_id ON jobs(prompt_id);
CREATE INDEX idx_results_job_id ON results(job_id);
CREATE INDEX idx_results_created_at ON results(created_at); 