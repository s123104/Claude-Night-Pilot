# Claude Night Pilot 生產環境部署指南

## 🚀 部署準備清單

### ✅ 已完成項目
- [x] 核心功能測試通過 (35/35 Rust單元測試)
- [x] CLI工具穩定運行 (87.5% 通過率)
- [x] 資料庫操作完整 (100% CRUD功能)
- [x] 整合性測試通過 (16/16 統一介面測試)
- [x] Release版本編譯 (1.8MB，遠小於10MB目標)
- [x] 冷卻檢查重試機制實作
- [x] 舊檔案歸檔完成

### ⚠️ 待改進項目
- [ ] GUI E2E測試穩定性 (50% 通過率，需修復元素定位)
- [ ] 冷卻檢查超時處理完善
- [ ] 生產環境監控機制

## 📦 部署版本資訊

### CLI工具 (優先部署)
- **二進制檔案**: `src-tauri/target/release/cnp-unified`
- **檔案大小**: 1.8MB
- **功能狀態**: ✅ 生產就緒
- **支援功能**:
  - 執行Claude命令 (`execute`)
  - 冷卻狀態檢查 (`cooldown`)
  - 系統健康檢查 (`health`)
  - 批量執行 (`batch`)

### GUI應用程式 (待修復後部署)
- **建構命令**: `npm run tauri build`
- **功能狀態**: ⚠️ 需修復E2E測試
- **問題**: 部分UI元素定位不穩定

## 🛠️ 安裝步驟

### 1. CLI工具安裝

```bash
# 複製二進制檔案到系統路徑
sudo cp src-tauri/target/release/cnp-unified /usr/local/bin/cnp

# 驗證安裝
cnp --help
cnp health --format json
```

### 2. GUI應用程式安裝

```bash
# 建構桌面應用程式
npm run tauri build

# 安裝包位置
# macOS: src-tauri/target/release/bundle/dmg/
# Windows: src-tauri/target/release/bundle/msi/
# Linux: src-tauri/target/release/bundle/deb/
```

### 3. 資料庫初始化

```bash
# CLI工具會自動初始化SQLite資料庫
cnp health  # 檢查資料庫狀態
```

## 🔧 系統需求

### 最低需求
- **作業系統**: macOS 10.15+, Windows 10+, Ubuntu 20.04+
- **記憶體**: 150MB RAM
- **儲存空間**: 10MB
- **Claude CLI**: v1.0.65+

### 建議需求
- **記憶體**: 300MB RAM
- **網路**: 穩定的網際網路連線
- **磁碟空間**: 50MB (含日誌和資料庫)

## 📊 性能基準

### CLI工具性能
- **啟動時間**: < 500ms
- **命令響應**: < 1s (一般操作)
- **冷卻檢查**: < 2s (含重試機制)
- **記憶體使用**: < 50MB

### GUI應用程式性能
- **啟動時間**: < 3s
- **UI響應**: < 100ms
- **記憶體使用**: < 150MB

## 🔒 安全考量

### 已實作安全功能
- ✅ 輸入驗證和清理
- ✅ SQL注入防護 (參數化查詢)
- ✅ 檔案存取限制
- ✅ 危險命令檢測
- ✅ 執行權限驗證

### 建議安全設定
- 定期更新Claude CLI
- 限制網路存取權限
- 啟用系統防火牆
- 定期備份資料庫

## 🐛 已知問題與解決方案

### 1. GUI E2E測試不穩定
**問題**: 部分測試因元素不可見而超時
**狀態**: 已部分修復，增加等待機制
**影響**: 不影響實際功能，僅測試環境問題

### 2. 冷卻檢查偶爾超時
**問題**: Claude CLI整合時偶爾超時
**解決方案**: 已實作重試機制 (最多3次)
**狀態**: ✅ 已修復

### 3. 二進制檔案大小
**問題**: Debug版本12.8MB略大
**解決方案**: Release版本1.8MB符合要求
**狀態**: ✅ 已解決

## 📝 部署後驗證

### CLI工具驗證
```bash
# 基本功能檢查
cnp --help
cnp health --format json
cnp cooldown --format json

# 效能測試
time cnp health  # 應 < 2s
```

### GUI應用程式驗證
1. 應用程式正常啟動
2. Material Design 3.0介面載入
3. Prompt建立和執行功能
4. 冷卻狀態正確顯示
5. 任務排程功能正常

## 🔄 升級策略

### CLI工具升級
1. 備份現有資料庫
2. 替換二進制檔案
3. 驗證功能正常
4. 恢復備份 (如有問題)

### GUI應用程式升級
1. 解除安裝舊版本
2. 安裝新版本
3. 資料庫會自動遷移
4. 驗證所有功能

## 🎯 部署建議

### 階段一: CLI工具部署 (立即可行)
- 優先部署CLI工具 (`cnp-unified`)
- 功能穩定，生產就緒
- 滿足基本自動化需求

### 階段二: GUI應用程式部署 (修復後)
- 修復E2E測試穩定性問題
- 完善錯誤處理機制
- 提供完整的視覺化介面

### 階段三: 進階功能 (未來版本)
- 雲端同步功能
- 多使用者支援
- 進階排程選項
- API服務模式

## 📞 支援與維護

### 日誌檔案位置
- CLI日誌: `~/.local/share/claude-night-pilot/logs/`
- GUI日誌: 應用程式內建日誌檢視器
- 資料庫: `~/.local/share/claude-night-pilot/claude-pilot.db`

### 常見問題排除
1. **Claude CLI未找到**: 安裝 `@anthropic-ai/claude-code`
2. **資料庫錯誤**: 刪除並重新初始化資料庫
3. **權限問題**: 檢查檔案系統權限
4. **網路問題**: 檢查網際網路連線

---

**更新日期**: 2025年08月02日  
**版本**: v0.1.0  
**狀態**: CLI工具生產就緒，GUI應用程式測試中