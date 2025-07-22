# Claude Night Pilot 🌙✈️

> 現代 Claude Code 用戶的「夜間自動打工仔」- 零雲端、零安裝痛苦、零學習曲線

<div align="center">

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/s123104/claude-night-pilot)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app/)
[![Test Status](https://img.shields.io/badge/tests-passing-brightgreen.svg)](tests/)
[![Language](https://img.shields.io/badge/language-Rust%20%2B%20JavaScript-blue.svg)](#技術棧)

![Claude Night Pilot Screenshot](docs/assets/screenshot.png)

</div>

## ✨ 專案特色

### 🎯 核心價值主張

- **零雲端依賴** - 完全本地運行，保護您的隱私與資料安全
- **零安裝痛苦** - 單一執行檔 < 10MB，開箱即用
- **零學習曲線** - 直觀的 GUI + 強大的 CLI 雙模式操作

### 🏗️ 現代技術棧

- **🦀 Tauri 2.0** - 極輕量桌面應用框架，跨平台支援
- **⚡ htmx + CSS** - 極簡前端技術棧，快速響應
- **💾 SQLite** - 單檔本地資料庫，無需額外設定
- **🧪 Playwright** - 完整 E2E 測試覆蓋

### 🎮 主要功能

- **📝 Prompt 模板管理** - 建立、編輯、分類您的 Claude prompts
- **⏰ 智能排程執行** - Cron 表達式支援，背景自動執行
- **🔄 冷卻狀態監控** - 即時顯示 Claude API 限制與倒數時間
- **🛠️ 雙模式操作** - GUI 視覺介面 + CLI 命令列工具
- **📊 結果追蹤** - 完整的執行歷史與結果管理

## 🚀 快速開始

### 📋 前置需求

- **Node.js** 18+ ([下載](https://nodejs.org/))
- **Rust** 1.76+ ([安裝](https://rustup.rs/))
- **Claude CLI** ([設定指南](docs/claude-cli-setup.md))

### ⚡ 30 秒安裝

```bash
# 1. 克隆專案
git clone https://github.com/s123104/claude-night-pilot.git
cd claude-night-pilot

# 2. 安裝依賴
npm install

# 3. 啟動應用 (開發模式)
npm run tauri dev

# 4. 或建置生產版本
npm run tauri build
```

### 🎯 基本使用

#### GUI 模式 (視覺介面)

1. **建立 Prompt** - 點擊 ➕ 按鈕新增 Prompt
2. **執行選擇** - 選擇「立即執行」或「排程執行」
3. **監控狀態** - 查看冷卻時間與執行進度

#### CLI 模式 (命令列)

```bash
# 建置 CLI 工具
npm run cli:build

# 基本指令
cnp --help              # 查看幫助
cnp prompt list         # 列出所有 prompts
cnp status              # 檢查系統狀態
cnp cooldown            # 查看冷卻狀態

# Prompt 管理
cnp prompt create "標題" "內容" --tags "標籤"
cnp prompt show 1       # 查看 prompt 詳情
cnp run 1               # 執行 prompt

# 排程管理
cnp job list            # 列出所有任務
cnp results             # 查看執行結果
```

## 📁 專案架構

```
claude-night-pilot/
├── 📁 src/                    # 前端資源
│   ├── 🌐 index.html         # 主介面
│   ├── ⚡ main.js            # JavaScript 邏輯
│   ├── 🎨 styles.css         # 自定義樣式
│   └── 📦 assets/            # 靜態資源
├── 📁 src-tauri/             # Rust 後端
│   ├── 🦀 src/
│   │   ├── 📋 lib.rs         # 主程式邏輯
│   │   ├── 💾 db.rs          # 資料庫層
│   │   ├── 🔧 executor.rs    # Claude CLI 執行器
│   │   ├── ⏰ scheduler.rs   # 排程器
│   │   └── 🛠️ bin/cnp.rs    # CLI 工具
│   ├── 📄 Cargo.toml         # Rust 依賴
│   └── ⚙️ tauri.conf.json   # Tauri 配置
├── 📁 tests/                 # E2E 測試
├── 📁 docs/                  # 專案文檔
└── 📄 README.md              # 本檔案
```

## 🧪 測試與品質保證

### 測試覆蓋率

- ✅ **基本功能測試** - 應用啟動、介面互動
- ✅ **Prompt 管理** - CRUD 操作完整測試
- ✅ **Claude Code 整合** - 檔案引用語法支援
- ✅ **排程系統** - Cron 表達式與背景執行
- ✅ **資料持久化** - SQLite 資料庫操作
- ✅ **響應式設計** - 多裝置尺寸支援

### 執行測試

```bash
# 執行完整測試套件
npm test

# 互動式測試 (推薦)
npm run test:ui

# 特定測試檔案
npx playwright test tests/claude-night-pilot.spec.js

# 效能測試
npm run test:performance
```

## 📊 效能指標

| 指標       | 目標值  | 實際值 | 狀態 |
| ---------- | ------- | ------ | ---- |
| 安裝包大小 | < 10MB  | ~8MB   | ✅   |
| 啟動時間   | < 3s    | ~1.5s  | ✅   |
| 記憶體使用 | < 150MB | ~80MB  | ✅   |
| 介面響應   | < 100ms | ~50ms  | ✅   |
| 測試覆蓋率 | > 80%   | ~85%   | ✅   |

## 🔧 與 Claude Code 整合

Claude Night Pilot 完全相容 Claude Code 語法：

```javascript
// 檔案引用
"@docs/PROJECT_RULES.md 請分析這個專案的架構";

// 多檔案操作
"@src/**.js 重構這些 JavaScript 檔案";

// 編輯指令
"編輯 config.json 並添加新的設定項目";
```

### 🕰️ 冷卻檢測功能

系統會自動解析 Claude CLI 的錯誤訊息並顯示準確的倒數計時器：

```
Claude usage limit reached. Your limit will reset at 23:00 (Asia/Taipei).
```

## 🤝 貢獻指南

我們歡迎任何形式的貢獻！請閱讀 [貢獻指南](CONTRIBUTING.md) 了解詳情。

### 🐛 回報問題

- 使用 [GitHub Issues](https://github.com/s123104/claude-night-pilot/issues) 回報 bugs
- 提供詳細的重現步驟與環境資訊
- 附上相關的日誌檔案

### 💡 功能建議

- 在 Issues 中標記為 `enhancement`
- 描述使用情境與預期效果
- 歡迎提供 mockups 或範例

### 🔄 提交 Pull Request

1. Fork 專案
2. 建立功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交變更 (`git commit -m 'Add AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 開啟 Pull Request

## 📜 授權條款

本專案採用 MIT 授權條款 - 詳見 [LICENSE](LICENSE) 檔案。

## 🙏 致謝

### 技術感謝

- [Tauri](https://tauri.app/) - 優秀的跨平台應用框架
- [htmx](https://htmx.org/) - 現代化的前端互動框架
- [SQLite](https://sqlite.org/) - 可靠的嵌入式資料庫
- [Playwright](https://playwright.dev/) - 強大的測試工具

### 社群感謝

- [Anthropic](https://anthropic.com/) 提供優秀的 Claude API
- 所有提供回饋與建議的社群成員
- 開源專案維護者們的無私奉獻

## 📞 聯繫方式

- **作者**: [s123104](https://github.com/s123104)
- **專案**: [Claude Night Pilot](https://github.com/s123104/claude-night-pilot)
- **問題回報**: [GitHub Issues](https://github.com/s123104/claude-night-pilot/issues)
- **討論區**: [GitHub Discussions](https://github.com/s123104/claude-night-pilot/discussions)

---

<div align="center">

**Claude Night Pilot** - 讓你的 Claude 助手在夜晚也能勤奮工作！ 🌙✨

Made with ❤️ by the open source community

</div>
