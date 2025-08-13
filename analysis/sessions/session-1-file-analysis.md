# Session 1: 檔案分析與清理

## 🎯 任務目標

檢測 Claude Night Pilot 專案中的過時檔案、無引用檔案和重複代碼，提供清理建議和自動化清理方案。

## 📋 具體任務

### 1. 過時檔案檢測

- 分析檔案最後修改時間
- 檢查檔案是否被其他檔案引用
- 識別已棄用的配置檔案
- 找出測試檔案對應的實際代碼是否存在

### 2. 無引用檔案分析

- 掃描所有 JavaScript/TypeScript 檔案的 import/require
- 分析 Rust 檔案的 use 和 mod 聲明
- 檢查 HTML 檔案的資源引用
- 識別孤立的資源檔案

### 3. 重複代碼檢測

- 分析相似的函數和模組
- 檢測重複的配置和常數
- 找出可以合併的工具腳本

### 4. 目錄結構分析

- 評估當前目錄結構的合理性
- 建議更好的檔案組織方式
- 識別可以簡化的目錄層級

## 🔧 分析工具

### 檔案依賴分析

```bash
# JavaScript/TypeScript 依賴分析
npx madge --image deps.svg src/
npx dependency-cruiser --output-type dot src/ | dot -T svg > deps.svg

# 檔案引用檢查
find . -name "*.js" -o -name "*.ts" -o -name "*.rs" | xargs grep -l "import\|require\|use\|mod"
```

### 過時檔案檢測

```bash
# 找出超過 30 天未修改的檔案
find . -type f -mtime +30 -not -path "./node_modules/*" -not -path "./target/*"

# 檢查 Git 歷史中長期未變更的檔案
git log --name-only --since="3 months ago" | sort | uniq -c | sort -n
```

### 重複代碼檢測

```bash
# 使用 jscpd 檢測 JavaScript 重複代碼
npx jscpd src/

# 使用自定義腳本檢測 Rust 重複代碼
```

## 📊 輸出格式

### 分析報告結構

```json
{
  "analysis_timestamp": "2025-08-14T03:00:00Z",
  "obsolete_files": [
    {
      "path": "path/to/file",
      "last_modified": "2024-01-01",
      "reason": "No references found",
      "confidence": 0.95
    }
  ],
  "unreferenced_files": [
    {
      "path": "path/to/file",
      "type": "javascript",
      "size": 1024,
      "potential_impact": "low"
    }
  ],
  "duplicate_code": [
    {
      "files": ["file1.js", "file2.js"],
      "similarity": 0.85,
      "lines": 50,
      "suggestion": "Extract to common module"
    }
  ],
  "directory_suggestions": [
    {
      "current": "src/utils/helpers/",
      "suggested": "src/lib/",
      "reason": "Simplify nested structure"
    }
  ]
}
```

## 🚀 執行步驟

1. **初始掃描**: 建立專案檔案清單
2. **依賴分析**: 分析所有檔案間的依賴關係
3. **引用檢查**: 檢查每個檔案是否被引用
4. **時間分析**: 分析檔案修改時間和 Git 歷史
5. **重複檢測**: 檢測重複代碼和配置
6. **報告生成**: 生成詳細的分析報告
7. **清理建議**: 提供自動化清理腳本

## 📝 預期成果

- **檔案清理清單**: 可安全刪除的檔案列表
- **重構建議**: 目錄結構優化建議
- **自動化腳本**: 一鍵清理腳本
- **風險評估**: 每個清理操作的風險等級
