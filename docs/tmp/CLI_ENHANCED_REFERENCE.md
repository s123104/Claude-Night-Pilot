# 🚀 Claude Night Pilot CLI 增強版功能參考手冊

**版本**: v0.1.1 Enhanced  
**更新時間**: 2025年8月15日  
**架構**: vibe-kanban modular + Git Worktree + Claude Session Management

---

## 📋 目錄

1. [概覽](#概覽)
2. [新增功能](#新增功能)
3. [CLI工具對比](#cli工具對比)
4. [命令參考](#命令參考)
5. [工作流程範例](#工作流程範例)
6. [高級功能](#高級功能)
7. [故障排除](#故障排除)

---

## 概覽

Claude Night Pilot 提供雙CLI架構，針對不同使用場景優化：

### 🏎️ cnp-optimized (性能優化版)
- **啟動時間**: 11.7ms (超越目標88%)
- **專注**: 極速執行、基礎功能
- **適用**: 頻繁使用、腳本整合

### 🔧 cnp-unified (統一功能版)  
- **功能**: 完整功能集、GUI一致性
- **適用**: 複雜操作、會話管理

---

## 🆕 新增功能

### 1. Claude 會話管理 (Session Management)
- 🚀 **創建會話**: 支援Git worktree整合
- 🔄 **恢復會話**: 跨終端會話持續性  
- ⚡ **會話執行**: 在指定會話中執行命令
- 📊 **會話統計**: Token使用量和成本追蹤

### 2. Git Worktree 管理
- 🌿 **創建Worktree**: 自動分支和目錄管理
- 🧹 **智能清理**: 安全的worktree和元數據清理
- 📋 **列表查看**: 實時worktree狀態顯示

### 3. 完整CRUD Job管理
- 📅 **創建任務**: 支援Cron表達式
- 📝 **更新任務**: 修改排程和描述
- 🗑️ **刪除任務**: 安全刪除確認
- 🔍 **查看詳情**: 任務執行歷史

---

## CLI工具對比

| 功能 | cnp-optimized | cnp-unified |
|------|---------------|-------------|
| **基礎執行** | ✅ 11.7ms啟動 | ✅ 完整功能 |
| **健康檢查** | ✅ 快速/標準模式 | ✅ 統一檢查 |
| **冷卻檢測** | ✅ 並行檢查 | ✅ 完整檢查 |
| **Prompt管理** | ❌ | ✅ 完整CRUD |
| **Job排程** | ❌ | ✅ 完整CRUD |
| **會話管理** | ❌ | ✅ 完整功能 |
| **Worktree管理** | ❌ | ✅ 完整功能 |
| **批量執行** | ❌ | ✅ 並發執行 |

---

## 命令參考

### cnp-optimized 命令集

#### `cnp-optimized execute` - 極速執行
```bash
# 基本執行
cnp-optimized execute -p "Hello Claude"

# 檔案輸入
cnp-optimized execute -f prompt.txt

# 輸出格式
cnp-optimized execute -p "分析報告" --format json
```

#### `cnp-optimized status` - 系統狀態
```bash
# 超快狀態檢查 (<10ms)
cnp-optimized status
# 輸出: {"database":"connected","prompts":0,"results":0,"tasks":0}
```

#### `cnp-optimized health` - 健康檢查
```bash
# 快速檢查 (12ms)
cnp-optimized health --fast

# 標準檢查 (311ms) 
cnp-optimized health --format json
```

#### `cnp-optimized benchmark` - 性能基準
```bash
# 預設5次迭代
cnp-optimized benchmark

# 自定義迭代
cnp-optimized benchmark --iterations 10

# JSON輸出
cnp-optimized benchmark --format json
```

### cnp-unified 命令集

#### Session 管理命令

##### `session create` - 創建新會話
```bash
# 基本會話創建
cnp-unified session create "功能開發會話"

# 帶描述
cnp-unified session create "API開發" --description "開發用戶API接口"

# 創建Git worktree
cnp-unified session create "功能分支開發" --create-worktree --branch "feature-xyz"

# 指定分支名稱
cnp-unified session create "修復Bug" --create-worktree --branch "fix-bug-123"
```

##### `session resume` - 恢復會話
```bash
# 根據會話UUID恢復
cnp-unified session resume 550e8400-e29b-41d4-a716-446655440000
```

##### `session execute` - 會話中執行
```bash
# 在指定會話中執行命令
cnp-unified session execute 550e8400-e29b-41d4-a716-446655440000 "分析當前代碼"
```

##### `session list` - 列出所有會話
```bash
# 查看會話列表
cnp-unified session list

# 示例輸出:
# 📋 Claude 會話列表
# ═══════════════════════════════════════
# 🟢 功能開發會話 (550e8400-e29b-41d4-a716-446655440000)
#    消息數: 5, Token: 1234
#    分支: feature-xyz
```

##### `session stats` - 會話統計
```bash
# 查看使用統計
cnp-unified session stats

# 示例輸出:
# 📊 會話統計
# ═══════════════════════════════════════
# 總會話數: 3
# 活躍會話: 2
# 暫停會話: 0
# 已完成會話: 1
# 總 Token 使用: 2500
# 總成本: $1.25
```

##### `session pause/complete` - 會話控制
```bash
# 暫停會話
cnp-unified session pause 550e8400-e29b-41d4-a716-446655440000

# 完成會話（自動清理worktree）
cnp-unified session complete 550e8400-e29b-41d4-a716-446655440000
```

#### Worktree 管理命令

##### `worktree create` - 創建Worktree
```bash
# 創建新的worktree
cnp-unified worktree create feature-branch

# 指定自定義路徑
cnp-unified worktree create hotfix-branch --path /custom/path/hotfix
```

##### `worktree list` - 列出Worktrees
```bash
# 查看所有worktrees
cnp-unified worktree list

# 示例輸出:
# 📋 Git Worktree 列表
# ═══════════════════════════════════════
# /project/main           c6d1ffd [main]
# /project/worktrees/dev  42d8b40 [feature-dev]
```

##### `worktree cleanup` - 清理Worktree
```bash
# 清理指定worktree
cnp-unified worktree cleanup /path/to/worktree
```

#### Job 排程管理 (增強版)

##### `job create` - 創建排程任務
```bash
# 創建定時任務
cnp-unified job create 1 "0 9 * * *" --description "每日早上9點分析"

# 參數說明:
# 1: Prompt ID
# "0 9 * * *": Cron表達式 (每日早上9點)
# --description: 任務描述
```

##### `job update` - 更新任務
```bash
# 更新Cron表達式
cnp-unified job update 1 --cron-expr "0 10 * * *"

# 更新描述
cnp-unified job update 1 --description "修改為早上10點執行"

# 同時更新多個屬性
cnp-unified job update 1 --cron-expr "0 8 * * *" --description "早上8點分析任務"
```

##### `job show` - 顯示任務詳情
```bash
# 查看特定任務詳情
cnp-unified job show 1
```

##### `job delete` - 刪除任務
```bash
# 刪除排程任務
cnp-unified job delete 1
```

#### 其他增強功能

##### `execute/run` - 執行命令 (別名等效)
```bash
# 兩個命令完全等效
cnp-unified execute -p "Hello Claude"
cnp-unified run -p "Hello Claude"

# 支援多種輸入方式
cnp-unified run -f input.txt --format json
cnp-unified run --stdin --work-dir /project
```

##### `batch` - 批量執行
```bash
# 批量執行prompts
cnp-unified batch -f prompts.json --concurrent 3

# JSON檔案格式範例:
# [
#   "第一個prompt",
#   {"content": "第二個prompt"},
#   {"prompt": "第三個prompt"}
# ]
```

---

## 🎯 工作流程範例

### 場景1: 功能開發工作流
```bash
# 1. 創建開發會話和worktree
cnp-unified session create "用戶登錄功能" \
  --description "開發OAuth2登錄系統" \
  --create-worktree \
  --branch "feature-oauth-login"

# 2. 在會話中執行開發任務
cnp-unified session execute <session-id> \
  "分析current OAuth2最佳實踐，生成實施計劃"

# 3. 繼續在同一會話中工作
cnp-unified session execute <session-id> \
  "實現用戶登錄API，包含JWT token生成"

# 4. 完成後清理
cnp-unified session complete <session-id>
```

### 場景2: 定期維護任務
```bash
# 1. 創建維護prompt
cnp-unified prompt create "系統健康檢查" \
  "檢查系統狀態、性能指標和潛在問題" \
  --tags "維護,監控"

# 2. 設置定期執行
cnp-unified job create 1 "0 9 * * *" \
  --description "每日早晨系統檢查"

# 3. 查看執行結果
cnp-unified results --format pretty
```

### 場景3: 批量代碼分析
```bash
# 1. 準備分析任務列表 (analysis-tasks.json)
# [
#   "分析src/auth.js的安全性",
#   "檢查api/users.js的性能",
#   "審核utils/helpers.js的代碼質量"
# ]

# 2. 並發執行分析
cnp-unified batch -f analysis-tasks.json \
  --concurrent 3 \
  --format json > analysis-results.json
```

---

## 🚀 高級功能

### 1. 會話與Worktree整合
- **自動分支創建**: 會話創建時自動生成Git分支
- **工作目錄隔離**: 每個會話獨立的工作環境
- **智能清理**: 會話完成後自動清理worktree
- **跨平台支持**: Windows/WSL相容性

### 2. 智能冷卻管理
- **並行檢測**: 同時檢查多個冷卻指標
- **預測性冷卻**: 根據使用模式預測冷卻時間
- **自動重試**: 智能重試機制避免API限制

### 3. 性能優化
- **並行執行**: 多任務並行處理
- **智能快取**: 重複查詢結果快取
- **增量更新**: 只更新變更的數據

### 4. 安全增強
- **執行審計**: 所有執行記錄SHA256雜湊
- **權限驗證**: 文件和目錄訪問權限檢查
- **風險評估**: 多層級風險評估系統

---

## 🔧 故障排除

### 常見問題

#### 1. Session 創建失敗
```bash
# 問題: 會話創建失敗
# 解決: 檢查Claude CLI可用性
cnp-unified health --format json

# 檢查權限
ls -la ~/.claude/

# 重新初始化
cnp-unified init
```

#### 2. Worktree 創建錯誤
```bash
# 問題: Git worktree創建失敗
# 解決: 檢查Git狀態
git status
git worktree list

# 清理已損壞的worktree
cnp-unified worktree cleanup /path/to/broken/worktree
```

#### 3. Job 排程不執行
```bash
# 問題: 定時任務沒有執行
# 解決: 檢查任務狀態
cnp-unified job list
cnp-unified job show <job-id>

# 檢查系統健康
cnp-unified health --format pretty
```

#### 4. 性能問題
```bash
# 問題: 執行速度慢
# 解決: 使用性能優化版
cnp-optimized execute -p "你的prompt"

# 檢查系統性能
cnp-optimized benchmark --iterations 10

# 查看詳細時間分析
cnp-optimized health --format json
```

### 性能基準參考

| 操作 | cnp-optimized | cnp-unified | 目標值 |
|------|---------------|-------------|--------|
| 啟動時間 | 11.7ms | ~50ms | 100ms |
| 狀態查詢 | <10ms | ~20ms | 50ms |
| 健康檢查(快速) | 12ms | N/A | 50ms |
| 健康檢查(標準) | 311ms | ~350ms | 200ms |
| 基本執行 | 96s* | 103s* | N/A |

*包含Claude API推理時間

### 日誌和調試

```bash
# 啟用詳細日誌
RUST_LOG=debug cnp-unified session create "Debug會話"

# 查看執行跟蹤
RUST_LOG=trace cnp-optimized execute -p "測試" --format json

# 輸出到檔案
cnp-unified batch -f tasks.json 2> error.log > results.json
```

---

## 📈 版本更新記錄

### v0.1.1 Enhanced (當前版本)
- ✅ **新增**: Claude Session 管理系統
- ✅ **新增**: Git Worktree 整合
- ✅ **增強**: Job管理完整CRUD操作
- ✅ **優化**: 性能達到11.7ms啟動時間
- ✅ **修復**: 健康檢查優化至12ms(快速模式)

### 路線圖 v0.2.0
- 🔮 **計劃**: MCP服務器整合
- 🔮 **計劃**: 配置文件支援
- 🔮 **計劃**: Shell自動完成
- 🔮 **計劃**: 結構化日誌系統

---

**文檔生成時間**: 2025年8月15日 週五  
**Claude Code SuperClaude Framework** | **vibe-kanban + Git Worktree + Session Management**