CREATE TABLE permission_grants (
  id TEXT PRIMARY KEY NOT NULL,
  subject_type TEXT NOT NULL CHECK(subject_type='module'),
  subject_id TEXT NOT NULL REFERENCES modules(id),
  capability TEXT NOT NULL CHECK(capability IN ('file_read','file_write')),
  resource_scope TEXT NOT NULL,
  grant_type TEXT NOT NULL CHECK(grant_type='directory'),
  revoked_at INTEGER
) STRICT;
CREATE INDEX grants_subject_capability ON permission_grants(subject_id,capability) WHERE revoked_at IS NULL;
