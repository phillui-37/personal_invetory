CREATE TABLE IF NOT EXISTS image_metas (
    resource_id TEXT PRIMARY KEY,
    width INTEGER,
    height INTEGER,
    file_format TEXT,
    file_size_bytes INTEGER,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
