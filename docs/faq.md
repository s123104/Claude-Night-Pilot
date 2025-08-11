# 常見問題 (FAQ)

本文檔收錄了 Claude Night Pilot 使用過程中的常見問題與解決方案。

## 安裝與設定

### Q: 如何確認 Claude Code 是否正確安裝？

**A**: 使用以下命令檢查：
```bash
# 檢查 Claude Code 版本
claude --version

# 使用 CNP 健康檢查
./cnp health --full
```

如果 Claude Code 未安裝，請參閱 [Claude Code 官方文檔](https://github.com/anthropics/claude-code)。

### Q: 初始化資料庫失敗，顯示權限錯誤

**A**: 這通常是檔案權限問題：
```bash
# 檢查目前目錄權限
ls -la claude-pilot.db

# 修正權限
chmod 644 claude-pilot.db
chmod 755 .

# 或完全重新初始化
rm claude-pilot.db
./cnp init --force
```

### Q: GUI 無法啟動，顯示「埠口被佔用」錯誤

**A**: 清理被佔用的埠口：
```bash
# 查找佔用埠口的程序
lsof -i :1420

# 終止相關程序
kill -9 <PID>

# 或使用專案提供的清理腳本
npm run cleanup-ports
```

## 使用問題

### Q: Prompt 執行時出現超時錯誤

**A**: 調整超時設定：
```bash
# 為特定執行設定較長超時時間
./cnp execute --id 1 --timeout 600

# 或修改全域預設值
./cnp config set default_timeout 600
```

### Q: @ 符號檔案引用無法正常工作

**A**: 確保檔案路徑正確且有存取權限：
```bash
# 檢查檔案是否存在
ls -la @src/main.js  # 錯誤：不要在 shell 中直接使用 @
ls -la src/main.js   # 正確：檢查實際檔案

# 確保在正確的工作目錄執行
./cnp execute --prompt "分析 @src/main.js" --working-dir /path/to/project
```

**@ 符號檔案引用正確格式**：
- `@file.txt` - 單一檔案
- `@folder/` - 整個資料夾  
- `@*.js` - 符合模式的檔案
- `@folder/*.py` - 特定資料夾中的檔案

### Q: 排程任務沒有按時執行

**A**: 檢查排程器狀態：
```bash
# 檢查排程器是否運行
./cnp status

# 查看特定任務狀態
./cnp schedule show <JOB_ID>

# 檢查 Cron 表達式是否正確
./cnp schedule list --format json | jq '.[] | .cron_expression'
```

**常見 Cron 問題**：
- `0 9 * * *` ✅ 正確：每日 9 AM
- `0 9 * * 1-5` ✅ 正確：工作日 9 AM  
- `9 * * *` ❌ 錯誤：缺少分鐘欄位
- `* * * * *` ⚠️ 警告：每分鐘執行（可能過於頻繁）

### Q: Claude API 回應「冷卻中」訊息

**A**: 等待冷卻期結束：
```bash
# 檢查冷卻狀態
./cnp cooldown

# 等待冷卻結束（自動等待）
./cnp cooldown --wait

# 或檢查預估等待時間
./cnp cooldown --estimate
```

**避免過度請求的策略**：
- 使用適當的執行間隔（建議至少 30 秒）
- 避免同時執行多個大型 Prompt
- 監控 API 使用量

## 效能問題

### Q: 應用程式啟動很慢

**A**: 啟用快速啟動模式：
```bash
# 跳過啟動時的健康檢查
./cnp config set skip_startup_checks true

# 啟用低記憶體模式
./cnp config set low_memory_mode true

# 清理快取
./cnp clean cache
```

### Q: 記憶體使用量過高

**A**: 調整記憶體使用設定：
```bash
# 設定較小的快取大小
./cnp config set cache_size 50MB

# 清理執行歷史
./cnp clean results --older-than 7d

# 啟用記憶體優化模式
./cnp config set memory_optimization true
```

### Q: GUI 介面反應遲緩

**A**: 優化 GUI 效能：
```bash
# 重建前端資源
npm run build

# 清除瀏覽器快取
rm -rf ~/.cache/claude-night-pilot/

# 啟用精簡模式
./cnp config set ui_mode minimal
```

## 整合問題

### Q: 如何在 CI/CD 管道中使用 CNP？

**A**: 設定無人值守模式：
```bash
#!/bin/bash
# CI/CD 腳本範例

# 設定環境變數
export CNP_NO_COLOR=1
export CNP_QUIET=1

# 初始化（如果需要）
./cnp init --config-only

# 執行檢查
if ./cnp health --fast --quiet; then
    ./cnp execute --id 1 --timeout 300
else
    echo "健康檢查失敗，跳過執行"
    exit 1
fi
```

### Q: 如何與其他工具整合？

**A**: 使用 JSON 輸出格式：
```bash
# 與 jq 整合處理結果
./cnp results list --format json | jq '.[] | select(.status == "success")'

# 與監控系統整合
./cnp status --format json | jq -r '.claude_code_available'

# 與日誌系統整合
./cnp logs --format json --since "1h" | jq -r '.[] | .message'
```

## 安全問題

### Q: 如何確保 Prompt 執行的安全性？

**A**: 啟用安全模式與審計：
```bash
# 啟用嚴格安全模式
./cnp config set security_level strict

# 啟用執行前確認
./cnp config set require_execution_confirmation true

# 查看審計日誌
./cnp logs --level security
```

### Q: 敏感資料會被記錄嗎？

**A**: CNP 有多層資料保護：
- 預設不記錄 Prompt 內容（僅記錄 ID）
- 可設定記錄等級控制詳細程度
- 審計日誌使用 SHA256 雜湊而非明文

```bash
# 設定最小記錄等級
./cnp config set log_level error

# 停用內容記錄
./cnp config set log_prompt_content false

# 啟用雜湊記錄
./cnp config set hash_sensitive_data true
```

## 故障排除

### Q: 如何收集除錯資訊？

**A**: 啟用除錯模式：
```bash
# 啟用詳細記錄
./cnp --verbose execute --id 1

# 輸出除錯資訊到檔案
CNP_LOG_LEVEL=debug ./cnp execute --id 1 2>&1 | tee debug.log

# 產生系統診斷報告
./cnp health --full --format json > system-diagnosis.json
```

### Q: 如何重置到預設狀態？

**A**: 完整重置步驟：
```bash
# 備份重要資料（可選）
./cnp backup create --path ./backup/

# 停止所有排程任務
./cnp schedule list --format json | jq -r '.[].id' | xargs -I {} ./cnp schedule stop {}

# 清除所有資料
./cnp clean all

# 重置設定
./cnp config reset

# 重新初始化
./cnp init --force
```

### Q: 資料庫損壞如何修復？

**A**: 資料庫修復步驟：
```bash
# 備份損壞的資料庫
cp claude-pilot.db claude-pilot.db.backup

# 嘗試修復
sqlite3 claude-pilot.db ".recover" > recovered.sql
sqlite3 claude-pilot-fixed.db < recovered.sql

# 或完全重建
rm claude-pilot.db
./cnp init
```

## 專案特定問題

### Q: 如何設定特定專案的配置？

**A**: 在專案根目錄建立 `.cnp-config.toml`：
```toml
[general]
working_directory = "."
default_timeout = 600

[execution]
security_level = "standard"
skip_confirmations = false

[prompts]
default_tags = ["project-name", "development"]
```

### Q: 多個專案如何共用 Prompts？

**A**: 使用全域 Prompts 或匯入/匯出功能：
```bash
# 匯出 Prompts
./cnp prompt export --output shared-prompts.json

# 在另一個專案中匯入
cd /other/project
./cnp prompt import --file shared-prompts.json

# 或使用全域模式
./cnp config set prompts_global true
```

## 獲得協助

如果上述解答無法解決您的問題：

1. **檢查日誌**: `./cnp logs --tail 100`
2. **查看文檔**: [docs/](.) 目錄中的完整文檔
3. **提交 Issue**: [GitHub Issues](https://github.com/s123104/claude-night-pilot/issues)
4. **參與討論**: [GitHub Discussions](https://github.com/s123104/claude-night-pilot/discussions)

**提交 Issue 時請包含**：
- CNP 版本：`./cnp --version`
- 系統資訊：`./cnp health --full`
- 錯誤訊息：完整的錯誤輸出
- 重現步驟：詳細的操作步驟

---

**文檔更新**：2025-08 • 如有其他問題歡迎提出建議