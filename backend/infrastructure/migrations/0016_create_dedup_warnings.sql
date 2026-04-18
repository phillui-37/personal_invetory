CREATE TABLE IF NOT EXISTS dedup_warnings (
    id TEXT PRIMARY KEY,
    resource_id_a TEXT NOT NULL REFERENCES resources(id),
    resource_id_b TEXT NOT NULL REFERENCES resources(id),
    similarity_score REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    UNIQUE(resource_id_a, resource_id_b)
);
