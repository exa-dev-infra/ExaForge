use std::{
    collections::HashMap,
    path::Path,
    time::Duration,
};
use tokio::sync::watch;

use crate::{
    modules::agents::dto::{ContainerBuildConfig, StepDefinition},
    pkg::errors::AppError,
};
use super::process_group::ProcessGroupExecutor;

pub struct StepRunner;

impl StepRunner {
    /// Executes a single pipeline step with $CI_ENV, $CI_OUTPUT, and $CI_STEP_SUMMARY support
    pub async fn execute_step<F>(
        step: &StepDefinition,
        workspace_dir: &Path,
        current_env: &mut HashMap<String, String>,
        job_outputs: &mut HashMap<String, String>,
        overall_summary: &mut String,
        is_previous_failed: bool,
        timeout: Duration,
        cancel_rx: watch::Receiver<bool>,
        log_callback: F,
    ) -> Result<bool, AppError>
    where
        F: Fn(String, bool) + Send + Sync + 'static + Clone,
    {
        // 1. Check `when` execution condition
        let when = step.when.as_deref().unwrap_or("on_success");
        let should_run = match when {
            "always" => true,
            "on_failure" => is_previous_failed,
            "on_success" => !is_previous_failed,
            _ => !is_previous_failed,
        };

        if !should_run {
            log_callback(format!("[system] Skipping step '{}' due to when condition ({})", step.name, when), false);
            return Ok(true);
        }

        log_callback(format!("==> Running step: {}", step.name), false);

        // 2. Setup temporary files for $CI_ENV, $CI_OUTPUT, $CI_STEP_SUMMARY
        let temp_dir = std::env::temp_dir().join(format!("exaforge_step_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await
            .map_err(|e| AppError::internal(format!("Failed to create step temp dir: {}", e)))?;

        let ci_env_file = temp_dir.join("ci_env.env");
        let ci_output_file = temp_dir.join("ci_output.env");
        let ci_summary_file = temp_dir.join("ci_summary.md");

        tokio::fs::write(&ci_env_file, "").await.unwrap_or_default();
        tokio::fs::write(&ci_output_file, "").await.unwrap_or_default();
        tokio::fs::write(&ci_summary_file, "").await.unwrap_or_default();

        // 3. Assemble environment variables
        let mut step_env_pairs: Vec<(String, String)> = current_env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if let Some(extra_env) = &step.env {
            for (k, v) in extra_env {
                step_env_pairs.push((k.clone(), v.clone()));
            }
        }

        step_env_pairs.push(("CI_ENV".to_string(), ci_env_file.to_string_lossy().to_string()));
        step_env_pairs.push(("CI_OUTPUT".to_string(), ci_output_file.to_string_lossy().to_string()));
        step_env_pairs.push(("CI_STEP_SUMMARY".to_string(), ci_summary_file.to_string_lossy().to_string()));

        // 4. Determine command: either `run` shell script or `container_build` directive
        let command_to_execute = if let Some(run_script) = &step.run {
            run_script.clone()
        } else if let Some(container_build) = &step.container_build {
            Self::generate_container_build_script(container_build)
        } else {
            log_callback("[system] Step has no run or container_build directive".to_string(), false);
            return Ok(true);
        };

        // 5. Execute in isolated OS process group
        let exit_code = ProcessGroupExecutor::execute_command(
            &command_to_execute,
            workspace_dir,
            &step_env_pairs,
            timeout,
            cancel_rx,
            log_callback.clone(),
        ).await?;

        // 6. Parse $CI_ENV and update current_env for subsequent steps
        if let Ok(content) = tokio::fs::read_to_string(&ci_env_file).await {
            for (k, v) in Self::parse_env_file(&content) {
                current_env.insert(k, v);
            }
        }

        // 7. Parse $CI_OUTPUT and update job_outputs for downstream jobs
        if let Ok(content) = tokio::fs::read_to_string(&ci_output_file).await {
            for (k, v) in Self::parse_env_file(&content) {
                job_outputs.insert(k, v);
            }
        }

        // 8. Collect $CI_STEP_SUMMARY markdown
        if let Ok(content) = tokio::fs::read_to_string(&ci_summary_file).await {
            if !content.trim().is_empty() {
                overall_summary.push_str(&format!("\n### {}\n{}\n", step.name, content));
            }
        }

        let _ = tokio::fs::remove_dir_all(temp_dir).await;

        if exit_code != 0 {
            log_callback(format!("[system] Step '{}' failed with exit code {}", step.name, exit_code), true);
            Ok(false)
        } else {
            log_callback(format!("[system] Step '{}' completed successfully", step.name), false);
            Ok(true)
        }
    }

    /// Generates container build shell script for Docker or Podman CLI
    fn generate_container_build_script(config: &ContainerBuildConfig) -> String {
        let dockerfile = config.dockerfile.as_deref().unwrap_or("Dockerfile");
        let mut script = String::new();

        if let Some(auth) = &config.registry_auth {
            script.push_str(&format!(
                "echo \"{}\" | docker login \"{}\" -u \"{}\" --password-stdin || echo \"{}\" | podman login \"{}\" -u \"{}\" --password-stdin\n",
                auth.password, auth.server, auth.username, auth.password, auth.server, auth.username
            ));
        }

        let mut tag_args = String::new();
        for t in &config.tags {
            tag_args.push_str(&format!(" -t \"{}:{}\"", config.image, t));
        }

        script.push_str(&format!(
            "docker build -f \"{}\"{} . || podman build -f \"{}\"{} .\n",
            dockerfile, tag_args, dockerfile, tag_args
        ));

        for t in &config.tags {
            script.push_str(&format!(
                "docker push \"{}:{}\" || podman push \"{}:{}\"\n",
                config.image, t, config.image, t
            ));
        }

        script
    }

    /// Parses KEY=VALUE format lines
    fn parse_env_file(content: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim().to_string();
                let mut val = v.trim().to_string();
                if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                    val = val[1..val.len() - 1].to_string();
                }
                result.push((key, val));
            }
        }
        result
    }
}
