# Phase 4 Design — Ecosystem Integrations

**Date:** 2026-04-18  
**Scope:** BookWalker, Kindle, Steam, DLSite, FANZA ecosystem connectors  
**Approach:** Sequential platform slices with trait extraction after first platform  
**Deferred:** Tags, device management, dedup (Phase 1 deferred), frontend platform builds, PostgreSQL — all stay deferred to Phase 5+

---

## 1. Credential Vault

Encrypted credential storage in SQLite for platform accounts.

### Domain

New trait in `domain`:

```rust
#[async_trait]
pub trait CredentialVault: Send + Sync {
    async fn store(&self, platform: &str, credential_type: &str, plaintext: &[u8]) -> Result<(), DomainError>;
    async fn retrieve(&self, platform: &str, credential_type: &str) -> Result<Vec<u8>, DomainError>;
    async fn delete(&self, platform: &str, credential_type: &str) -> Result<(), DomainError>;
    async fn list_platforms(&self) -> Result<Vec<String>, DomainError>;
}
```

Credential types: `cookie_jar`, `session_token`, `username_password`, `api_key`.

### Storage

New SQLite tables:

```sql
CREATE TABLE vault_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    salt BLOB NOT NULL,
    key_check BLOB NOT NULL,     -- encrypted known plaintext for password verification
    key_check_nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE credentials (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    credential_type TEXT NOT NULL,
    encrypted_blob BLOB NOT NULL,
    nonce BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(platform, credential_type)
);
```

### Encryption

- AES-256-GCM via `aes-gcm` crate
- Key derivation: Argon2id via `argon2` crate, 32-byte salt stored in `vault_config`
- Master password prompted once per session, derived key held in memory (`Arc<Mutex<Option<[u8; 32]>>>`)
- `key_check` field: encrypt a known plaintext ("vault-ok") to verify master password correctness on unlock

### First-Run Initialization

On first `POST /vault/unlock`, if `vault_config` has no row:
1. Generate random 32-byte salt
2. Derive key from provided master password + salt via Argon2id
3. Encrypt known plaintext "vault-ok" → store as `key_check` + `key_check_nonce`
4. Insert `vault_config` row
5. Return 200 (vault created and unlocked)

Subsequent unlocks: load salt → derive key → decrypt `key_check` → verify plaintext matches "vault-ok" → if yes, cache key in memory.

### API

```
POST /api/v1/vault/unlock         — accepts { master_password }, returns 200 or 401
POST /api/v1/vault/lock           — clears in-memory key
GET  /api/v1/vault/status         — returns { unlocked: bool, initialized: bool }
POST /api/v1/vault/credentials    — store credential (requires unlocked vault)
GET  /api/v1/vault/credentials    — list platforms with stored credentials
DELETE /api/v1/vault/credentials/:platform/:type — delete credential
```

---

## 2. Browser Automation Layer

Reuse existing `chromiumoxide` from Phase 2.

### BrowserSession Abstraction

New struct in `plugins` crate:

```rust
pub struct BrowserSession {
    // wraps chromiumoxide::Browser + Page
}

impl BrowserSession {
    pub async fn navigate(&self, url: &str) -> Result<(), DomainError>;
    pub async fn wait_for_selector(&self, css: &str, timeout_ms: u64) -> Result<(), DomainError>;
    pub async fn extract_text(&self, css: &str) -> Result<String, DomainError>;
    pub async fn extract_html(&self, css: &str) -> Result<String, DomainError>;
    pub async fn extract_all_text(&self, css: &str) -> Result<Vec<String>, DomainError>;
    pub async fn click(&self, css: &str) -> Result<(), DomainError>;
    pub async fn fill(&self, css: &str, value: &str) -> Result<(), DomainError>;
    pub async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError>;
    pub async fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<(), DomainError>;
    pub async fn screenshot(&self) -> Result<Vec<u8>, DomainError>; // debug aid
}
```

### Login Flow Pattern

Each platform connector follows:

1. Load cookies from vault (if stored)
2. Set cookies on browser session
3. Navigate to a "logged in" indicator page
4. If session valid → proceed
5. If expired → load stored username/password from vault → fill login form → submit → extract new cookies → store in vault

