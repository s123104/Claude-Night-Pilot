# Claude Night Pilot - 部署指南 🚀

> **目標**: 將應用程式從開發環境順利部署到生產環境

## 🎯 部署策略

### 部署階段

1. **開發環境** (Development) - 本機開發與測試
2. **測試環境** (Staging) - 預生產驗證
3. **生產環境** (Production) - 正式發布

### 發布類型

- **Alpha**: 內部測試版本
- **Beta**: 公開測試版本
- **Release**: 正式發布版本
- **Hotfix**: 緊急修復版本

## 🛠️ 建置流程

### 開發建置

```bash
# 開發模式（熱重載）
npm run tauri dev

# 檢查程式碼
npm run lint
npm test

# 本機建置測試
npm run tauri build -- --debug
```

### 生產建置

```bash
# 清理環境
rm -rf dist/
rm -rf src-tauri/target/release/

# 安裝依賴
npm ci

# 執行測試
npm test

# 生產建置
npm run tauri build

# 驗證建置結果
ls -la src-tauri/target/release/bundle/
```

## 📦 打包配置

### Tauri 建置目標

```json
// tauri.conf.json
{
  "bundle": {
    "active": true,
    "targets": ["app", "dmg", "deb", "msi"],
    "identifier": "com.claude-night-pilot.app",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "shortDescription": "Claude CLI 自動化工具",
    "longDescription": "現代 Claude Code 用戶的夜間自動打工仔"
  }
}
```

### 平台特定配置

```bash
# macOS (Intel + Apple Silicon)
npm run tauri build -- --target universal-apple-darwin

# Windows (x64)
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux (x64)
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## 🔧 環境配置

### 環境變數

```bash
# 生產環境變數
export NODE_ENV=production
export RUST_LOG=info
export CLAUDE_PILOT_MODE=production

# 建置配置
export TAURI_PRIVATE_KEY=/path/to/private.key
export TAURI_KEY_PASSWORD=your-key-password
```

### 配置檔案

```toml
# src-tauri/Cargo.toml
[profile.release]
opt-level = "s"          # 最佳化大小
lto = true              # 連結時最佳化
codegen-units = 1       # 減少程式碼大小
panic = "abort"         # 減少二進位大小
strip = true           # 移除除錯符號
```

## 🚀 自動化部署

### GitHub Actions

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ["v*"]

jobs:
  release:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-20.04, windows-latest]

    runs-on: ${{ matrix.platform }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install dependencies (ubuntu only)
        if: matrix.platform == 'ubuntu-20.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev

      - name: Rust setup
        uses: dtolnay/rust-toolchain@stable

      - name: Node.js setup
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Install frontend dependencies
        run: npm ci

      - name: Build the app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "Claude Night Pilot ${{ github.ref_name }}"
          releaseBody: "See the assets to download and install."
          releaseDraft: true
          prerelease: false
```

### 手動發布

```bash
# 建立發布標籤
git tag v1.0.0
git push origin v1.0.0

# 建置所有平台
npm run build:all

# 上傳到 GitHub Releases
gh release create v1.0.0 \
  --title "Claude Night Pilot v1.0.0" \
  --notes-file CHANGELOG.md \
  dist/*
```

## 📋 發布檢查清單

### 發布前檢查

- [ ] **程式碼品質**

  - [ ] 所有測試通過
  - [ ] 程式碼審查完成
  - [ ] 安全掃描通過
  - [ ] 效能測試達標

- [ ] **文檔更新**

  - [ ] README.md 更新
  - [ ] CHANGELOG.md 完整
  - [ ] API 文檔最新
  - [ ] 使用指南更新

- [ ] **版本管理**
  - [ ] 版本號正確遞增
  - [ ] Git 標籤建立
  - [ ] 發布說明準備
  - [ ] 相依性檢查

### 建置驗證

- [ ] **多平台建置**

  - [ ] macOS 建置成功
  - [ ] Windows 建置成功
  - [ ] Linux 建置成功
  - [ ] 檔案大小合理

- [ ] **功能驗證**
  - [ ] 應用程式啟動正常
  - [ ] 核心功能運作
  - [ ] CLI 工具可用
  - [ ] 資料庫遷移正確

### 發布後驗證

- [ ] **下載測試**

  - [ ] 下載連結有效
  - [ ] 安裝流程順暢
  - [ ] 首次啟動正常
  - [ ] 升級流程正確

- [ ] **使用者回饋**
  - [ ] 監控錯誤報告
  - [ ] 收集使用者回饋
  - [ ] 追蹤效能指標
  - [ ] 準備下一版本

## 🔒 安全考量

### 程式碼簽名

```bash
# macOS 程式碼簽名
codesign --sign "Developer ID Application: Your Name" \
  --options runtime \
  --entitlements entitlements.plist \
  your-app.app

# Windows 程式碼簽名
signtool sign /f certificate.p12 /p password your-app.exe
```

### 更新機制

```json
// tauri.conf.json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://your-domain.com/updater/{{target}}/{{current_version}}"
    ],
    "dialog": true,
    "pubkey": "your-public-key"
  }
}
```

## 📊 監控與分析

### 應用程式遙測

```rust
// src-tauri/src/lib.rs
use tauri_plugin_aptabase::EventTracker;

#[tauri::command]
async fn track_event(event: String, properties: Value) {
    app.track_event(&event, Some(properties)).await;
}
```

### 錯誤追蹤

```javascript
// src/main.js
window.addEventListener("error", (error) => {
  console.error("Application error:", error);
  // 發送錯誤報告到監控服務
});
```

## 🚨 緊急處理

### 回滾程序

```bash
# 緊急回滾到上一版本
git revert HEAD
git tag v1.0.1-hotfix
npm run tauri build
gh release create v1.0.1-hotfix --prerelease
```

### 熱修復流程

1. **識別問題**: 確認問題範圍與影響
2. **快速修復**: 建立最小修復方案
3. **緊急測試**: 執行核心功能測試
4. **快速部署**: 跳過常規流程，直接發布
5. **後續跟進**: 監控修復效果，準備正式版本

## 📈 效能優化

### 建置最佳化

```toml
# Cargo.toml
[profile.release-lto]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"
```

### 安裝包優化

- 移除未使用的依賴
- 壓縮靜態資源
- 優化圖示檔案
- 清理除錯資訊

## 🌍 國際化準備

### 多語言支援

```json
// locales/zh-TW.json
{
  "app": {
    "title": "Claude Night Pilot",
    "description": "夜間自動打工仔"
  }
}
```

### 地區特定配置

- 時區設定
- 貨幣格式
- 日期格式
- 預設語言

---

**記住**: 成功的部署不只是技術問題，更是用戶體驗的延續！🌟
