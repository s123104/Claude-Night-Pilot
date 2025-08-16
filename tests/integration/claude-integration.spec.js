import { test, expect } from "@playwright/test";

test.describe("Claude Code 整合測試", () => {
  test.beforeEach(async ({ page }) => {
    // 前往應用主頁
    await page.goto("http://localhost:8080");

    // 等待應用載入，使用更靈活的選擇器
    await Promise.race([
      page.waitForSelector('h1:has-text("Claude Night Pilot")', { timeout: 30000 }),
      page.waitForSelector('.app-container', { timeout: 30000 }),
      page.waitForFunction(() => document.readyState === 'complete', { timeout: 30000 })
    ]);

    // 檢查應用是否已初始化
    await page.waitForFunction(
      () => window.__APP_READY__ === true || document.querySelector('[data-tab]'),
      { timeout: 15000 }
    );
  });

  test("測試 Claude Code 語法 Prompt 建立", async ({ page }) => {
    // 建立包含 Claude Code 語法的 Prompt
    await page.fill("#prompt-title", "Claude Code 測試指令");
    await page.fill(
      "#prompt-content",
      "@claude-code-zh-tw.md 請分析這個文檔的主要內容並總結重點"
    );
    await page.fill("#prompt-tags", "claude-code,analysis,docs");

    // 點擊建立按鈕
    await page.click('button:has-text("建立 Prompt")');

    // 等待成功訊息
    await expect(page.locator("text=Prompt 建立成功")).toBeVisible({
      timeout: 10000,
    });

    // 檢查 Prompt 是否正確顯示 Claude Code 語法
    await expect(page.locator(".prompt-item")).toContainText(
      "Claude Code 測試指令"
    );
    await expect(page.locator(".prompt-item")).toContainText(
      "@claude-code-zh-tw.md"
    );
  });

  test("測試 Claude Code Prompt 立即執行", async ({ page }) => {
    // 建立測試 Prompt
    await page.fill("#prompt-title", "Claude Code 執行測試");
    await page.fill(
      "#prompt-content",
      "@claude-code-zh-tw.md 請回答：Claude Code 的主要功能是什麼？"
    );
    await page.click('button:has-text("建立 Prompt")');
    await expect(page.locator("text=Prompt 建立成功")).toBeVisible();

    // 執行 Prompt
    await page.click('.prompt-item button:has-text("立即執行")');

    // 等待執行完成（在開發模式下會返回模擬回應）
    await expect(page.locator("text=執行成功")).toBeVisible({ timeout: 30000 });

    // 驗證執行結果對話框內容
    const dialogContent = await page.locator('[role="dialog"]').textContent();
    expect(dialogContent).toContain("模擬的 Claude 回應");
  });

  test("測試冷卻時間偵測功能", async ({ page }) => {
    // 檢查冷卻狀態顯示
    const cooldownStatus = page.locator("#cooldown-status");
    await expect(cooldownStatus).toBeVisible();

    // 在開發模式下應該顯示可用狀態
    await expect(cooldownStatus).toContainText("Claude CLI 可用");

    // 檢查狀態是否會自動更新
    await page.waitForTimeout(6000); // 等待一個更新週期
    await expect(cooldownStatus).toBeVisible();
  });

  test("測試排程執行 Claude Code 任務", async ({ page }) => {
    // 建立用於排程的 Claude Code Prompt
    await page.fill("#prompt-title", "自動化文檔分析");
    await page.fill(
      "#prompt-content",
      "@docs/*.md 請定期分析專案文檔的更新情況"
    );
    await page.click('button:has-text("建立 Prompt")');
    await expect(page.locator("text=Prompt 建立成功")).toBeVisible();

    // 點擊排程執行
    await page.click('.prompt-item button:has-text("排程執行")');

    // 檢查排程對話框
    await expect(page.locator("#schedule-dialog")).toBeVisible();

    // 設定每小時執行
    await page.fill("#cron-expression", "0 * * * *");
    await page.selectOption("#execution-mode", "async");

    // 建立排程
    await page.click('button:has-text("建立排程")');

    // 等待成功訊息
    await expect(page.locator("text=排程任務建立成功")).toBeVisible({
      timeout: 10000,
    });
  });

  test("測試多種 Claude Code 語法支援", async ({ page }) => {
    const testCases = [
      {
        title: "檔案引用測試",
        content: "@README.md 請總結這個專案",
        description: "單一檔案引用",
      },
      {
        title: "多檔案模式測試",
        content: "@src/*.js 請重構這些 JavaScript 檔案",
        description: "glob 模式檔案引用",
      },
      {
        title: "編輯指令測試",
        content: "編輯 package.json 並添加新的依賴項目",
        description: "編輯指令",
      },
    ];

    for (const testCase of testCases) {
      // 建立測試 Prompt
      await page.fill("#prompt-title", testCase.title);
      await page.fill("#prompt-content", testCase.content);
      await page.fill("#prompt-tags", "claude-code,test");

      await page.click('button:has-text("建立 Prompt")');
      await expect(page.locator("text=Prompt 建立成功")).toBeVisible();

      // 驗證 Prompt 正確顯示
      await expect(page.locator(".prompt-item").last()).toContainText(
        testCase.title
      );

      // 清空表單準備下一個測試
      await page.fill("#prompt-title", "");
      await page.fill("#prompt-content", "");
      await page.fill("#prompt-tags", "");
    }
  });

  test("測試 Claude CLI 錯誤處理", async ({ page }) => {
    // 這個測試模擬 Claude CLI 不可用的情況
    // 在實際環境中會檢查真實的 CLI 狀態

    // 檢查 CLI 驗證功能
    await page.evaluate(() => {
      // 使用 Tauri API 檢查 CLI 狀態
      window.__TAURI__.tauri.invoke("verify_claude_cli").then((result) => {
        console.log("Claude CLI 狀態:", result);
      });
    });

    // 驗證錯誤狀態顯示
    const statusElement = page.locator("#cooldown-status");
    await expect(statusElement).toBeVisible();
  });

  test("測試任務結果查看功能", async ({ page }) => {
    // 先執行一個任務
    await page.fill("#prompt-title", "結果測試");
    await page.fill("#prompt-content", "@claude-code-zh-tw.md 測試結果功能");
    await page.click('button:has-text("建立 Prompt")');
    await expect(page.locator("text=Prompt 建立成功")).toBeVisible();

    await page.click('.prompt-item button:has-text("立即執行")');
    await expect(page.locator("text=執行成功")).toBeVisible({ timeout: 30000 });

    // 查看任務結果
    await page.click('.job-item button:has-text("查看結果")');

    // 驗證結果對話框
    await expect(page.locator("text=的執行結果")).toBeVisible({
      timeout: 10000,
    });

    // 檢查結果內容
    const resultDialog = page.locator('[role="dialog"]');
    await expect(resultDialog).toContainText("模擬的執行結果內容");
  });

  test("測試應用啟動效能", async ({ page }) => {
    // 測量頁面載入和初始化時間
    const startTime = Date.now();

    await page.goto("http://localhost:8080");
    await page.waitForSelector('h1:has-text("Claude Night Pilot")');
    await page.waitForSelector("#cooldown-status");

    const loadTime = Date.now() - startTime;

    // 驗證載入時間符合預期（< 5 秒）
    expect(loadTime).toBeLessThan(5000);

    console.log(`應用載入時間: ${loadTime}ms`);
  });

  test("測試完整的工作流程", async ({ page }) => {
    // 完整的端到端工作流程測試

    // 1. 建立 Claude Code Prompt
    await page.fill("#prompt-title", "E2E 工作流程測試");
    await page.fill(
      "#prompt-content",
      "@claude-code-zh-tw.md 請幫我建立一個自動化腳本"
    );
    await page.fill("#prompt-tags", "e2e,automation,claude-code");
    await page.click('button:has-text("建立 Prompt")');
    await expect(page.locator("text=Prompt 建立成功")).toBeVisible();

    // 2. 立即執行
    await page.click('.prompt-item button:has-text("立即執行")');
    await expect(page.locator("text=執行成功")).toBeVisible({ timeout: 30000 });

    // 3. 檢查任務狀態
    await expect(page.locator(".job-item .status-done")).toBeVisible();

    // 4. 查看執行結果
    await page.click('.job-item button:has-text("查看結果")');
    await expect(page.locator("text=的執行結果")).toBeVisible({
      timeout: 10000,
    });

    // 5. 建立排程任務
    await page.click('.prompt-item button:has-text("排程執行")');
    await expect(page.locator("#schedule-dialog")).toBeVisible();

    await page.fill("#cron-expression", "0 0 * * *"); // 每日午夜
    await page.selectOption("#execution-mode", "async");
    await page.click('button:has-text("建立排程")');
    await expect(page.locator("text=排程任務建立成功")).toBeVisible({
      timeout: 10000,
    });

    // 6. 驗證整個流程完成
    await expect(page.locator(".job-item")).toHaveCount(2); // 一個完成的任務 + 一個排程任務

    console.log("✅ 完整工作流程測試通過");
  });
});
