/**
 * Cargo.toml 版本更新器
 * 基於 standard-version 的自定義更新器
 */

const fs = require('fs');
const path = require('path');

module.exports = {
  readVersion: function (contents) {
    // 從 Cargo.toml 讀取當前版本
    const versionMatch = contents.match(/^version\s*=\s*"([^"]+)"/m);
    return versionMatch ? versionMatch[1] : null;
  },

  writeVersion: function (contents, version) {
    // 更新 Cargo.toml 中的版本號
    return contents.replace(
      /^version\s*=\s*"[^"]+"/m,
      `version = "${version}"`
    );
  }
};

/**
 * 同步更新其他 Cargo.toml 檔案中的版本
 */
function syncCargoVersions(newVersion) {
  const cargoFiles = [
    'src-tauri/Cargo.toml'
  ];

  cargoFiles.forEach(filePath => {
    const fullPath = path.resolve(__dirname, '../../', filePath);
    
    if (fs.existsSync(fullPath)) {
      let content = fs.readFileSync(fullPath, 'utf8');
      
      // 更新 package 版本
      content = content.replace(
        /^version\s*=\s*"[^"]+"/m,
        `version = "${newVersion}"`
      );
      
      // 更新內部依賴版本 (如果有的話)
      content = content.replace(
        /^claude-night-pilot\s*=\s*{\s*version\s*=\s*"[^"]+"/gm,
        `claude-night-pilot = { version = "${newVersion}"`
      );
      
      fs.writeFileSync(fullPath, content, 'utf8');
      console.log(`✅ 已更新 ${filePath} 版本為 ${newVersion}`);
    }
  });
}

// 如果直接執行此腳本
if (require.main === module) {
  const newVersion = process.argv[2];
  if (newVersion) {
    syncCargoVersions(newVersion);
  } else {
    console.error('❌ 請提供版本號: node cargo-version-updater.js <version>');
    process.exit(1);
  }
}

module.exports.syncCargoVersions = syncCargoVersions;