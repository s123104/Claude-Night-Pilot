import { test, expect } from "@playwright/test";

test.describe("Claude Night Pilot - 簡化核心測試", () => {
  test("應用能正確啟動並顯示基本介面", async ({ page }) => {
    // 等待更長時間讓應用啟動
    await page.goto("http://localhost:1420", { timeout: 60000 });

    // 檢查是否有標題或任何內容載入
    try {
      await page.waitForSelector("body", { timeout: 30000 });

      // 檢查頁面是否有內容
      const bodyContent = await page.textContent("body");
      expect(bodyContent).toBeTruthy();

      console.log("頁面內容:", bodyContent.substring(0, 200));

      // 截圖以查看實際狀況
      await page.screenshot({
        path: "test-results/startup-screenshot.png",
        fullPage: true,
      });
    } catch (error) {
      console.error("測試失敗:", error.message);

      // 嘗試截圖看看頁面狀況
      try {
        await page.screenshot({
          path: "test-results/error-screenshot.png",
          fullPage: true,
        });
      } catch (screenshotError) {
        console.error("截圖也失敗:", screenshotError.message);
      }

      throw error;
    }
  });

  test("檢查頁面標題", async ({ page }) => {
    await page.goto("http://localhost:1420", { timeout: 60000 });

    // 檢查頁面標題
    const title = await page.title();
    console.log("頁面標題:", title);

    // 基本檢查 - 只要不是空的就算通過
    expect(title).toBeTruthy();
  });
});
