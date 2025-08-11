/**
 * 資料庫測試設定工具
 * 
 * 提供測試環境下的資料庫初始化和清理功能
 */

import path from 'path';
import fs from 'fs/promises';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * 測試資料庫設定
 */
export const dbConfig = {
  testDbPath: path.join(__dirname, '../../src-tauri/test-claude-pilot.db'),
  backupDbPath: path.join(__dirname, '../../src-tauri/claude-pilot.db.backup'),
  migrationPath: path.join(__dirname, '../../src-tauri/migrations')
};

/**
 * 初始化測試資料庫
 */
export async function initTestDatabase() {
  try {
    // 備份現有資料庫（如果存在）
    const originalDbPath = path.join(__dirname, '../../src-tauri/claude-pilot.db');
    
    try {
      await fs.access(originalDbPath);
      await fs.copyFile(originalDbPath, dbConfig.backupDbPath);
      console.log('✅ 原始資料庫已備份');
    } catch (error) {
      // 原始資料庫不存在，跳過備份
      console.log('ℹ️ 原始資料庫不存在，跳過備份');
    }
    
    // 建立測試資料庫
    await createTestDatabase();
    
    console.log('✅ 測試資料庫初始化完成');
    return true;
  } catch (error) {
    console.error('❌ 測試資料庫初始化失敗:', error);
    return false;
  }
}

/**
 * 建立測試資料庫
 */
async function createTestDatabase() {
  const { exec } = await import('child_process');
  const { promisify } = await import('util');
  const execAsync = promisify(exec);
  
  // 使用 CLI 工具初始化資料庫
  const initCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo run --bin cnp-unified -- init --force`;
  
  try {
    const { stdout, stderr } = await execAsync(initCommand);
    console.log('資料庫初始化輸出:', stdout);
    if (stderr) console.warn('資料庫初始化警告:', stderr);
  } catch (error) {
    throw new Error(`資料庫初始化失敗: ${error.message}`);
  }
}

/**
 * 插入測試資料
 */
export async function insertTestData() {
  const testPrompts = [
    {
      title: "測試 Prompt 1",
      content: "這是第一個測試用的 Prompt 內容",
      tags: "test,sample,automation"
    },
    {
      title: "Claude Code 語法測試",
      content: "@README.md 請分析這個專案的主要功能並提供改進建議",
      tags: "claude-code,analysis,improvement"
    },
    {
      title: "多檔案分析測試",
      content: "@src/ 請檢查所有 JavaScript 檔案的程式碼品質",
      tags: "multi-file,quality,review"
    }
  ];
  
  try {
    const { exec } = await import('child_process');
    const { promisify } = await import('util');
    const execAsync = promisify(exec);
    
    for (const prompt of testPrompts) {
      const createCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo run --bin cnp-unified -- prompt create "${prompt.title}" "${prompt.content}" --tags "${prompt.tags}"`;
      
      try {
        await execAsync(createCommand);
        console.log(`✅ 已建立測試 Prompt: ${prompt.title}`);
      } catch (error) {
        console.warn(`⚠️ 建立測試 Prompt 失敗: ${prompt.title} - ${error.message}`);
      }
    }
    
    console.log('✅ 測試資料插入完成');
    return true;
  } catch (error) {
    console.error('❌ 插入測試資料失敗:', error);
    return false;
  }
}

/**
 * 清理測試資料庫
 */
export async function cleanupTestDatabase() {
  try {
    // 刪除測試資料庫
    try {
      await fs.unlink(dbConfig.testDbPath);
      console.log('✅ 測試資料庫已清理');
    } catch (error) {
      // 測試資料庫不存在，跳過
      console.log('ℹ️ 測試資料庫不存在，跳過清理');
    }
    
    // 恢復原始資料庫（如果有備份）
    try {
      await fs.access(dbConfig.backupDbPath);
      const originalDbPath = path.join(__dirname, '../../src-tauri/claude-pilot.db');
      await fs.copyFile(dbConfig.backupDbPath, originalDbPath);
      await fs.unlink(dbConfig.backupDbPath);
      console.log('✅ 原始資料庫已恢復');
    } catch (error) {
      // 沒有備份，跳過恢復
      console.log('ℹ️ 沒有資料庫備份，跳過恢復');
    }
    
    return true;
  } catch (error) {
    console.error('❌ 清理測試資料庫失敗:', error);
    return false;
  }
}

/**
 * 檢查資料庫狀態
 */
export async function checkDatabaseStatus() {
  try {
    const { exec } = await import('child_process');
    const { promisify } = await import('util');
    const execAsync = promisify(exec);
    
    const statusCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo run --bin cnp-unified -- status --format json`;
    
    const { stdout } = await execAsync(statusCommand);
    const status = JSON.parse(stdout);
    
    return {
      connected: status.database_status === 'connected',
      tables: status.tables || [],
      prompts_count: status.prompts_count || 0,
      jobs_count: status.jobs_count || 0
    };
  } catch (error) {
    console.error('檢查資料庫狀態失敗:', error);
    return {
      connected: false,
      error: error.message
    };
  }
}

/**
 * 執行資料庫遷移
 */
export async function runDatabaseMigrations() {
  try {
    const migrationFiles = await fs.readdir(dbConfig.migrationPath);
    const sqlFiles = migrationFiles.filter(file => file.endsWith('.sql'));
    
    console.log(`發現 ${sqlFiles.length} 個遷移檔案:`, sqlFiles);
    
    // 透過 CLI 工具執行遷移
    const { exec } = await import('child_process');
    const { promisify } = await import('util');
    const execAsync = promisify(exec);
    
    const migrateCommand = `cd ${path.join(__dirname, '../../src-tauri')} && cargo run --bin cnp-unified -- init --migrate`;
    
    const { stdout, stderr } = await execAsync(migrateCommand);
    console.log('遷移輸出:', stdout);
    if (stderr) console.warn('遷移警告:', stderr);
    
    return true;
  } catch (error) {
    console.error('執行資料庫遷移失敗:', error);
    return false;
  }
}

/**
 * 重置資料庫到初始狀態
 */
export async function resetDatabase() {
  try {
    await cleanupTestDatabase();
    await initTestDatabase();
    await insertTestData();
    
    console.log('✅ 資料庫重置完成');
    return true;
  } catch (error) {
    console.error('❌ 資料庫重置失敗:', error);
    return false;
  }
}

/**
 * 驗證資料庫完整性
 */
export async function validateDatabaseIntegrity() {
  try {
    const status = await checkDatabaseStatus();
    
    if (!status.connected) {
      throw new Error('資料庫連接失敗');
    }
    
    // 檢查必要的表是否存在
    const requiredTables = ['prompts', 'jobs', 'results'];
    const missingTables = requiredTables.filter(table => 
      !status.tables.includes(table)
    );
    
    if (missingTables.length > 0) {
      throw new Error(`缺少必要的資料表: ${missingTables.join(', ')}`);
    }
    
    console.log('✅ 資料庫完整性驗證通過');
    return true;
  } catch (error) {
    console.error('❌ 資料庫完整性驗證失敗:', error);
    return false;
  }
}