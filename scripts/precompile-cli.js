#!/usr/bin/env node

/**
 * Pre-compile CLI binaries to avoid Cargo lock contention during tests
 */

import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

async function precompileBinaries() {
  console.log("🔨 Pre-compiling CLI binaries to avoid test lock contention...");

  const binaries = ["cnp-unified", "cnp-optimized"];

  for (const binary of binaries) {
    try {
      console.log(`   Compiling ${binary}...`);
      const { stdout, stderr } = await execAsync(
        `cd src-tauri && cargo build --bin ${binary}`,
        { timeout: 300000 } // 5 minutes timeout
      );

      if (stderr && !stderr.includes("Finished")) {
        console.log(
          `   ⚠️  ${binary} warnings: ${stderr.substring(0, 200)}...`
        );
      }

      console.log(`   ✅ ${binary} compiled successfully`);
    } catch (error) {
      console.error(`   ❌ Failed to compile ${binary}:`, error.message);
      throw error;
    }
  }

  console.log("✅ All CLI binaries pre-compiled successfully!");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  precompileBinaries().catch((error) => {
    console.error("❌ Pre-compilation failed:", error);
    process.exit(1);
  });
}

export default precompileBinaries;
