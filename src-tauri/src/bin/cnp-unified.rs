// Claude Night Pilot 統一CLI工具
// 使用統一介面確保與GUI功能一致

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use claude_night_pilot_lib::unified_interface::{UnifiedClaudeInterface, UnifiedExecutionOptions};
use serde_json::json;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "cnp")]
#[command(about = "Claude Night Pilot - 統一的Claude自動化CLI工具")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 執行Claude命令
    Execute {
        /// 要執行的prompt內容
        #[arg(short, long, value_name = "TEXT")]
        prompt: Option<String>,
        
        /// 從檔案讀取prompt
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,
        
        /// 從stdin讀取prompt
        #[arg(long)]
        stdin: bool,
        
        /// 執行模式 (sync, async, scheduled)
        #[arg(short, long, default_value = "sync")]
        mode: String,
        
        /// 工作目錄
        #[arg(short, long)]
        work_dir: Option<String>,
        
        /// 啟用重試機制
        #[arg(long, default_value = "true")]
        retry: bool,
        
        /// 檢查冷卻狀態
        #[arg(long, default_value = "true")]
        cooldown_check: bool,
        
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    
    /// 檢查冷卻狀態
    Cooldown {
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    
    /// 系統健康檢查
    Health {
        /// 輸出格式 (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    
    /// 批量執行prompts
    Batch {
        /// 包含prompts的JSON檔案
        #[arg(short, long, value_name = "FILE")]
        file: String,
        
        /// 並發執行數量
        #[arg(short, long, default_value = "1")]
        concurrent: u32,
        
        /// 執行模式
        #[arg(short, long, default_value = "sync")]
        mode: String,
        
        /// 輸出格式
        #[arg(long, default_value = "pretty")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Execute {
            prompt,
            file,
            stdin,
            mode,
            work_dir,
            retry,
            cooldown_check,
            format,
        } => {
            execute_prompt(prompt, file, stdin, mode, work_dir, retry, cooldown_check, format).await
        }
        
        Commands::Cooldown { format } => {
            check_cooldown(format).await
        }
        
        Commands::Health { format } => {
            health_check(format).await
        }
        
        Commands::Batch { file, concurrent, mode, format } => {
            batch_execute(file, concurrent, mode, format).await
        }
    }
}

async fn execute_prompt(
    prompt: Option<String>,
    file: Option<String>,
    stdin: bool,
    mode: String,
    work_dir: Option<String>,
    retry: bool,
    cooldown_check: bool,
    format: String,
) -> Result<()> {
    // 獲取prompt內容
    let prompt_content = if let Some(content) = prompt {
        content
    } else if let Some(file_path) = file {
        std::fs::read_to_string(&file_path)
            .with_context(|| format!("無法讀取檔案: {}", file_path))?
    } else if stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .context("無法從stdin讀取內容")?;
        buffer
    } else {
        anyhow::bail!("必須提供prompt內容 (使用 -p, -f, 或 --stdin)");
    };

    // 準備執行選項
    let options = UnifiedExecutionOptions {
        mode,
        cron_expr: None,
        retry_enabled: Some(retry),
        cooldown_check: Some(cooldown_check),
        working_directory: work_dir,
    };

    // 執行命令
    if format != "json" {
        println!("🚀 正在執行Claude命令...");
    }
    
    let result = UnifiedClaudeInterface::execute_claude(prompt_content, options)
        .await
        .context("執行Claude命令失敗")?;

    // 輸出結果
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "text" => {
            println!("{}", result.completion);
        }
        "pretty" | _ => {
            print_pretty_result(&result);
        }
    }

    Ok(())
}

async fn check_cooldown(format: String) -> Result<()> {
    if format != "json" {
        println!("🕐 檢查冷卻狀態...");
    }
    
    let cooldown_info = UnifiedClaudeInterface::check_cooldown()
        .await
        .context("檢查冷卻狀態失敗")?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&cooldown_info)?);
        }
        "text" => {
            if cooldown_info.is_cooling {
                println!("系統冷卻中，剩餘 {} 秒", cooldown_info.seconds_remaining);
            } else {
                println!("系統可用");
            }
        }
        "pretty" | _ => {
            print_pretty_cooldown(&cooldown_info);
        }
    }

    Ok(())
}

async fn health_check(format: String) -> Result<()> {
    if format != "json" {
        println!("🏥 執行系統健康檢查...");
    }
    
    let health_status = UnifiedClaudeInterface::health_check()
        .await
        .context("系統健康檢查失敗")?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&health_status)?);
        }
        "pretty" | _ => {
            print_pretty_health(&health_status);
        }
    }

    Ok(())
}

