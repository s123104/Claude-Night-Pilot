# Claude Night Pilot - 快速開始指南

**最後更新**: 2025-07-24T23:23:43+08:00  
**適用版本**: v1.0.0  
**預期完成時間**: 15 分鐘  

## 🚀 什麼是 Claude Night Pilot？

Claude Night Pilot 是一個企業級的 Claude CLI 自動化管理工具，提供：

- ✅ **智能使用量監控** - 實時追蹤 Claude 使用狀況，避免超支
- ✅ **安全執行系統** - 多層安全檢查，保護系統免受危險操作
- ✅ **自動化排程** - 時區感知的智能任務排程系統
- ✅ **雙模式操作** - CLI 和 GUI 兩種操作方式滿足不同需求

## 📋 系統需求

### 最低系統需求
- **作業系統**: macOS 10.15+, Windows 10+, 或 Ubuntu 18.04+
- **記憶體**: 最少 4GB RAM (建議 8GB+)
- **儲存空間**: 100MB 可用空間
- **網路**: 需要網際網路連線進行 Claude CLI 通訊

### 前置需求
- **Claude CLI**: 必須已安裝並配置 (v1.0.58+)
- **Node.js**: 建議 v18+ (用於 ccusage 工具)
- **網路存取**: 需要能夠存取 Claude API

## 🛠️ 安裝步驟

### 方法 1: 從 GitHub Releases 下載 (建議)

1. **下載應用程式**
   ```bash
   # 前往 GitHub Releases 頁面
   https://github.com/s123104/Claude-Night-Pilot/releases
   
   # 下載適合您作業系統的版本：
   # • macOS: Claude-Night-Pilot_1.0.0_x64.dmg
   # • Windows: Claude-Night-Pilot_1.0.0_x64.msi
   # • Linux: Claude-Night-Pilot_1.0.0_amd64.deb
   ```

2. **安裝應用程式**
   - **macOS**: 雙擊 `.dmg` 檔案，將應用程式拖到應用程式資料夾
   - **Windows**: 執行 `.msi` 安裝程式，依照指示完成安裝
   - **Linux**: 使用 `dpkg -i` 命令安裝 `.deb` 套件

### 方法 2: 從原始碼編譯

```bash
# 1. 克隆倉庫
git clone https://github.com/s123104/Claude-Night-Pilot.git
cd Claude-Night-Pilot

# 2. 安裝依賴
npm install

# 3. 編譯應用程式
npm run tauri build

# 4. 安裝編譯結果
# 編譯完成的檔案位於 src-tauri/target/release/bundle/
```

## ⚙️ 初始設定

### 1. 驗證 Claude CLI
首先確認 Claude CLI 已正確安裝：

```bash
# 檢查 Claude CLI 版本
claude --version
# 應該顯示 v1.0.58 或更新版本

# 檢查 Claude CLI 狀態
claude auth status
# 應該顯示已認證狀態
```

### 2. 啟動 Claude Night Pilot

#### GUI 模式 (圖形介面)
- **macOS**: 從啟動台或應用程式資料夾啟動
- **Windows**: 從開始選單或桌面捷徑啟動
- **Linux**: 從應用程式選單啟動

#### CLI 模式 (命令行)
```bash
# 檢查系統狀態
cnp status

# 查看可用命令
cnp --help

# 檢查 Claude CLI 冷卻狀態
cnp cooldown
```

### 3. 首次執行檢查

啟動後，系統會自動執行以下檢查：

1. ✅ **Claude CLI 連線** - 驗證能否正常呼叫 Claude
2. ✅ **資料庫初始化** - 建立 SQLite 資料庫和必要表格
3. ✅ **ccusage 工具** - 檢查使用量追蹤工具可用性
4. ✅ **權限設定** - 確認安全執行環境

如果所有檢查都通過，您會看到綠色的 ✅ 狀態指示。

## 🎯 基本使用

### GUI 模式基本操作

