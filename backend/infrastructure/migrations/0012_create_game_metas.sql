CREATE TABLE IF NOT EXISTS game_metas (
    resource_id TEXT PRIMARY KEY,
    platform TEXT,
    store TEXT,
    developer TEXT,
    publisher TEXT,
    manual_notes TEXT,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
