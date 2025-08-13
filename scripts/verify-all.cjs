#!/usr/bin/env node
/**
 * Consolidated verification runner (CommonJS)
 * - Runs CLI help, benchmarks, full test suite, status and health
 * - Captures stdout/stderr of every step into a timestamped log file
 * - Produces a JSON summary with exit codes and durations
 */

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function ts() {
  const d = new Date();
  const pad = (n) => String(n).padStart(2, '0');
  const y = d.getFullYear();
  const m = pad(d.getMonth() + 1);
  const day = pad(d.getDate());
  const h = pad(d.getHours());
  const min = pad(d.getMinutes());
  const s = pad(d.getSeconds());
  return `${y}${m}${day}-${h}${min}${s}`;
}

function runStep(name, command, cwd) {
  const start = Date.now();
  const res = spawnSync(command, {
    cwd,
    shell: true,
    env: { ...process.env, CI: 'true' },
    encoding: 'utf8',
    maxBuffer: 1024 * 1024 * 100,
  });
  const durationMs = Date.now() - start;
  return {
    name,
    command,
    code: typeof res.status === 'number' ? res.status : -1,
    signal: res.signal || null,
    durationMs,
    stdout: res.stdout || '',
    stderr: res.stderr || '',
  };
}

(function main() {
  const projectRoot = process.cwd();
  const resultsDir = path.join(projectRoot, 'test-results');
  if (!fs.existsSync(resultsDir)) fs.mkdirSync(resultsDir, { recursive: true });

  const stamp = ts();
  const logPath = path.join(resultsDir, `verify-${stamp}.log`);
  const jsonPath = path.join(resultsDir, `verify-${stamp}.json`);

  const includeBench = process.env.VERIFY_INCLUDE_BENCH === 'true';
  const steps = [
    { name: 'cli-help', cmd: 'npm run -s cli:optimized -- --help' },
    ...(includeBench ? [{ name: 'bench-cli', cmd: 'npm run -s bench:cli' }] : []),
    { name: 'test-all', cmd: 'npm run -s test:all' },
    { name: 'cli-status', cmd: 'npm run -s cli:optimized -- status' },
    { name: 'cli-health', cmd: 'npm run -s cli:optimized -- health --format json' },
  ];

  const lines = [];
  const summary = [];

  lines.push(`# Claude Night Pilot Verification Log @ ${new Date().toISOString()}`);
  lines.push('');

  for (const s of steps) {
    lines.push(`\n===== [${s.name}] $ ${s.cmd}`);
    const result = runStep(s.name, s.cmd, projectRoot);
    summary.push({
      name: s.name,
      command: s.cmd,
      code: result.code,
      signal: result.signal,
      durationMs: result.durationMs,
    });
    if (result.stdout) {
      lines.push('\n--- stdout ---');
      lines.push(result.stdout.trimEnd());
    }
    if (result.stderr) {
      lines.push('\n--- stderr ---');
      lines.push(result.stderr.trimEnd());
    }
    lines.push(`\n--- exit: ${result.code} (${result.durationMs} ms)`);
  }

  const failed = summary.filter((s) => s.code !== 0);
  lines.push(`\n===== Summary: ${summary.length - failed.length} passed, ${failed.length} failed =====`);

  fs.writeFileSync(logPath, lines.join('\n'), 'utf8');
  fs.writeFileSync(jsonPath, JSON.stringify({ timestamp: new Date().toISOString(), summary }, null, 2), 'utf8');

  console.log(`\nVerification completed.\nLog: ${logPath}\nJSON: ${jsonPath}`);
  process.exit(failed.length ? 1 : 0);
})();


