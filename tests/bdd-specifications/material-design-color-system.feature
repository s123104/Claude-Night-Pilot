# Material Design 3.0 色彩系統 BDD 規範
# Claude Night Pilot 完整主題和色彩系統測試

@material-design @color-system @theming
功能: Material Design 3.0 色彩系統完整實作
  作為 UI/UX 設計師
  我希望應用程式完全符合 Material Design 3.0 色彩規範
  以便提供一致、專業且符合標準的視覺體驗

  背景:
    假設 Claude Night Pilot 應用程式已啟動
    並且 Material Design 3.0 樣式系統已載入
    並且 CSS 自定義屬性已正確定義

  @color-palette @primary-colors
  場景: 主要色彩系統驗證
    那麼 系統應該定義完整的主要色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 預期色值      | 用途描述           |
      | 主要色                | --md-sys-color-primary          | #2196f3       | 主要品牌色彩       |
      | 主要色 (dark)         | --md-sys-color-primary-dark     | #1976d2       | 深色主要色         |
      | 主要色 (light)        | --md-sys-color-primary-light    | #64b5f6       | 淺色主要色         |
      | 主要色對比            | --md-sys-color-on-primary       | #ffffff       | 主要色上的文字     |
      | 主要容器              | --md-sys-color-primary-container| #e3f2fd       | 主要色容器背景     |
      | 主要容器對比          | --md-sys-color-on-primary-container| #0d47a1    | 主要容器上的文字   |

  @color-palette @secondary-colors
  場景: 次要色彩系統驗證
    那麼 系統應該定義完整的次要色彩調色板:
      | 色彩角色              | CSS 變數名稱                      | 預期色值      | 用途描述           |
      | 次要色                | --md-sys-color-secondary          | #ff9800       | 次要品牌色彩       |
      | 次要色對比            | --md-sys-color-on-secondary       | #000000       | 次要色上的文字     |
      | 次要容器              | --md-sys-color-secondary-container| #fff3e0       | 次要色容器背景     |
      | 次要容器對比          | --md-sys-color-on-secondary-container| #e65100     | 次要容器上的文字   |

  @color-palette @tertiary-colors
  場景: 第三色彩系統驗證
    那麼 系統應該定義完整的第三色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 預期色值      | 用途描述           |
      | 第三色                | --md-sys-color-tertiary         | #9c27b0       | 第三品牌色彩       |
      | 第三色對比            | --md-sys-color-on-tertiary      | #ffffff       | 第三色上的文字     |
      | 第三容器              | --md-sys-color-tertiary-container| #f3e5f5      | 第三色容器背景     |
      | 第三容器對比          | --md-sys-color-on-tertiary-container| #4a148c    | 第三容器上的文字   |

  @color-palette @neutral-colors
  場景: 中性色彩系統驗證
    那麼 系統應該定義完整的中性色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 預期色值      | 用途描述           |
      | 背景色                | --md-sys-color-background       | #fef7ff       | 主要背景色         |
      | 背景對比色            | --md-sys-color-on-background    | #1c1b1f       | 背景上的文字       |
      | 表面色                | --md-sys-color-surface          | #fef7ff       | 卡片和元件表面     |
      | 表面對比色            | --md-sys-color-on-surface       | #1c1b1f       | 表面上的文字       |
      | 表面變體色            | --md-sys-color-surface-variant   | #e7e0ec       | 表面變體背景       |
      | 表面變體對比色        | --md-sys-color-on-surface-variant| #49454f       | 表面變體上的文字   |

  @color-palette @semantic-colors
  場景: 語意色彩系統驗證
    那麼 系統應該定義完整的語意色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 預期色值      | 用途描述           |
      | 錯誤色                | --md-sys-color-error            | #ba1a1a       | 錯誤狀態指示       |
      | 錯誤對比色            | --md-sys-color-on-error         | #ffffff       | 錯誤色上的文字     |
      | 錯誤容器              | --md-sys-color-error-container  | #ffdad6       | 錯誤容器背景       |
      | 錯誤容器對比          | --md-sys-color-on-error-container| #410002      | 錯誤容器上的文字   |
      | 成功色                | --md-sys-color-success          | #2e7d32       | 成功狀態指示       |
      | 成功對比色            | --md-sys-color-on-success       | #ffffff       | 成功色上的文字     |
      | 警告色                | --md-sys-color-warning          | #f57c00       | 警告狀態指示       |
      | 警告對比色            | --md-sys-color-on-warning       | #000000       | 警告色上的文字     |

  @color-palette @outline-colors
  場景: 輪廓色彩系統驗證
    那麼 系統應該定義完整的輪廓色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 預期色值      | 用途描述           |
      | 輪廓色                | --md-sys-color-outline          | #79747e       | 邊框和分隔線       |
      | 輪廓變體色            | --md-sys-color-outline-variant  | #cac4d0       | 輕微的邊框和分隔線 |

  @dark-theme @theme-switching
  場景: 深色主題色彩系統
    假設 我切換到深色主題
    那麼 系統應該應用深色主題的色彩調色板:
      | 色彩角色              | CSS 變數名稱                    | 深色主題色值  | 用途描述           |
      | 背景色                | --md-sys-color-background       | #10080e       | 深色背景           |
      | 背景對比色            | --md-sys-color-on-background    | #e6e1e5       | 深色背景上的文字   |
      | 表面色                | --md-sys-color-surface          | #10080e       | 深色表面           |
      | 表面對比色            | --md-sys-color-on-surface       | #e6e1e5       | 深色表面上的文字   |
      | 主要色                | --md-sys-color-primary          | #d0bcff       | 深色主要色         |
      | 主要色對比            | --md-sys-color-on-primary       | #381e72       | 深色主要色對比     |

  @theme-switching @state-management
  場景大綱: 主題切換功能完整測試
    假設 當前主題為 "<當前主題>"
    當 我點擊主題切換按鈕
    那麼 主題應該切換為 "<新主題>"
    並且 按鈕圖標應該變為 "<新圖標>"
    並且 CSS 屬性 "data-theme" 應該為 "<新主題>"
    並且 應該顯示通知訊息 "<通知訊息>"

    例子:
      | 當前主題 | 新主題 | 新圖標         | 通知訊息           |
      | auto     | light  | light_mode     | 已切換至淺色主題   |
      | light    | dark   | dark_mode      | 已切換至深色主題   |
      | dark     | auto   | brightness_auto| 已切換至自動主題   |

  @elevation @shadow-system
  場景: 高度和陰影系統驗證
    那麼 系統應該定義 Material Design 3.0 高度系統:
      | 高度等級 | CSS 變數名稱          | 陰影值                              | 用途描述       |
      | Level 0  | --md-elevation-0      | none                                | 無高度         |
      | Level 1  | --md-elevation-1      | 0px 1px 3px rgba(0,0,0,0.12)      | 卡片基礎高度   |
      | Level 2  | --md-elevation-2      | 0px 1px 5px rgba(0,0,0,0.14)      | 懸停狀態       |
      | Level 3  | --md-elevation-3      | 0px 1px 8px rgba(0,0,0,0.16)      | 模態和對話框   |
      | Level 4  | --md-elevation-4      | 0px 2px 10px rgba(0,0,0,0.18)     | 導航抽屜       |
      | Level 5  | --md-elevation-5      | 0px 4px 15px rgba(0,0,0,0.20)     | 浮動按鈕       |

  @typography @color-application
  場景: 文字顏色應用驗證
    那麼 不同類型的文字應該使用正確的顏色:
      | 文字類型     | 預期色彩變數                  | 應用場景               |
      | 主要文字     | --md-sys-color-on-surface    | 標題和重要內容         |
      | 次要文字     | --md-sys-color-on-surface-variant | 描述和輔助資訊     |
      | 按鈕文字     | --md-sys-color-on-primary    | 主要按鈕文字           |
      | 錯誤文字     | --md-sys-color-error         | 錯誤訊息和警告         |
      | 連結文字     | --md-sys-color-primary       | 可點擊的連結           |

  @component-theming @consistent-application
  場景: 元件主題一致性驗證
    那麼 所有 UI 元件應該一致地應用主題色彩:
      | 元件類型   | 主要色彩來源                | 次要色彩來源              |
      | 按鈕       | primary / primary-container | on-primary / on-primary-container |
      | 卡片       | surface / surface-container | on-surface / on-surface-variant   |
      | 輸入框     | surface-variant             | on-surface-variant                |
      | 導航       | surface-container           | on-surface                        |
      | 狀態指示器 | error / success / warning   | on-error / on-success / on-warning|

  @accessibility @color-contrast
  場景: 顏色對比度無障礙驗證
    那麼 所有顏色組合應該符合 WCAG 2.1 AA 標準:
      | 前景色彩                  | 背景色彩                | 最小對比度 | 用途               |
      | on-surface               | surface                 | 4.5:1      | 正常文字           |
      | on-primary               | primary                 | 4.5:1      | 按鈕文字           |
      | on-error                 | error                   | 4.5:1      | 錯誤訊息           |
      | on-surface-variant       | surface                 | 3:1        | 次要文字           |

  @state-colors @interaction-feedback
  場景: 互動狀態色彩驗證
    那麼 所有互動元件應該提供適當的狀態色彩回饋:
      | 互動狀態 | 色彩調整                      | 視覺效果描述           |
      | 懸停     | primary 色彩 + 8% 透明度覆層  | 輕微的色彩加深         |
      | 焦點     | primary 色彩 + 12% 透明度覆層 | 明顯的焦點指示         |
      | 按下     | primary 色彩 + 16% 透明度覆層 | 按下回饋效果           |
      | 禁用     | 38% 透明度                    | 灰化的禁用狀態         |

  @custom-properties @css-architecture
  場景: CSS 自定義屬性架構驗證
    那麼 色彩系統應該使用結構化的 CSS 自定義屬性:
      | 屬性類別       | 命名模式                    | 範例                           |
      | 系統色彩       | --md-sys-color-{role}      | --md-sys-color-primary         |
      | 參考色彩       | --md-ref-color-{color}{tone}| --md-ref-color-primary40       |
      | 主題覆寫       | --md-custom-color-{name}   | --md-custom-color-brand        |
      | 高度陰影       | --md-elevation-{level}     | --md-elevation-2               |

  @performance @css-optimization
  場景: 色彩系統效能驗證
    那麼 色彩系統應該具備良好的效能特性:
      | 效能指標       | 目標值      | 驗證方法                   |
      | CSS 載入時間   | < 50ms      | 測量樣式表載入時間         |
      | 主題切換速度   | < 100ms     | 測量主題切換動畫時間       |
      | 重新渲染開銷   | < 16ms      | 測量主題切換的重新渲染時間 |
      | 記憶體使用     | < 2MB       | CSS 樣式佔用的記憶體       |