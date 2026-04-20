CREATE TABLE IF NOT EXISTS web_reader_metas (
    resource_id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    site_name TEXT,
    last_checked_chapter TEXT,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_web_reader_metas_url ON web_reader_metas(url);
