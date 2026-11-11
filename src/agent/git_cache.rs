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

    /// Locates an existing local git directory, or synchronizes remote/bare cache mirror
    pub async fn locate_git_dir(&self, repo_url: &str) -> Result<PathBuf, AppError> {
        let local_path = Path::new(repo_url);
        if local_path.exists() {
            if local_path.join("HEAD").exists() {
                // Bare repository directly on disk
                return Ok(local_path.to_path_buf());
            }
            if local_path.join(".git").exists() {
                // Standard git working directory
                return Ok(local_path.join(".git"));
            }
        }

        // Remote repository URL or cached mirror
        self.sync_cache(repo_url).await
    }

    /// Reads pipeline configuration file (.ci.yaml, etc.) from git repository or working tree
    pub async fn read_pipeline_file(
        &self,
        repo_url: &str,
        git_ref: &str,
        pipeline_file: &str,
    ) -> Result<String, AppError> {
        let git_dir = self.locate_git_dir(repo_url).await?;

        // 1. Check direct working tree file if source is a working tree
        let working_dir = if git_dir.ends_with(".git") {
            git_dir.parent().unwrap_or(&git_dir)
        } else {
            &git_dir
        };

        let mut candidate_files = vec![pipeline_file];
        if pipeline_file != ".ci.yaml" {
            candidate_files.push(".ci.yaml");
        }
        if pipeline_file != ".ci.yml" {
            candidate_files.push(".ci.yml");
        }
        if pipeline_file != ".exaforge.yaml" {
            candidate_files.push(".exaforge.yaml");
        }

        for fname in &candidate_files {
            let direct_file = working_dir.join(fname);
            if direct_file.is_file() {
                if let Ok(content) = tokio::fs::read_to_string(&direct_file).await {
                    if !content.trim().is_empty() {
                        info!(path = ?direct_file, "Loaded pipeline config directly from local working tree");
                        return Ok(content);
                    }
                }
            }
        }

        // 2. Query git repository via `git --git-dir <git_dir> show <ref>:<file>`
        let clean_ref = git_ref.trim_start_matches("refs/heads/").trim_start_matches("refs/tags/");
        let candidate_refs = if clean_ref == "HEAD" || clean_ref.is_empty() {
            vec!["HEAD", "main", "master"]
        } else {
            vec![git_ref, clean_ref, "HEAD", "main"]
        };

        for r in candidate_refs {
            for fname in &candidate_files {
                let show_target = format!("{}:{}", r, fname);
                let out = Command::new("git")
                    .arg("--git-dir")
                    .arg(&git_dir)
                    .arg("show")
                    .arg(&show_target)
                    .output()
                    .await;

                if let Ok(output) = out {
                    if output.status.success() {
                        let content = String::from_utf8_lossy(&output.stdout).to_string();
                        if !content.trim().is_empty() {
                            info!(target = %show_target, "Loaded pipeline config via git show");
                            return Ok(content);
                        }
                    }
                }
            }
        }

        Err(AppError::not_found(
            format!("Pipeline config file '{}' not found in repository '{}' at ref '{}'", pipeline_file, repo_url, git_ref),
            "PIPELINE_FILE_NOT_FOUND",
        ))
    }

    /// Resolves actual 40-character commit SHA from ref
    pub async fn resolve_commit_sha(&self, repo_url: &str, git_ref: &str) -> Result<String, AppError> {
        let git_dir = self.locate_git_dir(repo_url).await?;
        let clean_ref = git_ref.trim_start_matches("refs/heads/").trim_start_matches("refs/tags/");
        let candidate_refs = if clean_ref.is_empty() {
            vec!["HEAD"]
        } else {
            vec![git_ref, clean_ref, "HEAD"]
        };

        for r in candidate_refs {
            let out = Command::new("git")
                .arg("--git-dir")
                .arg(&git_dir)
                .arg("rev-parse")
                .arg(r)
                .output()
                .await;

            if let Ok(output) = out {
                if output.status.success() {
                    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !sha.is_empty() {
                        return Ok(sha);
                    }
                }
            }
        }

        Err(AppError::internal("Failed to resolve commit SHA"))
    }

    /// Resolves commit message from ref
    pub async fn resolve_commit_message(&self, repo_url: &str, git_ref: &str) -> Result<String, AppError> {
        let git_dir = self.locate_git_dir(repo_url).await?;
        let clean_ref = git_ref.trim_start_matches("refs/heads/").trim_start_matches("refs/tags/");
        let target = if clean_ref.is_empty() { "HEAD" } else { clean_ref };

        let out = Command::new("git")
            .arg("--git-dir")
            .arg(&git_dir)
            .arg("log")
            .arg("-1")
            .arg("--format=%s")
            .arg(target)
            .output()
            .await;

        if let Ok(output) = out {
            if output.status.success() {
                let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !msg.is_empty() {
                    return Ok(msg);
                }
            }
        }

        Err(AppError::internal("Failed to resolve commit message"))
    }

    /// Checks out commit SHA / ref into job workspace directory with force clean
    pub async fn checkout_workspace(
        &self,
        repo_url: &str,
        git_ref: &str,
        commit_sha: &str,
        workspace_dir: &Path,
    ) -> Result<(), AppError> {
        let git_source = self.locate_git_dir(repo_url).await?;

        if !workspace_dir.exists() {
            tokio::fs::create_dir_all(workspace_dir).await
                .map_err(|e| AppError::internal(format!("Failed to create workspace: {}", e)))?;
        }

        let git_dir = workspace_dir.join(".git");
        if !git_dir.exists() {
            let clone_out = Command::new("git")
                .arg("clone")
                .arg(&git_source)
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
                .arg(&git_source)
                .arg("+refs/heads/*:refs/remotes/origin/*")
                .arg("+refs/tags/*:refs/tags/*")
                .output()
                .await;
        }

        // Determine checkout target
        let clean_ref = git_ref.trim_start_matches("refs/heads/").trim_start_matches("refs/tags/");
        let target = if !commit_sha.is_empty() && commit_sha != "HEAD" {
            commit_sha
        } else if !clean_ref.is_empty() && clean_ref != "HEAD" {
            clean_ref
        } else {
            "HEAD"
        };

        // Checkout with --force to ensure no dirty state blocks branch checkout
        let _ = Command::new("git")
            .current_dir(workspace_dir)
            .arg("checkout")
            .arg("--force")
            .arg(target)
            .output()
            .await;

        // Hard reset to ensure working tree matches target exactly
        let reset_out = Command::new("git")
            .current_dir(workspace_dir)
            .arg("reset")
            .arg("--hard")
            .arg(target)
            .output()
            .await
            .map_err(|e| AppError::internal(format!("git reset --hard failed: {}", e)))?;

        if !reset_out.status.success() {
            let err = String::from_utf8_lossy(&reset_out.stderr);
            warn!(err = %err, target = %target, "git reset warning, falling back to HEAD");
            let _ = Command::new("git")
                .current_dir(workspace_dir)
                .arg("reset")
                .arg("--hard")
                .arg("HEAD")
                .output()
                .await;
        }

        // Clean any untracked or ignored files from prior runs for a deterministic rebuild
        let _ = Command::new("git")
            .current_dir(workspace_dir)
            .arg("clean")
            .arg("-fdx")
            .output()
            .await;

        Ok(())
    }
}

