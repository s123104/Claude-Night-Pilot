import { test, expect } from "@playwright/test";

test.describe("Claude Night Pilot - 四核心模組測試", () => {
  test.beforeEach(async ({ page }) => {
    // 設置測試環境
    await page.goto("http://localhost:8081");
    await page.waitForSelector('h1:has-text("Claude Night Pilot")', {
      timeout: 30000,
    });

    // 切換到測試標籤頁
    await page.click('[data-tab="testing"]');
    await page.waitForSelector('[data-testid="core-001-section"]', {
      timeout: 10000,
    });
  });

  test.describe("CORE-001: ccusage API 整合模組", () => {
    test("應能夠獲取使用量資訊", async ({ page }) => {
      // 觸發使用量檢查
      await page.click('[data-testid="check-usage"]');

      // 等待結果
      await page.waitForSelector('[data-testid="usage-info"]', {
        timeout: 15000,
      });

      // 驗證使用量資訊顯示
      const usageInfo = page.locator('[data-testid="usage-info"]');
      await expect(usageInfo).toBeVisible();

      // 驗證必要的字段
      await expect(
        usageInfo.locator('[data-testid="remaining-minutes"]')
      ).toBeVisible();
      await expect(
        usageInfo.locator('[data-testid="usage-source"]')
      ).toBeVisible();
    });

    test("應支援多指令回退機制", async ({ page }) => {
      // 模擬 ccusage 命令不可用的情況
      await page.evaluate(() => {
        window.mockCcusageUnavailable = true;
      });

      await page.click('[data-testid="check-usage"]');

      // 應該自動回退到其他命令
      await page.waitForSelector('[data-testid="fallback-indicator"]', {
        timeout: 10000,
      });

      const fallbackInfo = page.locator('[data-testid="fallback-indicator"]');
      await expect(fallbackInfo).toContainText("使用回退方法");
    });

    test("應實現30秒智能快取", async ({ page }) => {
      // 第一次檢查
      const start = Date.now();
      await page.click('[data-testid="check-usage"]');
      await page.waitForSelector('[data-testid="usage-info"]');

      // 立即第二次檢查
      await page.click('[data-testid="check-usage"]');
      const end = Date.now();

      // 第二次檢查應該很快（使用快取）
      expect(end - start).toBeLessThan(5000);

      // 驗證快取指示器
      await expect(
        page.locator('[data-testid="cache-indicator"]')
      ).toBeVisible();
    });
  });

  test.describe("CORE-002: 安全執行系統", () => {
    test("應進行多層安全檢查", async ({ page }) => {
      // 輸入一個包含潛在危險模式的 prompt
      await page.fill(
        '[data-testid="prompt-input"]',
        "rm -rf / --dangerous-command"
      );

      // 嘗試執行
      await page.click('[data-testid="execute-prompt"]');

      // 應該顯示安全警告
      await expect(
        page.locator('[data-testid="security-warning"]')
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="security-warning"]')
      ).toContainText("檢測到危險模式");
    });

    test("應支援 --dangerously-skip-permissions 選項", async ({ page }) => {
      // 輸入危險 prompt
      await page.fill(
        '[data-testid="prompt-input"]',
        "rm -rf / --dangerous-command"
      );

      // 啟用跳過權限檢查
      await page.check('[data-testid="skip-permissions"]');

      // 執行
      await page.click('[data-testid="execute-prompt"]');

      // 應該顯示跳過權限警告但允許執行
      await expect(
        page.locator('[data-testid="permission-skipped"]')
      ).toBeVisible();
    });

    test("應記錄完整審計日誌", async ({ page }) => {
      // 執行一個簡單的 prompt
      await page.fill('[data-testid="prompt-input"]', "Hello, Claude!");
      await page.click('[data-testid="execute-prompt"]');

      // 等待執行完成
      await page.waitForSelector('[data-testid="execution-complete"]');

      // 檢查審計日誌
      await page.click('[data-testid="view-audit-log"]');

      const auditLog = page.locator('[data-testid="audit-log"]');
      await expect(auditLog).toBeVisible();
      await expect(auditLog).toContainText("SHA256");
      await expect(auditLog).toContainText("執行選項");
      await expect(auditLog).toContainText("安全檢查結果");
    });

    test("應支援乾運行模式", async ({ page }) => {
      // 輸入 prompt
      await page.fill('[data-testid="prompt-input"]', "Test dry run");

      // 啟用乾運行模式
      await page.check('[data-testid="dry-run"]');

      // 執行
      await page.click('[data-testid="execute-prompt"]');

      // 應該顯示乾運行結果
      await expect(
        page.locator('[data-testid="dry-run-result"]')
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="dry-run-result"]')
      ).toContainText("乾運行完成");
    });
  });

  test.describe("CORE-003: 自適應監控系統", () => {
    test("應顯示當前監控模式", async ({ page }) => {
      // 檢查監控狀態顯示
      const monitorStatus = page.locator('[data-testid="monitor-status"]');
      await expect(monitorStatus).toBeVisible();

      // 應該顯示六種模式之一
      const modes = [
        "Normal",
        "Approaching",
        "Imminent",
        "Critical",
        "Unavailable",
        "Unknown",
      ];
      const currentMode = await monitorStatus.textContent();
      expect(modes.some((mode) => currentMode.includes(mode))).toBeTruthy();
    });

    test("應動態調整監控間隔", async ({ page }) => {
      // 模擬接近限制的情況
      await page.evaluate(() => {
        window.mockUsageNearLimit = true;
      });

      // 觸發監控更新
      await page.click('[data-testid="update-monitor"]');

      // 等待監控模式變更
      await page.waitForFunction(() => {
        const status = document.querySelector('[data-testid="monitor-status"]');
        return (
          status &&
          (status.textContent.includes("Approaching") ||
            status.textContent.includes("Critical"))
        );
      });

      // 檢查間隔是否變短
      const intervalInfo = page.locator('[data-testid="monitor-interval"]');
      await expect(intervalInfo).toContainText("2分鐘");
    });

    test("應發送事件通知", async ({ page }) => {
      // 監聽事件
      await page.evaluate(() => {
        window.addEventListener("monitoring-event", () => {
          window.testEventReceived = true;
        });
      });

      // 觸發監控事件
      await page.click('[data-testid="trigger-monitor-event"]');

      // 檢查事件是否被接收
      await page.waitForFunction(() => window.testEventReceived);

      const wasEventReceived = await page.evaluate(
        () => window.testEventReceived
      );
      expect(wasEventReceived).toBeTruthy();
    });

    test("應追蹤監控統計", async ({ page }) => {
      // 觸發多次監控檢查
      for (let i = 0; i < 3; i++) {
        await page.click('[data-testid="check-monitor"]');
        await page.waitForTimeout(1000);
      }

      // 檢查統計資訊
      await page.click('[data-testid="view-stats"]');

      const stats = page.locator('[data-testid="monitor-stats"]');
      await expect(stats).toBeVisible();
      await expect(stats).toContainText("檢查次數");
      await expect(stats).toContainText("模式變更");
      await expect(stats).toContainText("運行時間");
    });
  });

  test.describe("CORE-004: 智能排程系統", () => {
    test("應支援時區感知排程", async ({ page }) => {
      // 設定排程任務
      await page.fill(
        '[data-testid="schedule-prompt"]',
        "Scheduled test prompt"
      );
      await page.fill('[data-testid="schedule-time"]', "2025-07-24 14:30");

      // 選擇時區
      await page.selectOption('[data-testid="timezone-select"]', "Asia/Taipei");

      // 建立排程
      await page.click('[data-testid="create-schedule"]');

      // 驗證排程資訊
      const scheduleInfo = page.locator('[data-testid="schedule-info"]');
      await expect(scheduleInfo).toBeVisible();
      await expect(scheduleInfo).toContainText("Asia/Taipei");
      await expect(scheduleInfo).toContainText("2025-07-24 14:30");
    });

    test("應實現5小時塊保護", async ({ page }) => {
      // 設定一個會導致用量耗盡的排程
      await page.fill('[data-testid="schedule-prompt"]', "Heavy usage prompt");
      await page.fill('[data-testid="required-minutes"]', "300"); // 5小時

      // 模擬只剩下4小時的情況
      await page.evaluate(() => {
        window.mockRemainingMinutes = 240; // 4小時
      });

      await page.click('[data-testid="create-schedule"]');

      // 應該顯示保護警告
      await expect(
        page.locator('[data-testid="block-protection"]')
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="block-protection"]')
      ).toContainText("5小時塊保護");
    });

    test("應計算效率分析", async ({ page }) => {
      // 設定排程參數
      await page.fill('[data-testid="schedule-prompt"]', "Efficiency test");
      await page.fill('[data-testid="required-minutes"]', "80"); // 80分鐘

      // 模擬剩餘100分鐘
      await page.evaluate(() => {
        window.mockRemainingMinutes = 100;
      });

      await page.click('[data-testid="analyze-efficiency"]');

      // 檢查效率分析結果
      const efficiency = page.locator('[data-testid="efficiency-analysis"]');
      await expect(efficiency).toBeVisible();
      await expect(efficiency).toContainText("理想使用率"); // 80% 使用率
      await expect(efficiency).toContainText("1.0"); // 效率分數
    });

    test("應遵循工作時間限制", async ({ page }) => {
      // 設定非工作時間的排程
      await page.fill('[data-testid="schedule-prompt"]', "After hours test");
      await page.fill('[data-testid="schedule-time"]', "2025-07-24 02:00"); // 凌晨2點

      await page.click('[data-testid="create-schedule"]');

      // 應該顯示工作時間警告
      await expect(
        page.locator('[data-testid="working-hours-warning"]')
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="working-hours-warning"]')
      ).toContainText("非工作時間");
    });

    test("應支援任務重試機制", async ({ page }) => {
      // 建立一個會失敗的排程任務
      await page.fill('[data-testid="schedule-prompt"]', "Failing task");
      await page.evaluate(() => {
        window.mockTaskFailure = true;
      });

      await page.click('[data-testid="create-schedule"]');

      // 等待任務執行和重試
      await page.waitForSelector('[data-testid="retry-indicator"]', {
        timeout: 15000,
      });

      // 檢查重試資訊
      const retryInfo = page.locator('[data-testid="retry-info"]');
      await expect(retryInfo).toBeVisible();
      await expect(retryInfo).toContainText("重試次數");
    });
  });

  test.describe("整合測試", () => {
    test("四大模組應協同工作", async ({ page }) => {
      // 1. 檢查使用量 (CORE-001)
      await page.click('[data-testid="check-usage"]');
      await page.waitForSelector('[data-testid="usage-info"]');

      // 2. 啟動監控 (CORE-003)
      await page.click('[data-testid="start-monitoring"]');
      await expect(
        page.locator('[data-testid="monitor-status"]')
      ).toContainText("Normal");

      // 3. 建立安全排程 (CORE-004 + CORE-002)
      await page.fill(
        '[data-testid="schedule-prompt"]',
        "Integration test prompt"
      );
      await page.check('[data-testid="enable-security"]');
      await page.click('[data-testid="create-schedule"]');

      // 4. 驗證所有模組狀態
      await expect(
        page.locator('[data-testid="integration-status"]')
      ).toContainText("所有模組正常運行");
    });

    test("應正確處理模組間的錯誤傳播", async ({ page }) => {
      // 模擬 ccusage 失敗
      await page.evaluate(() => {
        window.mockCcusageError = true;
      });

      // 嘗試建立排程（依賴使用量資訊）
      await page.click('[data-testid="create-schedule"]');

      // 應該顯示優雅的錯誤處理
      await expect(page.locator('[data-testid="module-error"]')).toBeVisible();
      await expect(page.locator('[data-testid="module-error"]')).toContainText(
        "使用量檢查失敗"
      );
    });

    test("應維護資料一致性", async ({ page }) => {
      // 建立多個操作
      await page.fill('[data-testid="prompt-input"]', "Consistency test 1");
      await page.click('[data-testid="execute-prompt"]');

      await page.fill('[data-testid="schedule-prompt"]', "Consistency test 2");
      await page.click('[data-testid="create-schedule"]');

      // 檢查資料庫一致性
      await page.click('[data-testid="check-consistency"]');

      const consistencyResult = page.locator(
        '[data-testid="consistency-result"]'
      );
      await expect(consistencyResult).toBeVisible();
      await expect(consistencyResult).toContainText("資料一致性檢查通過");
    });
  });
});
