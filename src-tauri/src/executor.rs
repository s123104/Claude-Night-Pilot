use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub completion: String,
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CooldownInfo {
    pub is_cooling: bool,
    pub seconds_remaining: u64,
    pub next_available_time: Option<SystemTime>,
}

// 新增：執行選項配置 [最佳實踐:2025-07-24T00:55:47+08:00]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOptions {
    pub skip_permissions: bool,           // 是否跳過許可權確認
    pub output_format: String,           // 輸出格式 ("json", "text")
    pub timeout_seconds: Option<u64>,    // 執行超時（秒）
    pub dry_run: bool,                   // 試運行模式
    pub working_directory: Option<String>, // 工作目錄限制
    pub allowed_operations: Vec<String>, // 允許的操作類型
    pub safety_check: bool,              // 是否執行安全檢查
    pub max_retries: u32,                // 最大重試次數
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            skip_permissions: false,  // 預設安全模式
            output_format: "json".to_string(),
            timeout_seconds: Some(300), // 5分鐘預設超時
            dry_run: false,
            working_directory: None,
            allowed_operations: vec![
                "read".to_string(),
                "write".to_string(),
                "compile".to_string(),
            ],
            safety_check: true,       // 預設啟用安全檢查
            max_retries: 3,
        }
    }
}

// 新增：安全檢查結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    pub passed: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// 新增：執行審計日誌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAudit {
    pub id: Option<i64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub prompt_hash: String,              // prompt的SHA256雜湊
    pub options: ExecutionOptions,
    pub security_check: SecurityCheckResult,
    pub execution_start: Option<chrono::DateTime<chrono::Utc>>,
    pub execution_end: Option<chrono::DateTime<chrono::Utc>>,
    pub result: ExecutionResult,
    pub output_length: Option<usize>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionResult {
    Success,
    Failed,
    Cancelled,
    Timeout,
    SecurityBlocked,
}

pub struct ClaudeExecutor;

