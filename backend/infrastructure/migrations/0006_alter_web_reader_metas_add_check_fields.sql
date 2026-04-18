ALTER TABLE web_reader_metas ADD COLUMN check_interval_secs INTEGER;
ALTER TABLE web_reader_metas ADD COLUMN last_checked_at TEXT;
ALTER TABLE web_reader_metas ADD COLUMN progress_css_selector TEXT;
