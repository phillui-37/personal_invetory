CREATE TABLE IF NOT EXISTS video_metas (
    resource_id TEXT PRIMARY KEY,
    duration_secs INTEGER,
    file_format TEXT,
    resolution TEXT,
    file_size_bytes INTEGER,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
