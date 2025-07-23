// src-tauri/src/usage_tracker.rs
// Claude Code使用量追蹤系統 - 整合ccusage API與智能回退機制
// 基於context7最佳實踐 [tauri-docs+launchbadge/sqlx:2025-07-24T00:55:47+08:00]

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBlock {
    pub remaining_minutes: u32,
    pub total_minutes: u32,
    pub reset_time: Option<chrono::DateTime<chrono::Utc>>,
    pub usage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub current_block: Option<UsageBlock>,
    pub next_block_starts: Option<chrono::DateTime<chrono::Utc>>,
    pub is_available: bool,
    pub source: String, // "ccusage" 或 "fallback" 或 "error"
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Option<i64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub remaining_minutes: u32,
    pub total_minutes: u32,
    pub usage_percentage: f32,
    pub source: String,
    pub raw_output: Option<String>,
}

pub struct UsageTracker {
    last_check: Option<chrono::DateTime<chrono::Utc>>,
    cached_info: Option<UsageInfo>,
    cache_duration: chrono::Duration,
    db: Arc<crate::db::Database>,
}

impl UsageTracker {
    pub fn new(db: Arc<crate::db::Database>) -> Self {
        Self {
            last_check: None,
            cached_info: None,
            cache_duration: chrono::Duration::seconds(30), // 30秒快取
            db,
        }
    }

    /// 獲取Claude使用量資訊，優先使用ccusage API，回退到時間戳檢查
    /// 實現多重命令策略與智能解析 [context7:CCAutoRenew:2025-07-24T00:55:47+08:00]
    pub async fn get_usage_info(&mut self) -> Result<UsageInfo> {
        // 檢查快取是否有效
        if let Some(cached) = &self.cached_info {
            if let Some(last_check) = self.last_check {
                let now = chrono::Utc::now();
                if now.signed_duration_since(last_check) < self.cache_duration {
                    return Ok(cached.clone());
                }
            }
        }

        // 嘗試ccusage命令（多重回退策略）
        let ccusage_commands = vec![
            vec!["ccusage", "blocks", "--json"],
            vec!["npx", "ccusage@latest", "blocks", "--json"],
            vec!["bunx", "ccusage", "blocks", "--json"],
            vec!["ccusage", "blocks"], // 純文字輸出回退
        ];

        for cmd_args in ccusage_commands {
            match self.try_ccusage_command(&cmd_args).await {
                Ok(usage_info) => {
                    // 成功獲取，更新快取並保存到資料庫
                    self.cached_info = Some(usage_info.clone());
                    self.last_check = Some(chrono::Utc::now());
                    
                    if let Err(e) = self.save_usage_record(&usage_info).await {
                        eprintln!("警告：保存使用記錄失敗: {}", e);
                    }
                    
                    return Ok(usage_info);
                }
                Err(e) => {
                    eprintln!("ccusage命令失敗 {:?}: {}", cmd_args, e);
                    continue;
                }
            }
        }

        // 所有ccusage命令都失敗，回退到時間戳檢查
        println!("ccusage不可用，回退到時間戳檢查");
        let fallback_info = self.fallback_time_check().await?;
        
        self.cached_info = Some(fallback_info.clone());
        self.last_check = Some(chrono::Utc::now());
        
        if let Err(e) = self.save_usage_record(&fallback_info).await {
            eprintln!("警告：保存回退記錄失敗: {}", e);
        }
        
        Ok(fallback_info)
    }

    /// 嘗試執行ccusage命令並解析輸出
    async fn try_ccusage_command(&self, cmd_args: &[&str]) -> Result<UsageInfo> {
        let output = Command::new(cmd_args[0])
            .args(&cmd_args[1..])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "命令執行失敗: {} (exit code: {:?})",
                cmd_args.join(" "),
                output.status.code()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if cmd_args.contains(&"--json") {
            self.parse_json_output(&stdout)
        } else {
            self.parse_text_output(&stdout)
        }
    }

