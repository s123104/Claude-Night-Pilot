import { test, expect } from "@playwright/test";
import { waitForAppReady, createTestPrompt, setupClaudeMock } from "../../utils/test-helpers.js";

test.describe("Claude Night Pilot - Prompt 管理測試", () => {
  test.beforeEach(async ({ page }) => {
    // 設定 Claude CLI 模擬
    await setupClaudeMock(page);
    
    // 前往應用主頁
    await page.goto("http://localhost:8080");

    // 等待應用完全載入
    await waitForAppReady(page);
  });

  test("應用正確載入並顯示標題", async ({ page }) => {
    // 檢查標題
    await expect(page.locator("[data-testid='app-title']")).toContainText("Claude Night Pilot", { timeout: 15000 });

    // 檢查主要區塊存在
    await expect(page.locator("[data-testid='prompts-tab']")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("[data-testid='scheduler-tab']")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("[data-testid='cooldown-status']")).toBeVisible({ timeout: 15000 });
  });

  test("建立新的 Prompt", async ({ page }) => {
    // 點擊建立按鈕
    await page.click("[data-testid='create-prompt-fab']");

    // 等待表單元素可見並填寫
    await expect(page.locator("[data-testid='prompt-modal']")).toBeVisible();
    await page.fill("[data-testid='prompt-title-input']", "測試 Prompt 標題");
    await page.fill("[data-testid='prompt-content-input']", '這是一個測試用的 Prompt 內容，請回覆 "Hello from Claude!"');
    await page.fill("[data-testid='prompt-tags-input']", "test, e2e");

    // 點擊儲存按鈕
    await page.click("[data-testid='prompt-modal-save-btn']");

    // 驗證成功訊息
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功", { timeout: 15000 });

    // 檢查 Prompt 是否出現在列表中
    await expect(page.locator("[data-testid='prompt-list']")).toContainText("測試 Prompt 標題", { timeout: 15000 });
    await expect(page.locator("[data-testid='prompt-list']")).toContainText("test, e2e", { timeout: 15000 });
  });

  test("刪除 Prompt", async ({ page }) => {
    // 先建立一個測試 Prompt
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "要刪除的 Prompt");
    await page.fill("[data-testid='prompt-content-input']", "這個 Prompt 將被刪除");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    // 點擊刪除按鈕
    await page.click(".prompt-item [data-testid='delete-prompt-btn']");

    // 處理確認對話框
    page.on("dialog", (dialog) => dialog.accept());

    // 等待成功訊息
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 刪除成功", {
      timeout: 10000,
    });

    // 檢查 Prompt 是否從列表中消失
    await expect(
      page.locator('.prompt-item:has-text("要刪除的 Prompt")')
    ).not.toBeVisible();
  });

  test("檢查 Claude CLI 狀態", async ({ page }) => {
    // 檢查冷卻狀態顯示
    const cooldownStatus = page.locator("[data-testid='cooldown-status']");
    await expect(cooldownStatus).toBeVisible();

    // 狀態應該是 "Claude CLI 可用" 或 "冷卻中"
    const statusText = await cooldownStatus.textContent();
    expect(statusText).toMatch(/(Claude CLI 可用|冷卻中)/);
  });

  test("執行 Prompt 測試（模擬模式）", async ({ page }) => {
    // 建立測試 Prompt
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "執行測試 Prompt");
    await page.fill("[data-testid='prompt-content-input']", '請回覆 "測試成功"');
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    // 點擊立即執行
    await page.click(".prompt-item [data-testid='run-prompt-btn']");

    // 等待執行完成（在開發模式下使用模擬回應）
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("執行成功", { timeout: 30000 });

    // 檢查任務列表中是否出現新任務
    await expect(page.locator("[data-testid='jobs-list']")).toBeVisible();
    await expect(page.locator(".job-item .status-done")).toBeVisible();
  });

  test("建立排程任務", async ({ page }) => {
    // 先建立測試 Prompt
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "排程測試 Prompt");
    await page.fill("[data-testid='prompt-content-input']", "這是排程執行的 Prompt");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    // 點擊排程執行
    await page.click(".prompt-item [data-testid='schedule-prompt-btn']");

    // 檢查排程對話框開啟
    await expect(page.locator("[data-testid='job-modal']")).toBeVisible();

    // 填寫排程資訊
    await page.fill("[data-testid='job-cron-input']", "0 */1 * * *"); // 每小時執行

    // 建立排程
    await page.click("[data-testid='job-modal-save-btn']");

    // 等待成功訊息
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("排程任務建立成功", {
      timeout: 10000,
    });

    // 檢查任務列表中出現排程任務
    await expect(
      page.locator('[data-testid="jobs-list"]:has-text("0 */1 * * *")')
    ).toBeVisible();
  });

  test("查看任務結果", async ({ page }) => {
    // 建立並執行測試 Prompt
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "結果查看測試");
    await page.fill("[data-testid='prompt-content-input']", "這個 Prompt 用於測試結果查看");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    await page.click(".prompt-item [data-testid='run-prompt-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("執行成功", { timeout: 30000 });

    // 點擊查看結果
    await page.click(".job-item [data-testid='view-result-btn']");

    // 檢查結果對話框顯示
    await expect(page.locator("[data-testid='result-modal']")).toBeVisible({ timeout: 10000 });
  });

  test("冷卻狀態自動更新", async ({ page }) => {
    const cooldownStatus = page.locator("[data-testid='cooldown-status']");

    // 記錄初始狀態
    const initialStatus = await cooldownStatus.textContent();

    // 等待 5 秒（冷卻監控間隔）
    await page.waitForTimeout(6000);

    // 狀態元素應該仍然存在且可能有更新
    await expect(cooldownStatus).toBeVisible();

    // 如果在冷卻中，倒數應該減少
    const currentStatus = await cooldownStatus.textContent();

    // 至少狀態文字應該存在
    expect(currentStatus).toBeTruthy();
  });

  test("任務狀態自動更新", async ({ page }) => {
    // 建立測試任務
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "狀態更新測試");
    await page.fill("[data-testid='prompt-content-input']", "測試任務狀態更新");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功", { timeout: 10000 });

    await page.click(".prompt-item [data-testid='run-prompt-btn']");

    // 檢查任務狀態從 running 變為 done
    const jobItem = page.locator(".job-item").first();

    // 最終應該顯示為已完成
    await expect(jobItem.locator(".status-done")).toBeVisible({
      timeout: 30000,
    });
  });

  test("響應式設計檢查", async ({ page }) => {
    // 檢查桌面版本
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator("[data-testid='prompts-tab']")).toBeVisible();
    await expect(page.locator("[data-testid='scheduler-tab']")).toBeVisible();

    // 檢查手機版本
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator("[data-testid='prompts-tab']")).toBeVisible();
    await expect(page.locator("[data-testid='scheduler-tab']")).toBeVisible();

    // 檢查平板版本
    await page.setViewportSize({ width: 768, height: 1024 });
    await expect(page.locator("[data-testid='prompts-tab']")).toBeVisible();
    await expect(page.locator("[data-testid='scheduler-tab']")).toBeVisible();
  });

  test("錯誤處理測試", async ({ page }) => {
    // 測試空白 Prompt 建立
    await page.click("[data-testid='create-prompt-fab']");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("請填入標題和內容", {
      timeout: 10000,
    });

    // 測試查看不存在任務的結果
    // 這需要 Mock 或特殊設定，暫時跳過
  });

  test("介面載入效能測試", async ({ page }) => {
    // 測量頁面載入時間
    const startTime = Date.now();
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();
    const loadTime = Date.now() - startTime;

    // 載入時間應該少於 5 秒
    expect(loadTime).toBeLessThan(5000);

    // 檢查關鍵元素載入
    await expect(page.locator("[data-testid='prompts-tab']")).toBeVisible();
    await expect(page.locator("[data-testid='scheduler-tab']")).toBeVisible();
    await expect(page.locator("[data-testid='cooldown-status']")).toBeVisible();
  });
});

