CREATE TABLE settings (
  key TEXT NOT NULL, value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  scope TEXT NOT NULL, updated_at INTEGER NOT NULL, PRIMARY KEY(key,scope)
) STRICT;
CREATE TABLE modules (
  id TEXT PRIMARY KEY NOT NULL, version TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind='builtin'), enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
  installed_path TEXT, manifest_version INTEGER NOT NULL CHECK(manifest_version=1),
  manifest_json TEXT NOT NULL CHECK(json_valid(manifest_json))
) STRICT;
CREATE TABLE audit_events (
  id INTEGER PRIMARY KEY, event_type TEXT NOT NULL, subject_id TEXT NOT NULL,
  outcome TEXT NOT NULL, timestamp INTEGER NOT NULL,
  safe_details_json TEXT NOT NULL CHECK(json_valid(safe_details_json))
) STRICT;
