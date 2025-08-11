# CLI 使用指南

Claude Night Pilot CLI (`cnp`) 提供完整的命令列介面，適合自動化整合與進階使用者。

## 基本語法

```bash
cnp [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS] [ARGS]
```

### 全域選項

```bash
-h, --help          顯示說明資訊
-V, --version       顯示版本資訊  
-v, --verbose       詳細輸出
-q, --quiet         安靜模式
--config <FILE>     指定配置檔案
--no-color          停用彩色輸出
```

## 命令參考

### 1. 初始化與設定

#### `cnp init`
初始化資料庫與基本設定

```bash
cnp init                    # 標準初始化
cnp init --force            # 強制重新初始化
cnp init --config-only      # 僅建立設定檔
```

#### `cnp config`
配置管理

```bash
# 查看設定
cnp config list
cnp config get <KEY>

# 修改設定  
cnp config set <KEY> <VALUE>
cnp config unset <KEY>

# 重置設定
cnp config reset
```

**常用設定項**：
- `working_directory`: 預設工作目錄
- `default_timeout`: 預設超時時間 (秒)
- `retry_count`: 失敗重試次數
- `log_level`: 日誌等級 (error/warn/info/debug)

### 2. Prompt 管理

#### `cnp prompt`
Prompt 模板管理

```bash
# 列出所有 Prompts
cnp prompt list
cnp prompt list --tag <TAG>         # 依標籤篩選
cnp prompt list --format json       # JSON 格式輸出

# 建立 Prompt
cnp prompt create --name "檢查程式碼" --content "請檢查這個檔案的程式碼品質"
cnp prompt create --file prompt.txt  # 從檔案讀取

# 查看 Prompt 詳情
cnp prompt show <ID>
cnp prompt show <NAME>

# 編輯 Prompt  
cnp prompt edit <ID>
cnp prompt edit <ID> --content "新內容"

# 刪除 Prompt
cnp prompt delete <ID>
cnp prompt delete --name <NAME>
```

### 3. 任務執行

#### `cnp execute`
執行 Prompt 或任務

```bash
# 立即執行
cnp execute --prompt "幫我優化這個專案"
cnp execute --id <PROMPT_ID>
cnp execute --file prompt.txt

# 執行選項
cnp execute --prompt "檢查安全性" --mode safe      # 安全模式
cnp execute --id 1 --timeout 600                   # 設定超時
cnp execute --id 1 --working-dir /path/to/project  # 指定目錄
cnp execute --id 1 --dry-run                       # 試運行模式

# @ 符號檔案引用
cnp execute --prompt "分析 @src/main.js 的效能問題"
cnp execute --prompt "檢查 @docs/*.md 文檔完整性"
```

#### `cnp run`
執行特定任務（別名：execute）

```bash
cnp run --id <PROMPT_ID>
cnp run --name <PROMPT_NAME>  
cnp run --interactive          # 互動模式
```

### 4. 排程管理

#### `cnp schedule`  
任務排程管理

```bash
# 建立排程任務
cnp schedule create --prompt-id 1 --cron "0 9 * * *"
cnp schedule create --name "每日檢查" --cron "0 9 * * *" --prompt-id 1

# 列出排程任務
cnp schedule list
cnp schedule list --active      # 僅顯示活躍任務
cnp schedule list --format table

# 控制排程任務
cnp schedule start <JOB_ID>     # 啟動任務
cnp schedule stop <JOB_ID>      # 停止任務  
cnp schedule pause <JOB_ID>     # 暫停任務
cnp schedule resume <JOB_ID>    # 恢復任務

# 排程任務資訊
cnp schedule show <JOB_ID>      # 查看詳情
cnp schedule logs <JOB_ID>      # 查看日誌
cnp schedule delete <JOB_ID>    # 刪除任務
```

**Cron 表達式範例**：
```bash
cnp schedule create --cron "0 9 * * *"     # 每日 9 AM
cnp schedule create --cron "*/30 * * * *"  # 每 30 分鐘  
cnp schedule create --cron "0 18 * * 1-5"  # 工作日 6 PM
cnp schedule create --cron "0 9 1 * *"     # 每月 1 號 9 AM
```

### 5. 結果與日誌

#### `cnp results`
查看執行結果