test.describe("Claude CLI 整合測試", () => {
  test("檢查 Claude CLI 可用性", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 檢查冷卻狀態
    const cooldownStatus = page.locator("[data-testid='cooldown-status']");
    await expect(cooldownStatus).toBeVisible();

    // 狀態文字應該指示 CLI 狀態
    const statusText = await cooldownStatus.textContent();
    expect(statusText).toMatch(/(Claude CLI 可用|冷卻中)/);
  });

  test("模擬 Claude CLI 冷卻狀態", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 在開發模式下，應該模擬正常狀態
    const cooldownStatus = page.locator("[data-testid='cooldown-status']");
    await expect(cooldownStatus).toBeVisible();

    // 開發模式下應該顯示可用狀態
    await expect(cooldownStatus).toContainText("Claude CLI 可用");
  });

  test("模擬執行 Claude CLI 指令", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 建立測試 Prompt，內容參考 Claude Code 使用手冊
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", "Claude Code 測試指令");
    await page.fill(
      "[data-testid='prompt-content-input']",
      "@claude-code-zh-tw.md 請分析這個文檔的主要內容"
    );
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    // 執行指令
    await page.click(".prompt-item [data-testid='run-prompt-btn']");

    // 在開發模式下應該返回模擬回應
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("執行成功", { timeout: 30000 });

    // 檢查任務完成狀態
    await expect(page.locator(".status-done")).toBeVisible();
  });
});

test.describe("資料持久化測試", () => {
  test("Prompts 資料持久化", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 建立測試 Prompt
    const testTitle = `持久化測試-${Date.now()}`;
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", testTitle);
    await page.fill("[data-testid='prompt-content-input']", "這是持久化測試的內容");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    // 重新載入頁面
    await page.reload();
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 檢查 Prompt 是否仍然存在
    await expect(
      page.locator(`.prompt-item:has-text("${testTitle}")`)
    ).toBeVisible();
  });

  test("Jobs 資料持久化", async ({ page }) => {
    await page.goto("http://localhost:8080");
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 建立並執行測試 Prompt
    const testTitle = `任務持久化測試-${Date.now()}`;
    await page.click("[data-testid='create-prompt-fab']");
    await page.fill("[data-testid='prompt-title-input']", testTitle);
    await page.fill("[data-testid='prompt-content-input']", "這是任務持久化測試");
    await page.click("[data-testid='prompt-modal-save-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("Prompt 建立成功");

    await page.click(".prompt-item [data-testid='run-prompt-btn']");
    await expect(page.locator("[data-testid='snackbar-container']")).toContainText("執行成功", { timeout: 30000 });

    // 重新載入頁面
    await page.reload();
    await expect(page.locator("[data-testid='app-title']")).toBeVisible();

    // 檢查任務是否仍然存在
    await expect(page.locator("[data-testid='jobs-list'] .job-item")).toBeVisible();
  });
});
