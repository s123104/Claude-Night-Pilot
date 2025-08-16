use serde_json::Value;

/// 提供統一的代理清單（MVP 版本）
/// 將使用者在需求中提供的代理分群整理為可供 GUI/CLI 顯示的資料結構。
/// 若未來需要細分能力矩陣或權限，可擴充為型別結構。
pub fn agents_catalog_json() -> Value {
    // 精簡版代理清單（依部門分組）。
    // 註：為避免二進位過大，僅保留關鍵欄位；詳細描述請參考 docs/agents/AGENTS.md。
    let data = r#"
    {
      "version": "2025.6.2",
      "departments": [
        {
          "id": "engineering",
          "name": "工程部門",
          "agents": [
            {
              "id": "ai-engineer",
              "name": "AI 工程師",
              "specialty": "整合可落地的 AI/ML 功能",
              "scenarios": ["應用加入 AI 功能","整合 LLM/ML 管線","計算機視覺","智能自動化"],
              "active": true
            },
            {
              "id": "backend-architect",
              "name": "後端架構師",
              "specialty": "可擴展 API 與伺服器架構",
              "scenarios": ["API/DB 設計","效能優化","OAuth2 與安全","擴展服務"],
              "active": true
            },
            {
              "id": "devops-automator",
              "name": "DevOps 自動化專家",
              "specialty": "穩健的 CI/CD 與部署",
              "scenarios": ["CI/CD","自動擴縮/負載平衡","監控與警報","IaC"],
              "active": true
            },
            {
              "id": "frontend-developer",
              "name": "前端開發工程師",
              "specialty": "高效能 UI 與體驗",
              "scenarios": ["儀表板/圖表","響應式/行動端","載入效能優化","a11y"],
              "active": true
            },
            {
              "id": "mobile-app-builder",
              "name": "移動應用建構師",
              "specialty": "原生 iOS/Android 體驗",
              "scenarios": ["短影音串流","推播/生物辨識","React Native","原生效能"],
              "active": true
            },
            {
              "id": "rapid-prototyper",
              "name": "快速原型師",
              "specialty": "數日內完成 MVP",
              "scenarios": ["原型/POC","病毒功能試驗","想法驗證","Demo"],
              "active": true
            },
            {
              "id": "test-writer-fixer",
              "name": "測試編寫修復專家",
              "specialty": "撰寫抓真錯的測試",
              "scenarios": ["變更後撰寫測試","失敗分析","重構後修測","關鍵路徑覆蓋"],
              "active": true,
              "auto_trigger": ["after_code_change","post_refactor"]
            }
          ]
        },
        {
          "id": "product",
          "name": "產品部門",
          "agents": [
            {"id":"feedback-synthesizer","name":"反饋綜合師","specialty":"將抱怨轉成功能","scenarios":["多源反饋分析","模式識別","功能優先級","洞察報告"],"active": true},
            {"id":"sprint-prioritizer","name":"衝刺優先級管理師","specialty":"6天內交付最大價值","scenarios":["6天週期","權衡決策","範疇重組","ROI 決策"],"active": true},
            {"id":"trend-researcher","name":"趨勢研究員","specialty":"尋找病毒式機會","scenarios":["趨勢導向創意","市場驗證","競品分析","病毒機制"],"active": true}
          ]
        },
        {
          "id": "marketing",
          "name": "行銷部門",
          "agents": [
            {"id":"app-store-optimizer","name":"應用商店優化師","specialty":"ASO 最佳化","scenarios":["清單/關鍵字","中介資料優化","排名分析","有機成長"],"active": true},
            {"id":"content-creator","name":"內容創作者","specialty":"跨平台內容","scenarios":["內容策略","社群貼文","品牌聲音","物料製作"],"active": true},
            {"id":"growth-hacker","name":"增長駭客","specialty":"建立增長迴路","scenarios":["獲取/留存漏斗","實驗","迭代優化","病毒活動"],"active": true},
            {"id":"instagram-curator","name":"Instagram 策劃師","specialty":"視覺內容","scenarios":["內容策略","品牌美學","趨勢分析","參與度優化"],"active": true},
            {"id":"reddit-community-builder","name":"Reddit 社群建設者","specialty":"合規社群營運","scenarios":["社群建立/互動","價值內容","聲譽管理","產品推廣"],"active": true},
            {"id":"tiktok-strategist","name":"TikTok 策略師","specialty":"可分享時刻","scenarios":["發布策略","病毒格式","創作者合作","功能可分享性"],"active": true},
            {"id":"twitter-engager","name":"Twitter 參與專家","specialty":"趨勢參與","scenarios":["內容策略","trending 利用","品牌營運","viral 活動"],"active": true}
          ]
        },
        {
          "id": "design",
          "name": "設計部門",
          "agents": [
            {"id":"brand-guardian","name":"品牌守護者","specialty":"一致視覺識別","scenarios":["品牌指南","一致性與合規","資產管理","識別演進"],"active": true},
            {"id":"ui-designer","name":"UI 設計師","specialty":"可實作的界面","scenarios":["新介面/元件","設計系統","視覺/體驗","響應式/a11y"],"active": true},
            {"id":"ux-researcher","name":"UX 研究員","specialty":"將洞察轉為改進","scenarios":["需求理解","可用性研究","旅程/人物誌","驗證設計"],"active": true},
            {"id":"visual-storyteller","name":"視覺故事講述者","specialty":"可轉換/可分享視覺","scenarios":["入門插圖","資訊圖表","簡報物料","品牌敘事"],"active": true},
            {"id":"whimsy-injector","name":"趣味注入師","specialty":"增加愉悅互動","scenarios":["UI/UX 後的趣味","錯誤正體驗","有趣載入/等待","完成驚喜"],"active": true,
             "auto_trigger":["post_ui_change"]}
          ]
        },
        {
          "id": "project",
          "name": "專案管理",
          "agents": [
            {"id":"experiment-tracker","name":"實驗追蹤師","specialty":"數據驅動驗證","scenarios":["功能旗標","A/B","里程碑追蹤","數據決策"],"active": true,
             "auto_trigger":["feature_flag_added"]},
            {"id":"project-shipper","name":"專案交付師","specialty":"穩健發布","scenarios":["重大發布準備","協調上市","多版本管理","發布後監控"],"active": true},
            {"id":"studio-producer","name":"工作室製作人","specialty":"提升交付節奏","scenarios":["跨隊協作","資源與衝突","流程優化","衝刺協調"],"active": true}
          ]
        },
        {
          "id": "operations",
          "name": "工作室營運",
          "agents": [
            {"id":"analytics-reporter","name":"分析報告師","specialty":"數據到洞察","scenarios":["月回顧","KPI 儀表板","趨勢/機會","決策支持"],"active": true},
            {"id":"finance-tracker","name":"財務追蹤師","specialty":"保持盈利","scenarios":["季度預算","成本優化","ROI 分析","資源配置"],"active": true},
            {"id":"infrastructure-maintainer","name":"基礎設施維護師","specialty":"可靠擴展","scenarios":["系統健康","容量規劃","可靠性保障","成本/資源優化"],"active": true},
            {"id":"legal-compliance-checker","name":"法律合規檢查師","specialty":"快速且合法","scenarios":["歐洲市場","ToS/隱私","監管合規","法律風險"],"active": true},
            {"id":"support-responder","name":"支援回應師","specialty":"憤怒轉擁護","scenarios":["上線支援","客訴處理","支援文檔/流程","模式分析與改進"],"active": true}
          ]
        },
        {
          "id": "testing",
          "name": "測試部門",
          "agents": [
            {"id":"api-tester","name":"API 測試師","specialty":"壓力下仍可用","scenarios":["負載測試","規範/合約","安全/認證","自動化/監控"],"active": true},
            {"id":"performance-benchmarker","name":"效能基準測試師","specialty":"讓一切更快","scenarios":["速度/優化","瓶頸分析","基準/目標","監控/警報"],"active": true},
            {"id":"test-results-analyzer","name":"測試結果分析師","specialty":"找出失敗模式","scenarios":["趨勢分析","根因定位","品質指標","數據洞察"],"active": true},
            {"id":"tool-evaluator","name":"工具評估師","specialty":"選對工具","scenarios":["框架/函式庫評估","工具比較","採用建議","整合策略"],"active": true},
            {"id":"workflow-optimizer","name":"工作流程優化師","specialty":"消除瓶頸","scenarios":["流程效率","人機協作","瓶頸分析","自動化建議"],"active": true}
          ]
        },
        {
          "id": "bonus",
          "name": "獎勵代理",
          "agents": [
            {"id":"joker","name":"幽默師","specialty":"技術幽默","scenarios":["壓力衝刺提氣","有趣錯誤/404","產品趣味","團隊氣氛"],"active": true},
            {"id":"studio-coach","name":"工作室教練","specialty":"激勵到卓越","scenarios":["多代理任務指導","遇阻支援","重大衝刺準備","勝利/復盤"],"active": true,
             "auto_trigger":["multi_agent_complex_start","agent_overwhelmed"]}
          ]
        }
      ]
    }
    "#;

    serde_json::from_str(data).expect("agents_catalog_json 解析失敗")
}