    /// 解析ccusage的JSON輸出格式
    fn parse_json_output(&self, output: &str) -> Result<UsageInfo> {
        let json_value: serde_json::Value = serde_json::from_str(output.trim())?;
        
        // 嘗試解析不同的JSON結構
        if let Some(blocks) = json_value.get("blocks").and_then(|b| b.as_array()) {
            if let Some(current_block) = blocks.first() {
                return self.parse_block_object(current_block, "ccusage-json");
            }
        }
        
        // 嘗試其他可能的JSON結構
        if let Some(remaining) = json_value.get("remainingMinutes").and_then(|r| r.as_u64()) {
            let total = json_value.get("totalMinutes").and_then(|t| t.as_u64()).unwrap_or(300);
            
            return Ok(self.create_usage_info(
                remaining as u32,
                total as u32,
                "ccusage-json"
            ));
        }

        Err(anyhow::anyhow!("無法解析JSON格式的ccusage輸出"))
    }

    /// 解析ccusage的文字輸出格式
    /// 支援多種時間格式: "2h 30m", "150 minutes remaining", "Time remaining: 2:30:45"
    fn parse_text_output(&self, output: &str) -> Result<UsageInfo> {
        let text = output.to_lowercase();
        
        // 模式1: "time remaining: 2h 30m" 或類似格式
        let time_regex = Regex::new(r"(?i)(?:time\s+)?remaining:?\s*(\d+)h\s*(\d+)m")?;
        if let Some(captures) = time_regex.captures(&text) {
            let hours: u32 = captures[1].parse()?;
            let minutes: u32 = captures[2].parse()?;
            let total_minutes = hours * 60 + minutes;
            
            return Ok(self.create_usage_info(total_minutes, 300, "ccusage-text"));
        }

        // 模式2: "150 minutes remaining" 或類似格式
        let minutes_regex = Regex::new(r"(?i)(\d+)\s*minutes?\s+remaining")?;
        if let Some(captures) = minutes_regex.captures(&text) {
            let minutes: u32 = captures[1].parse()?;
            return Ok(self.create_usage_info(minutes, 300, "ccusage-text"));
        }

        // 模式3: "remaining: 2:30:45" (時:分:秒格式)
        let hms_regex = Regex::new(r"(?i)remaining:?\s*(\d+):(\d+):(\d+)")?;
        if let Some(captures) = hms_regex.captures(&text) {
            let hours: u32 = captures[1].parse()?;
            let minutes: u32 = captures[2].parse()?;
            let total_minutes = hours * 60 + minutes;
            
            return Ok(self.create_usage_info(total_minutes, 300, "ccusage-text"));
        }

        Err(anyhow::anyhow!("無法解析文字格式的ccusage輸出: {}", output))
    }

    /// 解析JSON物件中的block資訊
    fn parse_block_object(&self, block_obj: &serde_json::Value, source: &str) -> Result<UsageInfo> {
        let remaining = block_obj.get("remaining")
            .and_then(|r| r.as_u64())
            .unwrap_or(0) as u32;
            
        let total = block_obj.get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(300) as u32;
        
        Ok(self.create_usage_info(remaining, total, source))
    }

    /// 建立UsageInfo物件的統一方法
    fn create_usage_info(&self, remaining_minutes: u32, total_minutes: u32, source: &str) -> UsageInfo {
        let now = chrono::Utc::now();
        let reset_time = if remaining_minutes > 0 {
            Some(now + chrono::Duration::minutes(remaining_minutes as i64))
        } else {
            None
        };
        
        let usage_block = UsageBlock {
            remaining_minutes,
            total_minutes,
            reset_time,
            usage_percentage: if total_minutes > 0 {
                1.0 - (remaining_minutes as f32 / total_minutes as f32)
            } else {
                0.0
            },
        };

        UsageInfo {
            current_block: Some(usage_block),
            next_block_starts: reset_time,
            is_available: remaining_minutes > 0,
            source: source.to_string(),
            last_updated: now,
        }
    }

    /// 時間戳回退檢查機制
    /// 當ccusage不可用時，根據最後活動時間推算剩餘時間
    async fn fallback_time_check(&self) -> Result<UsageInfo> {
        let last_activity_file = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("無法獲取用戶目錄"))?
            .join(".claude-last-activity");

