-- Migration 000004: Create ExaForge CI/CD Engine Core Tables

-- 1. Projects
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_projects_slug ON projects(slug);

-- 2. Services (FE / BE / Mobile)
CREATE TABLE IF NOT EXISTS services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    repo_url TEXT NOT NULL,
    repo_type VARCHAR(30) NOT NULL DEFAULT 'multi_repo', -- 'multi_repo' | 'monorepo'
    path_filters JSONB NOT NULL DEFAULT '[]'::jsonb,
    tag_prefix VARCHAR(50),
    default_branch VARCHAR(100) NOT NULL DEFAULT 'main',
    webhook_secret VARCHAR(128) NOT NULL,
    pipeline_file VARCHAR(100) NOT NULL DEFAULT '.ci.yaml',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_project_service_slug UNIQUE (project_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_services_project_id ON services(project_id);

-- 3. Granular RBAC: Project Memberships
CREATE TABLE IF NOT EXISTS project_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'maintainer' | 'viewer'
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_project_user UNIQUE (project_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_project_memberships_user ON project_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_project_memberships_project ON project_memberships(project_id);

-- 4. Granular RBAC: Service Memberships
CREATE TABLE IF NOT EXISTS service_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'developer' | 'viewer'
    can_deploy_staging_only BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_service_user UNIQUE (service_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_service_memberships_user ON service_memberships(user_id);
CREATE INDEX IF NOT EXISTS idx_service_memberships_service ON service_memberships(service_id);

-- 5. Hierarchical Scoped Secrets (AES-256-GCM)
CREATE TABLE IF NOT EXISTS secrets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope_type VARCHAR(20) NOT NULL, -- 'global' | 'project' | 'service'
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    service_id UUID REFERENCES services(id) ON DELETE CASCADE,
    key VARCHAR(150) NOT NULL,
    encrypted_value TEXT NOT NULL,
    iv VARCHAR(64) NOT NULL,
    auth_tag VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_secret_scope CHECK (
        (scope_type = 'global' AND project_id IS NULL AND service_id IS NULL) OR
        (scope_type = 'project' AND project_id IS NOT NULL AND service_id IS NULL) OR
        (scope_type = 'service' AND project_id IS NOT NULL AND service_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_secrets_scope_key ON secrets (
    scope_type,
    COALESCE(project_id, '00000000-0000-0000-0000-000000000000'::uuid),
    COALESCE(service_id, '00000000-0000-0000-0000-000000000000'::uuid),
    key
);

-- 6. Worker Agents Registry & Capability Tags
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    agent_token VARCHAR(128) UNIQUE NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb, -- e.g. ["linux", "docker", "k8s"]
    status VARCHAR(20) NOT NULL DEFAULT 'offline', -- 'online' | 'offline' | 'busy'
    socket_id VARCHAR(64),
    os_info JSONB DEFAULT '{}'::jsonb,
    last_heartbeat_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);

-- 7. Pipelines (Workflow Runs)
CREATE TABLE IF NOT EXISTS pipelines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    trigger_type VARCHAR(30) NOT NULL, -- 'webhook_push' | 'webhook_tag' | 'manual'
    trigger_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    git_ref VARCHAR(150) NOT NULL,
    git_commit_sha VARCHAR(64) NOT NULL,
    git_commit_message TEXT,
    concurrency_group VARCHAR(150),
    cancel_in_progress BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(30) NOT NULL DEFAULT 'pending', -- 'pending' | 'running' | 'success' | 'failed' | 'cancelled'
    inputs JSONB DEFAULT '{}'::jsonb,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_pipelines_service ON pipelines(service_id, status);
CREATE INDEX IF NOT EXISTS idx_pipelines_concurrency ON pipelines(project_id, concurrency_group) WHERE status = 'running';

-- 8. Jobs (DAG Step Units)
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pipeline_id UUID NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
    job_key VARCHAR(100) NOT NULL,
    name VARCHAR(150) NOT NULL,
    required_tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    needs JSONB NOT NULL DEFAULT '[]'::jsonb,
    status VARCHAR(30) NOT NULL DEFAULT 'pending_dependencies', -- 'pending_dependencies' | 'pending_agent' | 'assigned' | 'running' | 'success' | 'failed' | 'cancelled'
    assigned_agent_id UUID REFERENCES agents(id) ON DELETE SET NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    inputs JSONB DEFAULT '{}'::jsonb,
    outputs JSONB DEFAULT '{}'::jsonb,
    step_summaries TEXT DEFAULT '',
    error_message TEXT,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_jobs_pipeline ON jobs(pipeline_id);
CREATE INDEX IF NOT EXISTS idx_jobs_pending_agent ON jobs(status) WHERE status = 'pending_agent';

-- 9. Real-Time Job Logs
CREATE TABLE IF NOT EXISTS job_logs (
    id BIGSERIAL PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL DEFAULT 0,
    stream VARCHAR(10) NOT NULL DEFAULT 'stdout', -- 'stdout' | 'stderr' | 'system'
    line TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_job_logs_job_id ON job_logs(job_id, id);
