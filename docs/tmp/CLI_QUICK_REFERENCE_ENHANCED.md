# ⚡ Claude Night Pilot CLI 快速參考卡 v0.1.1 Enhanced

## 🏎️ cnp-optimized (極速執行)

```bash
# 基本執行 (11.7ms啟動)
cnp-optimized execute -p "Hello Claude"

# 狀態檢查 (<10ms)
cnp-optimized status

# 健康檢查
cnp-optimized health --fast     # 12ms
cnp-optimized health            # 311ms 標準模式

# 性能基準
cnp-optimized benchmark --iterations 5
```

## 🔧 cnp-unified (完整功能)

### Session 管理 🚀
```bash
# 創建會話 (支援worktree)
cnp-unified session create "開發會話" --create-worktree --branch "feature-xyz"

# 列出會話
cnp-unified session list

# 在會話中執行
cnp-unified session execute <session-id> "分析代碼"

# 會話統計  
cnp-unified session stats

# 暫停/完成會話
cnp-unified session pause <session-id>
cnp-unified session complete <session-id>
```

### Worktree 管理 🌿
```bash
# 創建worktree
cnp-unified worktree create feature-branch

# 列出worktrees
cnp-unified worktree list

# 清理worktree
cnp-unified worktree cleanup /path/to/worktree
```

### Job 管理 📅 (完整CRUD)
```bash
# 創建定時任務
cnp-unified job create 1 "0 9 * * *" --description "每日分析"

# 更新任務
cnp-unified job update 1 --cron-expr "0 10 * * *"

# 查看任務詳情
cnp-unified job show 1

# 刪除任務
cnp-unified job delete 1

# 列出所有任務
cnp-unified job list
```

### Prompt 管理 📝
```bash
# 創建prompt
cnp-unified prompt create "分析代碼" "分析這個文件的質量" --tags "代碼,質量"

# 列出prompts
cnp-unified prompt list
```

### 執行命令 ⚡
```bash
# 執行 (兩個命令等效)
cnp-unified execute -p "Hello Claude"
cnp-unified run -p "Hello Claude"

# 批量執行
cnp-unified batch -f prompts.json --concurrent 3
```

## 🎯 常用工作流

### 功能開發流程
```bash
# 1. 創建開發會話+worktree
cnp-unified session create "新功能" --create-worktree --branch "feature-xyz"

# 2. 在會話中工作
cnp-unified session execute <id> "分析需求並生成實施計劃"

# 3. 完成並清理
cnp-unified session complete <id>
```

### 定期維護
```bash
# 1. 創建維護prompt
cnp-unified prompt create "系統檢查" "檢查系統健康狀態"

# 2. 設置定期執行
cnp-unified job create 1 "0 9 * * *" --description "每日檢查"
```

## 📊 性能指標

| 操作 | cnp-optimized | cnp-unified |
|------|---------------|-------------|
| 啟動 | 11.7ms | ~50ms |
| 狀態 | <10ms | ~20ms |
| 健康(快) | 12ms | N/A |
| 健康(標) | 311ms | ~350ms |

## 🔧 故障排除

```bash
# 檢查系統健康
cnp-unified health --format json

# 性能基準測試
cnp-optimized benchmark

# 啟用調試日誌
RUST_LOG=debug cnp-unified session create "測試"

# 檢查worktree狀態
cnp-unified worktree list
git worktree list
```

## 💡 小貼士

- 🏎️ **日常使用**: 優先使用 `cnp-optimized` 
- 🔧 **複雜操作**: 使用 `cnp-unified`
- 🚀 **會話管理**: 大型項目使用session + worktree
- 📅 **定期任務**: 使用job CRUD管理排程
- ⚡ **批量處理**: 使用batch命令並行執行

---
**快速參考 v0.1.1** | **生成時間**: 2025-08-15 | **vibe-kanban + Session + Worktree**