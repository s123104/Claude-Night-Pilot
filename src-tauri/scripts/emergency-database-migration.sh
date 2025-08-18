#!/bin/bash
# 🚨 Claude Night Pilot - 緊急資料庫路徑統一腳本
# 基於 SCHEDULER_IMPLEMENTATION_ROADMAP.md Day 1 P0 任務
# 創建時間: 2025-08-17T17:45:00+00:00

set -euo pipefail

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日誌函數
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 配置變數
LEGACY_DB="claude-pilot.db"
UNIFIED_DB="claude-night-pilot.db"
BACKUP_DIR="./backups/$(date +%Y%m%d_%H%M%S)"
MIGRATION_LOG="$BACKUP_DIR/migration.log"

# 檢查依賴
check_dependencies() {
    log_info "檢查系統依賴..."
    
    if ! command -v sqlite3 &> /dev/null; then
        log_error "sqlite3 未安裝，請先安裝 SQLite"
        exit 1
    fi
    
    if ! command -v rsync &> /dev/null; then
        log_error "rsync 未安裝，請先安裝 rsync"
        exit 1
    fi
    
    log_success "依賴檢查完成"
}

# 創建備份目錄
create_backup_directory() {
    log_info "創建備份目錄: $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"
    echo "Migration started at $(date)" > "$MIGRATION_LOG"
}

# 備份現有資料庫
backup_existing_databases() {
    log_info "備份現有資料庫..."
    
    # 備份舊資料庫
    if [[ -f "$LEGACY_DB" ]]; then
        log_info "備份舊資料庫: $LEGACY_DB"
        cp "$LEGACY_DB" "$BACKUP_DIR/claude-pilot-backup.db"
        log_success "舊資料庫備份完成"
        echo "Legacy database backed up: $LEGACY_DB -> $BACKUP_DIR/claude-pilot-backup.db" >> "$MIGRATION_LOG"
    else
        log_warning "舊資料庫檔案不存在: $LEGACY_DB"
        echo "Legacy database not found: $LEGACY_DB" >> "$MIGRATION_LOG"
    fi
    
    # 備份統一資料庫（如果存在）
    if [[ -f "$UNIFIED_DB" ]]; then
        log_info "備份統一資料庫: $UNIFIED_DB"
        cp "$UNIFIED_DB" "$BACKUP_DIR/claude-night-pilot-backup.db"
        log_success "統一資料庫備份完成"
        echo "Unified database backed up: $UNIFIED_DB -> $BACKUP_DIR/claude-night-pilot-backup.db" >> "$MIGRATION_LOG"
    else
        log_info "統一資料庫檔案不存在: $UNIFIED_DB (這是正常的)"
        echo "Unified database not found: $UNIFIED_DB (this is normal)" >> "$MIGRATION_LOG"
    fi
}

# 檢查資料庫結構
check_database_schema() {
    local db_file="$1"
    local db_name="$2"
    
    log_info "檢查 $db_name 資料庫結構..."
    
    if [[ ! -f "$db_file" ]]; then
        log_warning "$db_name 資料庫不存在: $db_file"
        return 1
    fi
    
    # 檢查表結構
    local tables=$(sqlite3 "$db_file" ".tables")
    log_info "$db_name 包含的表: $tables"
    echo "$db_name tables: $tables" >> "$MIGRATION_LOG"
    
    # 檢查 prompts 表
    if sqlite3 "$db_file" ".schema prompts" &>/dev/null; then
        local prompt_count=$(sqlite3 "$db_file" "SELECT COUNT(*) FROM prompts;")
        log_info "$db_name prompts 表記錄數: $prompt_count"
        echo "$db_name prompts count: $prompt_count" >> "$MIGRATION_LOG"
    fi
    
    # 檢查 jobs 表
    if sqlite3 "$db_file" ".schema jobs" &>/dev/null; then
        local job_count=$(sqlite3 "$db_file" "SELECT COUNT(*) FROM jobs;")
        log_info "$db_name jobs 表記錄數: $job_count"
        echo "$db_name jobs count: $job_count" >> "$MIGRATION_LOG"
    fi
    
    # 檢查 schedules 表（舊版本）
    if sqlite3 "$db_file" ".schema schedules" &>/dev/null; then
        local schedule_count=$(sqlite3 "$db_file" "SELECT COUNT(*) FROM schedules;")
        log_info "$db_name schedules 表記錄數: $schedule_count"
        echo "$db_name schedules count: $schedule_count" >> "$MIGRATION_LOG"
    fi
    
    return 0
}

