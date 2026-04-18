CREATE TABLE IF NOT EXISTS chapter_checks (
  id TEXT PRIMARY KEY,
  resource_id TEXT NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
  has_new_chapter INTEGER NOT NULL DEFAULT 0,
  latest_chapter TEXT,
  checked_at TEXT NOT NULL,
  error_message TEXT
);
CREATE INDEX IF NOT EXISTS chapter_checks_resource_id ON chapter_checks(resource_id);
