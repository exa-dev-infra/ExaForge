use askama::Template;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    modules::{
        agents::{dto::AgentRecord, service::AgentService},
        pipelines::{
            dto::{JobRecord, PipelineHistoryItem, PipelineRecord},
            service::PipelineService,
        },
        projects::{
            dto::{ProjectRecord, ServiceRecord},
            service::ProjectService,
        },
        rbac::{dto::RoleRecord, service::RbacService},
        secrets::{dto::SecretSummaryResponse, service::SecretService},
        user::{dto::UserResponse, service::UserService},
    },
    pkg::errors::AppError,
};

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub total_projects: usize,
    pub total_services: usize,
    pub online_agents: usize,
    pub total_pipelines: usize,
    pub recent_pipelines: Vec<PipelineRecord>,
}

#[derive(Template)]
#[template(path = "projects.html")]
pub struct ProjectsTemplate {
    pub projects: Vec<ProjectRecord>,
    pub services: Vec<ServiceRecord>,
}

#[derive(Template)]
#[template(path = "pipeline.html")]
pub struct PipelineDetailTemplate {
    pub pipeline: PipelineRecord,
    pub jobs: Vec<JobRecord>,
}

#[derive(Template)]
#[template(path = "pipelines.html")]
pub struct PipelinesHistoryTemplate {
    pub pipelines: Vec<PipelineHistoryItem>,
    pub projects: Vec<ProjectRecord>,
    pub services: Vec<ServiceRecord>,
    pub current_status: Option<String>,
    pub current_project_id: Option<Uuid>,
    pub current_service_id: Option<Uuid>,
    pub current_page: u64,
    pub total_pages: u64,
    pub total_count: i64,
}

impl PipelinesHistoryTemplate {
    pub fn is_selected_project(&self, id: &Uuid) -> bool {
        self.current_project_id.as_ref() == Some(id)
    }

    pub fn is_selected_service(&self, id: &Uuid) -> bool {
        self.current_service_id.as_ref() == Some(id)
    }

    pub fn is_selected_status(&self, status: &str) -> bool {
        self.current_status.as_deref() == Some(status)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct WebPipelinesQuery {
    pub project_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u64>,
}

#[derive(Template)]
#[template(path = "agents.html")]
pub struct AgentsTemplate {
    pub agents: Vec<AgentRecord>,
}

#[derive(Template)]
#[template(path = "secrets.html")]
pub struct SecretsTemplate {
    pub secrets: Vec<SecretSummaryResponse>,
    pub projects: Vec<ProjectRecord>,
    pub services: Vec<ServiceRecord>,
}

#[derive(Template)]
#[template(path = "users.html")]
pub struct UsersTemplate {
    pub users: Vec<UserResponse>,
    pub roles: Vec<RoleRecord>,
    pub projects: Vec<ProjectRecord>,
    pub services: Vec<ServiceRecord>,
}

#[derive(Clone)]
pub struct WebContext {
    pub project_service: Arc<ProjectService>,
    pub pipeline_service: Arc<PipelineService>,
    pub agent_service: Arc<AgentService>,
    pub secret_service: Arc<SecretService>,
    pub user_service: Arc<UserService>,
    pub rbac_service: Arc<RbacService>,
}

pub async fn dashboard_view(
    State(ctx): State<WebContext>,
) -> Result<DashboardTemplate, AppError> {
    let projects = ctx.project_service.list_projects().await?;
    let services = ctx.project_service.list_all_services().await?;
    let agents = ctx.agent_service.list_agents().await?;
    let recent_pipelines = ctx.pipeline_service.list_recent_pipelines(15).await?;

    let online_count = agents.iter().filter(|a| a.status == "online").count();

    Ok(DashboardTemplate {
        total_projects: projects.len(),
        total_services: services.len(),
        online_agents: online_count,
        total_pipelines: recent_pipelines.len(),
        recent_pipelines,
    })
}

pub async fn projects_view(
    State(ctx): State<WebContext>,
) -> Result<ProjectsTemplate, AppError> {
    let projects = ctx.project_service.list_projects().await?;
    let services = ctx.project_service.list_all_services().await?;

    Ok(ProjectsTemplate { projects, services })
}

pub async fn pipeline_detail_view(
    State(ctx): State<WebContext>,
    Path(id): Path<Uuid>,
) -> Result<PipelineDetailTemplate, AppError> {
    let detail = ctx.pipeline_service.get_pipeline_detail(id).await?;
    Ok(PipelineDetailTemplate {
        pipeline: detail.pipeline,
        jobs: detail.jobs,
    })
}

pub async fn pipelines_history_view(
    State(ctx): State<WebContext>,
    Query(query): Query<WebPipelinesQuery>,
) -> Result<PipelinesHistoryTemplate, AppError> {
    let limit = 20i64;
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) as i64 * limit;

    let clean_status = query.status.as_deref().filter(|s| !s.is_empty() && *s != "all");

    let pipelines = ctx.pipeline_service.list_pipeline_history(
        query.project_id,
        query.service_id,
        clean_status,
        limit,
        offset,
    ).await?;

    let total_count = ctx.pipeline_service.count_pipeline_history(
        query.project_id,
        query.service_id,
        clean_status,
    ).await?;

    let total_pages = if limit == 0 || total_count == 0 {
        1
    } else {
        (total_count as f64 / limit as f64).ceil() as u64
    };

    let projects = ctx.project_service.list_projects().await?;
    let services = ctx.project_service.list_all_services().await?;

    Ok(PipelinesHistoryTemplate {
        pipelines,
        projects,
        services,
        current_status: clean_status.map(|s| s.to_string()),
        current_project_id: query.project_id,
        current_service_id: query.service_id,
        current_page: page,
        total_pages,
        total_count,
    })
}

pub async fn agents_view(
    State(ctx): State<WebContext>,
) -> Result<AgentsTemplate, AppError> {
    let agents = ctx.agent_service.list_agents().await?;
    Ok(AgentsTemplate { agents })
}

pub async fn secrets_view(
    State(ctx): State<WebContext>,
) -> Result<SecretsTemplate, AppError> {
    let secrets = ctx.secret_service.list_secrets_summary().await?;
    let projects = ctx.project_service.list_projects().await?;
    let services = ctx.project_service.list_all_services().await?;
    Ok(SecretsTemplate { secrets, projects, services })
}

pub async fn users_view(
    State(ctx): State<WebContext>,
) -> Result<UsersTemplate, AppError> {
    let (users, _, _, _) = ctx.user_service.list_users(Default::default()).await?;
    let roles = ctx.rbac_service.list_roles().await?;
    let projects = ctx.project_service.list_projects().await?;
    let services = ctx.project_service.list_all_services().await?;

    Ok(UsersTemplate {
        users,
        roles,
        projects,
        services,
    })
}

pub fn web_router(ctx: WebContext) -> Router {
    Router::new()
        .route("/", get(dashboard_view))
        .route("/projects", get(projects_view))
        .route("/pipelines", get(pipelines_history_view))
        .route("/pipelines/:id", get(pipeline_detail_view))
        .route("/agents", get(agents_view))
        .route("/secrets", get(secrets_view))
        .route("/users", get(users_view))
        .with_state(ctx)
}
