ALTER TABLE web_reader_metas ADD COLUMN IF NOT EXISTS check_interval_secs INTEGER;
ALTER TABLE web_reader_metas ADD COLUMN IF NOT EXISTS last_checked_at TEXT;
ALTER TABLE web_reader_metas ADD COLUMN IF NOT EXISTS progress_css_selector TEXT;
