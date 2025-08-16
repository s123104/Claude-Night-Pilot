#!/usr/bin/env node

/**
 * Claude Night Pilot - 穩定的前端開發伺服器
 * 用於測試和開發環境，支持端口自動切換和 SPA 路由
 */

import express from "express";
import path from "path";
import fs from "fs";
import { createServer } from "http";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class StableDevelopmentServer {
  constructor() {
    this.app = express();
    this.port = process.env.PORT || 8080;
    this.fallbackPort = 8081;
    this.srcDir = path.join(__dirname, "../src");
    this.server = null;

    this.setupMiddleware();
    this.setupRoutes();
  }

  setupMiddleware() {
    // CORS 支持
    this.app.use((req, res, next) => {
      res.header("Access-Control-Allow-Origin", "*");
      res.header(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS"
      );
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
    this.app.use(
      express.static(this.srcDir, {
        maxAge: "1h",
        etag: false,
        lastModified: false,
      })
    );

    // JSON 解析
    this.app.use(express.json());
  }

  setupRoutes() {
    // 健康檢查端點
    this.app.get("/health", (req, res) => {
      res.json({
        status: "ok",
        timestamp: new Date().toISOString(),
        port: this.port,
        srcDir: this.srcDir,
      });
    });

    // API 模擬端點 (用於測試)
    this.app.get("/api/test", (req, res) => {
      res.json({
        message: "Test API endpoint working",
        timestamp: new Date().toISOString(),
      });
    });

    // SPA 路由支持 - 處理所有未匹配的請求，返回 index.html
    this.app.use((req, res, next) => {
      // 如果請求是 API 路由或靜態文件，跳過
      if (req.path.startsWith('/api/') || req.path.includes('.')) {
        return next();
      }

      const indexPath = path.join(this.srcDir, "index.html");

      if (fs.existsSync(indexPath)) {
        try {
          // 讀取並修改 index.html 以支持測試模式
          let indexContent = fs.readFileSync(indexPath, "utf8");

          // 在測試環境中注入測試標記
          if (process.env.NODE_ENV === "test") {
            indexContent = indexContent.replace(
              '<body data-test-mode="false">',
              '<body data-test-mode="true">'
            );

            // 注入測試輔助腳本
            const testScript = `
            <script>
              // 測試模式輔助
              window.__TEST_MODE__ = true;
              window.__APP_READY__ = false;
              
              // 加速應用載入
              document.addEventListener('DOMContentLoaded', () => {
                setTimeout(() => {
                  window.__APP_READY__ = true;
                  console.log('🧪 Test mode: App ready signal sent');
                }, 100);
              });
              
              // 測試用的全域函數
              window.testHelpers = {
                waitForElement: (selector, timeout = 5000) => {
                  return new Promise((resolve, reject) => {
                    const element = document.querySelector(selector);
                    if (element) {
                      resolve(element);
                      return;
                    }
                    
                    const observer = new MutationObserver(() => {
                      const element = document.querySelector(selector);
                      if (element) {
                        observer.disconnect();
                        resolve(element);
                      }
                    });
                    
                    observer.observe(document.body, {
                      childList: true,
                      subtree: true
                    });
                    
                    setTimeout(() => {
                      observer.disconnect();
                      reject(new Error(\`Element \${selector} not found within \${timeout}ms\`));
                    }, timeout);
                  });
                },
                
                makeElementVisible: (selector) => {
                  const element = document.querySelector(selector);
                  if (element) {
                    element.style.display = 'block';
                    element.style.visibility = 'visible';
                    element.style.opacity = '1';
                    return true;
                  }
                  return false;
                }
              };
            </script>
          `;

            indexContent = indexContent.replace(
              "</head>",
              testScript + "</head>"
            );
          }

          res.send(indexContent);
        } catch (error) {
          console.error('Error serving index.html:', error);
          res.status(500).json({
            error: "Error reading index file",
            message: error.message
          });
        }
      } else {
        res.status(404).json({
          error: "Index file not found",
          path: indexPath,
          srcDir: this.srcDir,
        });
      }
    });
  }

  async start() {
    return new Promise((resolve, reject) => {
      this.server = this.app.listen(this.port, (err) => {
        if (err) {
          reject(err);
          return;
        }

        console.log(`🚀 Claude Night Pilot 前端開發伺服器啟動成功`);
        console.log(`📍 地址: http://localhost:${this.port}`);
        console.log(`📁 服務目錄: ${this.srcDir}`);
        console.log(
          `🧪 測試模式: ${process.env.NODE_ENV === "test" ? "啟用" : "停用"}`
        );
        console.log(`⏰ 啟動時間: ${new Date().toLocaleString("zh-TW")}`);

        resolve(this.server);
      });

      this.server.on("error", (err) => {
        if (err.code === "EADDRINUSE") {
          console.log(
            `⚠️  端口 ${this.port} 被佔用，嘗試端口 ${this.fallbackPort}`
          );

          if (this.port !== this.fallbackPort) {
            this.port = this.fallbackPort;
            this.start().then(resolve).catch(reject);
          } else {
            reject(new Error("所有端口都被佔用，請手動釋放端口 8080 或 8081"));
          }
        } else {
          reject(err);
        }
      });
    });
  }

  async stop() {
    return new Promise((resolve) => {
      if (this.server) {
        this.server.close(() => {
          console.log("🛑 前端開發伺服器已停止");
          resolve();
        });
      } else {
        resolve();
      }
    });
  }
}

// 優雅關閉處理
function setupGracefulShutdown(server) {
  const shutdown = async (signal) => {
    console.log(`\n收到 ${signal} 信號，正在優雅關閉伺服器...`);

    try {
      await server.stop();
      console.log("✅ 伺服器已安全關閉");
      process.exit(0);
    } catch (error) {
      console.error("❌ 關閉伺服器時發生錯誤:", error);
      process.exit(1);
    }
  };

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));
  process.on("SIGUSR2", () => shutdown("SIGUSR2")); // nodemon 重啟信號
}

// 主執行邏輯
async function main() {
  const server = new StableDevelopmentServer();

  try {
    await server.start();
    setupGracefulShutdown(server);

    // 在測試環境中，提供額外的狀態信息
    if (process.env.NODE_ENV === "test") {
      console.log("\n🧪 測試環境配置:");
      console.log(`   - 自動設置 data-test-mode="true"`);
      console.log(`   - 注入測試輔助函數`);
      console.log(`   - 加速應用載入`);
      console.log(`   - 健康檢查: http://localhost:${server.port}/health`);
    }
  } catch (error) {
    console.error("❌ 伺服器啟動失敗:", error.message);

    if (error.code === "EADDRINUSE") {
      console.log("\n💡 解決建議:");
      console.log("   1. 檢查是否有其他服務佔用端口:");
      console.log("      lsof -i :8080 -i :8081");
      console.log("   2. 終止佔用端口的進程:");
      console.log("      kill -9 <PID>");
      console.log("   3. 或使用不同端口:");
      console.log("      PORT=3000 npm run dev:frontend");
    }

    process.exit(1);
  }
}

// 如果直接執行此腳本
const isMainModule = __filename === path.resolve(process.argv[1]);
if (isMainModule) {
  main().catch(console.error);
}

export default StableDevelopmentServer;
