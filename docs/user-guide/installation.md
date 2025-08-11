# 安裝指南

本文檔提供 Claude Night Pilot 的詳細安裝說明。

## 系統需求

### 基本需求
- **作業系統**: Windows 10+, macOS 10.15+, 或 Linux
- **Claude Code**: 已安裝並配置（版本 ≥1.0.58）
- **記憶體**: 最少 4GB RAM (建議 8GB+)
- **儲存空間**: 100MB 可用空間

### 開發需求（從源碼建置）
- **Node.js**: 18.0+ (GUI 開發)
- **Rust**: 1.76+ (CLI 建置)
- **Git**: 最新版本

## 安裝方式

### 方式一：預編譯版本 (推薦)

```bash
# 下載最新發布版本
curl -L https://github.com/s123104/claude-night-pilot/releases/latest/download/cnp -o cnp
chmod +x cnp

# 驗證安裝
./cnp --version
```

### 方式二：從源碼建置

```bash
# 克隆專案
git clone https://github.com/s123104/claude-night-pilot.git
cd claude-night-pilot

# 安裝依賴
npm install

# 建置 CLI 工具
npm run cli:build

# 或建置完整應用程式
npm run tauri build
```

## 初始設定

### 1. 初始化資料庫
```bash
./cnp init
```

### 2. 驗證 Claude Code 整合
```bash
./cnp health
```

### 3. 基本配置
```bash
# 設定工作目錄
./cnp config set working_directory /path/to/your/workspace

# 設定預設執行選項
./cnp config set default_timeout 300
```

## 疑難排解

### 常見問題

**Q: Claude Code 未找到錯誤**
```bash
# 檢查 Claude Code 安裝
which claude

# 更新 PATH 環境變數
export PATH="$PATH:/path/to/claude"
```

**Q: 權限拒絕錯誤**
```bash
# 賦予執行權限
chmod +x ./cnp

# 或以管理員權限執行
sudo ./cnp init
```

**Q: 資料庫初始化失敗**
```bash
# 清理並重新初始化
rm -f claude-pilot.db
./cnp init --force
```

### 效能調優

**記憶體使用優化**:
```bash
# 設定較小的快取大小
./cnp config set cache_size 50MB

# 啟用低記憶體模式
./cnp config set low_memory_mode true
```

**啟動速度優化**:
```bash
# 跳過啟動檢查
./cnp config set skip_startup_checks true
```

## 下一步

安裝完成後，請參閱：
- [快速入門指南](quick-start.md) - 5分鐘學會基本操作
- [GUI 使用指南](gui-usage.md) - 桌面應用程式操作
- [CLI 參考文檔](cli-usage.md) - 命令列完整參考

## 更新與維護

### 更新到最新版本
```bash
# 預編譯版本更新
curl -L https://github.com/s123104/claude-night-pilot/releases/latest/download/cnp -o cnp-new
mv cnp-new cnp
chmod +x cnp

# 源碼版本更新
git pull origin main
npm run cli:build
```

### 清理與重置
```bash
# 清理快取
./cnp clean

# 完全重置
./cnp reset --all
```

---

如有安裝問題，請查看 [常見問題](../faq.md) 或提交 [GitHub Issue](https://github.com/s123104/claude-night-pilot/issues)。