# 創建統一資料庫結構
create_unified_database() {
    log_info "創建統一資料庫結構..."
    
    # 如果統一資料庫已存在，先備份
    if [[ -f "$UNIFIED_DB" ]]; then
        log_warning "統一資料庫已存在，將覆蓋現有檔案"
    fi
    
    # 創建新的統一資料庫
    cat > "$BACKUP_DIR/unified_schema.sql" << 'EOF'
-- Claude Night Pilot 統一資料庫結構
-- 基於 SCHEDULER_COMPREHENSIVE_REFACTORING_PLAN.md

-- 啟用外鍵約束
PRAGMA foreign_keys = ON;

-- Prompts 表 (保持現有結構)
CREATE TABLE IF NOT EXISTS prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    tags TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 統一 Jobs 表 (整合所有現有功能)
CREATE TABLE IF NOT EXISTS unified_jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    job_type TEXT NOT NULL DEFAULT 'scheduled',
    priority INTEGER DEFAULT 5,
    
    -- 企業級功能
    execution_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0,
    last_run_time TEXT,
    next_run_time TEXT,
    
    -- vibe-kanban 模式
    parent_job_id TEXT,
    
    -- 執行選項 (JSON)
    execution_options TEXT DEFAULT '{}',
    retry_config TEXT DEFAULT '{}',
    notification_config TEXT,
    
    -- 標籤與元數據
    tags TEXT DEFAULT '[]',
    metadata TEXT DEFAULT '{}',
    
    -- 時間戳
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by TEXT,
    
    FOREIGN KEY (parent_job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 任務執行流程表 (基於 vibe-kanban ExecutionProcess)
CREATE TABLE IF NOT EXISTS job_execution_processes (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    process_type TEXT NOT NULL CHECK (process_type IN ('setup', 'execution', 'cleanup', 'validation')),
    status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled', 'retrying')),
    start_time TEXT NOT NULL,
    end_time TEXT,
    output TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 任務執行結果表 (來自 RealTimeExecutor)
CREATE TABLE IF NOT EXISTS job_execution_results (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    prompt_id TEXT NOT NULL,
    status TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    output TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE CASCADE
);

-- 使用量追蹤表 (基於 ccusage 模式)
CREATE TABLE IF NOT EXISTS usage_tracking (
    id TEXT PRIMARY KEY,
    job_id TEXT,
    session_id TEXT,
    tokens_input INTEGER DEFAULT 0,
    tokens_output INTEGER DEFAULT 0,
    tokens_total INTEGER DEFAULT 0,
    cost_usd REAL DEFAULT 0.0,
    model_name TEXT,
    execution_duration_ms INTEGER DEFAULT 0,
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (job_id) REFERENCES unified_jobs(id) ON DELETE SET NULL
);

-- 系統配置表
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引優化
CREATE INDEX IF NOT EXISTS idx_unified_jobs_status ON unified_jobs(status);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_created_at ON unified_jobs(created_at);
CREATE INDEX IF NOT EXISTS idx_unified_jobs_parent ON unified_jobs(parent_job_id);
CREATE INDEX IF NOT EXISTS idx_execution_processes_job_id ON job_execution_processes(job_id);
CREATE INDEX IF NOT EXISTS idx_execution_processes_status ON job_execution_processes(status);
CREATE INDEX IF NOT EXISTS idx_execution_results_job_id ON job_execution_results(job_id);
CREATE INDEX IF NOT EXISTS idx_usage_tracking_job_id ON usage_tracking(job_id);
CREATE INDEX IF NOT EXISTS idx_usage_tracking_timestamp ON usage_tracking(timestamp);

-- 插入系統配置
INSERT OR REPLACE INTO system_config (key, value, description) VALUES
('database_version', '2.0.0-unified', 'Unified database schema version'),
('migration_date', datetime('now'), 'Date when migration was completed'),
('scheduler_type', 'unified', 'Type of scheduler implementation');
EOF

    # 應用資料庫結構
    log_info "應用統一資料庫結構到 $UNIFIED_DB"
    sqlite3 "$UNIFIED_DB" < "$BACKUP_DIR/unified_schema.sql"
    log_success "統一資料庫結構創建完成"
}

# 遷移舊資料庫資料
migrate_legacy_data() {
    if [[ ! -f "$LEGACY_DB" ]]; then
        log_info "舊資料庫不存在，跳過資料遷移"
        return 0
    fi
    
    log_info "開始遷移舊資料庫資料..."
    
    # 遷移 prompts 表
    if sqlite3 "$LEGACY_DB" ".schema prompts" &>/dev/null; then
        log_info "遷移 prompts 表..."
        sqlite3 "$LEGACY_DB" ".output '$BACKUP_DIR/prompts_export.sql'" ".dump prompts"
        sqlite3 "$UNIFIED_DB" < "$BACKUP_DIR/prompts_export.sql" 2>/dev/null || true
        log_success "prompts 表遷移完成"
    fi
    
    # 遷移 jobs 表到 unified_jobs
    if sqlite3 "$LEGACY_DB" ".schema jobs" &>/dev/null; then
        log_info "遷移 jobs 表到 unified_jobs..."
        
        # 導出 jobs 資料並轉換格式
        sqlite3 -header -csv "$LEGACY_DB" "SELECT * FROM jobs;" > "$BACKUP_DIR/jobs_export.csv"
        
        # 創建轉換腳本
        cat > "$BACKUP_DIR/convert_jobs.py" << 'EOF'
import csv
import sqlite3
import sys
import json
from datetime import datetime

def convert_jobs(legacy_csv, unified_db):
    conn = sqlite3.connect(unified_db)
    cursor = conn.cursor()
    
    with open(legacy_csv, 'r', encoding='utf-8') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            # 轉換資料格式
            cursor.execute("""
                INSERT OR REPLACE INTO unified_jobs 
                (id, name, prompt_id, cron_expression, status, job_type, 
                 execution_count, failure_count, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                row.get('id', ''),
                row.get('name', ''),
                row.get('prompt_id', ''),
                row.get('cron_expression', ''),
                row.get('status', 'active'),
                row.get('job_type', 'scheduled'),
                int(row.get('execution_count', 0) or 0),
                int(row.get('failure_count', 0) or 0),
                row.get('created_at', datetime.now().isoformat()),
                row.get('updated_at', datetime.now().isoformat())
            ))
    
    conn.commit()
    conn.close()
    print(f"Converted jobs from {legacy_csv} to {unified_db}")

if __name__ == "__main__":
    convert_jobs(sys.argv[1], sys.argv[2])
EOF
        
        # 執行轉換
        if command -v python3 &> /dev/null; then
            python3 "$BACKUP_DIR/convert_jobs.py" "$BACKUP_DIR/jobs_export.csv" "$UNIFIED_DB"
            log_success "jobs 表遷移完成"
        else
            log_warning "Python3 未安裝，跳過 jobs 表自動轉換"
        fi
    fi
    
    # 遷移 schedules 表（如果存在）
    if sqlite3 "$LEGACY_DB" ".schema schedules" &>/dev/null; then
        log_info "檢測到 schedules 表，需要手動合併到 unified_jobs"
        sqlite3 -header -csv "$LEGACY_DB" "SELECT * FROM schedules;" > "$BACKUP_DIR/schedules_export.csv"
        log_warning "schedules 表已導出到 $BACKUP_DIR/schedules_export.csv，需要手動檢查和合併"
    fi
}

# 驗證遷移結果
verify_migration() {
    log_info "驗證遷移結果..."
    
    # 檢查統一資料庫結構
    check_database_schema "$UNIFIED_DB" "統一資料庫"
    
    # 比較記錄數量
    if [[ -f "$LEGACY_DB" ]]; then
        log_info "比較遷移前後資料數量..."
        
        # 比較 prompts 表
        if sqlite3 "$LEGACY_DB" ".schema prompts" &>/dev/null && sqlite3 "$UNIFIED_DB" ".schema prompts" &>/dev/null; then
            local legacy_prompts=$(sqlite3 "$LEGACY_DB" "SELECT COUNT(*) FROM prompts;")
            local unified_prompts=$(sqlite3 "$UNIFIED_DB" "SELECT COUNT(*) FROM prompts;")
            
            if [[ "$legacy_prompts" -eq "$unified_prompts" ]]; then
                log_success "prompts 表遷移驗證通過: $legacy_prompts = $unified_prompts"
            else
                log_warning "prompts 表數量不一致: 舊=$legacy_prompts, 新=$unified_prompts"
            fi
        fi
        
        # 檢查 unified_jobs 表
        local unified_jobs=$(sqlite3 "$UNIFIED_DB" "SELECT COUNT(*) FROM unified_jobs;" 2>/dev/null || echo "0")
        log_info "unified_jobs 表記錄數: $unified_jobs"
    fi
    
    # 檢查資料庫完整性
    local integrity_check=$(sqlite3 "$UNIFIED_DB" "PRAGMA integrity_check;")
    if [[ "$integrity_check" == "ok" ]]; then
        log_success "資料庫完整性檢查通過"
    else
        log_error "資料庫完整性檢查失敗: $integrity_check"
        return 1
    fi
}

# 更新應用配置
update_application_config() {
    log_info "更新應用配置..."
    
    # 創建配置更新腳本
    cat > "$BACKUP_DIR/update_config.md" << EOF
# 應用配置更新指南

## 需要更新的檔案

1. **src-tauri/src/database_manager.rs**
   - 將所有 \`claude-pilot.db\` 改為 \`claude-night-pilot.db\`
   - 更新表名稱從 \`jobs\` 到 \`unified_jobs\`

2. **src-tauri/src/scheduler/real_time_executor.rs**
   - 確認使用 \`claude-night-pilot.db\`
   - 更新表結構對應新的 unified_jobs

3. **src-tauri/src/services/job_service.rs**
   - 更新查詢語句使用 unified_jobs 表
   - 適配新的資料結構

4. **測試檔案**
   - 更新所有測試中的資料庫路徑
   - 修改測試資料結構對應新表

## 驗證步驟

1. 執行測試套件確保功能正常
2. 檢查日誌中是否有資料庫路徑錯誤
3. 驗證排程器功能是否正常運作

## 回滾指南

如果遷移出現問題，可以從備份恢復：
\`\`\`bash
cp $BACKUP_DIR/claude-pilot-backup.db claude-pilot.db
cp $BACKUP_DIR/claude-night-pilot-backup.db claude-night-pilot.db
\`\`\`
EOF
    
    log_success "配置更新指南已創建: $BACKUP_DIR/update_config.md"
}

# 清理臨時檔案
cleanup_temporary_files() {
    log_info "清理臨時檔案..."
    
    # 保留重要的備份和日誌，清理其他臨時檔案
    rm -f "$BACKUP_DIR/convert_jobs.py" 2>/dev/null || true
    rm -f "$BACKUP_DIR/prompts_export.sql" 2>/dev/null || true
    
    log_success "臨時檔案清理完成"
}

# 生成遷移報告
generate_migration_report() {
    log_info "生成遷移報告..."
    
    cat > "$BACKUP_DIR/migration_report.md" << EOF
# Claude Night Pilot 資料庫遷移報告

**遷移時間**: $(date)
**遷移版本**: 從分散式資料庫到統一資料庫 v2.0.0
**備份位置**: $BACKUP_DIR

## 遷移摘要

### 檔案變更
- **舊資料庫**: $LEGACY_DB $(if [[ -f "$LEGACY_DB" ]]; then echo "✅ 已備份"; else echo "❌ 不存在"; fi)
- **統一資料庫**: $UNIFIED_DB ✅ 已創建

### 資料遷移狀態
$(if [[ -f "$LEGACY_DB" ]]; then
    echo "- Prompts 表: $(sqlite3 "$LEGACY_DB" "SELECT COUNT(*) FROM prompts;" 2>/dev/null || echo "0") 筆記錄已遷移"
    if sqlite3 "$LEGACY_DB" ".schema jobs" &>/dev/null; then
        echo "- Jobs 表: $(sqlite3 "$LEGACY_DB" "SELECT COUNT(*) FROM jobs;" 2>/dev/null || echo "0") 筆記錄已轉換為 unified_jobs"
    fi
    if sqlite3 "$LEGACY_DB" ".schema schedules" &>/dev/null; then
        echo "- Schedules 表: $(sqlite3 "$LEGACY_DB" "SELECT COUNT(*) FROM schedules;" 2>/dev/null || echo "0") 筆記錄需要手動檢查"
    fi
else
    echo "- 舊資料庫不存在，建立全新統一資料庫"
fi)

### 新資料庫結構
- unified_jobs: 統一任務表
- job_execution_processes: 執行流程追蹤
- job_execution_results: 執行結果記錄
- usage_tracking: 使用量追蹤
- system_config: 系統配置

## 後續行動項目

1. **立即行動**
   - [ ] 更新應用程式碼中的資料庫路徑
   - [ ] 修改表名稱從 jobs 到 unified_jobs
   - [ ] 執行完整測試套件

2. **驗證測試**
   - [ ] 排程器功能測試
   - [ ] 資料完整性測試
   - [ ] 性能基準測試

3. **清理工作**
   - [ ] 確認新系統穩定後，可以刪除舊資料庫檔案
   - [ ] 更新文檔和部署腳本

## 回滾計劃

如果需要回滾到舊版本：
\`\`\`bash
cd $(pwd)
cp $BACKUP_DIR/claude-pilot-backup.db claude-pilot.db
rm claude-night-pilot.db
git checkout HEAD~1  # 回到遷移前的程式碼版本
\`\`\`

## 支援資源

- 備份檔案: $BACKUP_DIR/
- 遷移日誌: $MIGRATION_LOG
- 配置指南: $BACKUP_DIR/update_config.md
- 問題回報: 請檢查遷移日誌檔案

---
**狀態**: ✅ 遷移完成
**下一步**: 請按照配置更新指南修改應用程式碼
EOF

    log_success "遷移報告已生成: $BACKUP_DIR/migration_report.md"
}

# 主執行函數
main() {
    echo "🚨 Claude Night Pilot - 緊急資料庫路徑統一工具"
    echo "=================================================="
    echo ""
    
    # 執行遷移步驟
    check_dependencies
    create_backup_directory
    backup_existing_databases
    check_database_schema "$LEGACY_DB" "舊資料庫" || true
    check_database_schema "$UNIFIED_DB" "統一資料庫" || true
    create_unified_database
    migrate_legacy_data
    verify_migration
    update_application_config
    cleanup_temporary_files
    generate_migration_report
    
    echo ""
    echo "🎉 資料庫遷移完成！"
    echo ""
    echo "📋 下一步行動："
    echo "1. 檢查遷移報告: $BACKUP_DIR/migration_report.md"
    echo "2. 閱讀配置指南: $BACKUP_DIR/update_config.md"
    echo "3. 更新應用程式碼中的資料庫路徑"
    echo "4. 執行測試套件驗證功能正常"
    echo ""
    echo "🔒 備份位置: $BACKUP_DIR"
    echo "📝 遷移日誌: $MIGRATION_LOG"
    echo ""
    echo "⚠️  重要提醒：請在確認新系統穩定後再刪除備份檔案"
}

# 執行主函數
main "$@"