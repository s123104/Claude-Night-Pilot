#!/usr/bin/env node

/**
 * 簡化的GUI資料庫整合驗證測試
 * 使用 Puppeteer 簡單檢測前端是否使用真實資料
 */

import puppeteer from 'puppeteer';
import { setTimeout } from 'timers/promises';

async function testGuiIntegration() {
  console.log('🧪 開始GUI資料庫整合驗證...');
  
  let browser, page;
  
  try {
    // 啟動瀏覽器
    browser = await puppeteer.launch({ 
      headless: false,
      devtools: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    page = await browser.newPage();
    
    // 設置視窗大小
    await page.setViewport({ width: 1280, height: 720 });
    
    // 監聽console訊息
    const consoleLogs = [];
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push({
        type: msg.type(),
        text: text
      });
      console.log(`Console [${msg.type()}]:`, text);
    });
    
    // 監聽網路請求
    const requests = [];
    page.on('request', request => {
      requests.push({
        url: request.url(),
        method: request.method()
      });
    });
    
    // 導航到頁面
    console.log('🌐 導航至 http://localhost:8080...');
    await page.goto('http://localhost:8080', { 
      waitUntil: 'networkidle2',
      timeout: 30000 
    });
    
    console.log('✅ 頁面載入成功');
    
    // 等待頁面完全載入
    await setTimeout(3000);
    
    // 檢查頁面標題
    const title = await page.title();
    console.log(`📄 頁面標題: ${title}`);
    
    // 檢查是否有模擬資料標識
    const bodyText = await page.evaluate(() => document.body.innerText);
    const hasMockIndicators = /模擬|測試|mock|fake|dummy/.test(bodyText);
    
    console.log(`🔍 頁面包含模擬資料指標: ${hasMockIndicators}`);
    
    // 檢查是否有錯誤
    const errors = consoleLogs.filter(log => log.type === 'error');
    console.log(`❌ Console錯誤數量: ${errors.length}`);
    if (errors.length > 0) {
      console.log('錯誤詳情:');
      errors.forEach((error, i) => {
        console.log(`  ${i + 1}. ${error.text}`);
      });
    }
    
    // 檢查網路請求
    console.log(`🌐 網路請求數量: ${requests.length}`);
    
    // 生成報告
    const report = {
      timestamp: new Date().toISOString(),
      pageTitle: title,
      hasMockIndicators,
      consoleErrors: errors.length,
      networkRequests: requests.length,
      status: hasMockIndicators || errors.length > 5 ? 'needs_attention' : 'healthy'
    };
    
    console.log('\n📊 測試報告:');
    console.log(JSON.stringify(report, null, 2));
    
    if (report.status === 'healthy') {
      console.log('🎉 GUI整合測試通過！');
    } else {
      console.log('⚠️  GUI可能需要進一步檢查');
    }
    
    // 等待一段時間便於觀察
    console.log('⏳ 等待5秒便於觀察...');
    await setTimeout(5000);
    
  } catch (error) {
    console.error('❌ 測試失敗:', error.message);
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}

// 執行測試
testGuiIntegration().catch(console.error);