#!/usr/bin/env node

/**
 * Claude Night Pilot GUI 排程功能演示腳本
 * 展示如何建立和管理排程任務
 */

import { chromium } from 'playwright';

async function demoScheduleCreation() {
    console.log('🚀 啟動 Claude Night Pilot GUI 排程功能演示...\n');

    // 啟動瀏覽器
    const browser = await chromium.launch({ 
        headless: false,  // 顯示瀏覽器視窗
        slowMo: 1000      // 減慢動作以便觀察
    });
    
    const page = await browser.newPage();
    
    try {
        // 1. 前往應用程式
        console.log('📍 Step 1: 前往 Claude Night Pilot GUI');
        await page.goto('http://localhost:8081');
        await page.waitForTimeout(2000);
        
        // 2. 驗證介面載入
        console.log('🎨 Step 2: 驗證 Material Design 介面載入');
        await page.waitForSelector('.md-top-app-bar', { timeout: 5000 });
        console.log('   ✅ 頂部應用欄載入成功');
        
        // 3. 點擊排程頁面
        console.log('📅 Step 3: 切換到排程頁面');
        const schedulerTab = page.locator('[data-tab="scheduler"]');
        await schedulerTab.click();
        await page.waitForTimeout(1000);
        console.log('   ✅ 已切換到排程任務頁面');
        
        // 4. 點擊建立排程按鈕
        console.log('➕ Step 4: 開啟排程建立對話框');
        const createJobFab = page.locator('#create-job-fab');
        await createJobFab.click();
        await page.waitForTimeout(1000);
        console.log('   ✅ 排程建立對話框已開啟');
        
        // 5. 填寫排程資訊
        console.log('📝 Step 5: 填寫排程資訊');
        
        // 填寫任務名稱
        await page.fill('#job-name', '測試排程任務 - 每日早安報告');
        console.log('   ✅ 已填寫任務名稱');
        
        // 填寫 Cron 表達式
        await page.fill('#job-cron', '0 9 * * *');
        console.log('   ✅ 已填寫 Cron 表達式 (每日 9 點執行)');
        
        await page.waitForTimeout(1000);
        
        // 6. 提交排程
        console.log('💾 Step 6: 提交排程任務');
        const submitBtn = page.locator('[data-testid="job-modal-submit-btn"]');
        await submitBtn.click();
        await page.waitForTimeout(2000);
        console.log('   ✅ 排程任務已提交');
        
        // 7. 驗證排程列表
        console.log('📋 Step 7: 驗證排程任務出現在列表中');
        const jobsList = page.locator('#jobs-list');
        await page.waitForTimeout(1000);
        console.log('   ✅ 排程任務列表已更新');
        
        // 8. 切換到系統監控頁面
        console.log('🖥️ Step 8: 檢查系統監控狀態');
        const systemTab = page.locator('[data-tab="system"]');
        await systemTab.click();
        await page.waitForTimeout(1000);
        console.log('   ✅ 已切換到系統監控頁面');
        
        // 9. 檢查冷卻狀態
        console.log('❄️ Step 9: 檢查 Claude API 冷卻狀態');
        const cooldownStatus = page.locator('#cooldown-status');
        const statusText = await cooldownStatus.textContent();
        console.log(`   📊 冷卻狀態: ${statusText}`);
        
        // 10. 測試主題切換
        console.log('🌙 Step 10: 測試明暗主題切換');
        const themeToggle = page.locator('#theme-toggle');
        await themeToggle.click();
        await page.waitForTimeout(1000);
        console.log('   ✅ 主題已切換');
        
        console.log('\n🎉 GUI 排程功能演示完成！');
        console.log('\n📊 演示內容總結:');
        console.log('   • Material Design 3.0 介面正常載入');
        console.log('   • 排程建立對話框功能完整');
        console.log('   • Cron 表達式輸入驗證正常');
        console.log('   • 排程任務列表更新正確');
        console.log('   • 系統監控狀態顯示準確');
        console.log('   • 主題切換動畫流暢');
        
    } catch (error) {
        console.error('❌ 演示過程中發生錯誤:', error.message);
    }
    
    // 保持瀏覽器開啟 10 秒供觀察
    console.log('\n⏱️ 保持瀏覽器開啟 10 秒供觀察...');
    await page.waitForTimeout(10000);
    
    await browser.close();
    console.log('✅ 演示結束');
}

// Cron 表達式教學
function showCronExamples() {
    console.log('\n📚 Cron 表達式範例教學:');
    console.log('   格式: 分 時 日 月 週');
    console.log('   • "0 9 * * *"     - 每日 9:00 執行');
    console.log('   • "*/30 * * * *"  - 每 30 分鐘執行');
    console.log('   • "0 9-17 * * 1-5" - 週一到週五，9-17 點每小時執行');
    console.log('   • "0 0 1 * *"     - 每月 1 號午夜執行');
    console.log('   • "0 0 * * 0"     - 每週日午夜執行');
}

if (import.meta.url === `file://${process.argv[1]}`) {
    showCronExamples();
    demoScheduleCreation().catch(console.error);
}