#!/usr/bin/env node

/**
 * Claude Night Pilot - CLI 指令分析工具
 * 自動發現和分析所有 CLI 指令
 */

import fs from "fs";
import path from "path";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

class CLIAnalyzer {
  constructor() {
    this.binaries = ["cnp-unified", "cnp-optimized"];
    this.commands = new Map();
    this.outputDir = path.join(
      process.cwd(),
      "analysis",
      "reports",
      "cli-analysis"
    );

    fs.mkdirSync(this.outputDir, { recursive: true });
  }

  /**
   * 分析所有 CLI 二進制檔案
   */
  async analyzeAllBinaries() {
    console.log("🔍 Analyzing CLI binaries...");

    for (const binary of this.binaries) {
      try {
        await this.analyzeBinary(binary);
      } catch (error) {
        console.error(`❌ Failed to analyze ${binary}:`, error.message);
      }
    }

    await this.generateReport();
    await this.generateBDDScenarios();
  }

  /**
   * 分析單個二進制檔案
   */
  async analyzeBinary(binary) {
    console.log(`📋 Analyzing ${binary}...`);

    try {
      // 獲取主要幫助信息
      const { stdout: helpOutput } = await execAsync(
        `cd src-tauri && cargo run --bin ${binary} -- --help`,
        { timeout: 30000 }
      );

      const binaryInfo = {
        name: binary,
        version: this.extractVersion(helpOutput),
        description: this.extractDescription(helpOutput),
        commands: new Map(),
      };

      // 解析主要指令
      const commands = this.parseCommands(helpOutput);

      // 分析每個子指令
      for (const command of commands) {
        try {
          const commandInfo = await this.analyzeCommand(binary, command);
          binaryInfo.commands.set(command, commandInfo);
        } catch (error) {
          console.warn(
            `⚠️  Failed to analyze command ${command}:`,
            error.message
          );
        }
      }

      this.commands.set(binary, binaryInfo);
    } catch (error) {
      throw new Error(`Failed to get help for ${binary}: ${error.message}`);
    }
  }

  /**
   * 分析單個指令
   */
  async analyzeCommand(binary, command) {
    console.log(`  📝 Analyzing command: ${command}`);

    try {
      const { stdout } = await execAsync(
        `cd src-tauri && cargo run --bin ${binary} -- ${command} --help`,
        { timeout: 15000 }
      );

      return {
        name: command,
        description: this.extractDescription(stdout),
        usage: this.extractUsage(stdout),
        options: this.parseOptions(stdout),
        examples: this.generateExamples(binary, command),
        testScenarios: this.generateTestScenarios(binary, command),
      };
    } catch (error) {
      // 有些指令可能沒有 --help，嘗試直接執行
      try {
        const { stdout, stderr } = await execAsync(
          `cd src-tauri && timeout 5 cargo run --bin ${binary} -- ${command}`,
          { timeout: 10000 }
        );

        return {
          name: command,
          description: "No help available",
          usage: `${binary} ${command}`,
          options: [],
          examples: [`${binary} ${command}`],
          testScenarios: this.generateBasicTestScenarios(binary, command),
          output: stdout || stderr,
        };
      } catch (execError) {
        throw new Error(`Command execution failed: ${execError.message}`);
      }
    }
  }

  /**
   * 解析指令列表
   */
  parseCommands(helpOutput) {
    const commandsSection = helpOutput.match(
      /Commands?:(.*?)(?=\\n\\n|Options?:|$)/s
    );
    if (!commandsSection) return [];

    const commands = [];
    const lines = commandsSection[1].split("\\n");

    for (const line of lines) {
      const match = line.trim().match(/^(\\w+)/);
      if (match && !["help"].includes(match[1])) {
        commands.push(match[1]);
      }
    }

    return commands;
  }

  /**
   * 解析選項
   */
  parseOptions(helpOutput) {
    const optionsSection = helpOutput.match(/Options?:(.*?)(?=\\n\\n|$)/s);
    if (!optionsSection) return [];

    const options = [];
    const lines = optionsSection[1].split("\\n");

    for (const line of lines) {
      const match = line
        .trim()
        .match(/^(-\\w|--[\\w-]+)(?:\\s+<([^>]+)>)?\\s+(.+)/);
      if (match) {
        options.push({
          flag: match[1],
          argument: match[2] || null,
          description: match[3],
        });
      }
    }

    return options;
  }

  /**
   * 提取版本信息
   */
  extractVersion(output) {
    const versionMatch = output.match(/version\\s+([\\d.]+)/i);
    return versionMatch ? versionMatch[1] : "unknown";
  }

