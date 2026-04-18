CREATE TABLE IF NOT EXISTS vault_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    salt BLOB NOT NULL,
    key_check BLOB NOT NULL,
    key_check_nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