impl ClaudeExecutor {
    /// 使用執行選項執行Claude prompt - 新增安全檢查機制
    /// [最佳實踐:安全執行:2025-07-24T00:55:47+08:00]
    pub async fn run_with_options(prompt: &str, options: ExecutionOptions) -> Result<String> {
        let start_time = chrono::Utc::now();
        let prompt_hash = Self::calculate_prompt_hash(prompt);
        
        // 1. 安全檢查
        let security_check = Self::perform_security_check(prompt, &options).await?;
        
        if !security_check.passed {
            let audit = ExecutionAudit {
                id: None,
                timestamp: start_time,
                prompt_hash,
                options: options.clone(),
                security_check: security_check.clone(),
                execution_start: None,
                execution_end: Some(chrono::Utc::now()),
                result: ExecutionResult::SecurityBlocked,
                output_length: None,
                error_message: Some("安全檢查失敗".to_string()),
            };
            
            // 記錄安全阻擋事件
            Self::log_execution_audit(&audit).await?;
            
            bail!("安全檢查失敗: {:?}", security_check.errors);
        }

        // 2. 試運行模式
        if options.dry_run {
            println!("🧪 試運行模式：將執行 claude -p \"{}\" {}", 
                prompt.chars().take(50).collect::<String>(),
                if options.skip_permissions { "--dangerously-skip-permissions" } else { "" }
            );
            
            let audit = ExecutionAudit {
                id: None,
                timestamp: start_time,
                prompt_hash,
                options,
                security_check,
                execution_start: Some(start_time),
                execution_end: Some(chrono::Utc::now()),
                result: ExecutionResult::Success,
                output_length: Some(0),
                error_message: None,
            };
            
            Self::log_execution_audit(&audit).await?;
            
            return Ok("試運行模式：命令檢查通過".to_string());
        }

        // 3. 實際執行
        let execution_start = chrono::Utc::now();
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count <= options.max_retries {
            match Self::execute_command_with_timeout(prompt, &options).await {
                Ok(output) => {
                    let audit = ExecutionAudit {
                        id: None,
                        timestamp: start_time,
                        prompt_hash,
                        options,
                        security_check,
                        execution_start: Some(execution_start),
                        execution_end: Some(chrono::Utc::now()),
                        result: ExecutionResult::Success,
                        output_length: Some(output.len()),
                        error_message: None,
                    };
                    
                    Self::log_execution_audit(&audit).await?;
                    return Ok(output);
                }
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                    
                    if retry_count <= options.max_retries {
                        println!("執行失敗，將進行重試 {}/{}", retry_count, options.max_retries);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }

        // 所有重試都失敗
        let error = last_error.unwrap();
        let audit = ExecutionAudit {
            id: None,
            timestamp: start_time,
            prompt_hash,
            options,
            security_check,
            execution_start: Some(execution_start),
            execution_end: Some(chrono::Utc::now()),
            result: ExecutionResult::Failed,
            output_length: None,
            error_message: Some(error.to_string()),
        };
        
        Self::log_execution_audit(&audit).await?;
        Err(error)
    }

    /// 帶超時的命令執行
    async fn execute_command_with_timeout(prompt: &str, options: &ExecutionOptions) -> Result<String> {
        let timeout = std::time::Duration::from_secs(options.timeout_seconds.unwrap_or(300));
        
        let execution_future = async {
            let mut cmd = AsyncCommand::new("claude");
            cmd.arg("-p").arg(prompt);

            // 添加跳過許可權選項
            if options.skip_permissions {
                cmd.arg("--dangerously-skip-permissions");
            }

            // 設定輸出格式
            if options.output_format == "json" {
                cmd.arg("--output-format").arg("json");
            }

            // 設定工作目錄
            if let Some(work_dir) = &options.working_directory {
                cmd.current_dir(work_dir);
            }

            let output = cmd.output().await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);

                // 檢查是否為冷卻錯誤
                if stderr.contains("429") || stderr.contains("cooldown") {
                    if let Some(seconds) = Self::parse_cooldown_from_error(&stderr) {
                        bail!("Claude CLI 正在冷卻中，剩餘 {} 秒", seconds);
                    } else {
                        bail!("Claude CLI 冷卻中，請稍後再試");
                    }
                }

                bail!("Claude CLI 執行失敗: {}", stderr);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);

            // 嘗試解析 JSON 回應
            match serde_json::from_str::<ClaudeResponse>(&stdout) {
                Ok(response) => Ok(response.completion),
                Err(_) => {
                    // 如果不是 JSON 格式，直接返回原始輸出
                    Ok(stdout.trim().to_string())
                }
            }
        };

        match tokio::time::timeout(timeout, execution_future).await {
            Ok(result) => result,
            Err(_) => bail!("執行超時，超過 {} 秒", timeout.as_secs()),
        }
    }

    /// 安全檢查機制 [最佳實踐:安全驗證:2025-07-24T00:55:47+08:00]
    async fn perform_security_check(prompt: &str, options: &ExecutionOptions) -> Result<SecurityCheckResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut risk_level = RiskLevel::Low;

        // 1. 檢查是否允許跳過許可權
        if options.skip_permissions {
            // 需要從配置檔、環境變數或資料庫檢查是否允許
            if !Self::is_skip_permissions_allowed().await {
                errors.push("未授權跳過許可權操作".to_string());
                risk_level = RiskLevel::Critical;
            } else {
                warnings.push("使用危險的跳過許可權模式".to_string());
                risk_level = RiskLevel::High;
            }
        }

        // 2. 檢查工作目錄安全性
        if let Some(work_dir) = &options.working_directory {
            if !Self::is_safe_working_directory(work_dir) {
                errors.push(format!("不安全的工作目錄: {}", work_dir));
                risk_level = RiskLevel::High;
            }
        }

        // 3. 檢查prompt內容是否包含危險模式
        let dangerous_patterns = vec![
            "rm -rf",
            "sudo",
            "chmod 777",
            "delete",
            "format",
            "mkfs",
        ];

