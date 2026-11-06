-- Migration 000005: Create System Roles & Permissions Matrix
CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '[]'::jsonb,
    is_system BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Seed System Roles
INSERT INTO roles (id, name, description, permissions, is_system)
VALUES
    ('super_admin', 'Super Administrator', 'Full platform control with unrestricted permissions across all organizations and resources', '["*"]'::jsonb, TRUE),
    ('admin', 'Administrator', 'Administrative access to manage users, projects, agents, and system configurations', '["users:read", "users:write", "projects:read", "projects:write", "services:read", "services:write", "agents:read", "agents:write", "secrets:read", "secrets:write", "pipelines:read", "pipelines:trigger", "pipelines:cancel"]'::jsonb, TRUE),
    ('developer', 'Developer', 'Read and trigger pipelines for assigned projects/services with secret read access', '["projects:read", "services:read", "pipelines:read", "pipelines:trigger", "secrets:read"]'::jsonb, TRUE),
    ('viewer', 'Viewer', 'Read-only access to view projects, services, agents, and pipeline execution logs', '["projects:read", "services:read", "pipelines:read", "agents:read"]'::jsonb, TRUE)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    permissions = EXCLUDED.permissions;

-- Ensure fast lookup
CREATE INDEX IF NOT EXISTS idx_roles_is_system ON roles(is_system);

