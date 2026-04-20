CREATE TABLE IF NOT EXISTS resources (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    notes TEXT,
    resource_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_resources_resource_type ON resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_resources_title ON resources(title);
CREATE UNIQUE INDEX IF NOT EXISTS idx_resources_title_unique_ci ON resources(LOWER(title));