```bash
# 列出結果
cnp results list
cnp results list --limit 10          # 限制顯示數量
cnp results list --since "2025-01-01" # 指定日期後的結果
cnp results list --status success     # 依狀態篩選
cnp results list --format json        # JSON 格式

# 查看特定結果
cnp results show <RESULT_ID>
cnp results show <RESULT_ID> --full   # 完整輸出
cnp results export <RESULT_ID>        # 匯出結果
```

#### `cnp logs`
系統日誌管理

```bash
cnp logs                    # 查看最新日誌
cnp logs --tail 100         # 顯示最後 100 行
cnp logs --follow           # 即時監控
cnp logs --level error      # 僅顯示錯誤
cnp logs --since "1h"       # 最近 1 小時
```

### 6. 系統監控

#### `cnp status`
系統狀態檢查

```bash
cnp status                  # 整體狀態
cnp status --detailed       # 詳細資訊
cnp status --format json    # JSON 格式
cnp status --health-check   # 執行健康檢查
```

#### `cnp health`
健康檢查

```bash
cnp health                  # 基本健康檢查
cnp health --full           # 完整檢查
cnp health --fast           # 快速檢查
cnp health --format json    # JSON 格式輸出
```

#### `cnp cooldown`
Claude API 冷卻檢查

```bash
cnp cooldown                # 檢查冷卻狀態
cnp cooldown --wait         # 等待冷卻結束
cnp cooldown --estimate     # 估算冷卻時間
```

### 7. 資料管理

#### `cnp backup`
資料備份與還原

```bash
cnp backup create                    # 建立備份
cnp backup create --path /backup/   # 指定備份路徑
cnp backup list                      # 列出備份
cnp backup restore <BACKUP_ID>       # 還原備份
```

#### `cnp clean`
清理與維護

```bash
cnp clean cache            # 清除快取
cnp clean logs             # 清除舊日誌  
cnp clean results --older-than 30d  # 清除 30 天前的結果
cnp clean all              # 清除所有暫存資料
```

## 進階用法

### 管道操作

```bash
# 將檔案內容作為 Prompt
cat prompt.txt | cnp execute --stdin

# 處理多個檔案
find . -name "*.js" | xargs -I {} cnp execute --prompt "檢查 @{} 的程式碼品質"
```

### 腳本整合

```bash
#!/bin/bash
# 自動化檢查腳本

# 檢查系統健康狀況
if cnp health --quiet; then
    echo "系統正常，開始執行任務..."
    cnp execute --id 1
else
    echo "系統異常，跳過執行"
    exit 1
fi
```

### JSON 輸出處理

```bash
# 使用 jq 處理 JSON 輸出
cnp results list --format json | jq '.[] | select(.status == "success")'

# 提取特定欄位
cnp schedule list --format json | jq -r '.[] | .name + ": " + .next_run'
```

## 設定檔案

### 全域設定檔案位置

- **Linux/macOS**: `~/.config/claude-night-pilot/config.toml`
- **Windows**: `%APPDATA%\claude-night-pilot\config.toml`

### 專案設定檔案

在專案根目錄建立 `.cnp-config.toml`：

```toml
[general]
working_directory = "."
default_timeout = 300

[execution]  
retry_count = 3
skip_confirmations = false

[logging]
level = "info"
file = "./cnp.log"
```

## 環境變數

```bash
export CNP_CONFIG_PATH="/path/to/config"     # 設定檔路徑
export CNP_WORKING_DIR="/path/to/workspace"  # 工作目錄
export CNP_LOG_LEVEL="debug"                 # 日誌等級
export CNP_NO_COLOR="1"                      # 停用彩色
```

## 疑難排解

### 常見問題

**命令找不到**：
```bash
# 檢查 cnp 是否在 PATH 中
which cnp
export PATH="$PATH:/path/to/cnp"
```

**權限問題**：
```bash
# 確保有執行權限
chmod +x cnp

# 資料庫權限
chmod 644 claude-pilot.db
```

**Claude Code 整合問題**：
```bash  
# 檢查 Claude Code 安裝
cnp health --full

# 手動測試 Claude Code
claude --version
```

### 除錯模式

```bash
# 啟用詳細輸出
cnp --verbose execute --id 1

# 除錯等級日誌
CNP_LOG_LEVEL=debug cnp execute --id 1

# 輸出到檔案
cnp execute --id 1 2>&1 | tee execution.log
```

---

更多範例請參閱 [範例目錄](../examples/) 或查看 [FAQ](../faq.md)。