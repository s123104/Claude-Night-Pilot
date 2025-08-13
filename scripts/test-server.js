#!/usr/bin/env node

import express from "express";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const port = process.env.PORT || 8081;
const srcDir = path.join(__dirname, "../src");

console.log(`🚀 啟動測試伺服器...`);
console.log(`📁 服務目錄: ${srcDir}`);

// CORS 支持
app.use((req, res, next) => {
  res.header("Access-Control-Allow-Origin", "*");
  res.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
  res.header(
    "Access-Control-Allow-Headers",
    "Origin, X-Requested-With, Content-Type, Accept, Authorization"
  );

  if (req.method === "OPTIONS") {
    res.sendStatus(200);
  } else {
    next();
  }
});

// 靜態文件服務
app.use(express.static(srcDir));

// 健康檢查端點
app.get("/health", (req, res) => {
  res.json({
    status: "ok",
    timestamp: new Date().toISOString(),
    port: port,
    srcDir: srcDir,
  });
});

// SPA 路由支持 - 修復 Express 5.x 路由問題
app.get("/*", (req, res) => {
  const indexPath = path.join(srcDir, "index.html");
  res.sendFile(indexPath);
});

app.listen(port, () => {
  console.log(`✅ 測試伺服器啟動成功: http://localhost:${port}`);
  console.log(`⏰ 啟動時間: ${new Date().toLocaleString("zh-TW")}`);
});
