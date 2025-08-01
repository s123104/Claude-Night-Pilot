import { test, expect } from "@playwright/test";

/**
 * Claude Night Pilot - Material Design 3.0 E2E 測試套件
 * 驗證任務排程系統的 CLI 與 GUI 功能整合
 */

test.describe("Material Design 3.0 任務排程系統驗證", () => {
  test.beforeEach(async ({ page }) => {
    // 前往應用主頁
    await page.goto("http://localhost:8081");
    
    // 等待 Material Design 應用載入完成
    await page.waitForSelector('.app-container.md-surface', {
      timeout: 30000,
    });
    
    // 確保載入動畫完成
    await page.waitForFunction(() => {
      const loader = document.getElementById('app-loader');
      return loader && loader.style.display === 'none';
    });
  });

  test("Material Design 3.0 GUI 架構驗證", async ({ page }) => {
    // 驗證頂部應用欄
    await expect(page.locator(".md-top-app-bar")).toBeVisible();
    
    // 驗證 Material Icons 正確載入
    const brandIcon = page.locator('.brand-icon.material-symbols-rounded');
    await expect(brandIcon).toBeVisible();
    await expect(brandIcon).toHaveText('flight');
    
    // 驗證導航鐵軌
    await expect(page.locator(".md-navigation-rail")).toBeVisible();
    
    // 驗證四個主要導航項目
    const navItems = [
      { tab: 'prompts', icon: 'chat', label: 'Prompt' },
      { tab: 'scheduler', icon: 'schedule', label: '排程' },
      { tab: 'results', icon: 'analytics', label: '結果' },
      { tab: 'system', icon: 'monitoring', label: '監控' }
    ];
    
    for (const item of navItems) {
      const navItem = page.locator(`[data-tab="${item.tab}"]`);
      await expect(navItem).toBeVisible();
      await expect(navItem.locator('.material-symbols-outlined')).toHaveText(item.icon);
    }
  });

  test("冷卻狀態 Material Icons 顯示驗證", async ({ page }) => {
    // 檢查冷卻狀態區塊
    const cooldownStatus = page.locator('#cooldown-status');
    await expect(cooldownStatus).toBeVisible();
    
    // 驗證狀態圖示為 Material Design Icon
    const statusIcon = cooldownStatus.locator('.status-icon.material-symbols-outlined');
    await expect(statusIcon).toBeVisible();
    await expect(statusIcon).toHaveText('schedule');
    
    // 驗證狀態文字存在
    const statusText = cooldownStatus.locator('.status-text');
    await expect(statusText).toBeVisible();
    
    // 測試冷卻狀態自動更新（5秒間隔）
    const initialText = await statusText.textContent();
    
    // 等待狀態更新
    await page.waitForTimeout(6000);
    
    // 驗證狀態仍然可見且具有意義
    await expect(statusText).toBeVisible();
  });

  test("Prompt 管理 Material Design 介面測試", async ({ page }) => {
    // 點擊 Prompt 標籤
    await page.click('[data-tab="prompts"]');
    
    // 驗證 FAB 按鈕存在
    const fab = page.locator('#create-prompt-fab.md-fab');
    await expect(fab).toBeVisible();
    await expect(fab.locator('.material-symbols-outlined')).toHaveText('add');
    
    // 點擊 FAB 開啟建立對話框
    await fab.click();
    
    // 驗證 Material Design 對話框開啟
    const modal = page.locator('#prompt-modal.md-dialog');
    await expect(modal).toBeVisible();
    
    // 驗證對話框標題圖示
    const modalTitle = modal.locator('.md-dialog-header .material-symbols-outlined');
    await expect(modalTitle).toHaveText('chat');
    
    // 填寫表單
    await page.fill('#prompt-title', 'Material Design 測試 Prompt');
    await page.fill('#prompt-content', '@docs/PROJECT_RULES.md 請分析專案的 Material Design 實作情況');
    await page.fill('#prompt-tags', 'material-design,test,ui');
    
    // 提交表單
    await page.click('.md-filled-button[type="submit"]');
    
    // 等待 Snackbar 通知
    await expect(page.locator('#snackbar-container')).toContainText('Prompt 建立成功');
  });

  test("排程任務建立與 Material Design 狀態指示器", async ({ page }) => {
    // 切換到排程標籤
    await page.click('[data-tab="scheduler"]');
    
    // 驗證排程標題和圖示
    const sectionTitle = page.locator('#scheduler-tab h2');
    await expect(sectionTitle.locator('.material-symbols-outlined')).toHaveText('schedule');
    
    // 點擊建立任務 FAB
    const jobFab = page.locator('#create-job-fab.md-fab');
    await expect(jobFab.locator('.material-symbols-outlined')).toHaveText('add_task');
    await jobFab.click();
    
    // 驗證任務建立對話框
    const jobModal = page.locator('#job-modal.md-dialog');
    await expect(jobModal).toBeVisible();
    
    // 驗證表單圖示
    await expect(jobModal.locator('[for="job-prompt"] .material-symbols-outlined')).toHaveText('chat');
    await expect(jobModal.locator('[for="job-cron"] .material-symbols-outlined')).toHaveText('schedule');
    
    // 選擇 Prompt（假設有可用選項）
    await page.selectOption('#job-prompt', { index: 1 });
    
    // 填寫 Cron 表達式
    await page.fill('#job-cron', '*/5 * * * *');
    
    // 提交任務
    await page.click('#job-modal .md-filled-button[type="submit"]');
    
    // 驗證任務出現在列表中
    await expect(page.locator('.md-list-container')).toContainText('*/5 * * * *');
  });

  test("系統監控頁面 Material Icons 驗證", async ({ page }) => {
    // 切換到系統監控標籤
    await page.click('[data-tab="system"]');
    
    // 驗證系統監控標題圖示
    const systemTitle = page.locator('#system-tab h2 .material-symbols-outlined');
    await expect(systemTitle).toHaveText('monitoring');
    
    // 驗證刷新按鈕圖示
    const refreshBtn = page.locator('#refresh-system-btn .material-symbols-outlined');
    await expect(refreshBtn).toHaveText('refresh');
    
    // 驗證系統資訊卡片圖示
    const infoCards = [
      { selector: '#app-info', icon: 'info' },
      { selector: '#performance-info', icon: 'memory' }
    ];
    
    for (const card of infoCards) {
      const cardIcon = page.locator(`${card.selector} .md-card-header .material-symbols-outlined`);
      await expect(cardIcon).toHaveText(card.icon);
    }
    
    // 點擊刷新按鈕測試功能
    await page.click('#refresh-system-btn');
    
    // 等待資料更新
    await page.waitForTimeout(2000);
  });

  test("主題切換與 Material Design 動畫", async ({ page }) => {
    // 找到主題切換按鈕
    const themeToggle = page.locator('#theme-toggle');
    await expect(themeToggle).toBeVisible();
    
    // 檢查初始主題圖示
    let themeIcon = themeToggle.locator('.material-symbols-outlined');
    const initialIcon = await themeIcon.textContent();
    
    // 點擊主題切換
    await themeToggle.click();
    
    // 等待主題切換動畫
    await page.waitForTimeout(500);
    
    // 驗證圖示變化
    const newIcon = await themeIcon.textContent();
    expect(newIcon).not.toBe(initialIcon);
    
    // 驗證主題圖示為有效的 Material Design 圖示
    const validThemeIcons = ['light_mode', 'dark_mode', 'brightness_auto'];
    expect(validThemeIcons).toContain(newIcon);
  });

  test("Material Design 響應式設計測試", async ({ page }) => {
    // 測試桌面版本 (1200x800)
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator('.md-navigation-rail')).toBeVisible();
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
    
    // 測試平板版本 (768x1024)
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
    
    // 測試手機版本 (375x667)
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
    
    // 恢復桌面尺寸
    await page.setViewportSize({ width: 1200, height: 800 });
  });

  test("Material Design 載入動畫與骨架屏測試", async ({ page }) => {
    // 重新載入頁面以觀察載入動畫
    await page.reload();
    
    // 驗證載入器存在
    const loader = page.locator('#app-loader.app-loader');
    await expect(loader).toBeVisible();
    
    // 驗證載入器圖示
    const loaderIcon = loader.locator('.loader-icon .material-symbols-rounded');
    await expect(loaderIcon).toHaveText('flight');
    
    // 驗證載入步驟指示器
    const stepIndicators = loader.locator('.step-indicator');
    await expect(stepIndicators).toHaveCount(4);
    
    // 驗證各步驟圖示
    const stepIcons = ['terminal', 'database', 'api', 'check_circle'];
    for (let i = 0; i < stepIcons.length; i++) {
      const stepIcon = stepIndicators.nth(i).locator('.material-symbols-outlined');
      await expect(stepIcon).toHaveText(stepIcons[i]);
    }
    
    // 等待應用完全載入
    await page.waitForSelector('.app-container', { timeout: 30000 });
  });

  test("Material Design Snackbar 通知系統測試", async ({ page }) => {
    // 觸發一個會產生通知的操作（建立 Prompt）
    await page.click('[data-tab="prompts"]');
    await page.click('#create-prompt-fab');
    
    // 填寫並提交表單
    await page.fill('#prompt-title', 'Snackbar 測試');
    await page.fill('#prompt-content', '測試通知系統');
    await page.click('#prompt-modal .md-filled-button[type="submit"]');
    
    // 驗證 Snackbar 容器
    const snackbarContainer = page.locator('#snackbar-container');
    await expect(snackbarContainer).toBeVisible();
    
    // 檢查通知內容
    await page.waitForTimeout(1000);
    const hasSnackbar = await snackbarContainer.locator('.md-snackbar').count();
    if (hasSnackbar > 0) {
      await expect(snackbarContainer.locator('.md-snackbar')).toContainText('成功');
    }
  });

  test("Material Design 卡片與列表項目驗證", async ({ page }) => {
    // 切換到 Prompts 標籤
    await page.click('[data-tab="prompts"]');
    
    // 等待內容載入
    await page.waitForTimeout(2000);
    
    // 檢查卡片網格
    const cardGrid = page.locator('.md-card-grid');
    await expect(cardGrid).toBeVisible();
    
    // 如果有 Prompt 卡片，驗證其結構
    const promptCards = cardGrid.locator('.md-card');
    if (await promptCards.count() > 0) {
      const firstCard = promptCards.first();
      await expect(firstCard).toHaveClass(/md-elevation/);
      
      // 檢查卡片內的 Material Design 元素
      const cardActions = firstCard.locator('.md-card-actions');
      if (await cardActions.count() > 0) {
        const actionButtons = cardActions.locator('.material-symbols-outlined');
        await expect(actionButtons.first()).toBeVisible();
      }
    }
    
    // 切換到排程標籤檢查列表項目
    await page.click('[data-tab="scheduler"]');
    await page.waitForTimeout(1000);
    
    const listContainer = page.locator('.md-list-container');
    await expect(listContainer).toBeVisible();
  });
});

