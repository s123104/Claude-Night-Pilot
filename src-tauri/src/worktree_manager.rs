// Worktree Manager - 整合 vibe-kanban 的 Git Worktree 管理功能
// 從 research-projects/vibe-kanban/backend/src/utils/worktree_manager.rs 適配

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use tracing::{debug, info};

// Global synchronization for worktree creation to prevent race conditions - Updated to use std::sync::LazyLock
static WORKTREE_CREATION_LOCKS: std::sync::LazyLock<Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct WorktreeManager;

impl WorktreeManager {
    /// Ensure worktree exists, recreating if necessary with proper synchronization
    /// This is the main entry point for ensuring a worktree exists and prevents race conditions
    pub async fn ensure_worktree_exists(
        repo_path: String,
        branch_name: String,
        worktree_path: PathBuf,
    ) -> Result<()> {
        let path_str = worktree_path.to_string_lossy().to_string();

        // Get or create a lock for this specific worktree path
        let lock = {
            let mut locks = WORKTREE_CREATION_LOCKS.lock().unwrap();
            locks
                .entry(path_str.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };

        // Acquire the lock for this specific worktree path
        let _guard = lock.lock().await;

        // Check if worktree already exists and is properly set up
        if Self::is_worktree_properly_set_up(&repo_path, &worktree_path).await? {
            debug!("Worktree already properly set up at path: {}", path_str);
            return Ok(());
        }

        // If worktree doesn't exist or isn't properly set up, recreate it
        info!("Worktree needs recreation at path: {}", path_str);
        Self::recreate_worktree_internal(repo_path, branch_name, worktree_path).await
    }

    /// Internal worktree recreation function (always recreates)
    async fn recreate_worktree_internal(
        repo_path: String,
        branch_name: String,
        worktree_path: PathBuf,
    ) -> Result<()> {
        let path_str = worktree_path.to_string_lossy().to_string();
        let branch_name_owned = branch_name.to_string();
        let worktree_path_owned = worktree_path.to_path_buf();

        // Use the provided repo path
        let git_repo_path = repo_path;

        // Get the worktree name for metadata operations
        let worktree_name = worktree_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid worktree path"))?
            .to_string();

        info!(
            "Creating worktree {} at path {}",
            branch_name_owned, path_str
        );

        // Step 1: Comprehensive cleanup of existing worktree and metadata (non-blocking)
        Self::comprehensive_worktree_cleanup_async(
            &git_repo_path,
            &worktree_path_owned,
            &worktree_name,
        )
        .await?;

        // Step 2: Ensure parent directory exists (non-blocking)
        if let Some(parent) = worktree_path_owned.parent() {
            let parent_path = parent.to_path_buf();
            tokio::task::spawn_blocking(move || std::fs::create_dir_all(&parent_path))
                .await
                .context("Task join error")?
                .context("Failed to create parent directory")?;
        }

        // Step 3: Create the worktree with retry logic for metadata conflicts (non-blocking)
        Self::create_worktree_with_retry(
            &git_repo_path,
            &branch_name_owned,
            &worktree_path_owned,
            &worktree_name,
            &path_str,
        )
        .await
    }

    /// Check if a worktree is properly set up (filesystem + git metadata)
    async fn is_worktree_properly_set_up(
        repo_path: &str,
        worktree_path: &Path,
    ) -> Result<bool> {
        let repo_path = repo_path.to_string();
        let worktree_path = worktree_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            // Check 1: Filesystem path must exist
            if !worktree_path.exists() {
                return Ok(false);
            }

            // Check 2: Use git command to check if worktree is valid
            let output = std::process::Command::new("git")
                .args(&["worktree", "list"])
                .current_dir(&repo_path)
                .output()
                .context("Failed to list worktrees")?;

            if !output.status.success() {
                return Ok(false);
            }

            let worktree_list = String::from_utf8_lossy(&output.stdout);
            let worktree_path_str = worktree_path.to_string_lossy();
            
            // Check if our worktree path is in the list
            let found = worktree_list
                .lines()
                .any(|line| line.contains(worktree_path_str.as_ref()));

            Ok(found)
        })
        .await
        .context("Task join error")?
    }

    /// Comprehensive cleanup of worktree path and metadata to prevent "path exists" errors
    async fn comprehensive_worktree_cleanup_async(
        git_repo_path: &str,
        worktree_path: &Path,
        worktree_name: &str,
    ) -> Result<()> {
        let git_repo_path_owned = git_repo_path.to_string();
        let worktree_path_owned = worktree_path.to_path_buf();
        let worktree_name_owned = worktree_name.to_string();

        tokio::task::spawn_blocking(move || {
            debug!("Performing cleanup for worktree: {}", worktree_name_owned);

            // Step 1: Try to remove worktree using git command
            let _ = std::process::Command::new("git")
                .args(&["worktree", "remove", "--force", worktree_path_owned.to_str().unwrap_or("")])
                .current_dir(&git_repo_path_owned)
                .output(); // Ignore errors as worktree might not exist

            // Step 2: Clean up physical worktree directory if it exists
            if worktree_path_owned.exists() {
                debug!(
                    "Removing existing worktree directory: {}",
                    worktree_path_owned.display()
                );
                std::fs::remove_dir_all(&worktree_path_owned)
                    .context("Failed to remove existing directory")?;
            }

            // Step 3: Force cleanup metadata directory
            Self::force_cleanup_worktree_metadata(&git_repo_path_owned, &worktree_name_owned)
                .unwrap_or_else(|e| {
                    debug!("Metadata cleanup failed (non-fatal): {}", e);
                });

            debug!(
                "Comprehensive cleanup completed for worktree: {}",
                worktree_name_owned
            );
            
            Ok::<(), anyhow::Error>(())
        })
        .await
        .context("Task join error")?
    }

    /// Create worktree with retry logic in non-blocking manner
    async fn create_worktree_with_retry(
        git_repo_path: &str,
        branch_name: &str,
        worktree_path: &Path,
        worktree_name: &str,
        path_str: &str,
    ) -> Result<()> {
        let git_repo_path = git_repo_path.to_string();
        let branch_name = branch_name.to_string();
        let worktree_path = worktree_path.to_path_buf();
        let worktree_name = worktree_name.to_string();
        let path_str = path_str.to_string();

        tokio::task::spawn_blocking(move || {
            // Check if branch exists, create if not
            let branch_exists = std::process::Command::new("git")
                .args(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
                .current_dir(&git_repo_path)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !branch_exists {
                // Create new branch
                let status = std::process::Command::new("git")
                    .args(&["checkout", "-b", &branch_name])
                    .current_dir(&git_repo_path)
                    .status()
                    .context("Failed to create new branch")?;

                if !status.success() {
                    return Err(anyhow::anyhow!("Failed to create branch {}", branch_name));
                }

                // Switch back to main branch
                let _ = std::process::Command::new("git")
                    .args(&["checkout", "main"])
                    .current_dir(&git_repo_path)
                    .status();
            }

            // Create worktree
            let output = std::process::Command::new("git")
                .args(&["worktree", "add", worktree_path.to_str().unwrap(), &branch_name])
                .current_dir(&git_repo_path)
                .output()
                .context("Failed to execute git worktree add")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // If metadata directory exists, try cleanup and retry
                if stderr.contains("already exists") {
                    debug!(
                        "Worktree metadata directory exists, attempting force cleanup: {}",
                        stderr
                    );

                    // Force cleanup metadata and try one more time
                    Self::force_cleanup_worktree_metadata(&git_repo_path, &worktree_name)
                        .context("Failed to cleanup worktree metadata")?;

                    // Try again after cleanup
                    let retry_output = std::process::Command::new("git")
                        .args(&["worktree", "add", worktree_path.to_str().unwrap(), &branch_name])
                        .current_dir(&git_repo_path)
                        .output()
                        .context("Failed to retry git worktree add")?;

                    if !retry_output.status.success() {
                        let retry_stderr = String::from_utf8_lossy(&retry_output.stderr);
                        return Err(anyhow::anyhow!(
                            "Worktree creation failed even after metadata cleanup: {}",
                            retry_stderr
                        ));
                    }
                } else {
                    return Err(anyhow::anyhow!("Failed to create worktree: {}", stderr));
                }
            }

            // Verify the worktree was actually created
            if !worktree_path.exists() {
                return Err(anyhow::anyhow!(
                    "Worktree creation reported success but path {} does not exist",
                    path_str
                ));
            }

            info!(
                "Successfully created worktree {} at {}",
                branch_name, path_str
            );

            Ok(())
        })
        .await
        .context("Task join error")?
    }

    /// Force cleanup worktree metadata directory
    fn force_cleanup_worktree_metadata(
        git_repo_path: &str,
        worktree_name: &str,
    ) -> Result<(), std::io::Error> {
        let git_worktree_metadata_path = Path::new(git_repo_path)
            .join(".git")
            .join("worktrees")
            .join(worktree_name);

        if git_worktree_metadata_path.exists() {
            debug!(
                "Force removing git worktree metadata: {}",
                git_worktree_metadata_path.display()
            );
            std::fs::remove_dir_all(&git_worktree_metadata_path)?;
        }

        Ok(())
    }

    /// Clean up a worktree path and its git metadata (non-blocking)
    pub async fn cleanup_worktree(
        worktree_path: &Path,
        git_repo_path: Option<&str>,
    ) -> Result<()> {
        let path_str = worktree_path.to_string_lossy().to_string();

        // Get the same lock to ensure we don't interfere with creation
        let lock = {
            let mut locks = WORKTREE_CREATION_LOCKS.lock().unwrap();
            locks
                .entry(path_str.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };

        let _guard = lock.lock().await;

        if let Some(worktree_name) = worktree_path.file_name().and_then(|n| n.to_str()) {
            // Try to determine the git repo path if not provided
            let resolved_repo_path = if let Some(repo_path) = git_repo_path {
                Some(repo_path.to_string())
            } else {
                Self::infer_git_repo_path(worktree_path).await
            };

            if let Some(repo_path) = resolved_repo_path {
                Self::comprehensive_worktree_cleanup_async(
                    &repo_path,
                    worktree_path,
                    worktree_name,
                )
                .await?;
            } else {
                // Can't determine repo path, just clean up the worktree directory
                debug!(
                    "Cannot determine git repo path for worktree {}, performing simple cleanup",
                    path_str
                );
                Self::simple_worktree_cleanup(worktree_path).await?;
            }
        } else {
            return Err(anyhow::anyhow!(
                "Invalid worktree path, cannot determine name",
            ));
        }

        Ok(())
    }

    /// Try to infer the git repository path from a worktree
    async fn infer_git_repo_path(worktree_path: &Path) -> Option<String> {
        let worktree_path_owned = worktree_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let output = std::process::Command::new("git")
                .args(&["rev-parse", "--git-common-dir"])
                .current_dir(&worktree_path_owned)
                .output()
                .ok()?;

            if output.status.success() {
                let git_common_dir = String::from_utf8(output.stdout).ok()?.trim().to_string();

                // git-common-dir gives us the path to the .git directory
                // We need the working directory (parent of .git)
                let git_dir_path = std::path::Path::new(&git_common_dir);
                if git_dir_path.file_name() == Some(std::ffi::OsStr::new(".git")) {
                    git_dir_path.parent()?.to_str().map(|s| s.to_string())
                } else {
                    // In case of bare repo or unusual setup, use the git-common-dir as is
                    Some(git_common_dir)
                }
            } else {
                None
            }
        })
        .await
        .ok()
        .flatten()
    }

    /// Simple worktree cleanup when we can't determine the main repo
    async fn simple_worktree_cleanup(worktree_path: &Path) -> Result<()> {
        let worktree_path_owned = worktree_path.to_path_buf();

        tokio::task::spawn_blocking(move || {
            if worktree_path_owned.exists() {
                std::fs::remove_dir_all(&worktree_path_owned)
                    .context("Failed to remove worktree directory")?;
                info!(
                    "Removed worktree directory: {}",
                    worktree_path_owned.display()
                );
            }
            Ok(())
        })
        .await
        .context("Task join error")?
    }
}