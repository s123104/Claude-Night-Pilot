#!/usr/bin/env node

/**
 * 簡化端口檢查腳本
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const PORTS = [8080, 8081, 8082, 3000, 3001, 4000, 4001, 5000, 5001, 1420];

async function checkPort(port) {
  try {
    const { stdout } = await execAsync(`lsof -ti:${port}`);
    const pids = stdout.trim().split('\n').filter(pid => pid);
    return pids.length > 0 ? pids : null;
  } catch (error) {
    return null;
  }
}

async function main() {
  console.log('🔍 檢查常用開發端口狀態...\n');
  
  for (const port of PORTS) {
    const pids = await checkPort(port);
    if (pids) {
      console.log(`❌ 端口 ${port} 被占用: PIDs ${pids.join(', ')}`);
    } else {
      console.log(`✅ 端口 ${port} 可用`);
    }
  }
}

main().catch(console.error);