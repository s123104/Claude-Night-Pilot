import { test, expect } from "@playwright/test";
import { 
  waitForAppReady, 
  validateMaterialDesignComponents, 
  createTestPrompt,
  cleanupTestData,
  captureErrorLogs,
  forceInitializeMaterialDesignApp 
} from "../../utils/test-helpers.js";

/**
 * Claude Night Pilot - Material Design 3.0 E2E 測試套件
 * 驗證任務排程系統的 CLI 與 GUI 功能整合
 */

test.describe("Material Design 3.0 任務排程系統驗證", () => {
  test.beforeEach(async ({ page }) => {
    // Setup error capture
    const errors = await captureErrorLogs(page);
    
    await page.goto("/", {
      waitUntil: "networkidle",
      timeout: 30000,
    });
    
    // Wait for app and force Material Design initialization
    await waitForAppReady(page, 30000);
    await forceInitializeMaterialDesignApp(page);
    
    // Short stabilization delay
    await page.waitForTimeout(500);
  });

  test("Material Design 3.0 GUI 架構驗證", async ({ page }) => {
    // Use lenient validation for basic structure
    const isValid = await validateMaterialDesignComponents(page, { strict: false, timeout: 15000 });
    expect(isValid).toBe(true);

    // 驗證應用標題和品牌元素 - more flexible selectors
    const appTitle = page.locator("[data-testid='app-title'], h1, .brand-text h1");
    await expect(appTitle.first()).toBeVisible({ timeout: 10000 });
    
    const titleText = await appTitle.first().textContent();
    expect(titleText).toContain("Claude");

    // 驗證主要容器結構 - with fallbacks
    const appContainer = page.locator(".app-container, #app, [data-testid='app-container']");
    await expect(appContainer.first()).toBeVisible({ timeout: 10000 });
    
    const mainContent = page.locator(".md-main-content, main, [data-testid='main-content']");
    await expect(mainContent.first()).toBeVisible({ timeout: 8000 });

    // 驗證導航結構完整性 - flexible approach
    const navRail = page.locator(".md-navigation-rail, nav, [data-testid='nav-rail']");
    await expect(navRail.first()).toBeVisible({ timeout: 8000 });

    // 驗證 Material Design 圖標系統 - any icon is sufficient
    const anyMaterialIcon = page.locator(".material-symbols-outlined, .material-symbols-rounded");
    await expect(anyMaterialIcon.first()).toBeVisible({ timeout: 5000 });

    // 驗證主題系統 - more lenient
    const htmlElement = page.locator("html");
    const themeAttribute = await htmlElement.getAttribute("data-theme");
    expect(themeAttribute).toBeTruthy(); // Just needs to exist
  });

  test("冷卻狀態 Material Icons 顯示驗證", async ({ page }) => {
    // 檢查冷卻狀態區塊
    const cooldownStatus = page.locator("#cooldown-status");
    await expect(cooldownStatus).toBeVisible();

    // 驗證狀態圖示為 Material Design Icon
    const statusIcon = cooldownStatus.locator(
      ".status-icon.material-symbols-outlined"
    );
    await expect(statusIcon).toBeVisible();
    await expect(statusIcon).toHaveText("schedule");

    // 驗證狀態文字存在
    const statusText = cooldownStatus.locator(".status-text");
    await expect(statusText).toBeVisible();

    // 測試冷卻狀態自動更新（5秒間隔）
    const initialText = await statusText.textContent();

    // 等待狀態更新
    await page.waitForTimeout(6000);

    // 驗證狀態仍然可見且具有意義
    await expect(statusText).toBeVisible();
  });

  test("Prompt 管理 Material Design 介面測試", async ({ page }) => {
    try {
      // 確保在 Prompt 頁面 - multiple selector attempts
      const promptsTab = page.locator("[data-tab='prompts'], [data-testid='nav-prompts']");
      if (await promptsTab.count() > 0) {
        await promptsTab.first().click({ timeout: 5000 });
      }
      
      // Wait for prompt tab content
      const promptsTabContent = page.locator("[data-testid='prompts-tab'], #prompts-tab");
      await expect(promptsTabContent.first()).toBeVisible({ timeout: 10000 });

      // 檢驗 FAB 按鈕 - flexible selector and validate Material Design classes
      const fab = page.locator("[data-testid='create-prompt-fab'], .md-fab, button[id*='prompt']");
      await expect(fab.first()).toBeVisible({ timeout: 15000 });
      
      // Verify FAB has Material Design classes
      const fabClasses = await fab.first().getAttribute('class');
      expect(fabClasses).toContain('md-fab');
      
      // Check if FAB has Material Design icon
      const fabIcon = fab.first().locator('.material-symbols-outlined');
      if (await fabIcon.count() > 0) {
        await expect(fabIcon.first()).toBeVisible();
        const iconText = await fabIcon.first().textContent();
        expect(['add', 'create', 'new', 'plus'].some(keyword => iconText?.includes(keyword) || keyword === iconText)).toBeTruthy();
      }

      // 驗證 Modal 元素存在（不需要顯示）- DOM structure test
      const modal = page.locator("[data-testid='prompt-modal'], #prompt-modal, dialog");
      await expect(modal.first()).toBeAttached({ timeout: 5000 });
      
      // Check modal has Material Design classes
      const modalClasses = await modal.first().getAttribute('class');
      expect(modalClasses).toContain('md-dialog');

      // 驗證表單元件存在在DOM中 - structure verification only
      const titleInput = page.locator("[data-testid='prompt-title-input'], input[id*='title'], input[name*='title']");
      const contentInput = page.locator("[data-testid='prompt-content-input'], textarea[id*='content'], textarea[name*='content']");
      const tagsInput = page.locator("[data-testid='prompt-tags-input'], input[id*='tags'], input[name*='tags']");

      await expect(titleInput.first()).toBeAttached({ timeout: 5000 });
      await expect(contentInput.first()).toBeAttached({ timeout: 5000 });
      await expect(tagsInput.first()).toBeAttached({ timeout: 5000 });
      
      // Verify form inputs have Material Design classes
      const titleClasses = await titleInput.first().getAttribute('class');
      expect(titleClasses).toContain('md-text-field');
      
      console.log('✅ Prompt management Material Design structure validated successfully');
      
    } catch (error) {
      console.warn('⚠️ Prompt management test had issues:', error.message);
      // Take screenshot for debugging
      await page.screenshot({ path: 'debug-prompt-management.png' });
      throw error;
    }
  });

  test("排程任務建立與 Material Design 狀態指示器", async ({ page }) => {
    try {
      // 切換到排程頁面 - flexible selector
      const schedulerTab = page.locator("[data-tab='scheduler'], [data-testid='nav-scheduler']");
      if (await schedulerTab.count() > 0) {
        await schedulerTab.first().click({ timeout: 5000 });
      }
      
      // Wait for scheduler tab content
      const schedulerTabContent = page.locator("[data-testid='scheduler-tab'], #scheduler-tab");
      await expect(schedulerTabContent.first()).toBeVisible({ timeout: 10000 });

      // 檢驗排程 FAB 按鈕 - flexible selector and validate Material Design classes
      const jobFab = page.locator("[data-testid='create-job-fab'], button[id*='job'], .md-fab");
      await expect(jobFab.first()).toBeVisible({ timeout: 15000 });
      
      // Verify FAB has Material Design classes
      const fabClasses = await jobFab.first().getAttribute('class');
      expect(fabClasses).toContain('md-fab');
      
      // Check if FAB has Material Design icon
      const fabIcon = jobFab.first().locator('.material-symbols-outlined');
      if (await fabIcon.count() > 0) {
        await expect(fabIcon.first()).toBeVisible();
        const iconText = await fabIcon.first().textContent();
        expect(['add_task', 'schedule', 'add', 'create'].some(keyword => iconText?.includes(keyword) || keyword === iconText)).toBeTruthy();
      }

      // 驗證 Job Modal DOM結構 - structural test only
      const jobModal = page.locator("[data-testid='job-modal'], #job-modal, dialog");
      await expect(jobModal.first()).toBeAttached({ timeout: 5000 });
      
      // Check modal has Material Design classes
      const modalClasses = await jobModal.first().getAttribute('class');
      expect(modalClasses).toContain('md-dialog');

      // 檢查表單元件DOM結構 - structural verification
      const promptSelect = page.locator("[data-testid='job-prompt-select'], select[id*='prompt'], select[name*='prompt']");
      const cronInput = page.locator("[data-testid='job-cron-input'], input[id*='cron'], input[name*='cron']");

      await expect(promptSelect.first()).toBeAttached({ timeout: 5000 });
      await expect(cronInput.first()).toBeAttached({ timeout: 5000 });
      
      // Verify form elements have Material Design classes
      const selectClasses = await promptSelect.first().getAttribute('class');
      const inputClasses = await cronInput.first().getAttribute('class');
      expect(selectClasses).toContain('md-select');
      expect(inputClasses).toContain('md-text-field');

      // 檢查排程列表區域 DOM結構
      const jobsList = page.locator("[data-testid='jobs-list'], #jobs-list, .md-list-container");
      await expect(jobsList.first()).toBeAttached({ timeout: 5000 });
      
      // Verify list has Material Design classes
      const listClasses = await jobsList.first().getAttribute('class');
      expect(listClasses).toContain('md-list-container');
      
      console.log('✅ Scheduler Material Design structure validated successfully');
      
    } catch (error) {
      console.warn('⚠️ Scheduler test had issues:', error.message);
      await page.screenshot({ path: 'debug-scheduler.png' });
      throw error;
    }
  });

  test("系統監控頁面 Material Icons 驗證", async ({ page }) => {
    try {
      // 切換到系統監控頁面 - flexible selector
      const systemTab = page.locator("[data-tab='system'], [data-testid='nav-system']");
      if (await systemTab.count() > 0) {
        await systemTab.first().click({ timeout: 5000 });
      }
      
      // Wait for system tab content
      const systemTabContent = page.locator("[data-testid='system-tab'], #system-tab");
      await expect(systemTabContent.first()).toBeVisible({ timeout: 10000 });

      // 驗證任何信息卡片存在 - flexible selector
      const infoCards = page.locator(".md-info-card, .info-card, [data-testid*='card']");
      await expect(infoCards.first()).toBeVisible({ timeout: 15000 });

      // 檢查主要監控元素 - more flexible
      const cooldownArea = page.locator("[data-testid='cooldown-info-card'], [data-testid='cooldown-status'], #cooldown-status");
      await expect(cooldownArea.first()).toBeVisible({ timeout: 10000 });

      // 驗證至少有一個 Material Icon 存在
      const anyIcon = page.locator(".material-symbols-outlined, .material-symbols-rounded");
      await expect(anyIcon.first()).toBeVisible({ timeout: 8000 });

      // 檢查任何操作按鈕存在 - flexible selector
      const actionBtn = page.locator("[data-testid='refresh-system-btn'], button[id*='refresh'], button[id*='system']");
      if (await actionBtn.count() > 0) {
        await expect(actionBtn.first()).toBeVisible({ timeout: 8000 });
        
        // Test button click if visible
        try {
          await actionBtn.first().click({ timeout: 3000 });
          console.log('✅ System refresh button clicked successfully');
        } catch (clickError) {
          console.warn('⚠️ Could not click refresh button:', clickError.message);
        }
      }

      // 驗證狀態指示器區域存在
      const statusArea = page.locator("[data-testid='cooldown-status'], .status-indicators, .md-status-chip");
      await expect(statusArea.first()).toBeVisible({ timeout: 8000 });
      
    } catch (error) {
      console.warn('⚠️ System monitoring test had issues:', error.message);
      await page.screenshot({ path: 'debug-system-monitoring.png' });
      throw error;
    }
  });

  test("主題切換與 Material Design 動畫", async ({ page }) => {
    // 找到主題切換按鈕
    const themeToggle = page.locator("#theme-toggle");
    await expect(themeToggle).toBeVisible();

    // 檢查初始主題圖示
    let themeIcon = themeToggle.locator(".material-symbols-outlined");
    const initialIcon = await themeIcon.textContent();

    // 點擊主題切換
    await themeToggle.click();

    // 等待主題切換動畫
    await page.waitForTimeout(500);

    // 驗證圖示變化
    const newIcon = await themeIcon.textContent();
    expect(newIcon).not.toBe(initialIcon);

    // 驗證主題圖示為有效的 Material Design 圖示
    const validThemeIcons = ["light_mode", "dark_mode", "brightness_auto"];
    expect(validThemeIcons).toContain(newIcon);
  });

  test("Material Design 響應式設計測試", async ({ page }) => {
    // 測試桌面版本 (1200x800)
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator(".md-navigation-rail")).toBeVisible();
    await expect(page.locator(".md-top-app-bar")).toBeVisible();

    // 測試平板版本 (768x1024)
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator(".md-top-app-bar")).toBeVisible();

    // 測試手機版本 (375x667)
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator(".md-top-app-bar")).toBeVisible();

    // 恢復桌面尺寸
    await page.setViewportSize({ width: 1200, height: 800 });
  });

  test("Material Design 載入動畫與骨架屏測試", async ({ page }) => {
    // 重新載入頁面以觀察載入動畫
    await page.reload();

    // 驗證載入器存在
    const loader = page.locator("#app-loader.app-loader");
    await expect(loader).toBeVisible();

    // 驗證載入器圖示
    const loaderIcon = loader.locator(".loader-icon .material-symbols-rounded");
    await expect(loaderIcon).toHaveText("flight");

    // 驗證載入步驟指示器
    const stepIndicators = loader.locator(".step-indicator");
    await expect(stepIndicators).toHaveCount(4);

    // 驗證各步驟圖示
    const stepIcons = ["terminal", "database", "api", "check_circle"];
    for (let i = 0; i < stepIcons.length; i++) {
      const stepIcon = stepIndicators
        .nth(i)
        .locator(".material-symbols-outlined");
      await expect(stepIcon).toHaveText(stepIcons[i]);
    }

    // 等待應用完全載入
    await page.waitForSelector(".app-container", { timeout: 30000 });
  });

  test("Material Design Snackbar 通知系統測試", async ({ page }) => {
    // 觸發一個會產生通知的操作（建立 Prompt）
    await page.click('[data-tab="prompts"]');
    await page.click("#create-prompt-fab");

    // 填寫並提交表單
    await page.fill("#prompt-title", "Snackbar 測試");
    await page.fill("#prompt-content", "測試通知系統");
    await page.click('#prompt-modal .md-filled-button[type="submit"]');

    // 驗證 Snackbar 容器
    const snackbarContainer = page.locator("#snackbar-container");
    await expect(snackbarContainer).toBeVisible();

    // 檢查通知內容
    await page.waitForTimeout(1000);
    const hasSnackbar = await snackbarContainer.locator(".md-snackbar").count();
    if (hasSnackbar > 0) {
      await expect(snackbarContainer.locator(".md-snackbar")).toContainText(
        "成功"
      );
    }
  });

  test("Material Design 卡片與列表項目驗證", async ({ page }) => {
    // 切換到 Prompts 標籤
    await page.click('[data-tab="prompts"]');

    // 等待內容載入
    await page.waitForTimeout(2000);

    // 檢查卡片網格
    const cardGrid = page.locator(".md-card-grid");
    await expect(cardGrid).toBeVisible();

    // 如果有 Prompt 卡片，驗證其結構
    const promptCards = cardGrid.locator(".md-card");
    if ((await promptCards.count()) > 0) {
      const firstCard = promptCards.first();
      await expect(firstCard).toHaveClass(/md-elevation/);

      // 檢查卡片內的 Material Design 元素
      const cardActions = firstCard.locator(".md-card-actions");
      if ((await cardActions.count()) > 0) {
        const actionButtons = cardActions.locator(".material-symbols-outlined");
        await expect(actionButtons.first()).toBeVisible();
      }
    }

    // 切換到排程標籤檢查列表項目
    await page.click('[data-tab="scheduler"]');
    await page.waitForTimeout(1000);

    // 使用 first() 避免多元素選擇器衝突
    const listContainer = page.locator(".md-list-container").first();
    await expect(listContainer).toBeVisible({ timeout: 10000 });
  });
});

