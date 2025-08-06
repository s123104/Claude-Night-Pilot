#!/usr/bin/env node

/**
 * 智能開發伺服器啟動腳本
 * 自動檢測可用端口並啟動前端開發伺服器
 */

import { spawn } from "child_process";
import path from "path";
import fs from "fs";
import net from "net";
import { fileURLToPath } from "url";
import { dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 動態導入 get-port (ES Module)
async function getAvailablePort() {
  try {
    const getPort = (await import("get-port")).default;
    return await getPort({
      port: [
        8080, 8081, 8082, 8083, 8084, 8085, 3000, 3001, 4000, 4001, 5000, 5001,
      ],
    });
  } catch (error) {
    console.error("❌ 無法導入 get-port，嘗試備用方案");
    return await fallbackPortDetection();
  }
}

// 備用端口檢測方案
async function fallbackPortDetection() {
  const ports = [
    8080, 8081, 8082, 8083, 8084, 8085, 3000, 3001, 4000, 4001, 5000, 5001,
  ];

  for (const port of ports) {
    if (await isPortAvailable(port)) {
      return port;
    }
  }

  // 隨機端口
  return Math.floor(Math.random() * (9999 - 8000) + 8000);
}

function isPortAvailable(port) {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.listen(port, () => {
      server.close(() => resolve(true));
    });
    server.on("error", () => resolve(false));
  });
}

async function startDevServer() {
  try {
    console.log("🔍 檢測可用端口...");
    const port = await getAvailablePort();
    console.log(`✅ 找到可用端口: ${port}`);

    // 更新 Tauri 配置
    await updateTauriConfig(port);

    // 啟動 Python HTTP 伺服器 (改用更穩定的方案)
    console.log(`🚀 啟動開發伺服器在端口 ${port}...`);
    const serverProcess = spawn(
      "python3",
      ["-m", "http.server", port.toString(), "--directory", "src"],
      {
        stdio: ["ignore", "pipe", "pipe"], // 捕獲輸出避免直接繼承
        cwd: path.join(__dirname, ".."),
        detached: false, // 確保子進程與父進程綁定
      }
    );

    // 監聽伺服器輸出
    serverProcess.stdout.on("data", (data) => {
      const message = data.toString().trim();
      if (message) {
        console.log(`📋 伺服器: ${message}`);
      }
    });

    serverProcess.stderr.on("data", (data) => {
      const message = data.toString().trim();
      if (message && !message.includes("Serving HTTP")) {
        console.error(`⚠️ 伺服器警告: ${message}`);
      }
    });

    // 確認伺服器啟動
    setTimeout(() => {
      console.log(`✅ 開發伺服器已成功啟動: http://localhost:${port}`);
      console.log("📝 伺服器將持續運行，請使用 Ctrl+C 停止");
      console.log("🔄 每30秒顯示一次運行狀態...");
    }, 1000);

    // 設置保持運行的狀態檢查
    const keepAliveInterval = setInterval(() => {
      if (!serverProcess.killed) {
        console.log(
          `🟢 伺服器運行中 (PID: ${
            serverProcess.pid
          }) - ${new Date().toLocaleTimeString()}`
        );
      }
    }, 30000);

    // 優雅關閉處理 - 改進版本
    const gracefulShutdown = (signal) => {
      console.log(`\n🛑 收到 ${signal} 信號，正在優雅關閉開發伺服器...`);
      clearInterval(keepAliveInterval);

      if (!serverProcess.killed) {
        serverProcess.kill("SIGTERM");

        // 3秒後強制終止
        setTimeout(() => {
          if (!serverProcess.killed) {
            console.log("⚡ 強制終止伺服器...");
            serverProcess.kill("SIGKILL");
          }
          process.exit(0);
        }, 3000);
      } else {
        process.exit(0);
      }
    };

    process.on("SIGINT", () => gracefulShutdown("SIGINT"));
    process.on("SIGTERM", () => gracefulShutdown("SIGTERM"));

    serverProcess.on("close", (code, signal) => {
      clearInterval(keepAliveInterval);
      console.log(`📋 開發伺服器已關閉 (code: ${code}, signal: ${signal})`);

      // 非正常關閉時不自動重啟，以免干擾測試
      if (code === 0 || signal) {
        process.exit(0);
      } else {
        console.log("❌ 伺服器異常關閉");
        process.exit(code || 1);
      }
    });

    serverProcess.on("error", (error) => {
      console.error("❌ 伺服器進程錯誤:", error.message);
      clearInterval(keepAliveInterval);
      process.exit(1);
    });
  } catch (error) {
    console.error("❌ 啟動開發伺服器失敗:", error.message);
    process.exit(1);
  }
}

async function updateTauriConfig(port) {
  const configPath = path.join(__dirname, "..", "src-tauri", "tauri.conf.json");

  try {
    if (fs.existsSync(configPath)) {
      const config = JSON.parse(fs.readFileSync(configPath, "utf8"));

      // 更新 devPath
      if (config.build) {
        config.build.devPath = `http://localhost:${port}`;
        console.log(`📝 更新 Tauri 配置 devPath: http://localhost:${port}`);
      }

      // 寫回配置文件
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
      console.log("✅ Tauri 配置已更新");
    }
  } catch (error) {
    console.warn("⚠️ 無法更新 Tauri 配置:", error.message);
  }
}

// 導出端口檢測函數供其他腳本使用
export { getAvailablePort, updateTauriConfig };

// 如果直接執行此腳本
if (import.meta.url === `file://${process.argv[1]}`) {
  startDevServer();
}
