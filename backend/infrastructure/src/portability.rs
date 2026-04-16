use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRow {
    pub id: String,
    pub title: String,
    pub notes: Option<String>,
    pub resource_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EbookMetaRow {
    pub resource_id: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub file_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebReaderMetaRow {
    pub resource_id: String,
    pub url: String,
    pub site_name: Option<String>,
    pub last_checked_chapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLocationRow {
    pub id: String,
    pub resource_id: String,
    pub device_id: String,
    pub path_or_url: String,
    pub storage_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalResourceSnapshot {
    pub resources: Vec<ResourceRow>,
    pub ebook_metas: Vec<EbookMetaRow>,
    pub web_reader_metas: Vec<WebReaderMetaRow>,
    pub resource_locations: Vec<ResourceLocationRow>,
}

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

    Ok(CanonicalResourceSnapshot {
        resources,
        ebook_metas,
        web_reader_metas,
        resource_locations,
    })
}

pub fn import_canonical_snapshot_sqlite(
    conn: &mut Connection,
    snapshot: &CanonicalResourceSnapshot,
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

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

    tx.commit()
}
