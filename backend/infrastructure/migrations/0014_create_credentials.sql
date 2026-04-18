CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    credential_type TEXT NOT NULL,
    encrypted_blob BLOB NOT NULL,
    nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(platform, credential_type)
);
