# 🔧 API 參考文檔

> [API 名稱] 完整程式介面參考

## 📋 目錄

- [認證](#-認證)
- [基礎 API](#-基礎-api)
- [核心功能 API](#-核心功能-api)
- [錯誤處理](#-錯誤處理)
- [使用限制](#-使用限制)
- [SDK 和範例](#-sdk-和範例)

## 🔐 認證

### 認證方式
```http
Authorization: Bearer [API_TOKEN]
Content-Type: application/json
```

### 取得 API Token
```bash
# CLI 命令
[cli_command] auth login

# 或透過環境變數
export API_TOKEN="your_token_here"
```

## 🏗️ 基礎 API

### 健康檢查
檢查 API 服務狀態

```http
GET /api/v1/health
```

**回應範例**:
```json
{
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2025-08-09T10:30:00Z",
  "services": {
    "database": "healthy",
    "scheduler": "healthy"
  }
}
```

### 系統資訊
取得系統配置和統計資訊

```http
GET /api/v1/system/info
```

**回應範例**:
```json
{
  "system": {
    "os": "linux",
    "version": "1.0.0",
    "uptime": 3600
  },
  "statistics": {
    "total_executions": 150,
    "active_jobs": 3,
    "last_execution": "2025-08-09T10:25:00Z"
  }
}
```

## 🎯 核心功能 API

### Prompt 管理

#### 列出所有 Prompts
```http
GET /api/v1/prompts
```

**查詢參數**:
| 參數 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `limit` | integer | ❌ | 每頁數量 (預設: 50) |
| `offset` | integer | ❌ | 偏移量 (預設: 0) |
| `search` | string | ❌ | 搜尋關鍵字 |
| `tag` | string | ❌ | 標籤篩選 |

**回應範例**:
```json
{
  "prompts": [
    {
      "id": 1,
      "name": "Daily Report",
      "content": "Generate daily status report",
      "tags": ["automation", "daily"],
      "created_at": "2025-08-01T09:00:00Z",
      "updated_at": "2025-08-01T09:00:00Z"
    }
  ],
  "total": 25,
  "limit": 50,
  "offset": 0
}
```

#### 創建新 Prompt
```http
POST /api/v1/prompts
```

**請求主體**:
```json
{
  "name": "My Custom Prompt",
  "content": "Detailed prompt content with @file.md references",
  "description": "Optional description",
  "tags": ["custom", "automation"],
  "is_active": true
}
```

**回應範例**:
```json
{
  "id": 26,
  "name": "My Custom Prompt",
  "content": "Detailed prompt content with @file.md references",
  "description": "Optional description",
  "tags": ["custom", "automation"],
  "is_active": true,
  "created_at": "2025-08-09T10:30:00Z",
  "updated_at": "2025-08-09T10:30:00Z"
}
```

#### 取得特定 Prompt
```http
GET /api/v1/prompts/{id}
```

#### 更新 Prompt
```http
PUT /api/v1/prompts/{id}
PATCH /api/v1/prompts/{id}
```

#### 刪除 Prompt
```http
DELETE /api/v1/prompts/{id}
```

### 任務排程

#### 列出排程任務
```http
GET /api/v1/jobs
```

**查詢參數**:
| 參數 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `status` | string | ❌ | 狀態篩選: `pending`, `running`, `completed`, `failed` |
| `prompt_id` | integer | ❌ | 依 Prompt ID 篩選 |

#### 創建排程任務
```http
POST /api/v1/jobs
```

**請求主體**:
```json
{
  "prompt_id": 1,
  "cron_expression": "0 9 * * *",
  "name": "Daily Automation",
  "description": "Run daily report generation",
  "execution_options": {
    "timeout_seconds": 300,
    "max_retries": 3,
    "output_format": "json"
  },
  "is_active": true
}
```

#### 執行任務
```http
POST /api/v1/jobs/{id}/execute
```

#### 取消任務
```http
POST /api/v1/jobs/{id}/cancel
```

### 執行結果

#### 列出執行結果
```http
GET /api/v1/results
```

#### 取得特定執行結果
```http
GET /api/v1/results/{id}
```

**回應範例**:
```json
{
  "id": 150,
  "job_id": 5,
  "status": "completed",
  "output": "Task completed successfully",
  "execution_time": 45.5,
  "token_usage": {
    "input_tokens": 1200,
    "output_tokens": 800,
    "total_cost": 0.0234
  },
  "started_at": "2025-08-09T10:00:00Z",
  "completed_at": "2025-08-09T10:00:45Z"
}
```

### 使用監控

#### 取得使用統計
```http
GET /api/v1/usage/stats
```

**查詢參數**:
| 參數 | 類型 | 必填 | 說明 |
|------|------|------|------|
| `start_date` | string | ❌ | 開始日期 (ISO 8601) |
| `end_date` | string | ❌ | 結束日期 (ISO 8601) |
| `granularity` | string | ❌ | 粒度: `day`, `week`, `month` |

**回應範例**:
```json
{
  "period": {
    "start": "2025-08-01T00:00:00Z",
    "end": "2025-08-09T23:59:59Z"
  },
  "summary": {
    "total_executions": 150,
    "successful_executions": 142,
    "failed_executions": 8,
    "total_tokens": 125000,
    "total_cost": 12.45
  },
  "daily_breakdown": [
    {
      "date": "2025-08-01",
      "executions": 18,
      "tokens": 15000,
      "cost": 1.50
    }
  ]
}
```

## ❌ 錯誤處理

### 標準錯誤格式
所有 API 錯誤遵循統一格式：

```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "The requested resource was not found",
    "details": {
      "resource": "prompt",
      "id": 999
    },
    "timestamp": "2025-08-09T10:30:00Z"
  }
}
```

### 常見錯誤碼
| HTTP 狀態碼 | 錯誤碼 | 說明 |
|-------------|--------|------|
| 400 | `INVALID_REQUEST` | 請求格式錯誤 |
| 401 | `UNAUTHORIZED` | 未提供有效認證 |
| 403 | `FORBIDDEN` | 權限不足 |
| 404 | `RESOURCE_NOT_FOUND` | 資源不存在 |
| 429 | `RATE_LIMIT_EXCEEDED` | 超過使用限制 |
| 500 | `INTERNAL_ERROR` | 伺服器內部錯誤 |

### 錯誤處理建議
```javascript
async function callAPI(endpoint, options) {
  try {
    const response = await fetch(endpoint, options);
    
    if (!response.ok) {
      const error = await response.json();
      throw new APIError(error.error.code, error.error.message);
    }
    
    return await response.json();
  } catch (error) {
    console.error('API Call failed:', error);
    throw error;
  }
}
```

## 📊 使用限制

### 速率限制
| 端點類型 | 限制 | 時間窗口 |
|----------|------|----------|
| 讀取操作 | 100 次 | 每分鐘 |
| 寫入操作 | 30 次 | 每分鐘 |
| 執行操作 | 10 次 | 每分鐘 |

### 資料限制
- **Prompt 內容**: 最大 50KB
- **批次操作**: 最多 100 項
- **檔案上傳**: 最大 10MB
- **查詢結果**: 最多 1000 筆

## 🛠️ SDK 和範例

### JavaScript SDK
```bash
npm install @claude-night-pilot/sdk
```

```javascript
import { CNPClient } from '@claude-night-pilot/sdk';

const client = new CNPClient({
  apiToken: 'your_token_here',
  baseURL: 'http://localhost:3000/api/v1'
});

// 創建 Prompt
const prompt = await client.prompts.create({
  name: 'Test Prompt',
  content: 'Hello world'
});

// 執行任務
const result = await client.jobs.execute(prompt.id);
```

### Python SDK
```bash
pip install claude-night-pilot-sdk
```

```python
from claude_night_pilot import CNPClient

client = CNPClient(
    api_token='your_token_here',
    base_url='http://localhost:3000/api/v1'
)

# 列出 Prompts
prompts = client.prompts.list(limit=10)

# 創建排程任務
job = client.jobs.create(
    prompt_id=1,
    cron_expression='0 9 * * *',
    name='Daily Task'
)
```

### cURL 範例
```bash
# 健康檢查
curl -X GET http://localhost:3000/api/v1/health

# 創建 Prompt
curl -X POST http://localhost:3000/api/v1/prompts \
  -H "Authorization: Bearer your_token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Prompt",
    "content": "Generate status report"
  }'

# 列出任務
curl -X GET "http://localhost:3000/api/v1/jobs?status=running" \
  -H "Authorization: Bearer your_token"
```

## 🔄 版本管理

### 版本策略
- API 版本採用 URL 路徑版本控制 (`/api/v1/`, `/api/v2/`)
- 向後相容性保證至少維持 2 個主版本
- 重大變更會提前 3 個月通知

### 版本歷史
- **v1.0** (2025-08): 初始發布版本
- **v1.1** (計劃中): 增強監控功能
- **v2.0** (計劃中): GraphQL 支援

---

**API 版本**: v1.0 • **最後更新**: 2025-08 • **維護狀態**: 積極維護

<!-- 
使用說明:
1. 根據實際 API 功能調整端點和參數
2. 更新範例程式碼以符合實際使用情況  
3. 確保錯誤碼和狀態碼正確
4. 測試所有 API 範例的正確性
5. 保持版本資訊同步更新
-->