# 🚀 Claude Night Pilot 完整CLI使用指南

**🎯 連小朋友都看得懂的全面使用說明書**

---

## ⚡ 快速啟動與測試指令

### 🏃‍♂️ 超快速開始（3分鐘上手）

```bash
# 1️⃣ 建置CLI工具（只需要一次）
cd /path/to/Claude-Night-Pilot
cargo build --release

# 2️⃣ 快速健康檢查（像給電腦量體溫）
./target/release/cnp-optimized health --fast

# 3️⃣ 查看有什麼功能可以用
./target/release/cnp-optimized --help
./target/release/cnp-unified --help

# 4️⃣ 試試看基本功能（不會弄壞任何東西）
./target/release/cnp-optimized status
./target/release/cnp-unified status
```

### 🧪 完整功能測試（確保一切正常）

```bash
# 效能測試（看看有多快）
./target/release/cnp-optimized benchmark --iterations 3

# 查看所有任務（現在應該是空的）
./target/release/cnp-unified job list

# 檢查系統狀態（包含資料庫連線等）
./target/release/cnp-unified status
```

---

## 📚 完整功能目錄

### 🎭 雙重人格CLI工具

Claude Night Pilot 有兩個CLI工具，就像超人和克拉克：

#### ⚡ `cnp-optimized` - 閃電俠模式
- **超能力**: 11.7毫秒啟動（比眨眼還快！）
- **適合**: 經常使用、腳本自動化、效能要求高
- **主要功能**: 健康檢查、狀態查詢、快速執行、效能測試

#### 🦸‍♂️ `cnp-unified` - 全能超人模式  
- **超能力**: 完整功能集、會話管理、Git整合
- **適合**: 複雜工作流程、專案開發、會話管理
- **主要功能**: 完整的任務管理、Git worktree、Claude會話

---

## 🔥 cnp-optimized 閃電俠指南

### 🩺 健康檢查命令

```bash
# 🚀 超快速健康檢查（12毫秒完成）
./target/release/cnp-optimized health --fast

# 🔍 詳細健康檢查（知道所有細節）
./target/release/cnp-optimized health

# 💾 JSON格式輸出（給程式讀的）
./target/release/cnp-optimized health --format json
```

**小朋友解釋**: 就像給電腦看醫生，檢查它是否健康！

**輸出範例**:
```
⚡ Claude Night Pilot 快速健康檢查
⏱️  檢查時間: 12ms

🔍 核心檢查:
✅ UnifiedClaudeInterface 初始化正常
⚠️  Claude CLI 二進位檔案檢查: 未找到 'claude' 命令
✅ 冷卻檢測系統: 運行正常

💡 建議:
- 請確保 Claude CLI 已安裝並在 PATH 中
```

### 📊 系統狀態查詢

```bash
# 🎯 系統狀態總覽
./target/release/cnp-optimized status

# 📈 JSON格式狀態（程式專用）
./target/release/cnp-optimized status --format json
```

**小朋友解釋**: 看看電腦現在的心情和狀態怎麼樣！

### ⚡ 快速執行

```bash
# 💬 執行文字指令
./target/release/cnp-optimized execute -p "Hello Claude!"

# 📄 從檔案執行
./target/release/cnp-optimized execute -f my-prompt.txt

# ⌨️ 從鍵盤輸入執行
echo "Tell me a joke" | ./target/release/cnp-optimized execute --stdin

# 🔧 進階選項
./target/release/cnp-optimized execute -p "Analyze this code" \
    --mode sync \
    --work-dir /path/to/project \
    --retry true \
    --format json
```

**小朋友解釋**: 就像跟Claude聊天，但是用命令列！

### 🏃‍♂️ 效能測試

```bash
# 🎯 標準效能測試
./target/release/cnp-optimized benchmark

# 🔥 高強度測試（5次迭代）
./target/release/cnp-optimized benchmark --iterations 5

# 📊 JSON格式結果
./target/release/cnp-optimized benchmark --format json --iterations 3
```

**小朋友解釋**: 測試電腦跑得有多快，就像賽車測速！

