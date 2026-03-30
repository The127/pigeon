-- Migrate signing_secret (single TEXT) to signing_secrets (TEXT array).
-- Existing secrets are preserved. Endpoints without a secret get a placeholder
-- that will be replaced by the application on next access.

ALTER TABLE endpoints ADD COLUMN signing_secrets TEXT[] NOT NULL DEFAULT '{}';

-- Migrate existing data: wrap single secret in array
UPDATE endpoints
SET signing_secrets = ARRAY[signing_secret]
WHERE signing_secret IS NOT NULL;

-- Generate simple secrets for endpoints that had none (using md5 of random values)
UPDATE endpoints
SET signing_secrets = ARRAY['whsec_' || md5(random()::text) || md5(random()::text)]
WHERE signing_secrets = '{}';

-- Drop the old column
ALTER TABLE endpoints DROP COLUMN signing_secret;
