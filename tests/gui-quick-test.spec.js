import { test, expect } from "@playwright/test";

test.describe("Claude Night Pilot - 快速 GUI 測試", () => {
  test.beforeEach(async ({ page }) => {
    // 設置測試環境
    await page.goto("http://localhost:8081");
    await page.waitForSelector('h1:has-text("Claude Night Pilot")', {
      timeout: 30000,
    });
  });

  test("應能切換到測試標籤頁", async ({ page }) => {
    // 點擊測試標籤
    await page.click('[data-tab="testing"]');

    // 等待測試標籤頁內容顯示
    await page.waitForSelector('[data-testid="core-001-section"]', {
      timeout: 10000,
    });

    // 驗證四大核心模組都存在
    await expect(
      page.locator('[data-testid="core-001-section"]')
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="core-002-section"]')
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="core-003-section"]')
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="core-004-section"]')
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="integration-section"]')
    ).toBeVisible();
  });

  test("CORE-001 模組應能點擊檢查使用量按鈕", async ({ page }) => {
    // 切換到測試標籤頁
    await page.click('[data-tab="testing"]');
    await page.waitForSelector('[data-testid="core-001-section"]', {
      timeout: 10000,
    });

    // 驗證檢查使用量按鈕存在且可點擊
    const checkUsageBtn = page.locator('[data-testid="check-usage"]');
    await expect(checkUsageBtn).toBeVisible();
    await expect(checkUsageBtn).toBeEnabled();

    // 點擊按鈕（但不期待特定結果，只測試基本功能）
    await checkUsageBtn.click();

    // 確保沒有崩潰或錯誤
    await page.waitForTimeout(1000);
  });

  test("CORE-002 模組應能輸入提示並顯示選項", async ({ page }) => {
    // 切換到測試標籤頁
    await page.click('[data-tab="testing"]');
    await page.waitForSelector('[data-testid="core-002-section"]', {
      timeout: 10000,
    });

    // 測試提示輸入框
    const promptInput = page.locator('[data-testid="prompt-input"]');
    await expect(promptInput).toBeVisible();
    await promptInput.fill("測試提示內容");

    // 測試複選框
    await expect(
      page.locator('[data-testid="skip-permissions"]')
    ).toBeVisible();
    await expect(page.locator('[data-testid="dry-run"]')).toBeVisible();
    await expect(page.locator('[data-testid="enable-security"]')).toBeVisible();

    // 測試執行按鈕
    await expect(page.locator('[data-testid="execute-prompt"]')).toBeVisible();
  });

  test("CORE-003 監控模組應顯示狀態信息", async ({ page }) => {
    // 切換到測試標籤頁
    await page.click('[data-tab="testing"]');
    await page.waitForSelector('[data-testid="core-003-section"]', {
      timeout: 10000,
    });

    // 驗證監控狀態顯示
    await expect(page.locator('[data-testid="monitor-status"]')).toBeVisible();
    await expect(
      page.locator('[data-testid="monitor-interval"]')
    ).toBeVisible();

    // 驗證監控控制按鈕
    await expect(
      page.locator('[data-testid="start-monitoring"]')
    ).toBeVisible();
    await expect(page.locator('[data-testid="update-monitor"]')).toBeVisible();
    await expect(page.locator('[data-testid="check-monitor"]')).toBeVisible();
  });

  test("CORE-004 排程模組應提供完整表單", async ({ page }) => {
    // 切換到測試標籤頁
    await page.click('[data-tab="testing"]');
    await page.waitForSelector('[data-testid="core-004-section"]', {
      timeout: 10000,
    });

    // 驗證排程表單元素
    await expect(page.locator('[data-testid="schedule-prompt"]')).toBeVisible();
    await expect(page.locator('[data-testid="schedule-time"]')).toBeVisible();
    await expect(page.locator('[data-testid="timezone-select"]')).toBeVisible();
    await expect(
      page.locator('[data-testid="required-minutes"]')
    ).toBeVisible();

    // 測試填寫表單
    await page.fill('[data-testid="schedule-prompt"]', "測試排程");
    await page.fill('[data-testid="required-minutes"]', "60");

    // 驗證按鈕存在
    await expect(page.locator('[data-testid="create-schedule"]')).toBeVisible();
    await expect(
      page.locator('[data-testid="analyze-efficiency"]')
    ).toBeVisible();
  });
});
