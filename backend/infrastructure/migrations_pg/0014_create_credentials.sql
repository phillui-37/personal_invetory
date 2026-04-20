CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    credential_type TEXT NOT NULL,
    encrypted_blob BYTEA NOT NULL,
    nonce BYTEA NOT NULL,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT),
    updated_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT),
    UNIQUE(platform, credential_type)
);