**效能目標與結果**:
```
🏆 效能評估: A 級
✅ 啟動時間: ~13ms (目標: <100ms) - 超越目標87%
⚠️  健康檢查: ~460ms (目標: <200ms) - 標準模式較慢但功能完整
✅ 快速健康檢查: <50ms - 完全符合快速模式目標
✅ 狀態查詢: <10ms (目標: <50ms) - 超越目標80%

💡 提示：使用 --fast 選項可獲得 <50ms 的健康檢查速度
```

---

## 🦸‍♂️ cnp-unified 全能超人指南

### 📊 系統狀態與監控

```bash
# 🎯 系統狀態總覽
./target/release/cnp-unified status
```

**小朋友解釋**: 就像儀表板，告訴你所有東西的狀況！

### 🎭 Claude會話管理

#### 創建新會話
```bash
# ✨ 創建基本會話
./target/release/cnp-unified session create "我的新專案"

# 🌳 創建帶Git分支的會話
./target/release/cnp-unified session create "功能開發" \
    --description "實作新的登入功能" \
    --create-worktree \
    --branch "feature-login"

# 🏷️ 帶標籤的會話
./target/release/cnp-unified session create "緊急修復" \
    --tags "bug,urgent,security"
```

#### 管理現有會話
```bash
# 📋 列出所有會話
./target/release/cnp-unified session list

# 🔍 查看特定會話詳情
./target/release/cnp-unified session show <session-id>

# ▶️ 恢復會話
./target/release/cnp-unified session resume <session-id>

# 🎯 在會話中執行命令
./target/release/cnp-unified session execute <session-id> \
    "分析這個專案的架構並提供改善建議"

# ⏸️ 暫停會話
./target/release/cnp-unified session pause <session-id>

# ✅ 完成並清理會話
./target/release/cnp-unified session complete <session-id>
```

**小朋友解釋**: 會話就像不同的工作空間，每個都有自己的記憶和檔案！

### 🌳 Git Worktree管理

```bash
# 🌱 創建新的工作樹
./target/release/cnp-unified worktree create feature-payments

# 📂 創建在指定位置
./target/release/cnp-unified worktree create feature-ui \
    --path /path/to/workdir \
    --branch feature-ui-redesign

# 📋 列出所有工作樹
./target/release/cnp-unified worktree list

# 📊 詳細列表（包含狀態）
./target/release/cnp-unified worktree list --detailed

# 🧹 清理特定工作樹
./target/release/cnp-unified worktree cleanup /path/to/worktree

# 🗑️ 清理所有無效工作樹
./target/release/cnp-unified worktree cleanup --all
```

**小朋友解釋**: Worktree就像給每個功能一個獨立的房間工作！

### 💬 提示詞管理

```bash
# 📋 列出所有提示詞
./target/release/cnp-unified prompt list

# 🔍 搜索提示詞
./target/release/cnp-unified prompt list --search "分析"

# ✨ 創建新提示詞
./target/release/cnp-unified prompt create \
    --title "程式碼審查" \
    --content "請審查這段程式碼的品質、安全性和性能" \
    --tags "review,code,quality"

# 📄 查看提示詞詳情
./target/release/cnp-unified prompt show <prompt-id>

# ✏️ 編輯提示詞
./target/release/cnp-unified prompt update <prompt-id> \
    --title "新標題" \
    --content "新內容"

# 🗑️ 刪除提示詞
./target/release/cnp-unified prompt delete <prompt-id>
```

**小朋友解釋**: 提示詞就像預先寫好的問題，可以重複使用！

### 🎯 任務管理

#### 創建和管理任務
```bash
# ✨ 創建簡單任務
./target/release/cnp-unified job create \
    --name "每日報告" \
    --prompt-id <prompt-id> \
    --cron "0 0 9 * * *"  # 每天早上9點

# 🔄 創建重複任務
./target/release/cnp-unified job create \
    --name "週報生成" \
    --prompt "生成本週工作總結" \
    --cron "0 0 18 * * 5"  # 週五下午6點
    --retry-attempts 3 \
    --timeout 3600

# 📋 列出所有任務
./target/release/cnp-unified job list

# 🔍 按狀態篩選
./target/release/cnp-unified job list --status active
./target/release/cnp-unified job list --status pending
./target/release/cnp-unified job list --status completed

# 📊 詳細列表
./target/release/cnp-unified job list --format detailed
```

