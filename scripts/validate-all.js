#!/usr/bin/env node
/*
  Unified validation pipeline
  - Runs: CLI help, CLI benchmark, full test suite (Rust + E2E + coverage), status, health
  - Captures stdout/stderr of each step into a single timestamped log under logs/
  - Exits non-zero if any step fails
*/
import { spawn } from 'node:child_process';
import { mkdirSync, createWriteStream, existsSync } from 'node:fs';
import { join } from 'node:path';

const timestamp = new Date()
  .toISOString()
  .replace(/[-:]/g, '')
  .replace('T', '_')
  .slice(0, 15);

const logsDir = join(process.cwd(), 'logs');
if (!existsSync(logsDir)) {
  mkdirSync(logsDir, { recursive: true });
}
const logFile = join(logsDir, `validate-${timestamp}.log`);
const log = createWriteStream(logFile, { flags: 'a' });

function runStep(name, cmd, args) {
  return new Promise((resolve) => {
    log.write(`\n===== STEP: ${name} =====\n`);
    const child = spawn(cmd, args, { shell: false });
    child.stdout.on('data', (d) => log.write(d));
    child.stderr.on('data', (d) => log.write(d));
    child.on('close', (code) => {
      log.write(`\n----- RESULT: ${name} -> exit ${code} -----\n`);
      resolve({ name, code });
    });
  });
}

// Allow quick mode: skip benchmarks and E2E to save time
const isQuick = process.argv.includes('--quick');

const steps = [
  { name: 'cli-help', cmd: 'npm', args: ['run', '-s', 'cli:optimized', '--', '--help'] },
  ...(isQuick ? [] : [{ name: 'bench-cli', cmd: 'npm', args: ['run', '-s', 'bench:cli'] }]),
  { name: 'test-all', cmd: 'npm', args: ['run', '-s', isQuick ? 'test:rust' : 'test:all'] },
  { name: 'status', cmd: 'npm', args: ['run', '-s', 'cli:optimized', '--', 'status'] },
  { name: 'health', cmd: 'npm', args: ['run', '-s', 'cli:optimized', '--', 'health', '--format', 'json'] },
];

(async () => {
  const results = [];
  for (const step of steps) {
    /* eslint-disable no-await-in-loop */
    const r = await runStep(step.name, step.cmd, step.args);
    results.push(r);
    if (r.code !== 0) {
      // continue to next steps but remember failure
    }
  }

  const failed = results.filter((r) => r.code !== 0);
  log.write(`\n===== SUMMARY =====\n`);
  for (const r of results) {
    log.write(`${r.name}: ${r.code === 0 ? 'OK' : 'FAIL(' + r.code + ')' }\n`);
  }
  log.end();

  // Print concise console summary
  const summary = results.map((r) => `${r.name}=${r.code}`).join(', ');
  console.log(`[validate-all] steps: ${summary}`);
  console.log(`[validate-all] log: ${logFile}`);

  process.exit(failed.length ? 1 : 0);
})();


