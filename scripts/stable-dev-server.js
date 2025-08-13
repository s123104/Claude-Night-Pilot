#!/usr/bin/env node

/**
 * 穩定的開發伺服器腳本
 * 使用固定端口 8080，簡化配置
 */

import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";
import { dirname } from "path";
import http from "http";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PORT = 8081;
const SRC_DIR = path.join(__dirname, "..", "src");

function checkPort() {
  return new Promise((resolve) => {
    const server = http.createServer();
    server.listen(PORT, () => {
      server.close(() => resolve(true));
    });
    server.on("error", () => resolve(false));
  });
}

async function startServer() {
  console.log(`🔍 檢查端口 ${PORT}...`);

  if (!(await checkPort())) {
    console.error(`❌ 端口 ${PORT} 已被占用，請先停止占用該端口的程序`);
    process.exit(1);
  }

  console.log(`🚀 啟動開發伺服器在端口 ${PORT}...`);

  const server = spawn(
    "python3",
    ["-m", "http.server", PORT.toString(), "--directory", SRC_DIR],
    {
      stdio: "inherit",
      cwd: path.join(__dirname, ".."),
    }
  );

  console.log(`✅ 開發伺服器已啟動: http://localhost:${PORT}`);
  console.log("📝 使用 Ctrl+C 停止伺服器");

  // 優雅關閉
  process.on("SIGINT", () => {
    console.log("\n🛑 正在停止開發伺服器...");
    server.kill("SIGTERM");
    setTimeout(() => {
      server.kill("SIGKILL");
      process.exit(0);
    }, 3000);
  });

  server.on("error", (error) => {
    console.error("❌ 伺服器錯誤:", error);
    process.exit(1);
  });

  server.on("close", (code) => {
    console.log(`📋 開發伺服器已關閉 (code: ${code})`);
    process.exit(code || 0);
  });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  startServer().catch(console.error);
}