#### 任務執行控制
```bash
# ▶️ 手動執行任務
./target/release/cnp-unified job run <job-id>

# ⏸️ 暫停任務
./target/release/cnp-unified job pause <job-id>

# ▶️ 恢復任務
./target/release/cnp-unified job resume <job-id>

# ✏️ 更新任務
./target/release/cnp-unified job update <job-id> \
    --name "新名稱" \
    --cron "0 0 10 * * *"

# 🗑️ 刪除任務
./target/release/cnp-unified job delete <job-id>

# 📄 查看任務詳情
./target/release/cnp-unified job show <job-id>
```

**小朋友解釋**: 任務就像鬧鐘，到時間就會自動執行你想要的工作！

### 🚀 執行命令

```bash
# 💬 基本執行
./target/release/cnp-unified run -p "Hello Claude!"

# 📄 從檔案執行
./target/release/cnp-unified run -f my-prompt.txt

# ⌨️ 從標準輸入
echo "Explain quantum computing" | ./target/release/cnp-unified run --stdin

# 🎯 在特定會話中執行
./target/release/cnp-unified run -p "分析程式碼" \
    --session <session-id>

# 🔧 完整選項執行
./target/release/cnp-unified execute -p "複雜任務" \
    --mode async \
    --work-dir /path/to/project \
    --retry true \
    --cooldown-check true \
    --format json \
    --session <session-id>
```

### 📊 批量執行

```bash
# 🚀 並行執行多個任務
./target/release/cnp-unified batch \
    --file tasks.json \
    --concurrent 3

# 📋 從檔案列表執行
./target/release/cnp-unified batch \
    --prompts prompt1.txt,prompt2.txt,prompt3.txt \
    --mode sync

# 📊 批量執行結果
./target/release/cnp-unified batch \
    --job-ids job1,job2,job3 \
    --format json
```

**批量任務檔案格式 (tasks.json)**:
```json
{
  "tasks": [
    {
      "name": "分析程式碼",
      "prompt": "分析 @src/main.js 的程式品質",
      "options": {
        "work_dir": "/project/path",
        "retry": true
      }
    },
    {
      "name": "測試執行",  
      "prompt": "執行所有單元測試並報告結果",
      "options": {
        "timeout_seconds": 300
      }
    }
  ]
}
```

### 📈 結果查看

```bash
# 📋 查看執行結果
./target/release/cnp-unified results

# 🔍 查看特定任務的結果
./target/release/cnp-unified results --job-id <job-id>

# 📊 詳細結果顯示
./target/release/cnp-unified results show <result-id>

# 🕐 按時間範圍篩選
./target/release/cnp-unified results \
    --since "2025-08-01" \
    --until "2025-08-31"

# 📄 匯出結果
./target/release/cnp-unified results export \
    --format json \
    --output results.json

# 📊 結果統計
./target/release/cnp-unified results stats
```

**小朋友解釋**: 結果就像考試成績單，記錄了所有任務的執行情況！

---

## 🎛️ Cron表達式指南

Claude Night Pilot 使用6欄位Cron格式：

```
秒 分 時 日 月 週
*  *  *  *  *  *
```

### 📅 常用時間表達式

```bash
# ⏰ 每分鐘執行
"0 * * * * *"

# 🌅 每天早上9點
"0 0 9 * * *"

# 🌃 每天晚上6點
"0 0 18 * * *"

# 📅 週一到週五早上10點（工作日）
"0 0 10 * * 1-5"

# 📊 每週五下午5點（週報）
"0 0 17 * * 5"

# 📆 每月1號早上8點（月報）
"0 0 8 1 * *"

# 🕐 每2小時執行一次
"0 0 */2 * * *"

# ⏰ 每30分鐘執行
"0 */30 * * * *"
```