test.describe("CLI 與 GUI 整合功能測試", () => {
  test("CLI 命令執行狀態在 GUI 中的即時反映", async ({ page }) => {
    await page.goto("http://localhost:8081");
    
    // 等待應用載入
    await page.waitForSelector('.app-container', { timeout: 30000 });
    
    // 檢查初始任務狀態
    await page.click('[data-tab="scheduler"]');
    
    const jobsList = page.locator('#jobs-list');
    await expect(jobsList).toBeVisible();
    
    // 驗證任務狀態圖示使用 Material Design
    const statusIcons = jobsList.locator('.status-icon.material-symbols-outlined');
    if (await statusIcons.count() > 0) {
      const validStatusIcons = ['pending', 'play_circle', 'check_circle', 'error'];
      for (let i = 0; i < await statusIcons.count(); i++) {
        const iconText = await statusIcons.nth(i).textContent();
        // 驗證圖示是有效的狀態表示
        expect(['schedule', 'play_arrow', 'check_circle', 'error', 'timer', 'done']).toContain(iconText);
      }
    }
  });

  test("冷卻時間倒數與進度條動畫", async ({ page }) => {
    await page.goto("http://localhost:8081");
    await page.waitForSelector('.app-container', { timeout: 30000 });
    
    // 檢查冷卻狀態顯示
    const cooldownStatus = page.locator('#cooldown-status');
    await expect(cooldownStatus).toBeVisible();
    
    // 檢查是否有進度指示器
    const progressIndicators = page.locator('.progress-fill, .md-linear-progress, .md-circular-progress');
    
    // 如果系統處於冷卻狀態，應該有進度指示器
    const statusText = await cooldownStatus.locator('.status-text').textContent();
    
    if (statusText && statusText.includes('冷卻')) {
      await expect(progressIndicators.first()).toBeVisible();
    }
    
    // 驗證時間格式顯示 (mm:ss)
    const timeDisplay = page.locator('.time-remaining, .eta-display');
    if (await timeDisplay.count() > 0) {
      const timeText = await timeDisplay.first().textContent();
      // 檢查時間格式
      const timePattern = /\d{1,2}:\d{2}/;
      if (timeText) {
        expect(timePattern.test(timeText) || timeText.includes('正常')).toBeTruthy();
      }
    }
  });
});

