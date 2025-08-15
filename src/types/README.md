# TypeScript Types

此目錄包含從 Rust 後端自動生成的 TypeScript 類型定義。

## 自動生成

這些類型使用 `ts-rs` 從 Rust 結構體自動生成：

- `ApiResponse.ts` - 統一 API 響應格式
- `PaginatedResponse.ts` - 分頁響應格式
- `ClaudeRequest.ts` - Claude 請求模型
- `ClaudeResponse.ts` - Claude 響應模型
- `ExecutionResult.ts` - 執行結果模型
- `Job.ts` - 工作任務模型
- `Prompt.ts` - 提示模型

## 使用方式

```typescript
import { ApiResponse, ClaudeRequest } from './types';

// 類型安全的 API 調用
const response: ApiResponse<ClaudeRequest[]> = await invoke('get_prompts');
```

## 更新類型

運行以下命令更新類型定義：

```bash
cd src-tauri && cargo check
```

## 注意事項

- 🚨 **不要手動編輯這些文件** - 它們會被自動覆蓋
- 類型定義與 Rust 後端保持 100% 同步
- 支援泛型和複雜類型結構
- 包含完整的 TypeScript 類型註解