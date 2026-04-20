CREATE TABLE IF NOT EXISTS site_configs (
    id TEXT PRIMARY KEY,
    url_pattern TEXT NOT NULL,
    css_selector TEXT,
    xpath TEXT,
    text_regex TEXT,
    check_interval_secs INTEGER
);
