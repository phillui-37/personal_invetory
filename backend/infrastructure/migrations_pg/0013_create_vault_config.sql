CREATE TABLE IF NOT EXISTS vault_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    salt BYTEA NOT NULL,
    key_check BYTEA NOT NULL,
    key_check_nonce BYTEA NOT NULL,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::TEXT)
);