---


---

## 🐛 疑難排解指南

### 🩺 常見問題診斷

```bash
# 🔍 基本系統健康檢查
./target/release/cnp-optimized health --fast

# 📊 查看詳細系統狀態  
./target/release/cnp-unified status

# 📋 列出所有會話
./target/release/cnp-unified session list

# 🌳 列出所有工作樹
./target/release/cnp-unified worktree list
```

### ⚠️ 常見錯誤解決

#### Claude CLI未找到
```bash
# 問題：Claude CLI 二進位檔案檢查: 未找到 'claude' 命令

# 解決方案1：安裝Claude CLI
npm install -g @anthropic-ai/claude-code

# 解決方案2：檢查PATH設定
echo $PATH | grep node
which claude

# 解決方案3：檢查Claude Code認證
claude auth status

# 解決方案4：重新登入Claude Code
claude auth login
```

#### Claude認證問題
```bash
# 問題：❌ Claude Code 認證失效或未設定

# 解決方案1：檢查認證狀態
claude auth status

# 解決方案2：重新登入
claude auth login

# 解決方案3：驗證API金鑰
echo $ANTHROPIC_API_KEY

# 注意：會話創建功能需要有效的Claude Code認證
```

#### 會話無法創建
```bash
# 問題：會話創建失敗

# 檢查Git配置
git config --global user.name
git config --global user.email

# 檢查工作目錄權限
ls -la /path/to/project

# 使用診斷工具
./target/release/cnp-unified session diagnose <session-id>
```

#### 任務執行失敗
```bash
# 檢查任務狀態
./target/release/cnp-unified job show <job-id>

# 查看錯誤日誌
./target/release/cnp-unified results show <result-id>

# 重試失敗的任務
./target/release/cnp-unified job retry <job-id>
```

---

## 🚀 效能優化技巧

### ⚡ 速度優化

```bash
# 1️⃣ 使用Release版本（快10倍）
cargo build --release
./target/release/cnp-optimized  # 而不是 ./target/debug/

# 2️⃣ 快速模式健康檢查
./target/release/cnp-optimized health --fast  # 12ms vs 311ms

# 3️⃣ 並行執行任務
./target/release/cnp-unified batch --concurrent 5

# 4️⃣ 查看系統狀態以監控效能
./target/release/cnp-unified status
```

### 💾 記憶體優化

```bash
# 查看系統資訊並優化效能
./target/release/cnp-unified status

# 檢查任務狀態減少資源使用
./target/release/cnp-unified job list --status active
```

---

## 📚 實用腳本範例

### 🤖 自動化腳本

#### 每日報告生成器
```bash
#!/bin/bash
# daily-report.sh

echo "📊 生成每日報告..."

# 執行分析任務
./target/release/cnp-unified execute -p "
分析今天的開發進度：
- 檢查Git提交記錄
- 統計程式碼變更
- 列出完成的功能
- 指出待解決的問題
請以清晰的格式整理報告
" --format json > daily-report.json

echo "✅ 報告已保存到 daily-report.json"
```

#### 專案健康檢查
```bash
#!/bin/bash
# project-health.sh

echo "🔍 檢查專案健康度..."

# 並行執行多項檢查
./target/release/cnp-unified batch --file - <<EOF
{
  "tasks": [
    {
      "name": "程式碼品質檢查",
      "prompt": "分析 @src/ 目錄的程式碼品質，檢查潛在問題"
    },
    {
      "name": "安全掃描", 
      "prompt": "掃描專案是否有安全漏洞或敏感資料洩露"
    },
    {
      "name": "效能分析",
      "prompt": "檢查程式碼效能瓶頸並提供優化建議"
    }
  ]
}
EOF
```

#### 自動修復腳本
```bash
#!/bin/bash
# auto-fix.sh

echo "🛠️ 自動修復常見問題..."

# 系統健康檢查
./target/release/cnp-optimized health

# 檢查系統狀態
./target/release/cnp-unified status

# 列出任務狀態
./target/release/cnp-unified job list

echo "✅ 系統檢查完成"
```