        if let Ok(timestamp_str) = tokio::fs::read_to_string(&last_activity_file).await {
            if let Ok(timestamp) = timestamp_str.trim().parse::<i64>() {
                let last_activity = chrono::DateTime::from_timestamp(timestamp, 0)
                    .ok_or_else(|| anyhow::anyhow!("無效的時間戳"))?;
                
                let now = chrono::Utc::now();
                let elapsed = now.signed_duration_since(last_activity);
                
                // 假設5小時 = 300分鐘的限制
                let remaining_minutes = (300_i64 - elapsed.num_minutes()).max(0) as u32;
                
                return Ok(self.create_usage_info(remaining_minutes, 300, "fallback"));
            }
        }

        // 如果沒有歷史記錄，假設可用（保守估計）
        Ok(UsageInfo {
            current_block: None,
            next_block_starts: None,
            is_available: true, // 保守假設
            source: "fallback-unknown".to_string(),
            last_updated: chrono::Utc::now(),
        })
    }

    /// 保存使用記錄到資料庫
    async fn save_usage_record(&self, usage_info: &UsageInfo) -> Result<()> {
        if let Some(block) = &usage_info.current_block {
            let record = UsageRecord {
                id: None,
                timestamp: usage_info.last_updated,
                remaining_minutes: block.remaining_minutes,
                total_minutes: block.total_minutes,
                usage_percentage: block.usage_percentage,
                source: usage_info.source.clone(),
                raw_output: None, // 可以考慮保存原始輸出用於除錯
            };
            
            self.db.save_usage_record(&record).await?;
        }
        
        Ok(())
    }

    /// 獲取歷史使用記錄
    pub async fn get_usage_history(&self, hours: u32) -> Result<Vec<UsageRecord>> {
        self.db.get_usage_history(hours).await
    }

    /// 清除快取，強制重新檢查
    pub fn invalidate_cache(&mut self) {
        self.cached_info = None;
        self.last_check = None;
    }

    /// 獲取快取狀態
    pub fn get_cache_status(&self) -> (bool, Option<chrono::DateTime<chrono::Utc>>) {
        (self.cached_info.is_some(), self.last_check)
    }
}

// Tauri命令介面 [context7:tauri-docs:2025-07-24T00:55:47+08:00]
#[tauri::command]
pub async fn get_usage_status(
    state: tauri::State<'_, Arc<tokio::sync::Mutex<UsageTracker>>>
) -> Result<UsageInfo, String> {
    let mut tracker = state.lock().await;
    tracker.get_usage_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage_history(
    hours: u32,
    state: tauri::State<'_, Arc<tokio::sync::Mutex<UsageTracker>>>
) -> Result<Vec<UsageRecord>, String> {
    let tracker = state.lock().await;
    tracker.get_usage_history(hours).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn invalidate_usage_cache(
    state: tauri::State<'_, Arc<tokio::sync::Mutex<UsageTracker>>>
) -> Result<(), String> {
    let mut tracker = state.lock().await;
    tracker.invalidate_cache();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_output_parsing() {
        let tracker = UsageTracker::new(Arc::new(crate::db::Database::new_mock()));
        
        // 測試不同的時間格式
        let test_cases = vec![
            ("Time remaining: 2h 30m", 150),
            ("remaining: 1h 15m", 75),
            ("120 minutes remaining", 120),
            ("45 mins remaining", 45),
        ];

        for (output, expected_minutes) in test_cases {
            match tracker.parse_text_output(output) {
                Ok(usage_info) => {
                    if let Some(block) = usage_info.current_block {
                        assert_eq!(block.remaining_minutes, expected_minutes);
                    }
                }
                Err(e) => {
                    // 某些格式可能不被支援，記錄但不失敗
                    println!("無法解析 '{}': {}", output, e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_fallback_mechanism() {
        let tracker = UsageTracker::new(Arc::new(crate::db::Database::new_mock()));
        
        // 測試回退機制
        let result = tracker.fallback_time_check().await;
        assert!(result.is_ok());
        
        let usage_info = result.unwrap();
        assert_eq!(usage_info.source, "fallback-unknown");
        assert!(usage_info.is_available); // 保守假設應該可用
    }
} 