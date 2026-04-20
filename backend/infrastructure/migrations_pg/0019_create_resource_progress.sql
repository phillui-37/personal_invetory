CREATE TABLE IF NOT EXISTS resource_progress (
    resource_id TEXT NOT NULL,
    progress    DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 1.0),
    notes       TEXT,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (resource_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
