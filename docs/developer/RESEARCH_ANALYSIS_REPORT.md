# 研究專案分析報告

## 執行摘要

本報告詳細分析了四個 Claude CLI 自動化專案，為 Claude Night Pilot 的功能整合提供技術參考。這些專案展示了不同的自動化策略：從簡單的排程執行到複雜的任務自動化系統。

## 1. CCAutoRenew - 自動續期系統

### 核心架構
- **主要功能**: 自動偵測 Claude 5小時使用視窗並在到期時啟動新會話
- **技術棧**: Bash Shell Scripts, ccusage 整合
- **授權**: MIT License by Aniket Karne

### 關鍵實現邏輯

#### 冷卻檢測機制
```bash
# ccusage 整合 - 精確時間監控
get_ccusage_cmd() {
    if command -v ccusage &> /dev/null; then
        echo "ccusage"
    elif command -v bunx &> /dev/null; then
        echo "bunx ccusage"
    elif command -v npx &> /dev/null; then
        echo "npx ccusage@latest"
    else
        return 1
    fi
}

# 解析剩餘時間
parse_time_remaining() {
    local output=$($ccusage_cmd blocks 2>/dev/null | grep -i "time remaining")
    # 支援多種時間格式: 1h 30m, 30m, 1:30:00
}
```

#### 排程啟動功能
```bash
# 防止會話燒毀 - 指定開始時間
./claude-daemon-manager.sh start --at "09:00"
./claude-daemon-manager.sh start --at "2025-01-28 14:30"
```

#### 智能監控頻率
- 正常: 每10分鐘檢查
- <30分鐘: 每2分鐘檢查
- <5分鐘: 每30秒檢查

### 技術優勢
1. **ccusage 深度整合**: 獲得最精確的使用視窗資訊
2. **防止會話燒毀**: 支援指定開始時間避免浪費使用時數
3. **容錯設計**: ccusage 不可用時自動回退到時間計算
4. **後台守護程序**: 完整的啟動/停止/狀態檢查系統

## 2. Claude-Autopilot - VS Code 自動化擴展

### 核心架構
- **平台**: VS Code Extension (TypeScript)
- **主要功能**: 佇列管理、批量任務處理、自動恢復
- **授權**: MIT License by benbasha

### 關鍵技術實現

#### 佇列管理系統
```typescript
// 自動恢復機制
if (config.session.autoStart) {
    setTimeout(() => {
        startClaudeAutopilot(context);
    }, 1000);
}

// 跳過權限設定
const startProcessing = (skipPermissions: boolean) => {
    if (skipPermissions) {
        cmd.arg("--dangerously-skip-permissions");
    }
};
```

#### 批量處理架構
- **佇列持久化**: 工作區級別的任務佇列存儲
- **進度追蹤**: 即時更新執行狀態
- **錯誤恢復**: 自動重試機制和錯誤處理
- **睡眠防護**: 防止電腦休眠影響長時間執行

### 獨特功能
1. **24/7 自動處理**: 設定後無需人工干預
2. **VS Code 深度整合**: 工作區感知和專案管理
3. **批量任務支援**: 可處理數百個任務
4. **智能恢復**: 使用限制重置時自動恢復

## 3. claude-code-schedule - Rust 排程器

### 核心架構
- **語言**: Rust 2024 Edition
- **主要功能**: 單次定時執行 Claude 命令
- **授權**: Apache 2.0 by Ian Macalinao

### 實現邏輯

#### 時間解析與執行
```rust
// 時間解析
fn parse_time(time_str: &str) -> Result<DateTime<Local>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    let hour: u32 = parts[0].parse().context("Invalid hour")?;
    let minute: u32 = parts[1].parse().context("Invalid minute")?;
    
    // 如果時間已過，排程到明日
    if target_time <= Local::now() {
        target_time + chrono::Duration::days(1)
    }
}

// 命令建構
fn build_claude_command(message: &str) -> String {
    format!(
        "claude --dangerously-skip-permissions \"{}\"",
        message.replace("\"", "\\\"")
    )
}
```

#### 持續監控邏輯
```rust
// 倒數計時顯示
loop {
    let now = Local::now();
    if now >= target_time {
        run_claude_command(&args.message)?;
        break;
    }
    
    let duration_until = target_time.signed_duration_since(now);
    let hours = duration_until.num_hours();
    let minutes = duration_until.num_minutes() % 60;
    let seconds = duration_until.num_seconds() % 60;
    
    print!("\rTime remaining: {hours:02}:{minutes:02}:{seconds:02}");
}
```

### 設計優勢
1. **極簡設計**: 專注單一功能，程式碼清晰
2. **Rust 性能**: 低資源消耗，高可靠性
3. **跨平台支援**: 統一的命令列介面
4. **即時回饋**: 清晰的倒數計時顯示

## 4. ClaudeNightsWatch - 任務自動化系統