### Usage Model

- Browser automation used **only for login** on DLSite/FANZA/BookWalker/Kindle
- After login, extract cookies and use **direct HTTP** (`reqwest` with cookie jar) for data fetching
- Steam uses public Web API (no browser needed)
- `spawn_blocking` wraps all Chromium calls to avoid nested-Tokio panics

---

## 3. Sync Engine

### Domain Entities

```rust
pub struct SyncJob {
    pub id: Uuid,
    pub platform: String,
    pub status: SyncJobStatus, // Pending, Running, Completed, Failed
    pub started_at: Option<DateTime>,
    pub completed_at: Option<DateTime>,
    pub items_found: u32,
    pub items_created: u32,
    pub items_skipped: u32,    // title conflict
    pub items_failed: u32,
    pub error_message: Option<String>,
}

pub enum SyncJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
```

### Storage

```sql
CREATE TABLE sync_jobs (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TEXT,
    completed_at TEXT,
    items_found INTEGER NOT NULL DEFAULT 0,
    items_created INTEGER NOT NULL DEFAULT 0,
    items_skipped INTEGER NOT NULL DEFAULT 0,
    items_failed INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Repository Trait

```rust
#[async_trait]
pub trait SyncJobRepository: Send + Sync {
    async fn create(&self, job: NewSyncJob) -> Result<SyncJob, DomainError>;
    async fn update(&self, job: &SyncJob) -> Result<(), DomainError>;
    async fn get(&self, id: Uuid) -> Result<SyncJob, DomainError>;
    async fn list_by_platform(&self, platform: &str) -> Result<Vec<SyncJob>, DomainError>;
}
```

### Scheduler Integration

Extend `plugins.toml`:

```toml
[ecosystem.steam]
enabled = true
sync_interval_secs = 86400    # daily
steam_api_key = "..."          # or from env var
steam_id = "..."

[ecosystem.dlsite]
enabled = true
sync_interval_secs = 86400

[ecosystem.fanza]
enabled = false

[ecosystem.bookwalker]
enabled = true
sync_interval_secs = 86400

[ecosystem.kindle]
enabled = true
sync_interval_secs = 604800   # weekly
```

Scheduler ticks check each enabled platform's last sync time against interval.

### API

```
POST /api/v1/ecosystem/:platform/sync          — manual trigger, returns sync job ID
GET  /api/v1/ecosystem/:platform/syncs          — sync history
GET  /api/v1/ecosystem/:platform/syncs/:id      — single sync job status
GET  /api/v1/ecosystem/status                   — all platforms summary
```

---

## 4. Dedup Warning System

### Storage

```sql
CREATE TABLE dedup_warnings (
    id TEXT PRIMARY KEY,
    resource_id_a TEXT NOT NULL REFERENCES resources(id),
    resource_id_b TEXT NOT NULL REFERENCES resources(id),
    similarity_score REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, dismissed, merged
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    UNIQUE(resource_id_a, resource_id_b)
);
```

### Similarity Algorithm

- Jaro-Winkler on normalized titles (lowercased, stripped of punctuation and common prefixes like "The")
- Threshold: configurable in `plugins.toml`, default 0.85
- Run after each sync job completes: compare newly created resources against all existing
- Only generate warnings for pairs not already dismissed

### API

```
GET    /api/v1/dedup-warnings                  — list pending warnings
POST   /api/v1/dedup-warnings/:id/dismiss      — mark as dismissed
POST   /api/v1/dedup-warnings/:id/merge        — keep resource A, delete B, merge B's locations into A
```

### Merge Behavior

When merging A ← B:
1. Copy all locations from B to A (skip duplicates by device_id + path_or_url)
2. Merge notes: append B's notes to A's if both exist
3. Delete resource B (cascade deletes its meta and locations)
4. Mark warning as `merged`

---

## 5. Platform Connectors

### 5.1 Steam

**Auth:** Steam Web API key + Steam ID (provided by user in ecosystem settings). No browser login needed.

**Library Sync:**
- `GET https://api.steampowered.com/IPlayerService/GetOwnedGames/v1?key={key}&steamid={id}&include_appinfo=true`
- Returns: appid, name, playtime_forever (minutes), img_icon_url