async fn batch_execute(file: String, concurrent: u32, mode: String, format: String) -> Result<()> {
    if format != "json" {
        println!("📦 批量執行模式 (並發: {})", concurrent);
    }
    
    // 讀取批量執行配置
    let content = std::fs::read_to_string(&file)
        .with_context(|| format!("無法讀取檔案: {}", file))?;
    
    let prompts: Vec<serde_json::Value> = serde_json::from_str(&content)
        .context("檔案格式錯誤，期望JSON陣列")?;

    let mut results = Vec::new();
    
    for (index, prompt_value) in prompts.iter().enumerate() {
        let prompt_text = prompt_value
            .as_str()
            .or_else(|| prompt_value["content"].as_str())
            .or_else(|| prompt_value["prompt"].as_str())
            .ok_or_else(|| anyhow::anyhow!("Prompt {} 格式錯誤", index + 1))?;

        println!("執行 Prompt {}/{}", index + 1, prompts.len());

        let options = UnifiedExecutionOptions {
            mode: mode.clone(),
            cron_expr: None,
            retry_enabled: Some(true),
            cooldown_check: Some(true),
            working_directory: None,
        };

        match UnifiedClaudeInterface::execute_claude(prompt_text.to_string(), options).await {
            Ok(result) => {
                results.push(json!({
                    "index": index + 1,
                    "status": "success",
                    "result": result
                }));
                println!("✅ Prompt {} 執行成功", index + 1);
            }
            Err(error) => {
                results.push(json!({
                    "index": index + 1,
                    "status": "failed",
                    "error": error.to_string()
                }));
                println!("❌ Prompt {} 執行失敗: {}", index + 1, error);
            }
        }
    }

    // 輸出批量執行結果
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        "pretty" | _ => {
            print_pretty_batch_results(&results);
        }
    }

    Ok(())
}

fn print_pretty_result(result: &claude_night_pilot_lib::enhanced_executor::EnhancedClaudeResponse) {
    println!("\n🎯 執行結果");
    println!("═══════════════════════════════════════");
    println!("{}", result.completion);
    
    if let Some(usage) = &result.usage {
        println!("\n📊 使用統計");
        println!("───────────────────────────");
        println!("輸入Token: {}", usage.input_tokens.unwrap_or(0));
        println!("輸出Token: {}", usage.output_tokens.unwrap_or(0));
    }
    
    let metadata = &result.execution_metadata;
    println!("\n🔍 執行信息");
    println!("───────────────────────────");
    println!("執行ID: {}", metadata.execution_id);
    println!("重試次數: {}", metadata.total_attempts);
    if let Some(scheduler) = &metadata.scheduler_used {
        println!("排程器: {}", scheduler);
    }
}

fn print_pretty_cooldown(cooldown: &claude_night_pilot_lib::core::CooldownInfo) {
    println!("\n🕐 冷卻狀態");
    println!("═══════════════════════════════════════");
    
    if cooldown.is_cooling {
        println!("❌ 系統冷卻中");
        println!("剩餘時間: {} 秒", cooldown.seconds_remaining);
        if let Some(reset_time) = &cooldown.reset_time {
            println!("重置時間: {}", reset_time.format("%H:%M:%S"));
        }
        if let Some(pattern) = &cooldown.cooldown_pattern {
            println!("冷卻模式: {:?}", pattern);
        }
    } else {
        println!("✅ 系統可用");
        println!("狀態: 可立即執行");
    }
    
    if !cooldown.original_message.is_empty() {
        println!("原始信息: {}", cooldown.original_message);
    }
}

fn print_pretty_health(health: &serde_json::Value) {
    println!("\n🏥 系統健康狀態");
    println!("═══════════════════════════════════════");
    
    if let Some(claude_available) = health["claude_cli_available"].as_bool() {
        println!("Claude CLI: {}", if claude_available { "✅ 可用" } else { "❌ 不可用" });
    }
    
    if let Some(cooldown_working) = health["cooldown_detection_working"].as_bool() {
        println!("冷卻檢測: {}", if cooldown_working { "✅ 正常" } else { "❌ 異常" });
    }
    
    if let Some(processes) = health["active_processes"].as_u64() {
        println!("活躍進程: {}", processes);
    }
    
    if let Some(last_check) = health["last_check"].as_str() {
        println!("最後檢查: {}", last_check);
    }
}

fn print_pretty_batch_results(results: &[serde_json::Value]) {
    println!("\n📦 批量執行結果");
    println!("═══════════════════════════════════════");
    
    let success_count = results.iter().filter(|r| r["status"] == "success").count();
    let failed_count = results.len() - success_count;
    
    println!("總計: {} 個任務", results.len());
    println!("成功: {} 個", success_count);
    println!("失敗: {} 個", failed_count);
    
    println!("\n詳細結果:");
    for result in results {
        let index = result["index"].as_u64().unwrap_or(0);
        let status = result["status"].as_str().unwrap_or("unknown");
        
        match status {
            "success" => println!("  ✅ Prompt {}: 執行成功", index),
            "failed" => {
                let error = result["error"].as_str().unwrap_or("未知錯誤");
                println!("  ❌ Prompt {}: {}", index, error);
            }
            _ => println!("  ❓ Prompt {}: 狀態未知", index),
        }
    }
}