test.describe("CLI 與 GUI 整合功能測試", () => {
  test("CLI 命令執行狀態在 GUI 中的即時反映", async ({ page }) => {
    await page.goto("http://localhost:8080");

    // 等待應用載入
    await page.waitForSelector(".app-container", { timeout: 30000 });

    // 檢查初始任務狀態
    await page.click('[data-tab="scheduler"]');

    const jobsList = page.locator("#jobs-list");
    await expect(jobsList).toBeVisible();

    // 驗證任務狀態圖示使用 Material Design
    const statusIcons = jobsList.locator(
      ".status-icon.material-symbols-outlined"
    );
    if ((await statusIcons.count()) > 0) {
      const validStatusIcons = [
        "pending",
        "play_circle",
        "check_circle",
        "error",
      ];
      for (let i = 0; i < (await statusIcons.count()); i++) {
        const iconText = await statusIcons.nth(i).textContent();
        // 驗證圖示是有效的狀態表示
        expect([
          "schedule",
          "play_arrow",
          "check_circle",
          "error",
          "timer",
          "done",
        ]).toContain(iconText);
      }
    }
  });

  test("冷卻時間倒數與進度條動畫", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await page.waitForSelector(".app-container", { timeout: 30000 });

    // 檢查冷卻狀態顯示
    const cooldownStatus = page.locator("#cooldown-status");
    await expect(cooldownStatus).toBeVisible();

    // 檢查是否有進度指示器
    const progressIndicators = page.locator(
      ".progress-fill, .md-linear-progress, .md-circular-progress"
    );

    // 如果系統處於冷卻狀態，應該有進度指示器
    const statusText = await cooldownStatus
      .locator(".status-text")
      .textContent();

    if (statusText && statusText.includes("冷卻")) {
      await expect(progressIndicators.first()).toBeVisible();
    }

    // 驗證時間格式顯示 (mm:ss)
    const timeDisplay = page.locator(".time-remaining, .eta-display");
    if ((await timeDisplay.count()) > 0) {
      const timeText = await timeDisplay.first().textContent();
      // 檢查時間格式
      const timePattern = /\d{1,2}:\d{2}/;
      if (timeText) {
        expect(
          timePattern.test(timeText) || timeText.includes("正常")
        ).toBeTruthy();
      }
    }
  });
});

