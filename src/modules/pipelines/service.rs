use std::{
    collections::HashMap,
    sync::Arc,
};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{
    constants::codes::{CODE_JOB_NOT_FOUND, CODE_PIPELINE_NOT_FOUND},
    modules::{
        agents::{
            dto::{ControllerToAgentMessage, StepDefinition},
            service::AgentService,
        },
        projects::service::ProjectService,
        secrets::service::SecretService,
    },
    pkg::errors::AppError,
};
use super::{
    dto::{JobRecord, PipelineDetailResponse, PipelineRecord},
    parser::{interpolate_variables, parse_pipeline_config, PipelineConfig},
    repository::PipelineRepository,
};

#[derive(Clone)]
pub struct PipelineService {
    pool: PgPool,
    repo: Arc<dyn PipelineRepository>,
    project_service: Arc<ProjectService>,
    secret_service: Arc<SecretService>,
    agent_service: Arc<AgentService>,
}

impl PipelineService {
    pub fn new(
        pool: PgPool,
        repo: Arc<dyn PipelineRepository>,
        project_service: Arc<ProjectService>,
        secret_service: Arc<SecretService>,
        agent_service: Arc<AgentService>,
    ) -> Self {
        Self {
            pool,
            repo,
            project_service,
            secret_service,
            agent_service,
        }
    }

    /// Triggers a pipeline execution with smart concurrency & cancel-in-progress support
    pub async fn trigger_pipeline(
        &self,
        project_id: Uuid,
        service_id: Uuid,
        trigger_type: &str,
        trigger_user_id: Option<Uuid>,
        git_ref: &str,
        git_commit_sha: &str,
        git_commit_message: Option<&str>,
        runtime_inputs: Option<HashMap<String, String>>,
        pipeline_config_raw: Option<&str>,
    ) -> Result<PipelineRecord, AppError> {
        let service = self.project_service.get_service(service_id).await?;

        // 1. Parse pipeline config (or use default simple CI config if not provided)
        let default_config = r#"
name: Default Pipeline
concurrency:
  group: default
  cancel_in_progress: true
jobs:
  build:
    name: Build & Verify
    tags: ["linux", "docker"]
    steps:
      - name: Checkout & Test
        run: |
          echo "Building commit ${CI_COMMIT_SHA} on ref ${CI_REF}..."
          git status
"#;
        let config_str = pipeline_config_raw.unwrap_or(default_config);
        let config: PipelineConfig = parse_pipeline_config(config_str)?;

        let concurrency_group = config.concurrency.as_ref()
            .and_then(|c| if c.group.is_empty() || c.group == "default" { None } else { Some(c.group.clone()) })
            .unwrap_or_else(|| format!("{}-{}", service.slug, git_ref));
        let cancel_in_progress = config.concurrency.as_ref().map(|c| c.cancel_in_progress).unwrap_or(false);

        // 2. Concurrency Control: Cancel in progress if flagged
        if cancel_in_progress {
            let running = self.repo.find_running_in_concurrency_group(&self.pool, project_id, &concurrency_group).await?;
            for old_pipe in running {
                info!(pipeline_id = %old_pipe.id, concurrency_group = %concurrency_group, "Cancelling in-progress pipeline");
                self.cancel_pipeline(old_pipe.id).await?;
            }
        }

        // 3. Resolve inputs: default inputs merged with runtime inputs
        let mut final_inputs = HashMap::new();
        for (k, v) in &config.inputs {
            if let Some(default_val) = &v.default {
                final_inputs.insert(k.clone(), default_val.clone());
            }
        }
        if let Some(overrides) = runtime_inputs {
            for (k, v) in overrides {
                final_inputs.insert(k, v);
            }
        }
        final_inputs.insert("CI_COMMIT_SHA".to_string(), git_commit_sha.to_string());
        final_inputs.insert("CI_REF".to_string(), git_ref.to_string());

        let inputs_json = serde_json::to_value(&final_inputs).unwrap_or_else(|_| serde_json::json!({}));

        // 4. Create Pipeline in Database
        let mut tx = self.pool.begin().await
            .map_err(|e| AppError::database(format!("Failed to begin transaction: {}", e)))?;

        let pipeline = self.repo.create_pipeline(
            &mut tx,
            project_id,
            service_id,
            trigger_type,
            trigger_user_id,
            git_ref,
            git_commit_sha,
            git_commit_message,
            Some(&concurrency_group),
            cancel_in_progress,
            inputs_json,
        ).await?;

        // 5. Create Jobs in Database
        let mut created_jobs = Vec::new();
        for (job_key, job_cfg) in &config.jobs {
            let tags_json = serde_json::to_value(&job_cfg.tags).unwrap_or_else(|_| serde_json::json!([]));
            let needs_json = serde_json::to_value(&job_cfg.needs).unwrap_or_else(|_| serde_json::json!([]));

            let job_record = self.repo.create_job(
                &mut tx,
                pipeline.id,
                job_key,
                &job_cfg.name,
                tags_json,
                needs_json,
                job_cfg.timeout_seconds as i32,
                serde_json::to_value(&job_cfg.env).unwrap_or_else(|_| serde_json::json!({})),
            ).await?;

            created_jobs.push((job_record, job_cfg.clone()));
        }

        tx.commit().await
            .map_err(|e| AppError::database(format!("Failed to commit transaction: {}", e)))?;

        // 6. Dispatch Jobs without dependencies immediately
        for (job_rec, job_cfg) in created_jobs {
            if job_cfg.needs.is_empty() {
                let _ = self.try_dispatch_job(
                    job_rec,
                    &service.repo_url,
                    git_ref,
                    git_commit_sha,
                    job_cfg.steps,
                    job_cfg.env,
                    &final_inputs,
                ).await;
            }
        }

        Ok(pipeline)
    }