#### 1. 監控使用量
- 點擊「**使用量**」標籤頁
- 查看當前 Claude 使用狀況
- 設定使用量警告閾值

#### 2. 執行 Claude 指令
- 點擊「**執行**」標籤頁
- 輸入您的提示內容
- 選擇安全等級
- 點擊「執行」按鈕

#### 3. 查看執行結果
- 點擊「**結果**」標籤頁
- 瀏覽歷史執行記錄
- 檢視詳細的執行資訊

#### 4. 設定自動化排程
- 點擊「**排程**」標籤頁
- 建立新的排程任務
- 設定執行時間和重複規則

### CLI 模式基本操作

#### 1. 查看系統狀態
```bash
cnp status
# 顯示：資料庫連線、Claude CLI 狀態、使用量資訊
```

#### 2. 執行 Claude 指令
```bash
# 基本執行
cnp run --prompt "解釋量子電腦的原理"

# 安全模式執行
cnp run --prompt "列出目前目錄檔案" --mode safe

# 危險模式執行 (需要明確授權)
cnp run --prompt "系統健康檢查" --dangerously-skip-permissions
```

#### 3. 查看執行結果
```bash
# 列出最近的執行結果
cnp results

# 查看特定結果詳情
cnp results --id 123
```

#### 4. 管理排程任務
```bash
# 列出所有排程任務
cnp job list

# 新增排程任務
cnp job add --name "每日報告" --prompt "生成今日工作總結" --schedule "0 18 * * *"
```

## 🔒 安全使用指南

### 安全等級說明

| 等級 | 說明 | 使用場景 |
|------|------|----------|
| **Safe** | 預設安全模式 | 一般文本處理、分析任務 |
| **Monitored** | 監控模式 | 需要系統資訊但無風險的任務 |
| **Dangerous** | 危險模式 | 需要系統管理權限的任務 |

### 最佳實踐建議

1. **從安全模式開始**: 新手用戶建議先使用 Safe 模式熟悉系統
2. **檢查執行結果**: 執行後總是檢查結果和系統日誌
3. **定期備份**: 重要的 prompt 和設定要備份
4. **監控使用量**: 定期檢查 Claude 使用量避免超支
5. **更新軟體**: 保持軟體為最新版本獲得安全修復

## 🆘 常見問題排解

### Q: 顯示「Claude CLI 未找到」錯誤
**A**: 請確認：
1. Claude CLI 已正確安裝 (`claude --version`)
2. Claude CLI 在系統 PATH 中
3. 已完成 Claude CLI 認證 (`claude auth login`)

### Q: 無法連接到資料庫
**A**: 請嘗試：
1. 重新啟動應用程式
2. 檢查應用程式是否有寫入權限
3. 清除應用程式資料重新初始化

### Q: ccusage 命令失效
**A**: 系統會自動嘗試：
1. 直接呼叫 `ccusage`
2. 透過 `npx ccusage` 執行
3. 透過 `bunx ccusage` 執行
4. 如果都失效，會使用時間戳回退機制

### Q: GUI 畫面顯示異常
**A**: 請嘗試：
1. 重新整理頁面 (Ctrl+R / Cmd+R)
2. 重新啟動應用程式
3. 清除應用程式快取

## 📚 下一步

完成快速開始後，您可以：

1. **深入學習**: 閱讀 [完整使用手冊](user-manual.md)
2. **進階設定**: 查看 [部署指南](deployment-guide.md)
3. **開發擴展**: 參考 [開發者文檔](../developer/)
4. **獲得支援**: 前往 [GitHub Issues](https://github.com/s123104/Claude-Night-Pilot/issues)

## 🎉 恭喜！

您已成功設定 Claude Night Pilot！現在您可以享受智能化的 Claude CLI 管理體驗。

---

**需要幫助？** 請訪問我們的 [GitHub 倉庫](https://github.com/s123104/Claude-Night-Pilot) 或查看 [詳細文檔](../README.md)。 