        for pattern in dangerous_patterns {
            if prompt.to_lowercase().contains(pattern) {
                warnings.push(format!("檢測到潛在危險操作: {}", pattern));
                if risk_level == RiskLevel::Low {
                    risk_level = RiskLevel::Medium;
                }
            }
        }

        // 4. 檢查prompt長度
        if prompt.len() > 10000 {
            warnings.push("Prompt長度異常，可能包含大量數據".to_string());
        }

        // 5. 檢查允許的操作類型
        if options.allowed_operations.is_empty() {
            warnings.push("未定義允許的操作類型".to_string());
        }

        Ok(SecurityCheckResult {
            passed: errors.is_empty(),
            warnings,
            errors,
            risk_level,
        })
    }

    /// 檢查是否允許跳過許可權
    async fn is_skip_permissions_allowed() -> bool {
        // 這裡可以檢查配置檔、環境變數或資料庫設定
        // 暫時通過環境變數檢查
        std::env::var("CLAUDE_ALLOW_SKIP_PERMISSIONS")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false)
    }

    /// 檢查工作目錄是否安全
    fn is_safe_working_directory(work_dir: &str) -> bool {
        let dangerous_paths = vec![
            "/bin",
            "/usr/bin",
            "/etc",
            "/var",
            "/sys",
            "/proc",
        ];

        // 檢查是否是根目錄
        if work_dir == "/" {
            return false;
        }

        for dangerous_path in dangerous_paths {
            if work_dir.starts_with(dangerous_path) {
                return false;
            }
        }

        // 檢查是否是相對路徑包含 ..
        if work_dir.contains("..") {
            return false;
        }

        true
    }

    /// 計算prompt的SHA256雜湊
    fn calculate_prompt_hash(prompt: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 記錄執行審計日誌
    async fn log_execution_audit(audit: &ExecutionAudit) -> Result<()> {
        // 這裡應該保存到資料庫，暫時輸出到日誌
        println!("🔒 執行審計: {} | 結果: {:?} | 風險: {:?}", 
            audit.timestamp.format("%Y-%m-%d %H:%M:%S"),
            audit.result,
            audit.security_check.risk_level
        );
        
        if !audit.security_check.warnings.is_empty() {
            println!("⚠️  安全警告: {:?}", audit.security_check.warnings);
        }

        // TODO: 實際保存到資料庫
        // self.db.save_execution_audit(audit).await?;
        
        Ok(())
    }

    /// 同步執行 Claude prompt (保持向後相容)
    pub async fn run_sync(prompt: &str) -> Result<String> {
        let options = ExecutionOptions::default();
        Self::run_with_options(prompt, options).await
    }

    /// 檢查 Claude CLI 狀態和冷卻時間
    pub async fn check_cooldown() -> Result<CooldownInfo> {
        let output = AsyncCommand::new("claude")
            .arg("doctor")
            .arg("--json")
            .output()
            .await?;

        if !output.status.success() {
            // 如果 doctor 命令失敗，假設沒有冷卻
            return Ok(CooldownInfo {
                is_cooling: false,
                seconds_remaining: 0,
                next_available_time: None,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 嘗試解析診斷資訊
        if let Ok(diag) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(cooldown_secs) = diag.get("cooldown_seconds").and_then(|v| v.as_u64()) {
                return Ok(CooldownInfo {
                    is_cooling: cooldown_secs > 0,
                    seconds_remaining: cooldown_secs,
                    next_available_time: if cooldown_secs > 0 {
                        Some(SystemTime::now() + std::time::Duration::from_secs(cooldown_secs))
                    } else {
                        None
                    },
                });
            }
        }

        // 如果無法解析，假設沒有冷卻
        Ok(CooldownInfo {
            is_cooling: false,
            seconds_remaining: 0,
            next_available_time: None,
        })
    }

    /// 驗證 Claude CLI 是否可用
    pub async fn verify_claude_cli() -> Result<bool> {
        let output = AsyncCommand::new("claude").arg("--version").output().await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 從錯誤訊息中解析冷卻時間
    pub fn parse_cooldown_from_error(error_message: &str) -> Option<u64> {
        // 尋找冷卻時間模式: "cooldown: 123s" 或 "wait 123 seconds"
        use regex::Regex;

        let patterns = [
            r"cooldown[:\s]+(\d+)s",
            r"wait\s+(\d+)\s+seconds?",
            r"retry\s+in\s+(\d+)\s+seconds?",
            r"(\d+)\s+seconds?\s+remaining",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(error_message) {
                    if let Some(seconds_str) = captures.get(1) {
                        if let Ok(seconds) = seconds_str.as_str().parse::<u64>() {
                            return Some(seconds);
                        }
                    }
                }
            }
        }

        None
    }

    /// 建立測試用的 mock executor（開發階段使用）
    #[cfg(debug_assertions)]
    pub async fn run_mock(prompt: &str) -> Result<String> {
        // 模擬執行時間
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(format!(
            "模擬回應 [{}]: 這是對 prompt '{}' 的模擬回應。此為開發階段的測試功能。",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            prompt.chars().take(50).collect::<String>()
        ))
    }
}

// Tauri命令介面 [最佳實踐:tauri-docs:2025-07-24T00:55:47+08:00]
#[tauri::command]
pub async fn execute_claude_with_options(
    prompt: String,
    options: ExecutionOptions,
) -> Result<String, String> {
    ClaudeExecutor::run_with_options(&prompt, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_claude_safe(prompt: String) -> Result<String, String> {
    let options = ExecutionOptions::default();
    ClaudeExecutor::run_with_options(&prompt, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_execution_options(options: ExecutionOptions) -> Result<SecurityCheckResult, String> {
    ClaudeExecutor::perform_security_check("", &options)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cooldown_from_error() {
        let test_cases = [
            ("Error: cooldown: 123s", Some(123)),
            ("Please wait 45 seconds before retrying", Some(45)),
            ("Rate limited. retry in 60 seconds", Some(60)),
            ("30 seconds remaining", Some(30)),
            ("No cooldown message", None),
        ];

        for (error_msg, expected) in &test_cases {
            assert_eq!(
                ClaudeExecutor::parse_cooldown_from_error(error_msg),
                *expected,
                "Failed for: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_security_check() {
        let options = ExecutionOptions::default();
        let result = ClaudeExecutor::perform_security_check("test prompt", &options).await;
        assert!(result.is_ok());

        let security_check = result.unwrap();
        assert!(security_check.passed);
    }

    #[tokio::test]
    async fn test_dangerous_patterns() {
        let options = ExecutionOptions::default();
        let result = ClaudeExecutor::perform_security_check("sudo rm -rf /", &options).await;
        assert!(result.is_ok());

        let security_check = result.unwrap();
        assert!(!security_check.warnings.is_empty());
        assert!(matches!(security_check.risk_level, RiskLevel::Medium));
    }

    #[test]
    fn test_safe_working_directory() {
        assert!(ClaudeExecutor::is_safe_working_directory("/home/user/project"));
        assert!(!ClaudeExecutor::is_safe_working_directory("/etc"));
        assert!(!ClaudeExecutor::is_safe_working_directory("../../../"));
        assert!(!ClaudeExecutor::is_safe_working_directory("/bin"));
    }

    #[tokio::test]
    async fn test_mock_executor() {
        let result = ClaudeExecutor::run_mock("Test prompt").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("模擬回應"));
        assert!(response.contains("Test prompt"));
    }

    #[test]
    fn test_execution_options_default() {
        let options = ExecutionOptions::default();
        assert!(!options.skip_permissions);
        assert!(options.safety_check);
        assert_eq!(options.output_format, "json");
        assert_eq!(options.timeout_seconds, Some(300));
    }
}
