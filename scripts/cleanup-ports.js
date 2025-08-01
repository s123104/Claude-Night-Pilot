#!/usr/bin/env node

/**
 * 端口清理腳本
 * 清理被占用的開發端口並提供端口狀態報告
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const execAsync = promisify(exec);

const COMMON_DEV_PORTS = [8080, 8081, 8082, 3000, 3001, 4000, 4001, 5000, 5001, 1420];

async function checkPort(port) {
  try {
    const { stdout } = await execAsync(`lsof -ti:${port}`);
    const pids = stdout.trim().split('\n').filter(pid => pid);
    return pids.length > 0 ? pids : null;
  } catch (error) {
    // 端口未被占用
    return null;
  }
}

async function getProcessInfo(pid) {
  try {
    const { stdout } = await execAsync(`ps -p ${pid} -o pid,ppid,comm,args --no-headers`);
    return stdout.trim();
  } catch (error) {
    return `PID ${pid} (進程信息不可用)`;
  }
}

async function killProcess(pid, signal = 'TERM') {
  try {
    await execAsync(`kill -${signal} ${pid}`);
    return true;
  } catch (error) {
    return false;
  }
}

async function reportPortStatus() {
  console.log('🔍 檢查常用開發端口狀態...\n');
  
  const portStatus = [];
  
  for (const port of COMMON_DEV_PORTS) {
    const pids = await checkPort(port);
    if (pids) {
      console.log(`❌ 端口 ${port} 被占用:`);
      for (const pid of pids) {
        const processInfo = await getProcessInfo(pid);
        console.log(`   PID ${pid}: ${processInfo}`);
        portStatus.push({ port, pid, processInfo });
      }
    } else {
      console.log(`✅ 端口 ${port} 可用`);
    }
  }
  
  return portStatus;
}

async function cleanupPorts(force = false) {
  console.log('🧹 開始端口清理...\n');
  
  const portStatus = await reportPortStatus();
  const occupiedPorts = portStatus.filter(item => item.pid);
  
  if (occupiedPorts.length === 0) {
    console.log('\n✅ 所有常用開發端口都可用！');
    return;
  }
  
  console.log(`\n🚨 發現 ${occupiedPorts.length} 個被占用的端口`);
  
  if (!force) {
    console.log('💡 使用 --force 參數自動清理這些端口');
    return;
  }
  
  console.log('⚠️ 開始強制清理占用的端口...');
  
  for (const { port, pid, processInfo } of occupiedPorts) {
    console.log(`\n🔥 正在終止端口 ${port} 的進程 ${pid}...`);
    console.log(`   進程信息: ${processInfo}`);
    
    // 先嘗試優雅終止
    const graceful = await killProcess(pid, 'TERM');
    if (graceful) {
      console.log(`✅ 進程 ${pid} 已優雅終止`);
      
      // 等待一秒鐘確認進程已終止
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // 檢查進程是否真的終止了
      const stillRunning = await checkPort(port);
      if (stillRunning) {
        console.log(`⚠️ 進程 ${pid} 仍在運行，嘗試強制終止...`);
        await killProcess(pid, 'KILL');
        console.log(`💀 進程 ${pid} 已強制終止`);
      }
    } else {
      console.log(`❌ 無法終止進程 ${pid}`);
    }
  }
  
  console.log('\n🔍 重新檢查端口狀態...');
  await reportPortStatus();
}

async function findAvailablePort(startPort = 8080) {
  for (let port = startPort; port < startPort + 100; port++) {
    const pids = await checkPort(port);
    if (!pids) {
      return port;
    }
  }
  return null;
}

// 命令行介面
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  
  switch (command) {
    case 'status':
    case 'check':
      await reportPortStatus();
      break;
      
    case 'cleanup':
    case 'clean':
      const force = args.includes('--force') || args.includes('-f');
      await cleanupPorts(force);
      break;
      
    case 'find':
      const startPort = parseInt(args[1]) || 8080;
      const availablePort = await findAvailablePort(startPort);
      if (availablePort) {
        console.log(`✅ 找到可用端口: ${availablePort}`);
      } else {
        console.log('❌ 在指定範圍內未找到可用端口');
      }
      break;
      
    case 'help':
    case '--help':
    case '-h':
    default:
      console.log(`
🔧 端口清理工具

用法:
  node scripts/cleanup-ports.js <command> [options]

命令:
  status, check     檢查常用開發端口狀態
  cleanup, clean    清理被占用的端口
    --force, -f     強制終止占用端口的進程
  find [起始端口]    查找可用端口 (默認從8080開始)
  help             顯示此幫助信息

示例:
  node scripts/cleanup-ports.js status
  node scripts/cleanup-ports.js cleanup --force  
  node scripts/cleanup-ports.js find 3000
      `);
      break;
  }
}

// 導出函數供其他腳本使用
export { 
  checkPort, 
  reportPortStatus, 
  cleanupPorts, 
  findAvailablePort 
};

// 如果直接執行此腳本
const isMainModule = import.meta.url.startsWith('file:') && 
  import.meta.url === `file://${process.argv[1]}`;

if (isMainModule) {
  console.log('🚀 啟動端口清理工具...');
  main().catch(error => {
    console.error('❌ 腳本執行失敗:', error);
    process.exit(1);
  });
}