    /// Tries to find an online agent matching job tags and dispatches the job
    pub async fn try_dispatch_job(
        &self,
        job: JobRecord,
        repo_url: &str,
        git_ref: &str,
        git_commit_sha: &str,
        steps: Vec<StepDefinition>,
        step_env: HashMap<String, String>,
        inputs: &HashMap<String, String>,
    ) -> Result<bool, AppError> {
        let required_tags: Vec<String> = job.required_tags.as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let matching_agent = self.agent_service.find_matching_online_agent(&required_tags).await?;

        if let Some(agent) = matching_agent {
            // Get pipeline info for project/service IDs
            let pipeline = self.repo.find_pipeline_by_id(&self.pool, job.pipeline_id).await?
                .ok_or_else(|| AppError::not_found("Pipeline not found", CODE_PIPELINE_NOT_FOUND))?;

            // 1. Resolve scoped secrets (Global -> Project -> Service)
            let secrets = self.secret_service.resolve_secrets_for_job(pipeline.project_id, pipeline.service_id).await?;

            // 2. Prepare interpolated environment
            let mut combined_env = inputs.clone();
            for (k, v) in step_env {
                combined_env.insert(k, v);
            }

            // 3. Interpolate variables in step commands
            let mut interpolated_steps = Vec::new();
            for mut step in steps {
                if let Some(cmd) = step.run {
                    step.run = Some(interpolate_variables(&cmd, &combined_env));
                }
                interpolated_steps.push(step);
            }

            // 4. Send AssignJob message to Agent via WebSocket
            let assign_msg = ControllerToAgentMessage::AssignJob {
                job_id: job.id,
                pipeline_id: job.pipeline_id,
                repo_url: repo_url.to_string(),
                git_ref: git_ref.to_string(),
                git_commit_sha: git_commit_sha.to_string(),
                timeout_seconds: job.timeout_seconds as u64,
                steps: interpolated_steps,
                secrets,
                env: combined_env,
            };

            if self.agent_service.send_to_agent(agent.id, assign_msg).await {
                self.repo.update_job_status(&self.pool, job.id, "assigned", Some(agent.id)).await?;
                info!(job_id = %job.id, agent_id = %agent.id, "Job assigned to online agent");
                return Ok(true);
            }
        }

        // Keep in pending_agent if no agent matches or sending failed
        self.repo.update_job_status(&self.pool, job.id, "pending_agent", None).await?;
        info!(job_id = %job.id, "Job queued in pending_agent state");
        Ok(false)
    }

    /// Cancels a pipeline and notifies active agents
    pub async fn cancel_pipeline(&self, pipeline_id: Uuid) -> Result<(), AppError> {
        let jobs = self.repo.list_jobs_by_pipeline(&self.pool, pipeline_id).await?;
        for job in jobs {
            if job.status == "running" || job.status == "assigned" {
                if let Some(agent_id) = job.assigned_agent_id {
                    let _ = self.agent_service.send_to_agent(agent_id, ControllerToAgentMessage::CancelJob { job_id: job.id }).await;
                }
                let _ = self.repo.update_job_status(&self.pool, job.id, "cancelled", None).await;
            } else if job.status == "pending_agent" || job.status == "pending_dependencies" {
                let _ = self.repo.update_job_status(&self.pool, job.id, "cancelled", None).await;
            }
        }

        self.repo.update_pipeline_status(&self.pool, pipeline_id, "cancelled").await?;
        Ok(())
    }

    pub async fn get_pipeline_detail(&self, id: Uuid) -> Result<PipelineDetailResponse, AppError> {
        let pipeline = self.repo.find_pipeline_by_id(&self.pool, id).await?
            .ok_or_else(|| AppError::not_found("Pipeline not found", CODE_PIPELINE_NOT_FOUND))?;
        let jobs = self.repo.list_jobs_by_pipeline(&self.pool, id).await?;
        Ok(PipelineDetailResponse { pipeline, jobs })
    }

    pub async fn list_pipelines_by_service(&self, service_id: Uuid) -> Result<Vec<PipelineRecord>, AppError> {
        self.repo.list_pipelines_by_service(&self.pool, service_id).await
    }

    pub async fn list_recent_pipelines(&self, limit: i64) -> Result<Vec<PipelineRecord>, AppError> {
        self.repo.list_recent_pipelines(&self.pool, limit).await
    }

    pub async fn get_job_detail(&self, job_id: Uuid) -> Result<JobRecord, AppError> {
        self.repo.find_job_by_id(&self.pool, job_id).await?
            .ok_or_else(|| AppError::not_found("Job not found", CODE_JOB_NOT_FOUND))
    }

    pub async fn get_job_logs(&self, job_id: Uuid) -> Result<Vec<String>, AppError> {
        let records = self.repo.get_job_logs(&self.pool, job_id).await?;
        Ok(records.into_iter().map(|r| r.line).collect())
    }
}