### 📊 監控腳本

```bash
#!/bin/bash
# monitor.sh - 持續監控系統狀態

while true; do
    echo "⏰ $(date): 檢查系統狀態"
    
    # 快速健康檢查
    if ./target/release/cnp-optimized health --fast --format json | jq -e '.healthy'; then
        echo "✅ 系統正常"
    else
        echo "⚠️ 發現問題，發送通知..."
        ./target/release/cnp-unified execute -p "系統異常，請檢查" 
    fi
    
    sleep 300  # 每5分鐘檢查一次
done
```

---

## 🔗 整合與擴展

### 🐙 Git Hooks整合

#### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔍 執行提交前檢查..."

# 使用Claude分析變更
git diff --cached | ./target/release/cnp-optimized execute --stdin \
    -p "分析這個Git diff，檢查：
1. 程式碼品質問題
2. 潛在的bug
3. 安全問題
4. 建議的改善
請給出簡潔的建議" \
    --format json > /tmp/pre-commit-analysis.json

# 檢查分析結果
if grep -q "CRITICAL\|HIGH_RISK" /tmp/pre-commit-analysis.json; then
    echo "❌ 發現高風險問題，提交被阻止"
    exit 1
fi

echo "✅ 提交檢查通過"
```

### 📊 CI/CD整合

#### GitHub Actions
```yaml
# .github/workflows/claude-review.yml
name: Claude Code Review

on: [pull_request]

jobs:
  claude-review:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Claude Night Pilot
      run: |
        cargo build --release
        
    - name: Review PR Changes
      run: |
        git diff origin/main...HEAD | \
        ./target/release/cnp-optimized execute --stdin \
        -p "分析這個PR的變更，提供詳細的程式碼審查建議" \
        --format json > review.json
        
    - name: Comment PR
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const review = JSON.parse(fs.readFileSync('review.json', 'utf8'));
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: `## 🤖 Claude Code Review\n\n${review.content}`
          });
```

### 🔌 API整合

```bash
# 作為HTTP API使用
./target/release/cnp-unified server start --port 8080

# curl範例
curl -X POST http://localhost:8080/api/execute \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello Claude!", "format": "json"}'
```

---

## 📖 API參考

### 🔧 命令列選項

#### 全域選項
```bash
--help, -h          # 顯示幫助資訊
--version, -V       # 顯示版本資訊
--config PATH       # 指定配置檔路徑
--verbose, -v       # 詳細輸出模式
--quiet, -q         # 安靜模式
--format FORMAT     # 輸出格式 (json|text|pretty)
```

#### 執行選項
```bash
--prompt, -p TEXT              # 執行的提示內容
--file, -f FILE               # 從檔案讀取提示
--stdin                       # 從標準輸入讀取
--mode MODE                   # 執行模式 (sync|async|scheduled)
--work-dir DIR                # 工作目錄
--retry BOOL                  # 啟用重試機制
--cooldown-check BOOL         # 檢查冷卻狀態
--timeout SECONDS             # 執行超時時間
--session SESSION_ID          # 指定會話ID
--dangerously-skip-permissions # 跳過權限檢查（危險！）
```

### 📊 回傳格式

#### JSON格式
```json
{
  "status": "success|error",
  "data": {
    "content": "執行結果內容",
    "execution_time": 1250,
    "tokens_used": 150,
    "cost_usd": 0.002
  },
  "metadata": {
    "session_id": "uuid",
    "timestamp": "2025-08-20T10:30:00Z",
    "model": "claude-3-sonnet"
  },
  "error": null
}
```

#### 狀態格式
```json
{
  "system": {
    "healthy": true,
    "version": "0.1.0",
    "uptime": 3600
  },
  "claude": {
    "available": false,
    "version": null,
    "cooling": false
  },
  "database": {
    "connected": true,
    "records": 42
  },
  "sessions": {
    "active": 2,
    "total": 15
  }
}
```

---

## 🧪 指令驗證測試

讓我系統性驗證所有文檔中的指令是否正確：

### ✅ 基本功能驗證