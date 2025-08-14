// Standard Version Configuration
// 基於 Conventional Commits 的版本管理配置

module.exports = {
  // 版本號格式
  header: '# Claude Night Pilot 更新日誌\n\n所有重要變更都會記錄在此檔案中。\n\n此專案遵循 [語義版本控制](https://semver.org/) 和 [約定式提交](https://www.conventionalcommits.org/) 規範。\n\n',
  
  // 比較 URL 模式
  compareUrlFormat: 'https://github.com/s123104/claude-night-pilot/compare/{{previousTag}}...{{currentTag}}',
  
  // 提交 URL 模式
  commitUrlFormat: 'https://github.com/s123104/claude-night-pilot/commit/{{hash}}',
  
  // 議題 URL 模式
  issueUrlFormat: 'https://github.com/s123104/claude-night-pilot/issues/{{id}}',
  
  // 用戶 URL 模式
  userUrlFormat: 'https://github.com/{{user}}',
  
  // 發布提交訊息模式
  releaseCommitMessageFormat: 'chore(release): {{currentTag}}',
  
  // 是否跳過不穩定版本
  skip: {
    bump: false,
    changelog: false,
    commit: false,
    tag: false
  },
  
  // 版本號文件配置
  packageFiles: [
    {
      filename: 'package.json',
      type: 'json'
    }
  ],
  
  // 版本更新文件配置
  bumpFiles: [
    {
      filename: 'package.json',
      type: 'json'
    },
    {
      filename: 'src-tauri/Cargo.toml',
      updater: require('./scripts/versioning/cargo-version-updater.js')
    },
    {
      filename: 'README.md',
      updater: require('./scripts/versioning/readme-version-updater.js')
    }
  ],
  
  // 提交類型配置
  types: [
    {
      type: 'feat',
      section: '✨ 新功能',
      hidden: false
    },
    {
      type: 'fix', 
      section: '🐛 錯誤修復',
      hidden: false
    },
    {
      type: 'perf',
      section: '⚡ 性能改進',
      hidden: false
    },
    {
      type: 'refactor',
      section: '♻️ 代碼重構',
      hidden: false
    },
    {
      type: 'docs',
      section: '📚 文檔更新',
      hidden: false
    },
    {
      type: 'test',
      section: '🧪 測試相關',
      hidden: false
    },
    {
      type: 'build',
      section: '📦 建置系統',
      hidden: false
    },
    {
      type: 'ci',
      section: '🔧 CI/CD',
      hidden: false
    },
    {
      type: 'style',
      section: '💄 代碼風格',
      hidden: true
    },
    {
      type: 'chore',
      section: '🔨 雜項變更',
      hidden: true
    },
    {
      type: 'revert',
      section: '⏪ 回滾變更',
      hidden: false
    }
  ],
  
  // 自定義變換器
  conventionalcommits: {
    issueUrlFormat: 'https://github.com/s123104/claude-night-pilot/issues/{{id}}',
    commitUrlFormat: 'https://github.com/s123104/claude-night-pilot/commit/{{hash}}',
    compareUrlFormat: 'https://github.com/s123104/claude-night-pilot/compare/{{previousTag}}...{{currentTag}}',
    
    // 預設設定
    parserOpts: {
      noteKeywords: ['BREAKING CHANGE', 'BREAKING CHANGES', '重大變更']
    },
    
    writerOpts: {
      commitGroupsSort: (a, b) => {
        const order = [
          '✨ 新功能',
          '🐛 錯誤修復', 
          '⚡ 性能改進',
          '♻️ 代碼重構',
          '📚 文檔更新',
          '🧪 測試相關',
          '📦 建置系統',
          '🔧 CI/CD',
          '⏪ 回滾變更'
        ];
        return order.indexOf(a.title) - order.indexOf(b.title);
      },
      
      commitsSort: ['subject', 'scope'],
      
      noteGroupsSort: 'title',
      
      notesSort: 'text'
    }
  }
};