**Metadata Fetch:**
- `GET https://store.steampowered.com/api/appdetails?appids={appid}`
- Returns: name, developers, publishers, categories, genres, platforms

**Resource Mapping:**
- Resource: `{ title: name, type: Game, notes: "Steam playtime: {hours}h" }`
- GameMeta: `{ platform: "PC", store: "Steam", developer: developers[0], publisher: publishers[0] }`
- Location: `{ device_id: <configured_device>, path_or_url: "steam://rungameid/{appid}", storage_type: "platform" }`

### 5.2 DLSite

**Auth:** Browser login → extract cookies → store in vault. Direct HTTP with cookies for data.

**Library Sync (reverse-engineered API):**
- Login page: `https://login.dlsite.com/login`
- Purchase history API: discover via network inspection during implementation. Expected pattern: paginated JSON endpoint returning work IDs + titles.
- Fallback: browser DOM scraping of purchase history page if no clean API found.

**Metadata:**
- Work detail page API or DOM: title, circle (developer), tags, file size, category
- Content type detection from DLSite category:
  - `game` → ResourceType::Game
  - `manga`/`cg`/`illustration` → ResourceType::Image
  - `video`/`anime` → ResourceType::Video
  - Default → ResourceType::Game (most DLSite content)

**Resource Mapping:**
- Resource: `{ title, type: <detected>, notes: "DLSite RJ{id}" }`
- Meta: type-specific (GameMeta/ImageMeta/VideoMeta) with available fields
- Location: `{ storage_type: "platform", path_or_url: "https://www.dlsite.com/maniax/work/=/product_id/RJ{id}" }`

### 5.3 FANZA

**Auth:** DMM account browser login → cookies → vault. Direct HTTP with cookies.

**Library Sync:** Same approach as DLSite — discover API via network inspection.

**Metadata:** Product page data — title, maker, genre tags, content type.

**Resource Mapping:** Same pattern as DLSite with content type detection from genre tags.

**Shared with DLSite:** After both work, extract common helpers for:
- Japanese storefront login flow
- Paginated purchase history traversal
- Content type detection from category/genre tags

### 5.4 BookWalker

**Auth:** Browser login (email/password) → cookies → vault. Direct HTTP for data.

**Library Sync:** Purchased books list API (discover via network inspection).

**Metadata:** Title, author, publisher, series, volume, format, page count.

**DRM Note:** BookWalker content uses heavy DRM. If the platform API does not expose sufficient metadata (author, page count, etc.), file-level extraction may be required. This could involve decrypting or parsing DRM-protected files, which is a last resort. Prefer API/web data first; fall back to file hacking only when metadata is otherwise unavailable.

**Resource Mapping:**
- Resource: `{ title, type: Ebook }`
- EbookMeta: `{ author, file_format: "bookwalker_digital", page_count }`
- Location: `{ storage_type: "platform", path_or_url: "https://bookwalker.jp/..." }`

### 5.5 Kindle

**Auth:** Two modes:
1. **Browser sync:** Amazon login (complex — CAPTCHA, 2FA, device verification). Cookies stored in vault.
2. **File import:** User exports library from Amazon's "Manage Your Content and Devices" as CSV.

**Library Sync (browser):** Hit Kindle library API with cookies. Paginate.

**Library Sync (file import):**
- Frontend parses CSV/JSON export
- Maps rows to add-resource requests
- Uses existing batch-import pattern

**Metadata:** Title, author, ASIN, format, file size.

**DRM Note:** Kindle content uses heavy DRM (AZW/KFX formats). If the Kindle API or export does not expose sufficient metadata, file-level hacking may be required to extract embedded metadata from DRM-protected files. This is a last-resort path — prefer API/web/export data first. If file hacking is needed, it will likely require a dedicated Rust or native tool for AZW/KFX parsing.

**Resource Mapping:**
- Resource: `{ title, type: Ebook }`
- EbookMeta: `{ author, file_format: "kindle" }`
- Location: `{ storage_type: "platform", path_or_url: "kindle://asin/{asin}" }`

---

## 6. EcosystemConnector Trait (Extracted After Steam)

After Steam connector works end-to-end, extract this trait:

```rust
pub struct SyncItem {
    pub external_id: String,
    pub title: String,
    pub resource_type: ResourceType,
    pub notes: Option<String>,
    pub metadata: PlatformMetadata,
    pub location: Option<NewLocation>,
}

pub enum PlatformMetadata {
    Game(NewGameMeta),
    Ebook(NewEbookMeta),
    Image(NewImageMeta),
    Video(NewVideoMeta),
}

#[async_trait]
pub trait EcosystemConnector: Send + Sync {
    fn platform(&self) -> &str;
    async fn authenticate(&self, vault: &dyn CredentialVault) -> Result<(), DomainError>;
    async fn is_authenticated(&self) -> bool;
    async fn sync_library(&self) -> Result<Vec<SyncItem>, DomainError>;
    async fn fetch_metadata(&self, external_id: &str) -> Result<PlatformMetadata, DomainError>;
}
```

Plugin registry in `plugins.toml` maps platform names to connector implementations. Config-driven enable/disable per platform.

---

## 7. Frontend UX

### New Screens

1. **Vault Unlock Screen** — master password input. Shown on first ecosystem access. Session-scoped unlock.
2. **Ecosystem Settings Screen** — list of platforms, each with:
   - Enable/disable toggle
   - Credential config (username/password or API key fields, depending on platform)
   - Sync interval config
   - "Test Connection" button
   - Last sync status badge
3. **Sync Dashboard Screen** — per-platform sync status, "Sync Now" button, expandable sync history with items found/created/skipped/failed counts
4. **Dedup Review Screen** — list of warnings with:
   - Side-by-side title comparison
   - Similarity score
   - Dismiss / Merge actions
   - Links to both resources

### New BLoCs

- `VaultBloc` — events: Unlock, Lock, CheckStatus. States: Locked, Unlocked, Error.
- `EcosystemBloc` — events: LoadPlatforms, UpdateConfig, TriggerSync, LoadSyncHistory. States: Loading, PlatformsLoaded, SyncInProgress, SyncHistoryLoaded, Error.
- `DedupBloc` — events: LoadWarnings, Dismiss, Merge. States: Loading, WarningsLoaded, OperationSuccess, Error.

### Navigation

- New "Ecosystem" section in app navigation (tab or drawer item)
- Dedup warnings surface as notifications via existing notification infrastructure
- Vault unlock triggered automatically when accessing ecosystem features while locked

---

## 8. Execution Order

| Track | Description | Dependencies |
|-------|-------------|-------------|
| P4-A | Shared foundations: vault, browser session, sync engine, dedup system | None |
| P4-B | Steam connector (end-to-end) | P4-A |
| P4-C | Trait extraction from Steam | P4-B |
| P4-D | DLSite connector | P4-C |
| P4-E | FANZA connector + shared DLSite/FANZA helpers | P4-C, P4-D |
| P4-F | BookWalker connector | P4-C |
| P4-G | Kindle connector (browser + file import) | P4-C |
| P4-H | Frontend ecosystem UX (all screens + BLoCs) | P4-A (start), P4-B (full test) |
| P4-I | Verification & CONTEXT.md update | All |

### Notes

- P4-D through P4-G can run in parallel after P4-C (trait is stable)
- P4-H frontend work can start alongside P4-B (shared foundations provide the APIs)
- Each platform connector follows TDD: Red → Green → Refactor
- API discovery for DLSite/FANZA/BookWalker/Kindle is a manual research step before coding

---

## 9. New Dependencies

### Backend (Cargo)

| Crate | Purpose |
|-------|---------|
| `aes-gcm` | AES-256-GCM encryption for credential vault |
| `argon2` | Argon2id key derivation |
| `reqwest` | Direct HTTP client for platform APIs (with cookie support) |
| `strsim` | Jaro-Winkler string similarity for dedup |

### Frontend (pubspec)

| Package | Purpose |
|---------|---------|
| `csv` | Kindle CSV import parsing |

---

## 10. Migration Summary

New migrations (numbered 0013+):

| Migration | Table |
|-----------|-------|
| 0013 | `vault_config` |
| 0014 | `credentials` |
| 0015 | `sync_jobs` |
| 0016 | `dedup_warnings` |
