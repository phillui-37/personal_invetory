CREATE TABLE IF NOT EXISTS ebook_metas (
    resource_id TEXT PRIMARY KEY,
    author TEXT,
    isbn TEXT,
    publisher TEXT,
    language TEXT,
    file_format TEXT,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ebook_metas_author ON ebook_metas(author);
