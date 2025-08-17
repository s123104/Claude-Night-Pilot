// Claude Code Authentication Auto-Detection
// 實現智能檢測已存在的 Claude Code 認證，避免重複設定

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    /// Anthropic Console OAuth (推薦方式)
    ConsoleOAuth {
        token_path: PathBuf,
        expires_at: Option<String>,
    },
    /// Claude App Pro/Max 訂閱
    ClaudeApp {
        app_session: String,
    },
    /// API Key 認證
    ApiKey {
        source: ApiKeySource,
        masked_key: String,
    },
    /// AWS Bedrock 企業認證
    Bedrock {
        region: String,
        profile: Option<String>,
    },
    /// Google Vertex AI 企業認證
    VertexAI {
        project_id: String,
        region: String,
    },
    /// 未檢測到認證
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeySource {
    EnvironmentVariable,
    ConfigFile,
    Helper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationStatus {
    pub method: AuthenticationMethod,
    pub is_valid: bool,
    pub last_verified: chrono::DateTime<chrono::Utc>,
    pub user_info: Option<UserInfo>,
    pub capabilities: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub subscription_type: Option<String>,
    pub organization: Option<String>,
}

pub struct ClaudeAuthDetector {
    config_paths: Vec<PathBuf>,
    token_paths: Vec<PathBuf>,
}

impl ClaudeAuthDetector {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        let config_paths = vec![
            home_dir.join(".claude"),
            home_dir.join(".config/claude"),
            PathBuf::from("./.claude"),
        ];

        let token_paths = vec![
            home_dir.join(".claude/auth"),
            home_dir.join(".claude/tokens"),
            home_dir.join(".config/claude/auth"),
            home_dir.join(".anthropic"),
        ];

        Self {
            config_paths,
            token_paths,
        }
    }

    /// 執行完整的認證檢測
    pub async fn detect_authentication(&self) -> Result<AuthenticationStatus> {
        tracing::info!("開始 Claude Code 認證自動檢測...");

        // 1. 檢測 API Key 認證
        if let Some(auth_method) = self.detect_api_key_auth().await? {
            let status = self.verify_authentication(&auth_method).await?;
            if status.is_valid {
                tracing::info!("檢測到有效的 API Key 認證");
                return Ok(status);
            }
        }

        // 2. 檢測 OAuth 認證
        if let Some(auth_method) = self.detect_oauth_auth().await? {
            let status = self.verify_authentication(&auth_method).await?;
            if status.is_valid {
                tracing::info!("檢測到有效的 OAuth 認證");
                return Ok(status);
            }
        }

        // 3. 檢測企業認證 (Bedrock)
        if let Some(auth_method) = self.detect_bedrock_auth().await? {
            let status = self.verify_authentication(&auth_method).await?;
            if status.is_valid {
                tracing::info!("檢測到有效的 Bedrock 認證");
                return Ok(status);
            }
        }

        // 4. 檢測 Vertex AI 認證
        if let Some(auth_method) = self.detect_vertex_auth().await? {
            let status = self.verify_authentication(&auth_method).await?;
            if status.is_valid {
                tracing::info!("檢測到有效的 Vertex AI 認證");
                return Ok(status);
            }
        }

        // 5. 檢測 Claude App 認證
        if let Some(auth_method) = self.detect_claude_app_auth().await? {
            let status = self.verify_authentication(&auth_method).await?;
            if status.is_valid {
                tracing::info!("檢測到有效的 Claude App 認證");
                return Ok(status);
            }
        }

        // 沒有檢測到有效認證
        tracing::warn!("未檢測到有效的 Claude Code 認證");
        Ok(AuthenticationStatus {
            method: AuthenticationMethod::None,
            is_valid: false,
            last_verified: chrono::Utc::now(),
            user_info: None,
            capabilities: vec![],
            recommendations: self.generate_setup_recommendations(),
        })
    }

    /// 檢測 API Key 認證
    async fn detect_api_key_auth(&self) -> Result<Option<AuthenticationMethod>> {
        // 檢查環境變數
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            if self.is_valid_api_key(&api_key) {
                return Ok(Some(AuthenticationMethod::ApiKey {
                    source: ApiKeySource::EnvironmentVariable,
                    masked_key: self.mask_api_key(&api_key),
                }));
            }
        }

        if let Ok(auth_token) = env::var("ANTHROPIC_AUTH_TOKEN") {
            if self.is_valid_api_key(&auth_token) {
                return Ok(Some(AuthenticationMethod::ApiKey {
                    source: ApiKeySource::EnvironmentVariable,
                    masked_key: self.mask_api_key(&auth_token),
                }));
            }
        }

        // 檢查設定檔案中的 API Key
        for config_path in &self.config_paths {
            let settings_file = config_path.join("settings.json");
            if settings_file.exists() {
                if let Ok(content) = fs::read_to_string(&settings_file).await {
                    if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(api_key) = settings.get("api_key").and_then(|v| v.as_str()) {
                            if self.is_valid_api_key(api_key) {
                                return Ok(Some(AuthenticationMethod::ApiKey {
                                    source: ApiKeySource::ConfigFile,
                                    masked_key: self.mask_api_key(api_key),
                                }));
                            }
                        }
                    }
                }
            }
        }

        // 檢查 apiKeyHelper 設定
        if let Some(helper_result) = self.check_api_key_helper().await? {
            return Ok(Some(helper_result));
        }

        Ok(None)
    }

    /// 檢測 OAuth 認證
    async fn detect_oauth_auth(&self) -> Result<Option<AuthenticationMethod>> {
        for token_path in &self.token_paths {
            // 檢查 OAuth token 檔案
            let oauth_token_file = token_path.join("oauth_token");
            let session_file = token_path.join("session.json");
            
            if oauth_token_file.exists() || session_file.exists() {
                let token_file = if oauth_token_file.exists() {
                    oauth_token_file
                } else {
                    session_file
                };

                // 檢查 token 是否過期
                if let Ok(metadata) = fs::metadata(&token_file).await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                        let now = chrono::Utc::now();
                        
                        // 如果 token 檔案在過去 24 小時內修改過，認為可能有效
                        if (now - modified_time).num_hours() < 24 {
                            return Ok(Some(AuthenticationMethod::ConsoleOAuth {
                                token_path: token_file,
                                expires_at: None, // 實際實作中應該解析 token 檔案獲取過期時間
                            }));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 檢測 AWS Bedrock 認證
    async fn detect_bedrock_auth(&self) -> Result<Option<AuthenticationMethod>> {
        // 檢查 CLAUDE_CODE_USE_BEDROCK 環境變數
        if env::var("CLAUDE_CODE_USE_BEDROCK").unwrap_or_default() == "1" {
            let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
            let profile = env::var("AWS_PROFILE").ok();

            // 檢查 AWS 認證
            if self.check_aws_credentials().await {
                return Ok(Some(AuthenticationMethod::Bedrock { region, profile }));
            }
        }

        Ok(None)
    }

    /// 檢測 Google Vertex AI 認證
    async fn detect_vertex_auth(&self) -> Result<Option<AuthenticationMethod>> {
        if env::var("CLAUDE_CODE_USE_VERTEX").unwrap_or_default() == "1" {
            let project_id = env::var("ANTHROPIC_VERTEX_PROJECT_ID")
                .or_else(|_| env::var("GOOGLE_CLOUD_PROJECT"))
                .unwrap_or_default();
            let region = env::var("CLOUD_ML_REGION").unwrap_or_else(|_| "us-east5".to_string());

            if !project_id.is_empty() && self.check_gcp_credentials().await {
                return Ok(Some(AuthenticationMethod::VertexAI {
                    project_id,
                    region,
                }));
            }
        }

        Ok(None)
    }

    /// 檢測 Claude App 認證
    async fn detect_claude_app_auth(&self) -> Result<Option<AuthenticationMethod>> {
        // 檢查 Claude App 相關的 session 檔案
        // 這個需要根據實際的 Claude App 儲存位置來實作
        // 目前暫時回傳 None
        Ok(None)
    }

    /// 驗證認證方法是否有效
    async fn verify_authentication(&self, auth_method: &AuthenticationMethod) -> Result<AuthenticationStatus> {
        tracing::debug!("驗證認證方法: {:?}", auth_method);

        // 使用 claude doctor 命令來驗證認證
        let verification_result = self.run_claude_doctor().await?;
        
        let is_valid = verification_result.success;
        let user_info = self.extract_user_info(&verification_result).await;
        let capabilities = self.extract_capabilities(&verification_result).await;

        Ok(AuthenticationStatus {
            method: auth_method.clone(),
            is_valid,
            last_verified: chrono::Utc::now(),
            user_info,
            capabilities,
            recommendations: if is_valid {
                vec![]
            } else {
                self.generate_fix_recommendations(auth_method)
            },
        })
    }

    /// 執行 claude doctor 進行健康檢查
    async fn run_claude_doctor(&self) -> Result<DoctorResult> {
        let output = Command::new("claude")
            .args(["doctor", "--json"])
            .output()
            .context("無法執行 claude doctor 命令")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(DoctorResult {
            success: output.status.success(),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        })
    }

    /// 檢查 API Key 格式是否有效
    fn is_valid_api_key(&self, key: &str) -> bool {
        // Anthropic API Key 格式: sk-ant-apiXX-XXXXXXX
        key.starts_with("sk-ant-api") && key.len() > 20
    }

    /// 遮罩 API Key 敏感資訊
    fn mask_api_key(&self, key: &str) -> String {
        if key.len() > 10 {
            format!("{}...{}", &key[..10], &key[key.len()-4..])
        } else {
            "***".to_string()
        }
    }

    /// 檢查 API Key Helper
    async fn check_api_key_helper(&self) -> Result<Option<AuthenticationMethod>> {
        for config_path in &self.config_paths {
            let settings_file = config_path.join("settings.json");
            if settings_file.exists() {
                if let Ok(content) = fs::read_to_string(&settings_file).await {
                    if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(helper_path) = settings.get("apiKeyHelper").and_then(|v| v.as_str()) {
                            // 測試執行 helper
                            if let Ok(output) = Command::new(helper_path).output() {
                                if output.status.success() {
                                    let key = String::from_utf8_lossy(&output.stdout).trim().to_string();
                                    if self.is_valid_api_key(&key) {
                                        return Ok(Some(AuthenticationMethod::ApiKey {
                                            source: ApiKeySource::Helper,
                                            masked_key: self.mask_api_key(&key),
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// 檢查 AWS 認證
    async fn check_aws_credentials(&self) -> bool {
        // 嘗試使用 AWS CLI 檢查認證
        if let Ok(output) = Command::new("aws")
            .args(["sts", "get-caller-identity"])
            .output()
        {
            output.status.success()
        } else {
            false
        }
    }

    /// 檢查 GCP 認證
    async fn check_gcp_credentials(&self) -> bool {
        // 嘗試使用 gcloud CLI 檢查認證
        if let Ok(output) = Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
        {
            output.status.success()
        } else {
            false
        }
    }

    /// 提取使用者資訊
    async fn extract_user_info(&self, doctor_result: &DoctorResult) -> Option<UserInfo> {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&doctor_result.stdout) {
            let user_id = json.get("user_id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let email = json.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
            let subscription = json.get("subscription").and_then(|v| v.as_str()).map(|s| s.to_string());
            let org = json.get("organization").and_then(|v| v.as_str()).map(|s| s.to_string());

            if user_id.is_some() || email.is_some() {
                return Some(UserInfo {
                    user_id,
                    email,
                    subscription_type: subscription,
                    organization: org,
                });
            }
        }
        None
    }

    /// 提取功能清單
    async fn extract_capabilities(&self, doctor_result: &DoctorResult) -> Vec<String> {
        let mut capabilities = vec![];

        if doctor_result.success {
            capabilities.push("基本 Claude 查詢".to_string());
            capabilities.push("檔案讀寫操作".to_string());
            capabilities.push("Git 整合".to_string());
            
            // 根據 doctor 結果檢測額外功能
            if doctor_result.stdout.contains("mcp") {
                capabilities.push("MCP 伺服器支援".to_string());
            }
            if doctor_result.stdout.contains("subagents") {
                capabilities.push("Subagents 多代理系統".to_string());
            }
        }

        capabilities
    }

    /// 生成設定建議
    fn generate_setup_recommendations(&self) -> Vec<String> {
        vec![
            "建議使用 OAuth 認證：執行 `claude` 命令並遵循瀏覽器登入流程".to_string(),
            "或設定 API Key：export ANTHROPIC_API_KEY=\"sk-ant-apiXX-XXXXXXX\"".to_string(),
            "企業用戶可考慮 AWS Bedrock 或 Google Vertex AI 整合".to_string(),
            "執行 `claude doctor` 檢查認證狀態".to_string(),
        ]
    }

    /// 生成修復建議
    fn generate_fix_recommendations(&self, auth_method: &AuthenticationMethod) -> Vec<String> {
        match auth_method {
            AuthenticationMethod::ApiKey { .. } => vec![
                "API Key 可能已過期，請更新 ANTHROPIC_API_KEY 環境變數".to_string(),
                "檢查 API Key 格式是否正確（以 sk-ant-api 開頭）".to_string(),
            ],
            AuthenticationMethod::ConsoleOAuth { .. } => vec![
                "OAuth token 可能已過期，請重新執行 `claude` 登入".to_string(),
                "或執行 `claude auth login` 重新認證".to_string(),
            ],
            AuthenticationMethod::Bedrock { .. } => vec![
                "檢查 AWS 認證是否有效：`aws sts get-caller-identity`".to_string(),
                "確認 AWS_REGION 環境變數設定正確".to_string(),
            ],
            AuthenticationMethod::VertexAI { .. } => vec![
                "檢查 GCP 認證：`gcloud auth print-access-token`".to_string(),
                "確認 ANTHROPIC_VERTEX_PROJECT_ID 設定正確".to_string(),
            ],
            _ => vec!["請設定有效的認證方式".to_string()],
        }
    }
}

#[derive(Debug)]
struct DoctorResult {
    success: bool,
    stdout: String,
    stderr: String,
}

impl Default for ClaudeAuthDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Commands
#[tauri::command]
pub async fn detect_claude_authentication() -> Result<AuthenticationStatus, String> {
    let detector = ClaudeAuthDetector::new();
    detector
        .detect_authentication()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_claude_authentication() -> Result<bool, String> {
    let detector = ClaudeAuthDetector::new();
    let status = detector
        .detect_authentication()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(status.is_valid)
}

#[tauri::command]
pub async fn get_authentication_recommendations() -> Result<Vec<String>, String> {
    let detector = ClaudeAuthDetector::new();
    let status = detector
        .detect_authentication()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(status.recommendations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        let detector = ClaudeAuthDetector::new();
        
        assert!(detector.is_valid_api_key("sk-ant-apiXX-XXXXXXXXXXXXXXXXXXXXXXX"));
        assert!(!detector.is_valid_api_key("invalid-key"));
        assert!(!detector.is_valid_api_key(""));
    }

    #[test]
    fn test_api_key_masking() {
        let detector = ClaudeAuthDetector::new();
        let key = "sk-ant-apiXX-ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let masked = detector.mask_api_key(key);
        
        assert!(masked.contains("sk-ant-api"));
        assert!(masked.contains("..."));
        assert!(!masked.contains("ABCDEFGHIJKLMNOPQRST"));
    }

    #[tokio::test]
    async fn test_auth_detector_creation() {
        let detector = ClaudeAuthDetector::new();
        assert!(!detector.config_paths.is_empty());
        assert!(!detector.token_paths.is_empty());
    }
}