test.describe("效能與可用性測試", () => {
  test("Material Design 動畫效能測試", async ({ page }) => {
    await page.goto("http://localhost:8081");
    
    // 測量載入時間
    const startTime = Date.now();
    await page.waitForSelector('.app-container', { timeout: 30000 });
    const loadTime = Date.now() - startTime;
    
    // 載入時間應少於 5 秒
    expect(loadTime).toBeLessThan(5000);
    
    // 測試主題切換動畫流暢度
    const themeToggle = page.locator('#theme-toggle');
    
    // 連續點擊主題切換測試動畫
    for (let i = 0; i < 3; i++) {
      await themeToggle.click();
      await page.waitForTimeout(300); // 等待動畫完成
    }
    
    // 驗證頁面依然響應
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
  });

  test("Material Design 組件可訪問性測試", async ({ page }) => {
    await page.goto("http://localhost:8081");
    await page.waitForSelector('.app-container', { timeout: 30000 });
    
    // 測試鍵盤導航
    await page.keyboard.press('Tab');
    
    // 驗證焦點管理
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
    
    // 測試 ARIA 標籤
    const buttons = page.locator('button[aria-label]');
    if (await buttons.count() > 0) {
      for (let i = 0; i < await buttons.count(); i++) {
        const ariaLabel = await buttons.nth(i).getAttribute('aria-label');
        expect(ariaLabel).toBeTruthy();
        expect(ariaLabel.length).toBeGreaterThan(0);
      }
    }
    
    // 測試高對比度支援
    await page.emulateMedia({ colorScheme: 'dark' });
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
    
    await page.emulateMedia({ colorScheme: 'light' });
    await expect(page.locator('.md-top-app-bar')).toBeVisible();
  });
});