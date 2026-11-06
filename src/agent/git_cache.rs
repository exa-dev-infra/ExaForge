use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use tokio::process::Command;
use tracing::{info, warn};

use crate::pkg::errors::AppError;

pub struct GitCacheManager {
    base_cache_dir: PathBuf,
}

impl GitCacheManager {
    pub fn new(base_cache_dir: PathBuf) -> Self {
        Self { base_cache_dir }
    }

    pub fn default_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".exaforge").join("cache").join("repos")
    }

    /// Derives deterministic directory name from repository URL
    pub fn get_cache_path(&self, repo_url: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(repo_url.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.base_cache_dir.join(&hash[..16])
    }

    /// Performs incremental fetch into local repository cache
    pub async fn sync_cache(&self, repo_url: &str) -> Result<PathBuf, AppError> {
        let cache_path = self.get_cache_path(repo_url);

        if !cache_path.exists() {
            tokio::fs::create_dir_all(&self.base_cache_dir).await
                .map_err(|e| AppError::internal(format!("Failed to create cache dir: {}", e)))?;

            info!(repo = %repo_url, path = ?cache_path, "Initializing new bare git cache mirror");
            let output = Command::new("git")
                .arg("clone")
                .arg("--mirror")
                .arg(repo_url)
                .arg(&cache_path)
                .output()
                .await
                .map_err(|e| AppError::internal(format!("git clone --mirror failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!(stderr = %stderr, "git clone mirror warning");
            }
        } else {
            info!(path = ?cache_path, "Fetching incremental updates in git cache mirror");
            let output = Command::new("git")
                .current_dir(&cache_path)
                .arg("fetch")
                .arg("--all")
                .arg("--prune")
                .output()
                .await
                .map_err(|e| AppError::internal(format!("git fetch --all failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!(stderr = %stderr, "git fetch warning");
            }
        }

        Ok(cache_path)
    }

    /// Checks out commit SHA into job workspace directory
    pub async fn checkout_workspace(
        &self,
        repo_url: &str,
        commit_sha: &str,
        workspace_dir: &Path,
    ) -> Result<(), AppError> {
        let cache_path = self.sync_cache(repo_url).await?;

        if !workspace_dir.exists() {
            tokio::fs::create_dir_all(workspace_dir).await
                .map_err(|e| AppError::internal(format!("Failed to create workspace: {}", e)))?;
        }

        let git_dir = workspace_dir.join(".git");
        if !git_dir.exists() {
            let clone_out = Command::new("git")
                .arg("clone")
                .arg(&cache_path)
                .arg(workspace_dir)
                .output()
                .await
                .map_err(|e| AppError::internal(format!("Failed to clone from cache: {}", e)))?;

            if !clone_out.status.success() {
                let err = String::from_utf8_lossy(&clone_out.stderr);
                return Err(AppError::internal(format!("git clone from cache failed: {}", err)));
            }
        } else {
            let _ = Command::new("git")
                .current_dir(workspace_dir)
                .arg("fetch")
                .arg(&cache_path)
                .output()
                .await;
        }

        // Hard reset to target commit SHA (or HEAD if empty/HEAD)
        let target_ref = if commit_sha.is_empty() || commit_sha == "HEAD" { "HEAD" } else { commit_sha };
        let reset_out = Command::new("git")
            .current_dir(workspace_dir)
            .arg("reset")
            .arg("--hard")
            .arg(target_ref)
            .output()
            .await
            .map_err(|e| AppError::internal(format!("git reset --hard failed: {}", e)))?;

        if !reset_out.status.success() {
            let err = String::from_utf8_lossy(&reset_out.stderr);
            warn!(err = %err, target = %target_ref, "git reset warning, falling back to HEAD");
            let _ = Command::new("git")
                .current_dir(workspace_dir)
                .arg("reset")
                .arg("--hard")
                .arg("HEAD")
                .output()
                .await;
        }

        Ok(())
    }
}

