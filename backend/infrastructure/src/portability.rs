use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceRow {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub resource_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EbookMetaRow {
    pub resource_id: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WebReaderMetaRow {
    pub resource_id: String,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceLocationRow {
    pub id: String,
    pub resource_id: String,
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageMetaRow {
    pub resource_id: String,
    pub file_format: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VideoMetaRow {
    pub resource_id: String,
    pub duration_secs: Option<i64>,
    pub resolution: Option<String>,
    pub file_format: Option<String>,
    pub studio: Option<String>,
    pub series: Option<String>,
    pub episode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameMetaRow {
    pub resource_id: String,
    pub platform: Option<String>,
    pub store: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub manual_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProgressRow {
    pub resource_id: String,
    pub progress: f64,
    pub notes: Option<String>,
    pub updated_at: String,
}

impl Eq for ProgressRow {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagRow {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceTagRow {
    pub resource_id: String,
    pub tag_id: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanonicalResourceSnapshot {
    pub resources: Vec<ResourceRow>,
    pub ebook_metas: Vec<EbookMetaRow>,
    pub web_reader_metas: Vec<WebReaderMetaRow>,
    pub resource_locations: Vec<ResourceLocationRow>,
    pub image_metas: Vec<ImageMetaRow>,
    pub video_metas: Vec<VideoMetaRow>,
    pub game_metas: Vec<GameMetaRow>,
    pub progress: Vec<ProgressRow>,
    pub tags: Vec<TagRow>,
    pub resource_tags: Vec<ResourceTagRow>,
}

impl Eq for CanonicalResourceSnapshot {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortabilityBackend {
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresParityHooks {
    pub export_contract: &'static str,
    pub import_contract: &'static str,
}

pub fn postgres_parity_hooks() -> PostgresParityHooks {
    PostgresParityHooks {
        export_contract: r#"
SELECT id, title, notes, resource_type, created_at, updated_at
FROM resources
ORDER BY id;
SELECT resource_id, author, isbn, publisher, language, file_format
FROM ebook_metas
ORDER BY resource_id;
SELECT resource_id, url, site_name, last_checked_chapter
FROM web_reader_metas
ORDER BY resource_id;
SELECT id, resource_id, device_id, path_or_url, storage_type
FROM resource_locations
ORDER BY id;
"#,
        import_contract: r#"
INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO UPDATE
SET title = EXCLUDED.title,
    notes = EXCLUDED.notes,
    resource_type = EXCLUDED.resource_type,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at;

INSERT INTO ebook_metas (resource_id, author, isbn, publisher, language, file_format)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (resource_id) DO UPDATE
SET author = EXCLUDED.author,
    isbn = EXCLUDED.isbn,
    publisher = EXCLUDED.publisher,
    language = EXCLUDED.language,
    file_format = EXCLUDED.file_format;

INSERT INTO web_reader_metas (resource_id, url, site_name, last_checked_chapter)
VALUES ($1, $2, $3, $4)
ON CONFLICT (resource_id) DO UPDATE
SET url = EXCLUDED.url,
    site_name = EXCLUDED.site_name,
    last_checked_chapter = EXCLUDED.last_checked_chapter;

INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO UPDATE
SET resource_id = EXCLUDED.resource_id,
    device_id = EXCLUDED.device_id,
    path_or_url = EXCLUDED.path_or_url,
    storage_type = EXCLUDED.storage_type;
"#,
    }
}

pub fn normalize_resource_rows(mut rows: Vec<ResourceRow>) -> Vec<ResourceRow> {
    rows.sort_by(|left, right| left.id.cmp(&right.id));
    rows
}

pub fn normalize_ebook_meta_rows(mut rows: Vec<EbookMetaRow>) -> Vec<EbookMetaRow> {
    rows.sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
    rows
}

pub fn normalize_web_reader_meta_rows(mut rows: Vec<WebReaderMetaRow>) -> Vec<WebReaderMetaRow> {
    rows.sort_by(|left, right| left.resource_id.cmp(&right.resource_id));
    rows
}

pub fn normalize_resource_location_rows(
    mut rows: Vec<ResourceLocationRow>,
) -> Vec<ResourceLocationRow> {
    rows.sort_by(|left, right| left.id.cmp(&right.id));
    rows
}

pub fn export_canonical_snapshot_sqlite(
    conn: &Connection,
) -> rusqlite::Result<CanonicalResourceSnapshot> {
    let resources = {
        let mut stmt = conn.prepare(
            "SELECT id, title, notes, resource_type, created_at, updated_at
             FROM resources
             ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ResourceRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    notes: row.get(2)?,
                    resource_type: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        normalize_resource_rows(rows)
    };

    let ebook_metas = {
        let mut stmt = conn.prepare(
            "SELECT resource_id, author, isbn, publisher, language, file_format
             FROM ebook_metas
             ORDER BY resource_id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(EbookMetaRow {
                    resource_id: row.get(0)?,
                    author: row.get(1)?,
                    isbn: row.get(2)?,
                    publisher: row.get(3)?,
                    language: row.get(4)?,
                    file_format: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        normalize_ebook_meta_rows(rows)
    };

    let web_reader_metas = {
        let mut stmt = conn.prepare(
            "SELECT resource_id, url, site_name, last_checked_chapter
             FROM web_reader_metas
             ORDER BY resource_id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(WebReaderMetaRow {
                    resource_id: row.get(0)?,
                    url: row.get(1)?,
                    site_name: row.get(2)?,
                    last_checked_chapter: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        normalize_web_reader_meta_rows(rows)
    };

    let resource_locations = {
        let mut stmt = conn.prepare(
            "SELECT id, resource_id, device_id, path_or_url, storage_type
             FROM resource_locations
             ORDER BY id",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ResourceLocationRow {
                    id: row.get(0)?,
                    resource_id: row.get(1)?,
                    device_id: row.get(2)?,
                    path_or_url: row.get(3)?,
                    storage_type: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        normalize_resource_location_rows(rows)
    };

    let image_metas = export_image_metas_sqlite(conn).unwrap_or_default();
    let video_metas = export_video_metas_sqlite(conn).unwrap_or_default();
    let game_metas = export_game_metas_sqlite(conn).unwrap_or_default();
    let progress = export_progress_sqlite(conn).unwrap_or_default();
    let tags = export_tags_sqlite(conn).unwrap_or_default();
    let resource_tags = export_resource_tags_sqlite(conn).unwrap_or_default();

    Ok(CanonicalResourceSnapshot {
        resources,
        ebook_metas,
        web_reader_metas,
        resource_locations,
        image_metas,
        video_metas,
        game_metas,
        progress,
        tags,
        resource_tags,
    })
}

fn export_image_metas_sqlite(conn: &Connection) -> rusqlite::Result<Vec<ImageMetaRow>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, file_format, width, height, tags FROM image_metas ORDER BY resource_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ImageMetaRow {
            resource_id: row.get(0)?,
            file_format: row.get(1)?,
            width: row.get(2)?,
            height: row.get(3)?,
            tags: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn export_video_metas_sqlite(conn: &Connection) -> rusqlite::Result<Vec<VideoMetaRow>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, duration_secs, resolution, file_format, studio, series, episode
         FROM video_metas ORDER BY resource_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(VideoMetaRow {
            resource_id: row.get(0)?,
            duration_secs: row.get(1)?,
            resolution: row.get(2)?,
            file_format: row.get(3)?,
            studio: row.get(4)?,
            series: row.get(5)?,
            episode: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn export_game_metas_sqlite(conn: &Connection) -> rusqlite::Result<Vec<GameMetaRow>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, platform, store, developer, publisher, manual_notes
         FROM game_metas ORDER BY resource_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GameMetaRow {
            resource_id: row.get(0)?,
            platform: row.get(1)?,
            store: row.get(2)?,
            developer: row.get(3)?,
            publisher: row.get(4)?,
            manual_notes: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn export_progress_sqlite(conn: &Connection) -> rusqlite::Result<Vec<ProgressRow>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, progress, notes, updated_at FROM resource_progress ORDER BY resource_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProgressRow {
            resource_id: row.get(0)?,
            progress: row.get(1)?,
            notes: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn export_tags_sqlite(conn: &Connection) -> rusqlite::Result<Vec<TagRow>> {
    let mut stmt =
        conn.prepare("SELECT id, name, created_at FROM tags ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(TagRow {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn export_resource_tags_sqlite(conn: &Connection) -> rusqlite::Result<Vec<ResourceTagRow>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, tag_id FROM resource_tags ORDER BY resource_id, tag_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ResourceTagRow {
            resource_id: row.get(0)?,
            tag_id: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn import_canonical_snapshot_sqlite(
    conn: &mut Connection,
    snapshot: &CanonicalResourceSnapshot,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    // Delete in reverse dependency order
    tx.execute("DELETE FROM resource_tags", [])?;
    tx.execute("DELETE FROM tags", [])?;
    tx.execute("DELETE FROM resource_progress", [])?;
    tx.execute("DELETE FROM game_metas", [])?;
    tx.execute("DELETE FROM video_metas", [])?;
    tx.execute("DELETE FROM image_metas", [])?;
    tx.execute("DELETE FROM resource_locations", [])?;
    tx.execute("DELETE FROM web_reader_metas", [])?;
    tx.execute("DELETE FROM ebook_metas", [])?;
    tx.execute("DELETE FROM resources", [])?;

    for row in normalize_resource_rows(snapshot.resources.clone()) {
        tx.execute(
            "INSERT INTO resources (id, title, notes, resource_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row.id,
                row.title,
                row.notes,
                row.resource_type,
                row.created_at,
                row.updated_at
            ],
        )?;
    }

    for row in normalize_ebook_meta_rows(snapshot.ebook_metas.clone()) {
        tx.execute(
            "INSERT INTO ebook_metas (resource_id, author, isbn, publisher, language, file_format)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row.resource_id,
                row.author,
                row.isbn,
                row.publisher,
                row.language,
                row.file_format
            ],
        )?;
    }

    for row in normalize_web_reader_meta_rows(snapshot.web_reader_metas.clone()) {
        tx.execute(
            "INSERT INTO web_reader_metas (resource_id, url, site_name, last_checked_chapter)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                row.resource_id,
                row.url,
                row.site_name,
                row.last_checked_chapter
            ],
        )?;
    }

    for row in normalize_resource_location_rows(snapshot.resource_locations.clone()) {
        tx.execute(
            "INSERT INTO resource_locations (id, resource_id, device_id, path_or_url, storage_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                row.id,
                row.resource_id,
                row.device_id,
                row.path_or_url,
                row.storage_type
            ],
        )?;
    }

    for row in &snapshot.image_metas {
        tx.execute(
            "INSERT INTO image_metas (resource_id, file_format, width, height, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![row.resource_id, row.file_format, row.width, row.height, row.tags],
        )?;
    }

    for row in &snapshot.video_metas {
        tx.execute(
            "INSERT INTO video_metas (resource_id, duration_secs, resolution, file_format, studio, series, episode)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                row.resource_id, row.duration_secs, row.resolution,
                row.file_format, row.studio, row.series, row.episode
            ],
        )?;
    }

    for row in &snapshot.game_metas {
        tx.execute(
            "INSERT INTO game_metas (resource_id, platform, store, developer, publisher, manual_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row.resource_id, row.platform, row.store,
                row.developer, row.publisher, row.manual_notes
            ],
        )?;
    }

    for row in &snapshot.progress {
        tx.execute(
            "INSERT INTO resource_progress (resource_id, progress, notes, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![row.resource_id, row.progress, row.notes, row.updated_at],
        )?;
    }

    for row in &snapshot.tags {
        tx.execute(
            "INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![row.id, row.name, row.created_at],
        )?;
    }

    for row in &snapshot.resource_tags {
        tx.execute(
            "INSERT INTO resource_tags (resource_id, tag_id) VALUES (?1, ?2)",
            params![row.resource_id, row.tag_id],
        )?;
    }

    tx.commit()
}
