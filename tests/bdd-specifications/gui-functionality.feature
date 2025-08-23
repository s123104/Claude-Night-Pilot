# Claude Night Pilot GUI 功能 BDD 規範
# 基於 Material Design 3.0 的完整介面測試規範

@gui @material-design @chinese-ui
功能: Claude Night Pilot GUI 完整功能測試
  作為 Claude 用戶
  我希望能夠透過直觀的圖形介面管理我的 Claude 自動化任務
  以便提高我的工作效率和體驗

  背景:
    假設 Claude Night Pilot 應用程式已啟動
    並且 應用程式正在 localhost:8080 運行
    並且 Material Design 3.0 樣式已正確載入
    並且 中文介面已正確顯示

  @navigation @core
  場景大綱: 導航系統功能驗證
    當 我點擊 "<導航項目>" 導航按鈕
    那麼 我應該看到 "<頁面標題>" 頁面
    並且 導航按鈕應該顯示為活躍狀態
    並且 頁面內容應該正確載入

    例子:
      | 導航項目 | 頁面標題   | 預期圖標 |
      | Prompt   | Prompt 管理 | chat     |
      | 排程     | 排程任務   | schedule |
      | 結果     | 執行結果   | analytics |
      | 監控     | 系統監控   | monitoring |
      | 測試     | 核心模組測試 | science  |

  @theme @material-design
  場景: 主題切換功能
    假設 當前主題為 "auto"
    當 我點擊主題切換按鈕
    那麼 主題應該切換為 "light"
    並且 按鈕圖標應該變為 "light_mode"
    並且 應該顯示主題切換通知 "已切換至淺色主題"
    並且 頁面顏色應該更新為淺色主題

  @prompts @crud
  場景: Prompt 管理功能
    假設 我在 Prompt 管理頁面
    那麼 我應該看到 Prompt 列表
    並且 每個 Prompt 卡片應該包含:
      | 元素     | 描述                    |
      | 標題     | Prompt 名稱             |
      | 內容     | Prompt 內容預覽         |
      | 標籤     | 分類標籤               |
      | 日期     | 創建或更新日期         |
      | 執行按鈕 | play_arrow 圖標按鈕    |
      | 刪除按鈕 | delete 圖標按鈕        |

  @prompts @file-references
  場景: Prompt 中的檔案引用支援
    假設 我查看 "架構分析 Prompt"
    那麼 我應該看到檔案引用語法 "@README.md @src/"
    並且 系統應該支援以下引用格式:
      | 格式        | 描述           |
      | @file.ext   | 單一檔案       |
      | @folder/    | 整個資料夾     |
      | @**/*.ext   | 萬用字元模式   |

  @scheduler @automation
  場景: 排程任務管理
    假設 我在排程任務頁面
    那麼 我應該看到排程任務列表
    並且 每個排程項目應該顯示:
      | 屬性       | 格式                      |
      | 任務名稱   | 對應的 Prompt 標題        |
      | Cron 表達式| 標準 cron 格式           |
      | 狀態       | 運行中/已暫停/錯誤       |
      | 操作按鈕   | 刪除按鈕                 |

  @results @history
  場景: 執行結果查看
    假設 我在執行結果頁面
    那麼 我應該看到執行歷史記錄
    並且 應該有結果狀態篩選器:
      | 狀態選項 | 描述       |
      | 所有結果 | 顯示全部   |
      | 成功     | 成功執行   |
      | 錯誤     | 執行失敗   |
      | 進行中   | 正在執行   |

  @results @detailed-view
  場景: 執行結果詳細資訊
    假設 我查看一個成功的執行結果
    那麼 我應該看到結果卡片包含:
      | 元素         | 格式                    |
      | 任務名稱     | Prompt 標題             |
      | 狀態圖標     | check_circle (成功)     |
      | 執行內容     | 結果文字內容            |
      | 執行時間戳   | YYYY年MM月DD日 時間格式 |
      | 執行耗時     | XXXXms 格式             |

  @monitoring @system-status
  場景: 系統監控資訊
    假設 我在系統監控頁面
    那麼 我應該看到三個監控區塊:
      | 區塊名稱     | 包含資訊                     |
      | Claude API 狀態 | 狀態、最後檢查時間、版本    |
      | 應用資訊     | 版本、Tauri版本、平台、建置日期 |
      | 效能監控     | 記憶體、CPU、執行時間、統計數據 |

  @testing @core-modules
  場景: 核心模組測試介面
    假設 我在測試頁面
    那麼 我應該看到四大核心模組:
      | 模組代號    | 模組名稱           | 主要功能按鈕           |
      | CORE-001    | ccusage API 整合   | 檢查使用量            |
      | CORE-002    | 安全執行系統       | 執行提示、查看審計日誌 |
      | CORE-003    | 自適應監控系統     | 啟動監控、更新監控     |
      | CORE-004    | 智能排程系統       | 建立排程、效率分析     |

  @security @execution-options
  場景: 安全執行系統測試
    假設 我在 CORE-002 安全執行系統區塊
    那麼 我應該看到安全選項:
      | 選項         | 預設狀態 | 描述                    |
      | 跳過權限檢查 | 未勾選   | 危險操作，謹慎使用      |
      | 乾運行模式   | 未勾選   | 測試模式，不實際執行    |
      | 啟用安全檢查 | 已勾選   | 安全驗證，建議保持開啟  |

  @ui-components @material-design
  場景: Material Design 3.0 元件驗證
    那麼 所有頁面應該使用統一的 Material Design 元件:
      | 元件類型   | 實作特徵                        |
      | 按鈕       | material-symbols-outlined 圖標  |
      | 卡片       | 統一的圓角和陰影               |
      | 輸入框     | Material 樣式的邊框和標籤      |
      | 下拉選單   | 一致的樣式和互動效果           |
      | 通知訊息   | 統一的通知樣式和關閉按鈕       |

  @color-system @theming
  場景: 色彩系統驗證
    那麼 應用程式應該實作完整的 Material Design 色彩系統:
      | 色彩類型        | CSS 變數名稱                | 預期值       |
      | 主要色彩        | --md-sys-color-primary      | #2196f3      |
      | 表面色彩        | --md-sys-color-surface      | #fef7ff      |
      | 背景色彩        | --md-sys-color-background   | #fef7ff      |
      | 主要色彩對比    | --md-sys-color-on-primary   | #ffffff      |

  @accessibility @inclusive-design
  場景: 無障礙設計驗證
    那麼 所有互動元素應該具備無障礙特性:
      | 無障礙特徵     | 實作要求                      |
      | 鍵盤導航       | 所有按鈕可用 Tab 鍵導航      |
      | 螢幕閱讀器     | 適當的 aria-label 屬性       |
      | 顏色對比       | 符合 WCAG 2.1 AA 標準       |
      | 焦點指示器     | 清楚的焦點視覺回饋           |

  @responsive @mobile-friendly
  場景: 響應式設計驗證
    當 我調整瀏覽器視窗大小
    那麼 介面應該適應不同螢幕尺寸:
      | 斷點     | 寬度範圍    | 預期行為              |
      | 手機     | < 768px     | 單欄佈局，隱藏側邊欄  |
      | 平板     | 768-1024px  | 調整間距和字體大小    |
      | 桌面     | > 1024px    | 完整佈局顯示          |

  @error-handling @user-feedback
  場景: 錯誤處理和使用者回饋
    當 系統發生錯誤或完成操作
    那麼 應該提供適當的使用者回饋:
      | 情況類型   | 回饋方式               | 範例訊息                 |
      | 成功操作   | 成功通知               | "已切換至淺色主題"       |
      | 錯誤狀況   | 錯誤訊息顯示           | "Connection timeout"     |
      | 載入狀態   | 載入指示器             | 旋轉動畫或進度條         |
      | 空狀態     | 友善的空狀態說明       | "尚無執行結果"           |

  @performance @optimization
  場景: 效能驗證
    那麼 應用程式應該符合效能標準:
      | 效能指標     | 目標值    | 測量方法                 |
      | 首次載入時間 | < 3 秒    | 頁面完整載入時間         |
      | 互動回應時間 | < 100ms   | 點擊到視覺回饋的延遲     |
      | 記憶體使用   | < 150MB   | 瀏覽器記憶體監控         |
      | API 回應時間 | < 200ms   | 後端 API 呼叫延遲       |

  @integration @backend
  場景: 前後端整合驗證
    那麼 前端應該正確與後端 Tauri 命令整合:
      | API 端點                    | 預期行為                |
      | prompt_service_list_prompts | 載入 Prompt 列表        |
      | job_service_list_jobs       | 載入排程任務列表        |
      | get_unified_cooldown_status | 檢查 Claude API 狀態    |
      | sync_service_get_status     | 同步服務狀態檢查        |

  @chinese-localization @internationalization
  場景: 中文本地化驗證
    那麼 所有文字內容應該使用繁體中文:
      | 功能區域 | 中文文字範例              |
      | 導航     | "Prompt", "排程", "結果"  |
      | 狀態     | "運行中", "已暫停", "成功" |
      | 操作     | "執行", "刪除", "刷新資訊" |
      | 通知     | "已切換至淺色主題"        |