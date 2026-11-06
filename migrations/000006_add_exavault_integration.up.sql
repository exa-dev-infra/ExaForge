-- Add ExaVault integration fields to services
ALTER TABLE services ADD COLUMN IF NOT EXISTS exavault_url VARCHAR(255);
ALTER TABLE services ADD COLUMN IF NOT EXISTS exavault_token VARCHAR(150);

-- Add ExaVault integration fields to projects
ALTER TABLE projects ADD COLUMN IF NOT EXISTS exavault_url VARCHAR(255);
ALTER TABLE projects ADD COLUMN IF NOT EXISTS exavault_token VARCHAR(150);

-- Add source tracking to secrets (manual | exavault)
ALTER TABLE secrets ADD COLUMN IF NOT EXISTS source VARCHAR(50) DEFAULT 'manual';

