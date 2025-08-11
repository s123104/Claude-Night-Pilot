/**
 * 全域測試設定
 * 
 * 在所有測試開始前執行的初始化作業
 */

import { initTestDatabase, insertTestData } from './db-setup.js';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function globalSetup(config) {
  console.log('🚀 開始全域測試設定...');
  
  try {
    // 1. 初始化測試資料庫
    console.log('📦 初始化測試資料庫...');
    const dbInitialized = await initTestDatabase();
    if (!dbInitialized) {
      throw new Error('資料庫初始化失敗');
    }
    
    // 2. 插入測試資料
    console.log('📝 插入測試資料...');
    const dataInserted = await insertTestData();
    if (!dataInserted) {
      console.warn('⚠️ 部分測試資料插入失敗，將使用模擬資料');
    }
    
    // 3. 檢查 CLI 工具可用性
    console.log('🔧 檢查 CLI 工具可用性...');
    await checkCliAvailability();
    
    // 4. 設定環境變數
    process.env.TEST_MODE = 'true';
    process.env.NODE_ENV = 'test';
    process.env.PLAYWRIGHT_TEST = 'true';
    
    // 5. 清理舊的測試結果
    await cleanupOldResults();
    
    console.log('✅ 全域測試設定完成');
    
  } catch (error) {
    console.error('❌ 全域測試設定失敗:', error);
    process.exit(1);
  }
}

/**
 * 檢查 CLI 工具可用性
 */
async function checkCliAvailability() {
  try {
    const { exec } = await import('child_process');
    const { promisify } = await import('util');
    const execAsync = promisify(exec);
    
    // 檢查 Rust CLI 是否可以編譯
    const buildCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo build --bin cnp-unified`;
    
    console.log('  🔨 編譯 CLI 工具...');
    const { stdout, stderr } = await execAsync(buildCommand, { timeout: 120000 });
    
    if (stderr && !stderr.includes('warning')) {
      console.warn('  ⚠️ CLI 編譯警告:', stderr);
    }
    
    // 測試基本功能
    const testCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo run --bin cnp-unified -- --version`;
    
    try {
      const { stdout: version } = await execAsync(testCommand, { timeout: 10000 });
      console.log('  ✅ CLI 工具可用，版本:', version.trim());
    } catch (error) {
      console.warn('  ⚠️ CLI 工具測試失敗，將使用模擬模式');
      process.env.USE_MOCK_CLI = 'true';
    }
    
  } catch (error) {
    console.warn('  ⚠️ CLI 工具檢查失敗，將使用模擬模式:', error.message);
    process.env.USE_MOCK_CLI = 'true';
  }
}

/**
 * 清理舊的測試結果
 */
async function cleanupOldResults() {
  try {
    const fs = await import('fs/promises');
    
    // 清理覆蓋率目錄
    const coverageDir = path.join(__dirname, '../../coverage');
    
    try {
      await fs.access(coverageDir);
      await fs.rm(coverageDir, { recursive: true });
      console.log('  🧹 已清理舊的覆蓋率報告');
    } catch (error) {
      // 目錄不存在，跳過清理
    }
    
    // 重新建立覆蓋率目錄
    await fs.mkdir(coverageDir, { recursive: true });
    
    // 清理 Playwright 報告
    const playwrightReportDir = path.join(__dirname, '../../playwright-report');
    
    try {
      await fs.access(playwrightReportDir);
      await fs.rm(playwrightReportDir, { recursive: true });
      console.log('  🧹 已清理舊的 Playwright 報告');
    } catch (error) {
      // 目錄不存在，跳過清理
    }
    
  } catch (error) {
    console.warn('  ⚠️ 清理舊結果時發生錯誤:', error.message);
  }
}

/**
 * 設定測試環境變數
 */
function setupEnvironmentVariables() {
  const testEnvVars = {
    'TEST_MODE': 'true',
    'NODE_ENV': 'test',
    'PLAYWRIGHT_TEST': 'true',
    'LOG_LEVEL': 'warn', // 減少測試期間的日誌輸出
    'RUST_LOG': 'warn',
    'TAURI_ENV': 'test'
  };
  
  Object.entries(testEnvVars).forEach(([key, value]) => {
    if (!process.env[key]) {
      process.env[key] = value;
    }
  });
  
  console.log('  🌍 測試環境變數設定完成');
}

export default globalSetup;