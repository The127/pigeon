ALTER TABLE applications ADD COLUMN org_id UUID NOT NULL REFERENCES organizations(id);
CREATE INDEX idx_applications_org_id ON applications(org_id);
