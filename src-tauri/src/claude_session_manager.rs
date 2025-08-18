// Claude Session Manager - 整合 Git Worktree 和 Claude Code Session 管理
use crate::claude_auth_detector::{AuthenticationMethod, AuthenticationStatus, ClaudeAuthDetector};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSession {
    pub id: Uuid,
    pub session_id: String, // Claude CLI session ID
    pub worktree_path: Option<String>,
    pub branch_name: Option<String>,
    pub project_path: String,
    pub status: SessionStatus,
    pub created_at: SystemTime,
    pub last_active: SystemTime,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub title: String,
    pub description: Option<String>,
    pub total_messages: u32,
    pub total_tokens: u32,
    pub total_cost: f64,
    pub last_command: Option<String>,
    pub output_format: String,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionExecutionOptions {
    pub output_format: String, // json, stream-json, text
    pub allowed_tools: Vec<String>,
    pub skip_permissions: bool,
    pub max_turns: Option<u32>,
    pub model: Option<String>,
    pub resume_session_id: Option<String>,
}

impl Default for SessionExecutionOptions {
    fn default() -> Self {
        Self {
            output_format: "stream-json".to_string(),
            allowed_tools: vec![
                "Bash(git:*)".to_string(),
                "Write".to_string(),
                "Read".to_string(),
                "Edit".to_string(),
                "MultiEdit".to_string(),
            ],
            skip_permissions: false,
            max_turns: None,
            model: Some("claude-sonnet-4-20250514".to_string()),
            resume_session_id: None,
        }
    }
}

pub struct ClaudeSessionManager {
    active_sessions: HashMap<Uuid, ClaudeSession>,
    database_path: String,
    project_root: PathBuf,
    auth_detector: ClaudeAuthDetector,
    cached_auth_status: Option<AuthenticationStatus>,
}

impl ClaudeSessionManager {
    pub fn new(database_path: String, project_root: PathBuf) -> Self {
        Self {
            active_sessions: HashMap::new(),
            database_path,
            project_root,
            auth_detector: ClaudeAuthDetector::new(),
            cached_auth_status: None,
        }
    }

    /// 智能檢測並確保 Claude Code 認證可用
    pub async fn ensure_authentication(&mut self) -> Result<AuthenticationStatus> {
        tracing::info!("執行 Claude Code 認證自動檢測...");

        // 如果有快取且在 5 分鐘內，直接使用
        if let Some(ref cached_status) = self.cached_auth_status {
            let cache_age = chrono::Utc::now() - cached_status.last_verified;
            if cache_age.num_minutes() < 5 && cached_status.is_valid {
                tracing::debug!("使用快取的認證狀態");
                return Ok(cached_status.clone());
            }
        }

        // 執行完整檢測
        let auth_status = self.auth_detector.detect_authentication().await?;

        if auth_status.is_valid {
            tracing::info!("✅ 檢測到有效的 Claude Code 認證: {:?}", auth_status.method);
            self.log_authentication_info(&auth_status).await;
        } else {
            tracing::warn!("❌ 未檢測到有效的 Claude Code 認證");
            self.log_authentication_recommendations(&auth_status).await;
        }

        // 更新快取
        self.cached_auth_status = Some(auth_status.clone());

        Ok(auth_status)
    }

    /// 記錄認證資訊
    async fn log_authentication_info(&self, auth_status: &AuthenticationStatus) {
        match &auth_status.method {
            AuthenticationMethod::ApiKey { source, masked_key } => {
                tracing::info!("🔑 使用 API Key 認證 (來源: {:?}): {}", source, masked_key);
            }
            AuthenticationMethod::ConsoleOAuth { token_path, .. } => {
                tracing::info!("🌐 使用 OAuth 認證 (Token 路徑: {})", token_path.display());
            }
            AuthenticationMethod::Bedrock { region, profile } => {
                tracing::info!(
                    "☁️ 使用 AWS Bedrock 認證 (區域: {}, 配置檔: {:?})",
                    region,
                    profile
                );
            }
            AuthenticationMethod::VertexAI { project_id, region } => {
                tracing::info!(
                    "🏢 使用 Google Vertex AI 認證 (專案: {}, 區域: {})",
                    project_id,
                    region
                );
            }
            AuthenticationMethod::ClaudeApp { app_session } => {
                tracing::info!("📱 使用 Claude App 認證 (會話: {})", app_session);
            }
            AuthenticationMethod::None => {
                tracing::warn!("❌ 未檢測到認證");
            }
        }

        if let Some(ref user_info) = auth_status.user_info {
            if let Some(ref email) = user_info.email {
                tracing::info!("👤 使用者: {}", email);
            }
            if let Some(ref subscription) = user_info.subscription_type {
                tracing::info!("💼 訂閱類型: {}", subscription);
            }
        }

        if !auth_status.capabilities.is_empty() {
            tracing::info!("🚀 可用功能: {}", auth_status.capabilities.join(", "));
        }
    }

    /// 記錄認證建議
    async fn log_authentication_recommendations(&self, auth_status: &AuthenticationStatus) {
        if !auth_status.recommendations.is_empty() {
            tracing::warn!("📋 認證設定建議:");
            for recommendation in &auth_status.recommendations {
                tracing::warn!("   • {}", recommendation);
            }
        }
    }

    /// 取得當前認證狀態
    pub async fn get_authentication_status(&self) -> Result<Option<AuthenticationStatus>> {
        Ok(self.cached_auth_status.clone())
    }

    /// 驗證認證是否仍然有效
    pub async fn verify_authentication(&mut self) -> Result<bool> {
        let auth_status = self.ensure_authentication().await?;
        Ok(auth_status.is_valid)
    }

    /// 創建新的 Claude 會話，可選擇性創建 worktree
    /// 自動檢測並確保 Claude Code 認證可用
    pub async fn create_session(
        &mut self,
        title: String,
        description: Option<String>,
        create_worktree: bool,
        branch_name: Option<String>,
        options: SessionExecutionOptions,
    ) -> Result<ClaudeSession> {
        // 🔍 自動檢測認證狀態
        let auth_status = self.ensure_authentication().await?;
        if !auth_status.is_valid {
            return Err(anyhow::anyhow!(
                "❌ Claude Code 認證失效或未設定。請檢查認證狀態或按照建議進行設定。"
            ));
        }
        let session_uuid = Uuid::new_v4();
        let now = SystemTime::now();

        let (worktree_path, actual_branch) = if create_worktree {
            let branch = branch_name.unwrap_or_else(|| format!("session-{}", session_uuid));
            let worktree_path = self.create_worktree(&branch).await?;
            (
                Some(worktree_path.to_string_lossy().to_string()),
                Some(branch),
            )
        } else {
            (None, branch_name)
        };

        // 執行初始 Claude 命令來獲取 session ID
        let initial_prompt = format!(
            "Claude Night Pilot Session initialized: {}\n{}",
            title,
            description.as_deref().unwrap_or("No description provided")
        );

        let claude_session_id = self
            .execute_claude_command(
                &initial_prompt,
                &options,
                worktree_path.as_ref().map(Path::new),
            )
            .await?;

        let session = ClaudeSession {
            id: session_uuid,
            session_id: claude_session_id,
            worktree_path,
            branch_name: actual_branch,
            project_path: self.project_root.to_string_lossy().to_string(),
            status: SessionStatus::Active,
            created_at: now,
            last_active: now,
            metadata: SessionMetadata {
                title,
                description,
                total_messages: 1,
                total_tokens: 0,
                total_cost: 0.0,
                last_command: Some(initial_prompt),
                output_format: options.output_format.clone(),
                allowed_tools: options.allowed_tools.clone(),
            },
        };

        // 保存到資料庫
        self.save_session_to_db(&session).await?;

        // 添加到活躍會話
        self.active_sessions.insert(session_uuid, session.clone());

        tracing::info!(
            "Created new Claude session: {} ({})",
            session_uuid,
            session.session_id
        );
        Ok(session)
    }

    /// 恢復已存在的會話
    /// 自動檢測並確保 Claude Code 認證可用
    pub async fn resume_session(
        &mut self,
        session_uuid: Uuid,
        options: Option<SessionExecutionOptions>,
    ) -> Result<ClaudeSession> {
        // 🔍 自動檢測認證狀態
        let auth_status = self.ensure_authentication().await?;
        if !auth_status.is_valid {
            tracing::warn!("認證失效，但嘗試恢復現有會話");
        }
        let mut session = self
            .get_session_from_db(session_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_uuid))?;

        // 更新狀態
        session.status = SessionStatus::Active;
        session.last_active = SystemTime::now();

        // 如果有 worktree，確保它存在
        if let Some(ref worktree_path) = session.worktree_path {
            if !Path::new(worktree_path).exists() {
                if let Some(ref branch_name) = session.branch_name {
                    // 重新創建 worktree
                    let new_worktree_path = self.create_worktree(branch_name).await?;
                    session.worktree_path = Some(new_worktree_path.to_string_lossy().to_string());
                }
            }
        }

        // 準備恢復選項
        let _resume_options = options.unwrap_or_else(|| {
            let mut opts = SessionExecutionOptions::default();
            opts.resume_session_id = Some(session.session_id.clone());
            opts.output_format = session.metadata.output_format.clone();
            opts.allowed_tools = session.metadata.allowed_tools.clone();
            opts
        });

        // 保存更新
        self.save_session_to_db(&session).await?;
        self.active_sessions.insert(session_uuid, session.clone());

        tracing::info!(
            "Resumed Claude session: {} ({})",
            session_uuid,
            session.session_id
        );
        Ok(session)
    }

    /// 執行 Claude 命令在指定會話中
    /// 執行前自動驗證認證狀態
    pub async fn execute_in_session(
        &mut self,
        session_uuid: Uuid,
        prompt: String,
        options: Option<SessionExecutionOptions>,
    ) -> Result<String> {
        // 🔍 快速驗證認證狀態（使用快取）
        if !self.verify_authentication().await? {
            return Err(anyhow::anyhow!(
                "❌ Claude Code 認證失效。請重新設定認證或檢查網路連線。"
            ));
        }
        let mut session = self
            .active_sessions
            .get(&session_uuid)
            .ok_or_else(|| anyhow::anyhow!("Session {} not active", session_uuid))?
            .clone();

        let execution_options = options.unwrap_or_else(|| {
            let mut opts = SessionExecutionOptions::default();
            opts.resume_session_id = Some(session.session_id.clone());
            opts
        });

        let working_dir = session.worktree_path.as_ref().map(Path::new);
        let result = self
            .execute_claude_command(&prompt, &execution_options, working_dir)
            .await?;

        // 更新會話統計
        session.last_active = SystemTime::now();
        session.metadata.total_messages += 1;
        session.metadata.last_command = Some(prompt);

        // 嘗試從結果中提取使用統計
        if let Ok(parsed_result) = serde_json::from_str::<serde_json::Value>(&result) {
            if let Some(usage) = parsed_result.get("usage") {
                if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                    session.metadata.total_tokens += input_tokens as u32;
                }
                if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                    session.metadata.total_tokens += output_tokens as u32;
                }
            }
        }

        // 保存更新的會話
        self.save_session_to_db(&session).await?;
        self.active_sessions.insert(session_uuid, session);

        Ok(result)
    }

    /// 創建 Git worktree
    async fn create_worktree(&self, branch_name: &str) -> Result<PathBuf> {
        let worktree_dir = self.project_root.join("worktrees");
        fs::create_dir_all(&worktree_dir)
            .await
            .context("Failed to create worktrees directory")?;

        let worktree_path = worktree_dir.join(format!(
            "{}-{}",
            branch_name,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        // 檢查分支是否存在，如不存在則創建
        let branch_exists = Command::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", branch_name),
            ])
            .current_dir(&self.project_root)
            .status()
            .context("Failed to check branch existence")?
            .success();

        if !branch_exists {
            // 創建新分支
            let status = Command::new("git")
                .args(["checkout", "-b", branch_name])
                .current_dir(&self.project_root)
                .status()
                .context("Failed to create new branch")?;

            if !status.success() {
                return Err(anyhow::anyhow!("Failed to create branch {}", branch_name));
            }

            // 切回主分支
            Command::new("git")
                .args(["checkout", "main"])
                .current_dir(&self.project_root)
                .status()
                .context("Failed to return to main branch")?;
        }

        // 創建 worktree
        let status = Command::new("git")
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap(),
                branch_name,
            ])
            .current_dir(&self.project_root)
            .status()
            .context("Failed to create worktree")?;

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create worktree for branch {}",
                branch_name
            ));
        }

        tracing::info!(
            "Created worktree: {} for branch: {}",
            worktree_path.display(),
            branch_name
        );
        Ok(worktree_path)
    }

    /// 執行 Claude CLI 命令
    async fn execute_claude_command(
        &self,
        prompt: &str,
        options: &SessionExecutionOptions,
        working_dir: Option<&Path>,
    ) -> Result<String> {
        let mut cmd = Command::new("claude");

        // 基本參數
        cmd.arg("-p").arg(prompt);

        // 輸出格式
        cmd.args(["--output-format", &options.output_format]);

        // 工具權限
        if !options.allowed_tools.is_empty() {
            cmd.arg("--allowedTools");
            for tool in &options.allowed_tools {
                cmd.arg(tool);
            }
        }

        // 跳過權限檢查
        if options.skip_permissions {
            cmd.arg("--dangerously-skip-permissions");
        }

        // 最大輪數
        if let Some(max_turns) = options.max_turns {
            cmd.args(["--max-turns", &max_turns.to_string()]);
        }

        // 模型選擇
        if let Some(ref model) = options.model {
            cmd.args(["--model", model]);
        }

        // 會話恢復
        if let Some(ref session_id) = options.resume_session_id {
            cmd.args(["--resume", session_id]);
        }

        // 工作目錄
        if let Some(work_dir) = working_dir {
            cmd.current_dir(work_dir);
        } else {
            cmd.current_dir(&self.project_root);
        }

        // 執行命令
        let output = cmd.output().context("Failed to execute Claude CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Claude CLI execution failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // 如果是 JSON 格式，嘗試提取 session_id
        if options.output_format.contains("json") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(session_id) = json.get("session_id").and_then(|s| s.as_str()) {
                    return Ok(session_id.to_string());
                }
            }
        }

        Ok(stdout.trim().to_string())
    }

    /// 列出所有會話
    pub async fn list_sessions(&self) -> Result<Vec<ClaudeSession>> {
        // 從資料庫載入所有會話
        self.load_sessions_from_db().await
    }

    /// 暫停會話
    pub async fn pause_session(&mut self, session_uuid: Uuid) -> Result<()> {
        if let Some(mut session) = self.active_sessions.remove(&session_uuid) {
            session.status = SessionStatus::Paused;
            session.last_active = SystemTime::now();
            self.save_session_to_db(&session).await?;
            tracing::info!("Paused session: {}", session_uuid);
        }
        Ok(())
    }

    /// 完成會話
    pub async fn complete_session(&mut self, session_uuid: Uuid) -> Result<()> {
        if let Some(mut session) = self.active_sessions.remove(&session_uuid) {
            session.status = SessionStatus::Completed;
            session.last_active = SystemTime::now();
            self.save_session_to_db(&session).await?;

            // 清理 worktree（可選）
            if let Some(ref worktree_path) = session.worktree_path {
                self.cleanup_worktree(Path::new(worktree_path)).await.ok();
            }

            tracing::info!("Completed session: {}", session_uuid);
        }
        Ok(())
    }

    /// 清理 worktree
    async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<()> {
        if worktree_path.exists() {
            let status = Command::new("git")
                .args(["worktree", "remove", worktree_path.to_str().unwrap()])
                .current_dir(&self.project_root)
                .status()
                .context("Failed to remove worktree")?;

            if status.success() {
                tracing::info!("Cleaned up worktree: {}", worktree_path.display());
            }
        }
        Ok(())
    }

    /// 保存會話到資料庫（簡化版本，實際應該使用真實資料庫）
    async fn save_session_to_db(&self, session: &ClaudeSession) -> Result<()> {
        let sessions_dir = Path::new(&self.database_path)
            .parent()
            .unwrap()
            .join("sessions");
        fs::create_dir_all(&sessions_dir).await?;

        let session_file = sessions_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)?;
        fs::write(session_file, json).await?;

        Ok(())
    }

    /// 從資料庫載入會話
    async fn get_session_from_db(&self, session_uuid: Uuid) -> Result<Option<ClaudeSession>> {
        let sessions_dir = Path::new(&self.database_path)
            .parent()
            .unwrap()
            .join("sessions");
        let session_file = sessions_dir.join(format!("{}.json", session_uuid));

        if session_file.exists() {
            let content = fs::read_to_string(session_file).await?;
            let session: ClaudeSession = serde_json::from_str(&content)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// 從資料庫載入所有會話
    async fn load_sessions_from_db(&self) -> Result<Vec<ClaudeSession>> {
        let sessions_dir = Path::new(&self.database_path)
            .parent()
            .unwrap()
            .join("sessions");

        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let mut entries = fs::read_dir(sessions_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Ok(content) = fs::read_to_string(entry.path()).await {
                        if let Ok(session) = serde_json::from_str::<ClaudeSession>(&content) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// 獲取會話統計
    pub async fn get_session_stats(&self) -> Result<SessionStats> {
        let sessions = self.list_sessions().await?;

        let active_count = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Active))
            .count();
        let paused_count = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Paused))
            .count();
        let completed_count = sessions
            .iter()
            .filter(|s| matches!(s.status, SessionStatus::Completed))
            .count();
        let total_tokens: u32 = sessions.iter().map(|s| s.metadata.total_tokens).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.metadata.total_cost).sum();

        Ok(SessionStats {
            total_sessions: sessions.len(),
            active_sessions: active_count,
            paused_sessions: paused_count,
            completed_sessions: completed_count,
            total_tokens,
            total_cost,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub paused_sessions: usize,
    pub completed_sessions: usize,
    pub total_tokens: u32,
    pub total_cost: f64,
}

// Tauri Commands
#[tauri::command]
pub async fn create_claude_session(
    title: String,
    description: Option<String>,
    create_worktree: bool,
    branch_name: Option<String>,
) -> Result<ClaudeSession, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);
    let options = SessionExecutionOptions::default();

    manager
        .create_session(title, description, create_worktree, branch_name, options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_claude_session(session_id: String) -> Result<ClaudeSession, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);
    let session_uuid =
        Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    manager
        .resume_session(session_uuid, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_in_claude_session(
    session_id: String,
    prompt: String,
) -> Result<String, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);
    let session_uuid =
        Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    manager
        .execute_in_session(session_uuid, prompt, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_claude_sessions() -> Result<Vec<ClaudeSession>, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let manager = ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    manager.list_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_stats() -> Result<SessionStats, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let manager = ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    manager.get_session_stats().await.map_err(|e| e.to_string())
}

// === 新增：Claude Code 認證自動檢測相關 Tauri Commands ===

#[tauri::command]
pub async fn check_claude_authentication() -> Result<AuthenticationStatus, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    manager
        .ensure_authentication()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_claude_auth_status() -> Result<bool, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    manager
        .verify_authentication()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_auth_status() -> Result<Option<AuthenticationStatus>, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let manager = ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    manager
        .get_authentication_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn force_refresh_authentication() -> Result<AuthenticationStatus, String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let mut manager =
        ClaudeSessionManager::new("./claude-night-pilot.db".to_string(), project_root);

    // 清除快取，強制重新檢測
    manager.cached_auth_status = None;

    manager
        .ensure_authentication()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_metadata_creation() {
        let metadata = SessionMetadata {
            title: "Test Session".to_string(),
            description: Some("A test session".to_string()),
            total_messages: 0,
            total_tokens: 0,
            total_cost: 0.0,
            last_command: None,
            output_format: "json".to_string(),
            allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        };

        assert_eq!(metadata.title, "Test Session");
        assert_eq!(metadata.total_messages, 0);
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ClaudeSessionManager::new(
            temp_dir
                .path()
                .join("test.db")
                .to_string_lossy()
                .to_string(),
            temp_dir.path().to_path_buf(),
        );

        assert_eq!(manager.active_sessions.len(), 0);
    }
}
