# 🚀 Claude Night Pilot CLI 快速參考卡片

## 快速選擇指南

| 需求場景 | 推薦CLI | 原因 |
|---------|---------|------|
| 自動化腳本 | `cnp-optimized` | 11.4ms超快啟動 |
| 日常使用 | `cnp-unified` | 完整功能集 |
| 性能監控 | `cnp-optimized` | 專業基準測試 |
| Prompt管理 | `cnp-unified` | 內建管理功能 |

## 🔥 最常用命令 (Top 10)

### 系統狀態檢查
```bash
# 🚀 超快系統概覽 (<10ms)
cnp-optimized status

# 🏥 快速健康檢查 (<50ms)  
cnp-optimized health --fast

# 🕐 冷卻狀態檢查
cnp-optimized cooldown
```

### Claude命令執行
```bash
# 📝 直接執行prompt
cnp-unified execute -p "分析這段代碼"

# 📄 從檔案執行
cnp-unified execute -f prompt.txt

# ⌨️ 管道輸入執行
echo "分析代碼" | cnp-unified execute --stdin
```

### Prompt管理
```bash
# 📋 列出所有prompts
cnp-unified prompt list

# ➕ 創建新prompt
cnp-unified prompt create "代碼審查" "請審查以下代碼" --tags "開發"
```

### 批量處理
```bash
# 📦 批量執行prompts
cnp-unified batch -f prompts.json -c 2
```

## ⚡ 性能數據速覽

| 指標 | cnp-optimized | 目標值 | 狀態 |
|------|---------------|--------|------|
| 啟動時間 | 11.4ms | 100ms | ✅ 超越88% |
| 快速健康檢查 | <50ms | 200ms | ✅ 超越75% |
| 標準健康檢查 | 451ms | 200ms | ❌ 待優化 |
| 狀態查詢 | <10ms | - | ✅ 極速 |

## 🎯 輸出格式選擇

| 格式 | 適用場景 | 範例 |
|------|----------|------|
| `--format json` | 程式處理、API整合 | `{"status": "ok"}` |
| `--format text` | 管道操作、腳本解析 | `系統可用` |
| `--format pretty` | 人類閱讀、終端顯示 | `✅ 系統可用` |

## 🛠️ NPM快速腳本

```bash
# 常用快捷方式
npm run cli -- status                # 快速狀態
npm run cli -- health --fast         # 快速檢查
npm run cli:build                     # 編譯release版本

# 開發除錯
export CNP_DEBUG_TIMING=1            # 啟用時間統計
npm run cli -- health                # 查看執行時間
```

## 🚨 故障排除速查

| 問題症狀 | 診斷命令 | 可能原因 |
|----------|----------|----------|
| 命令無響應 | `cnp-optimized health` | Claude CLI未安裝 |
| 執行失敗 | `cnp-optimized cooldown` | 系統冷卻中 |
| 效能異常 | `cnp-optimized benchmark -i 3` | 系統資源不足 |
| 權限錯誤 | `--dangerously-skip-permissions` | 權限配置問題 |

## 📊 批量處理範例

**prompts.json**:
```json
[
  "分析第一段代碼的性能瓶頸",
  {"content": "審查第二段代碼的安全性"},
  {"prompt": "優化第三段代碼的可讀性"}
]
```

**執行命令**:
```bash
cnp-unified batch -f prompts.json -c 2 --format json
```

## 🔧 開發者模式

```bash
# 啟用詳細時間統計
export CNP_DEBUG_TIMING=1

# 執行並查看性能數據
cnp-optimized execute -p "測試" 
# 輸出會包含: 總執行時間, 執行耗時等統計

# 清除除錯模式
unset CNP_DEBUG_TIMING
```

## 📈 進階功能預覽

| 功能 | 狀態 | 預計版本 |
|------|------|----------|
| GUI整合 | ✅ 已實現 | v0.1.1 |
| 排程任務 | 🔄 開發中 | v0.2.0 |
| 插件系統 | 📋 計劃中 | v0.3.0 |
| 雲端同步 | 💭 構思中 | v1.0.0 |

---

**💡 提示**: 使用 `--help` 獲取任何命令的最新參數資訊  
**🔗 完整文檔**: 查看 [CLI_FUNCTIONALITY_REFERENCE.md](./CLI_FUNCTIONALITY_REFERENCE.md)  
**⚡ 版本**: Claude Night Pilot v0.1.1 | 架構: vibe-kanban modular