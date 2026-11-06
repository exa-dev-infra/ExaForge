ALTER TABLE services DROP COLUMN IF EXISTS exavault_url;
ALTER TABLE services DROP COLUMN IF EXISTS exavault_token;

ALTER TABLE projects DROP COLUMN IF EXISTS exavault_url;
ALTER TABLE projects DROP COLUMN IF EXISTS exavault_token;

ALTER TABLE secrets DROP COLUMN IF EXISTS source;

