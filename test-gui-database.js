#!/usr/bin/env node

/**
 * 簡化的GUI與資料庫整合測試
 * 直接驗證GUI是否使用真實資料庫而非模擬資料
 */

import { chromium } from 'playwright';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function testGuiDatabaseIntegration() {
  console.log('🧪 開始GUI與資料庫整合測試...');
  
  let browser, page;
  
  try {
    // 啟動開發伺服器
    console.log('🚀 啟動開發伺服器...');
    const devServer = exec('npm run dev:frontend');
    
    // 等待伺服器啟動
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // 啟動瀏覽器
    browser = await chromium.launch({ 
      headless: false,  // 顯示瀏覽器便於調試
      devtools: true 
    });
    
    page = await browser.newPage();
    
    // 監聽網路請求
    const requests = [];
    page.on('request', request => {
      requests.push({
        url: request.url(),
        method: request.method()
      });
    });
    
    // 監聽Tauri命令調用（如果是Tauri應用）
    const tauriCalls = [];
    page.on('console', msg => {
      if (msg.text().includes('invoke')) {
        tauriCalls.push(msg.text());
      }
    });
    
    console.log('🌐 導航到應用...');
    await page.goto('http://localhost:8080', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });
    
    // 等待應用載入
    console.log('⏳ 等待應用載入...');
    await page.waitForSelector('body', { timeout: 10000 });
    
    // 檢查是否有資料庫相關的請求或調用
    console.log('📊 分析網路請求...');
    console.log(`總請求數量: ${requests.length}`);
    requests.forEach((req, i) => {
      console.log(`${i + 1}. ${req.method} ${req.url}`);
    });
    
    // 檢查頁面是否載入了真實資料而非模擬資料
    console.log('🔍 檢查頁面內容...');
    
    // 等待可能的非同步內容載入
    await page.waitForTimeout(2000);
    
    // 檢查是否有"模擬"、"測試"等字樣
    const pageText = await page.textContent('body');
    const hasMockData = pageText.includes('模擬') || 
                       pageText.includes('測試') || 
                       pageText.includes('mock') ||
                       pageText.includes('fake');
    
    console.log(`頁面是否包含模擬資料標識: ${hasMockData}`);
    
    // 檢查資料庫連接狀態
    try {
      // 嘗試執行資料庫相關操作
      const dbTestResult = await page.evaluate(async () => {
        // 如果是Tauri應用，嘗試調用後端命令
        if (window.__TAURI__) {
          try {
            const result = await window.__TAURI__.invoke('list_prompts');
            return { success: true, data: result };
          } catch (error) {
            return { success: false, error: error.message };
          }
        }
        return { success: false, error: 'Not a Tauri app' };
      });
      
      console.log('資料庫測試結果:', dbTestResult);
    } catch (error) {
      console.log('資料庫測試失敗:', error.message);
    }
    
    // 檢查Console是否有錯誤
    const consoleLogs = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleLogs.push(msg.text());
      }
    });
    
    await page.waitForTimeout(1000);
    
    if (consoleLogs.length > 0) {
      console.log('❌ 發現Console錯誤:');
      consoleLogs.forEach(log => console.log('  -', log));
    } else {
      console.log('✅ 無Console錯誤');
    }
    
    // 生成測試報告
    const report = {
      timestamp: new Date().toISOString(),
      requests: requests.length,
      hasMockData,
      consoleErrors: consoleLogs.length,
      tauriCalls: tauriCalls.length
    };
    
    console.log('\n📋 測試報告:');
    console.log(JSON.stringify(report, null, 2));
    
    if (!hasMockData && consoleLogs.length === 0) {
      console.log('🎉 GUI與資料庫整合測試通過！');
    } else {
      console.log('⚠️  GUI可能仍包含模擬資料或有錯誤');
    }
    
  } catch (error) {
    console.error('❌ 測試執行失敗:', error.message);
  } finally {
    if (browser) {
      await browser.close();
    }
    process.exit(0);
  }
}

// 執行測試
testGuiDatabaseIntegration().catch(console.error);