### 核心架構
- **基礎**: 基於 CCAutoRenew 擴展
- **主要功能**: 基於 markdown 檔案的任務自動執行
- **授權**: MIT License by Aniket Karne

### 任務執行系統

#### 任務準備機制
```bash
prepare_task_prompt() {
    local prompt=""
    
    # 添加安全規則
    if [ -f "$TASK_DIR/$RULES_FILE" ]; then
        prompt="IMPORTANT RULES TO FOLLOW:\n\n"
        prompt+=$(cat "$TASK_DIR/$RULES_FILE")
        prompt+="\n\n---END OF RULES---\n\n"
    fi
    
    # 添加任務內容
    if [ -f "$TASK_DIR/$TASK_FILE" ]; then
        prompt+="TASK TO EXECUTE:\n\n"
        prompt+=$(cat "$TASK_DIR/$TASK_FILE")
        prompt+="\n\n請閱讀上述任務，創建待辦清單並逐步執行。"
    fi
    
    echo -e "$prompt"
}
```

#### 安全約束系統
- **rules.md**: 定義安全規則和最佳實踐
- **task.md**: 定義要執行的具體任務
- **環境隔離**: 限制檔案系統存取範圍

### 創新特點
1. **基於檔案的任務定義**: 使用 markdown 格式定義複雜任務
2. **安全規則系統**: 通過 rules.md 約束自動執行行為
3. **綜合日誌記錄**: 完整記錄提示和回應
4. **任務複雜度升級**: 從簡單續期升級到複雜任務執行

## 5. ccusage - 使用監控工具

### 核心架構
- **語言**: TypeScript (Node.js)
- **主要功能**: 分析 Claude Code 本地使用資料
- **授權**: MIT License by ryoppippi

### 關鍵技術能力

#### 多維度資料分析
```typescript
// 支援多種報告格式
ccusage daily    // 日使用報告
ccusage monthly  // 月使用報告  
ccusage session  // 會話使用報告
ccusage blocks   // 5小時計費窗口報告

// 即時監控
ccusage blocks --live  // 即時使用儀表板
```

#### 計費窗口追蹤
- **精確監控**: 追蹤 Claude 的 5小時計費週期
- **即時資料**: 提供剩餘時間和使用預測
- **成本計算**: 基於 token 使用計算成本

### 整合價值
1. **精確時間資訊**: 為自動化系統提供準確的使用視窗資料
2. **成本追蹤**: 幫助優化使用策略
3. **多格式支援**: JSON 輸出便於程式化整合
4. **輕量設計**: 極小的軟體包大小，適合整合

## 整合可行性分析

### 共同技術模式

#### 1. --dangerously-skip-permissions 使用
所有專案都使用此參數實現自動化執行：
```bash
claude --dangerously-skip-permissions "prompt"
```

#### 2. ccusage 整合模式
多個專案展示了 ccusage 整合的最佳實踐：
```bash
# 檢測 ccusage 可用性
if command -v ccusage &> /dev/null; then
    ccusage_cmd="ccusage"
elif command -v bunx &> /dev/null; then
    ccusage_cmd="bunx ccusage"
else
    # 回退機制
fi
```

#### 3. 時間監控策略
共同的自適應檢查頻率模式：
- 距離到期時間越近，檢查越頻繁
- 支援多種時間格式解析
- 容錯處理和回退機制

### 技術債務識別

#### 安全性考量
1. **權限跳過風險**: 所有專案都依賴 `--dangerously-skip-permissions`
2. **檔案系統存取**: 需要嚴格的沙盒限制
3. **命令注入風險**: 需要輸入驗證和參數轉義

#### 可靠性問題
1. **ccusage 依賴**: 多個專案依賴外部工具
2. **網路連線需求**: 成本計算需要線上資源
3. **Claude CLI 版本相容性**: 需要處理 API 變更

## 建議整合策略

### 階段一：核心功能整合
1. **ccusage 整合**: 實現精確的使用視窗監控
2. **自動續期**: 實現基本的會話自動續期
3. **安全執行**: 集成 `ExecutionOptions` 系統

### 階段二：進階功能
1. **任務佇列**: 實現持久化任務管理
2. **排程系統**: 支援 Cron 式排程表達式  
3. **批量處理**: 支援多任務並行執行

### 階段三：企業功能
1. **任務範本**: 支援複雜任務定義
2. **安全規則**: 實現細緻的安全約束
3. **審計日誌**: 完整的執行追蹤和審計

## 結論

這四個專案展示了 Claude CLI 自動化的不同面向，為 Claude Night Pilot 提供了豐富的技術參考。通過整合這些專案的最佳實踐，可以建構一個功能完整、安全可靠的 Claude 自動化平台。

重點整合領域：
1. **ccusage 深度整合**用於精確監控
2. **安全執行框架**基於現有 `ExecutionOptions`
3. **多層級任務系統**從簡單續期到複雜自動化
4. **企業級安全與審計**滿足生產環境需求

下一步將基於此分析制定詳細的實現計劃和技術規範。