  /**
   * 提取描述
   */
  extractDescription(output) {
    const lines = output.split("\\n");
    for (let i = 0; i < Math.min(5, lines.length); i++) {
      const line = lines[i].trim();
      if (line && !line.startsWith("Usage:") && !line.includes("--")) {
        return line;
      }
    }
    return "No description available";
  }

  /**
   * 提取用法
   */
  extractUsage(output) {
    const usageMatch = output.match(/Usage:\\s*(.+)/);
    return usageMatch ? usageMatch[1].trim() : "";
  }

  /**
   * 生成使用範例
   */
  generateExamples(binary, command) {
    const examples = [`${binary} ${command}`];

    // 根據指令類型生成更多範例
    switch (command) {
      case "health":
        examples.push(`${binary} ${command} --format json`);
        break;
      case "cooldown":
        examples.push(`${binary} ${command} --format pretty`);
        break;
      case "prompt":
        examples.push(
          `${binary} ${command} create "Test Title" "Test Content"`
        );
        examples.push(`${binary} ${command} list`);
        break;
      case "run":
        examples.push(
          `${binary} ${command} --prompt "Hello World" --mode sync`
        );
        break;
    }

    return examples;
  }

  /**
   * 生成測試場景
   */
  generateTestScenarios(binary, command) {
    return [
      {
        name: `${command} basic functionality`,
        given: "The CLI tool is available",
        when: `I run '${binary} ${command}'`,
        then: [
          "The command should execute without errors",
          "The output should be properly formatted",
          "The exit code should be 0",
        ],
      },
      {
        name: `${command} help information`,
        given: "The CLI tool is available",
        when: `I run '${binary} ${command} --help'`,
        then: [
          "Help information should be displayed",
          "Usage examples should be shown",
          "Available options should be listed",
        ],
      },
    ];
  }

  /**
   * 生成基本測試場景
   */
  generateBasicTestScenarios(binary, command) {
    return [
      {
        name: `${command} execution`,
        given: "The system is initialized",
        when: `I run '${binary} ${command}'`,
        then: [
          "The command should complete",
          "Appropriate output should be generated",
        ],
      },
    ];
  }

  /**
   * 生成分析報告
   */
  async generateReport() {
    console.log("📊 Generating CLI analysis report...");

    const report = {
      timestamp: new Date().toISOString(),
      summary: {
        totalBinaries: this.binaries.length,
        analyzedBinaries: this.commands.size,
        totalCommands: Array.from(this.commands.values()).reduce(
          (sum, binary) => sum + binary.commands.size,
          0
        ),
      },
      binaries: {},
    };

    // 轉換 Map 為普通對象以便 JSON 序列化
    for (const [binaryName, binaryInfo] of this.commands) {
      report.binaries[binaryName] = {
        ...binaryInfo,
        commands: Object.fromEntries(binaryInfo.commands),
      };
    }

    const reportFile = path.join(this.outputDir, "cli-analysis-report.json");
    fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));

    console.log(`✅ Report saved to: ${reportFile}`);
  }

  /**
   * 生成 BDD 測試場景
   */
  async generateBDDScenarios() {
    console.log("🧪 Generating BDD test scenarios...");

    const scenarios = {
      feature: "Claude Night Pilot CLI",
      description: "Comprehensive CLI functionality testing",
      scenarios: [],
    };

    for (const [binaryName, binaryInfo] of this.commands) {
      for (const [commandName, commandInfo] of binaryInfo.commands) {
        scenarios.scenarios.push(...commandInfo.testScenarios);
      }
    }

    const scenariosFile = path.join(this.outputDir, "bdd-scenarios.yaml");
    const yamlContent = this.convertToYAML(scenarios);
    fs.writeFileSync(scenariosFile, yamlContent);

    console.log(`✅ BDD scenarios saved to: ${scenariosFile}`);
  }

  /**
   * 簡單的 YAML 轉換
   */
  convertToYAML(obj, indent = 0) {
    const spaces = "  ".repeat(indent);
    let yaml = "";

    for (const [key, value] of Object.entries(obj)) {
      if (Array.isArray(value)) {
        yaml += `${spaces}${key}:\\n`;
        for (const item of value) {
          if (typeof item === "object") {
            yaml += `${spaces}  -\\n`;
            yaml += this.convertToYAML(item, indent + 2).replace(/^/gm, "    ");
          } else {
            yaml += `${spaces}  - ${item}\\n`;
          }
        }
      } else if (typeof value === "object") {
        yaml += `${spaces}${key}:\\n`;
        yaml += this.convertToYAML(value, indent + 1);
      } else {
        yaml += `${spaces}${key}: ${value}\\n`;
      }
    }

    return yaml;
  }
}

// 主執行邏輯
async function main() {
  const analyzer = new CLIAnalyzer();
  await analyzer.analyzeAllBinaries();
  console.log("🎉 CLI analysis completed!");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export default CLIAnalyzer;
