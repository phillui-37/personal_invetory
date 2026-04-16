CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    linked_at TEXT NOT NULL,
    delinked_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_devices_active_device_id ON devices(device_id) WHERE delinked_at IS NULL;