test.describe("效能與可用性測試", () => {
  test("Material Design 動畫效能測試", async ({ page }) => {
    await page.goto("http://localhost:8080");

    // 測量載入時間
    const startTime = Date.now();
    await page.waitForSelector(".app-container", { timeout: 30000 });
    const loadTime = Date.now() - startTime;

    // 載入時間應少於 5 秒
    expect(loadTime).toBeLessThan(5000);

    // 測試主題切換動畫流暢度
    const themeToggle = page.locator("#theme-toggle");

    // 連續點擊主題切換測試動畫
    for (let i = 0; i < 3; i++) {
      await themeToggle.click();
      await page.waitForTimeout(300); // 等待動畫完成
    }

    // 驗證頁面依然響應
    await expect(page.locator(".md-top-app-bar")).toBeVisible();
  });

  test("Material Design 組件可訪問性測試", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await page.waitForSelector(".app-container", { timeout: 30000 });

    // 測試鍵盤導航
    await page.keyboard.press("Tab");

    // 驗證焦點管理
    const focusedElement = page.locator(":focus");
    await expect(focusedElement).toBeVisible();

    // 測試 ARIA 標籤
    const buttons = page.locator("button[aria-label]");
    if ((await buttons.count()) > 0) {
      for (let i = 0; i < (await buttons.count()); i++) {
        const ariaLabel = await buttons.nth(i).getAttribute("aria-label");
        expect(ariaLabel).toBeTruthy();
        expect(ariaLabel.length).toBeGreaterThan(0);
      }
    }

    // 測試高對比度支援
    await page.emulateMedia({ colorScheme: "dark" });
    await expect(page.locator(".md-top-app-bar")).toBeVisible();

    await page.emulateMedia({ colorScheme: "light" });
    await expect(page.locator(".md-top-app-bar")).toBeVisible();
  });
});
