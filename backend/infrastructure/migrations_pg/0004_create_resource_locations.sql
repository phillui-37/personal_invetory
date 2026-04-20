CREATE TABLE IF NOT EXISTS resource_locations (
    id TEXT PRIMARY KEY,
    resource_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    path_or_url TEXT NOT NULL,
    storage_type TEXT NOT NULL,
    FOREIGN KEY(resource_id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_resource_locations_resource_id ON resource_locations(resource_id);
