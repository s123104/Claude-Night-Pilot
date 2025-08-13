#!/usr/bin/env node
/*
  Sync version across package.json, src-tauri/Cargo.toml and README badge.
  Usage: node scripts/sync-version.js [version]
  - If version is omitted, uses package.json version
*/
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();
const pkgPath = join(root, 'package.json');
const cargoPath = join(root, 'src-tauri', 'Cargo.toml');
const readmePath = join(root, 'README.md');

const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'));
const newVersion = process.argv[2] || pkg.version;

// Update Cargo.toml version
let cargo = readFileSync(cargoPath, 'utf8');
cargo = cargo.replace(/version\s*=\s*"[^"]+"/g, `version = "${newVersion}"`);
writeFileSync(cargoPath, cargo);

// Update README badge version
let readme = readFileSync(readmePath, 'utf8');
readme = readme.replace(/badge\/version-[0-9]+\.[0-9]+\.[0-9]+-blue\.svg/, `badge/version-${newVersion}-blue.svg`);
writeFileSync(readmePath, readme);

console.log(`[sync-version] Synchronized to ${newVersion}`);


