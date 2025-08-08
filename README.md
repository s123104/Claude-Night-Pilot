# Claude Night Pilot

> 自動化 Claude Code 管理工具 - 本地運行、隱私安全、簡單易用

<p align="center">
  <a href="#installation"><img alt="Version" src="https://img.shields.io/badge/version-0.1.0-blue.svg" /></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green.svg" /></a>
  <a href="#"><img alt="Status" src="https://img.shields.io/badge/status-production%20ready-brightgreen.svg" /></a>
</p>

## 概覽

Claude Night Pilot 為 Claude Code 用戶提供自動化管理功能，支持排程執行、使用監控、冷卻檢測等核心功能。

**核心特色**：
- **本地運行** - 零雲端依賴，保護隱私
- **輕量化** - 單一執行檔 < 2MB  
- **雙模式** - GUI + CLI 靈活使用
- **生產就緒** - 企業級安全標準

## 安裝

確保您已安裝並配置好 Claude Code，然後：

```bash
# 下載最新發布版本
curl -L https://github.com/s123104/claude-night-pilot/releases/latest/download/cnp-unified -o cnp
chmod +x cnp

# 或使用 Rust 從源碼建置
cargo install --path .
```

## 使用

### GUI 模式
```bash
npm run tauri dev
```

### CLI 模式
```bash
# 立即執行 prompt
./cnp execute -p "幫我檢查這個檔案的程式碼品質"

# 排程執行
./cnp execute -p "早安報告" --mode scheduled --cron "0 9 * * *"

# 檢查 Claude API 使用狀況
./cnp cooldown

# 系統健康檢查  
./cnp health
```

## 主要功能

- Prompt 模板管理與分類
- 智能排程執行 (Cron 支援)
- Claude API 冷卻監控
- 使用量追蹤與分析
- 安全執行與審計

## 文檔

完整文檔請參閱 [docs/](docs/) 目錄：

- **[快速開始](docs/user-guide/quick-start.md)** - 新手指南
- **[CLAUDE.md](CLAUDE.md)** - 開發者完整指南
- **[部署指南](docs/user-guide/DEPLOYMENT_GUIDE.md)** - 生產環境部署

## 開發

### 前置需求
- Node.js 18+
- Rust 1.76+
- Claude CLI 1.0.58+

### 開發命令
```bash
# 開發服務
npm run tauri dev      # 啟動 GUI 開發模式
npm run cli:build      # 建置 CLI 工具
npm test               # 運行測試套件
npm run lint:check     # 程式碼檢查 (提交前必須執行)
```

## 支持

如有問題或建議，請開啟 [GitHub Issue](https://github.com/s123104/claude-night-pilot/issues)。

## 貢獻

歡迎提交 Pull Request！請先閱讀 [CONTRIBUTING.md](CONTRIBUTING.md) 了解開發規範。

## 授權

此專案採用 MIT 授權條款。
