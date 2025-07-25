/**
 * @type {import('@commitlint/types').UserConfig}
 */
export default {
  extends: ["@commitlint/config-conventional"],
  rules: {
    // Type 類型限制
    "type-enum": [
      2,
      "always",
      [
        "feat",     // 新功能
        "fix",      // 修復bug
        "docs",     // 文檔更新
        "style",    // 代碼格式調整
        "refactor", // 重構
        "perf",     // 性能優化
        "test",     // 測試相關
        "build",    // 構建系統
        "ci",       // CI配置
        "chore",    // 其他雜項
        "revert",    // 回滾
      ],
    ],
    // 主題行長度限制
    "header-max-length": [2, "always", 100],
    // 主題不能為空
    "subject-empty": [2, "never"],
    // 主題結尾不加句號
    "subject-full-stop": [2, "never", "."],
    // 主題大小寫 (允許句子格式)
    "subject-case": [
      2,
      "never",
      ["sentence-case", "start-case", "pascal-case", "upper-case"],
    ],
    // Body 前需要空行
    "body-leading-blank": [1, "always"],
    // Footer 前需要空行
    "footer-leading-blank": [1, "always"],
    // Body 每行最大長度
    "body-max-line-length": [2, "always", 100],
    // Footer 每行最大長度
    "footer-max-line-length": [2, "always", 100],
    // Type 必須小寫
    "type-case": [2, "always", "lower-case"],
    // Scope 必須小寫 (如果有的話)
    "scope-case": [2, "always", "lower-case"],
    // 允許的 Scope 範圍 (針對本專案)
    "scope-enum": [
      1,
      "always",
      [
        "core",        // 核心功能
        "gui",         // GUI 介面
        "cli",         // CLI 工具
        "db",          // 資料庫
        "scheduler",   // 排程器
        "executor",    // 執行器
        "security",    // 安全性
        "test",        // 測試
        "docs",        // 文檔
        "deps",        // 依賴更新
        "config",      // 配置
        "ci",          // CI/CD
        "release",      // 發布相關
      ],
    ],
  },
  prompt: {
    messages: {
      type: "選擇你要提交的變更類型:",
      scope: "選擇本次提交的變更範圍 (可選):",
      customScope: "請輸入自定義的變更範圍:",
      subject: "填寫簡短精練的變更描述:",
      body: "填寫更加詳細的變更描述 (可選)。使用 \"|\" 分隔換行:",
      breaking: "列舉非兼容性重大變更 (可選):",
      footer: "列舉出所有變更的 issues (可選)。 例如: #31, #34:",
      confirmCommit: "是否提交或修改commit?",
    },
    types: [
      { value: "feat", name: "feat:     🚀  新增功能" },
      { value: "fix", name: "fix:      🐛  修復缺陷" },
      { value: "docs", name: "docs:     📚  文檔變更" },
      { value: "style", name: "style:    💎  代碼格式" },
      { value: "refactor", name: "refactor: 📦  代碼重構" },
      { value: "perf", name: "perf:     🚀  性能優化" },
      { value: "test", name: "test:     🚨  添加測試" },
      { value: "build", name: "build:    🛠   構建相關" },
      { value: "ci", name: "ci:       ⚙️   持續集成" },
      { value: "chore", name: "chore:    ♻️   其他修改" },
      { value: "revert", name: "revert:   🗑   回滾提交" },
    ],
    useEmoji: false,
    emojiAlign: "center",
    allowCustomIssuePrefix: false,
    allowEmptyIssuePrefix: false,
  },
};