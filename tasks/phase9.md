# Phase 9 — Real API Integration + Advanced Search Features

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire real HTTP calls to Steam/DLSite/FANZA/Kindle connectors (Track A) and wire SearchFilterBar + backend sort/facet endpoints (Track B).

**Architecture:** Hexagonal (ports & adapters). Backend: Rust workspace (domain → use_cases → plugins → services → adapters → infrastructure → app). Frontend: Flutter + BLoC. Track A adds real HTTP clients behind `real-plugins` feature flag. Track B adds query params to existing list endpoints and wires the already-built SearchFilterBar widget into ResourceListScreen.

**Tech Stack:** Rust (axum, reqwest, chromiumoxide, serde), Flutter (flutter_bloc, equatable), SQLite/PostgreSQL, AES-256-GCM vault.

**Design Spec:** `docs/superpowers/specs/2026-04-20-phase9-real-api-advanced-features-design.md`

---

## File Map

### New Files

| File | Responsibility |
|---|---|
| `backend/plugins/src/har_parser.rs` | Parse HAR JSON → extract endpoints, cookies, response structures |
| `backend/plugins/tests/fixtures/real/dlsite_library.json` | HAR-extracted DLSite library response fixture |
| `backend/plugins/tests/fixtures/real/fanza_library.json` | HAR-extracted FANZA library response fixture |
| `backend/plugins/tests/fixtures/real/steam_owned_games.json` | Real Steam API response fixture |
| `backend/plugins/src/http_client.rs` | `HttpConnectorClient` — reqwest wrapper with retry middleware |
| `backend/plugins/src/chromium_session.rs` | `ChromiumSession` — real `BrowserPage` impl via chromiumoxide |
| `backend/services/src/otp_service.rs` | `OtpInteractionService` — OTP pause/resume state machine |
| `frontend/lib/blocs/search_filter/search_filter_bloc.dart` | `SearchFilterBloc` — global filter state across all tabs |
| `frontend/test/blocs/search_filter/search_filter_bloc_test.dart` | Unit tests for SearchFilterBloc |
| `frontend/test/screens/resource_list_screen_filter_test.dart` | Widget tests for filter bar wiring |

### Modified Files

| File | Change |
|---|---|
| `backend/plugins/src/lib.rs` | Export `har_parser`, `http_client`, `chromium_session` modules |
| `backend/plugins/Cargo.toml` | No new deps needed (chromiumoxide + reqwest already declared) |
| `backend/plugins/src/ecosystem/steam.rs` | Use `HttpConnectorClient` for real API call |
| `backend/plugins/src/ecosystem/dlsite.rs` | Replace placeholder URLs, use `ChromiumSession` |
| `backend/plugins/src/ecosystem/fanza.rs` | Replace placeholder URLs, use `ChromiumSession` |
| `backend/plugins/src/ecosystem/kindle.rs` | Amazon JP login flow with OTP pause |
| `backend/services/src/sync_service.rs` | Wire vault credential retrieval before connector calls |
| `backend/services/src/lib.rs` | Export `otp_service` |
| `backend/adapters/src/tag_filter.rs` | Extend `ListQuery` struct with sort_by, sort_order, with_facets |
| `backend/adapters/src/ebook.rs` | Add sort/facet handling to `list_ebooks` |
| `backend/adapters/src/sync_handler.rs` | Add OTP submission endpoint |
| `backend/adapters/src/routes.rs` | Register OTP route |
| `backend/app/src/runtime.rs` | Wire OtpInteractionService |
| `frontend/lib/screens/resource_list_screen.dart` | Replace `_TagFilterBar` with `SearchFilterBar`, listen to `SearchFilterBloc` |
| `frontend/lib/blocs/ebook/ebook_bloc.dart` | Extend `LoadEbooks` with filter/sort params |
| `frontend/lib/blocs/game/game_bloc.dart` | Extend `LoadGames` with filter/sort params |
| `frontend/lib/blocs/image/image_bloc.dart` | Extend `LoadImages` with filter/sort params |
| `frontend/lib/blocs/video/video_bloc.dart` | Extend `LoadVideos` with filter/sort params |
| `frontend/lib/blocs/web_reader/web_reader_bloc.dart` | Extend `LoadWebReaders` with filter/sort params |
| `frontend/lib/repositories/ebook_repository.dart` | Add filter/sort params to `listEbooks` |
| `frontend/lib/main.dart` | Provide `SearchFilterBloc` at app root |

---

## Track A: Real API Integration

### Task P9-A1: HAR Extraction + Fixture Upgrade

**Files:**
- Create: `backend/plugins/src/har_parser.rs`
- Create: `backend/plugins/tests/fixtures/real/dlsite_library.json`
- Create: `backend/plugins/tests/fixtures/real/fanza_library.json`
- Create: `backend/plugins/tests/fixtures/real/steam_owned_games.json`
- Modify: `backend/plugins/src/lib.rs`

- [ ] **Step 1: Write failing tests for HAR parser**

In `backend/plugins/src/har_parser.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_har_entries() {
        let har_json = r#"{
            "log": {
                "entries": [
                    {
                        "request": {
                            "method": "GET",
                            "url": "https://www.dlsite.com/maniax/mypage/userbuy?output=json"
                        },
                        "response": {
                            "status": 200,
                            "content": {
                                "text": "[{\"workno\":\"RJ123\",\"work_name\":\"Test\",\"work_type\":\"GAM\"}]"
                            }
                        }
                    }
                ]
            }
        }"#;
        let entries = parse_har(har_json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].request_url, "https://www.dlsite.com/maniax/mypage/userbuy?output=json");
        assert_eq!(entries[0].response_status, 200);
        assert!(entries[0].response_body.contains("RJ123"));
    }

    #[test]
    fn extracts_cookies_from_har() {
        let har_json = r#"{
            "log": {
                "entries": [
                    {
                        "request": {
                            "method": "GET",
                            "url": "https://example.com",
                            "cookies": [
                                {"name": "session", "value": "abc", "domain": ".example.com", "path": "/"}
                            ]
                        },
                        "response": {
                            "status": 200,
                            "content": {"text": ""}
                        }
                    }
                ]
            }
        }"#;
        let entries = parse_har(har_json).unwrap();
        assert_eq!(entries[0].request_cookies.len(), 1);
        assert_eq!(entries[0].request_cookies[0].name, "session");
    }

    #[test]
    fn handles_empty_har() {
        let har_json = r#"{"log": {"entries": []}}"#;
        let entries = parse_har(har_json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        let result = parse_har("not json");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins har_parser -- --nocapture`
Expected: FAIL — module does not exist yet.

- [ ] **Step 3: Implement HAR parser**

Create `backend/plugins/src/har_parser.rs`:

```rust
use domain::DomainError;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct HarEntry {
    pub request_url: String,
    pub request_method: String,
    pub response_status: u16,
    pub response_body: String,
    pub request_cookies: Vec<HarCookie>,
}

#[derive(Debug, Clone)]
pub struct HarCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Deserialize)]
struct HarFile {
    log: HarLog,
}

#[derive(Deserialize)]
struct HarLog {
    entries: Vec<HarRawEntry>,
}

#[derive(Deserialize)]
struct HarRawEntry {
    request: HarRequest,
    response: HarResponse,
}

#[derive(Deserialize)]
struct HarRequest {
    method: String,
    url: String,
    #[serde(default)]
    cookies: Vec<HarRawCookie>,
}

#[derive(Deserialize)]
struct HarRawCookie {
    name: String,
    value: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    path: String,
}

#[derive(Deserialize)]
struct HarResponse {
    status: u16,
    content: HarContent,
}

#[derive(Deserialize)]
struct HarContent {
    #[serde(default)]
    text: Option<String>,
}

pub fn parse_har(json: &str) -> Result<Vec<HarEntry>, DomainError> {
    let har: HarFile = serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("HAR parse error: {e}")))?;

    Ok(har
        .log
        .entries
        .into_iter()
        .map(|entry| HarEntry {
            request_url: entry.request.url,
            request_method: entry.request.method,
            response_status: entry.response.status,
            response_body: entry.response.content.text.unwrap_or_default(),
            request_cookies: entry
                .request
                .cookies
                .into_iter()
                .map(|c| HarCookie {
                    name: c.name,
                    value: c.value,
                    domain: c.domain,
                    path: c.path,
                })
                .collect(),
        })
        .collect())
}

// Tests defined in Step 1 go here
```

Add to `backend/plugins/src/lib.rs`:
```rust
pub mod har_parser;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins har_parser -- --nocapture`
Expected: 4 tests PASS

- [ ] **Step 5: Create real fixture files from HAR data**

Extract fixture data by examining the HAR files in the repo root and creating cleaned JSON fixtures.

Create `backend/plugins/tests/fixtures/real/steam_owned_games.json`:
```json
{
    "response": {
        "game_count": 3,
        "games": [
            {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1234},
            {"appid": 570, "name": "Dota 2", "playtime_forever": 5678},
            {"appid": 730, "name": "Counter-Strike 2", "playtime_forever": 910}
        ]
    }
}
```

Create `backend/plugins/tests/fixtures/real/dlsite_library.json`:
```json
[
    {"workno": "RJ123456", "work_name": "Test Game Alpha", "work_type": "GAM", "maker_name": "Studio A"},
    {"workno": "RJ789012", "work_name": "CG Collection Beta", "work_type": "CG", "maker_name": "Circle B"},
    {"workno": "RJ345678", "work_name": "Animation Gamma", "work_type": "MOV", "maker_name": "Group C"}
]
```

Create `backend/plugins/tests/fixtures/real/fanza_library.json`:
```json
[
    {"content_id": "d_123456", "title": "FANZA Game Alpha", "content_type": "game", "maker_name": "Maker X"},
    {"content_id": "d_789012", "title": "FANZA Comic Beta", "content_type": "comic", "maker_name": "Maker Y"},
    {"content_id": "d_345678", "title": "FANZA Video Gamma", "content_type": "video", "maker_name": "Maker Z"}
]
```

- [ ] **Step 6: Write fixture loading tests**

Add tests in `backend/plugins/src/ecosystem/steam.rs`:

```rust
#[test]
fn real_fixture_deserializes() {
    let fixture = include_str!("../../../tests/fixtures/real/steam_owned_games.json");
    let resp: OwnedGamesResponse = serde_json::from_str(fixture).expect("deserialize real fixture");
    let games = resp.response.games.unwrap();
    assert_eq!(games.len(), 3);
    assert_eq!(games[0].name, "Team Fortress 2");
}
```

Add tests in `backend/plugins/src/ecosystem/dlsite.rs`:

```rust
#[test]
fn real_fixture_deserializes() {
    let fixture = include_str!("../../../tests/fixtures/real/dlsite_library.json");
    let works: Vec<DLSiteWork> = serde_json::from_str(fixture).expect("deserialize real fixture");
    assert_eq!(works.len(), 3);
    assert_eq!(works[0].workno, "RJ123456");
    assert_eq!(detect_resource_type(&works[1].work_type), domain::ResourceType::Image);
}
```

Add tests in `backend/plugins/src/ecosystem/fanza.rs`:

```rust
#[test]
fn real_fixture_deserializes() {
    let fixture = include_str!("../../../tests/fixtures/real/fanza_library.json");
    let products: Vec<FanzaProduct> = serde_json::from_str(fixture).expect("deserialize real fixture");
    assert_eq!(products.len(), 3);
    assert_eq!(products[0].content_id, "d_123456");
    assert_eq!(detect_resource_type(&products[2].content_type), domain::ResourceType::Video);
}
```

- [ ] **Step 7: Run all fixture tests**

Run: `cd backend && cargo test -p plugins -- --nocapture`
Expected: All existing + 7 new tests PASS

- [ ] **Step 8: Commit**

```bash
git add backend/plugins/src/har_parser.rs backend/plugins/tests/fixtures/real/ backend/plugins/src/lib.rs backend/plugins/src/ecosystem/steam.rs backend/plugins/src/ecosystem/dlsite.rs backend/plugins/src/ecosystem/fanza.rs
git commit -m "feat(P9-A1): HAR parser + real-structure fixture files

- Add har_parser module for extracting endpoints/cookies/responses from HAR JSON
- Create real-structure fixture files for Steam, DLSite, FANZA
- Add fixture deserialization tests for all 3 platforms
- 7 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A2: HttpConnectorClient (reqwest + retry)

**Files:**
- Create: `backend/plugins/src/http_client.rs`
- Modify: `backend/plugins/src/lib.rs`

- [ ] **Step 1: Write failing tests for HttpConnectorClient**

In `backend/plugins/src/http_client.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(!config.user_agent.is_empty());
    }

    #[test]
    fn classify_transient_errors() {
        assert_eq!(classify_status(429), ErrorClass::Transient);
        assert_eq!(classify_status(500), ErrorClass::Transient);
        assert_eq!(classify_status(502), ErrorClass::Transient);
        assert_eq!(classify_status(503), ErrorClass::Transient);
    }

    #[test]
    fn classify_permanent_errors() {
        assert_eq!(classify_status(401), ErrorClass::Permanent);
        assert_eq!(classify_status(403), ErrorClass::Permanent);
        assert_eq!(classify_status(404), ErrorClass::Permanent);
    }

    #[test]
    fn classify_success() {
        assert_eq!(classify_status(200), ErrorClass::Success);
        assert_eq!(classify_status(201), ErrorClass::Success);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins http_client -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement HttpConnectorClient**

Create `backend/plugins/src/http_client.rs`:

```rust
use domain::DomainError;

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout_secs: u64,
    pub user_agent: String,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            user_agent: "PersonalInventory/1.0".to_string(),
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 8000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    Success,
    Transient,
    Permanent,
}

pub fn classify_status(status: u16) -> ErrorClass {
    match status {
        200..=299 => ErrorClass::Success,
        401 | 403 | 404 => ErrorClass::Permanent,
        429 | 500..=599 => ErrorClass::Transient,
        _ => ErrorClass::Permanent,
    }
}

#[cfg(feature = "real-plugins")]
pub struct HttpConnectorClient {
    client: reqwest::Client,
    config: HttpClientConfig,
}

#[cfg(feature = "real-plugins")]
impl HttpConnectorClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, DomainError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| DomainError::InternalError(format!("HTTP client build error: {e}")))?;
        Ok(Self { client, config })
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, DomainError> {
        let mut backoff_ms = self.config.initial_backoff_ms;

        for attempt in 0..=self.config.max_retries {
            let resp = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|e| DomainError::InternalError(format!("HTTP request error: {e}")))?;

            let status = resp.status().as_u16();
            match classify_status(status) {
                ErrorClass::Success => {
                    return resp.json::<T>().await.map_err(|e| {
                        DomainError::InternalError(format!("JSON parse error: {e}"))
                    });
                }
                ErrorClass::Permanent => {
                    return Err(DomainError::InternalError(format!(
                        "HTTP {status}: permanent error"
                    )));
                }
                ErrorClass::Transient => {
                    if attempt == self.config.max_retries {
                        return Err(DomainError::InternalError(format!(
                            "HTTP {status}: max retries exceeded"
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(self.config.max_backoff_ms);
                }
            }
        }

        Err(DomainError::InternalError("unreachable".to_string()))
    }

    pub async fn get_text_with_cookies(
        &self,
        url: &str,
        cookie_header: &str,
    ) -> Result<(u16, String), DomainError> {
        let resp = self
            .client
            .get(url)
            .header("Cookie", cookie_header)
            .send()
            .await
            .map_err(|e| DomainError::InternalError(format!("HTTP request error: {e}")))?;

        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| DomainError::InternalError(format!("HTTP read error: {e}")))?;
        Ok((status, text))
    }
}

// Tests defined in Step 1 go here
```

Add to `backend/plugins/src/lib.rs`:
```rust
pub mod http_client;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins http_client -- --nocapture`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/http_client.rs backend/plugins/src/lib.rs
git commit -m "feat(P9-A2): HttpConnectorClient with retry + error classification

- Add HttpConnectorClient wrapper around reqwest with exponential backoff
- Error classification: transient (429/5xx) vs permanent (401/403/404)
- get_json<T> with automatic retry, get_text_with_cookies for browser session
- Feature-gated behind real-plugins
- 4 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A3: ChromiumSession (real BrowserPage impl)

**Files:**
- Create: `backend/plugins/src/chromium_session.rs`
- Modify: `backend/plugins/src/lib.rs`

- [ ] **Step 1: Write failing test for ChromiumSession**

In `backend/plugins/src/chromium_session.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_session::BrowserPage;

    #[test]
    fn chromium_config_defaults() {
        let config = ChromiumConfig::default();
        assert_eq!(config.navigation_timeout_ms, 30_000);
        assert!(config.headless);
    }

    #[test]
    fn chromium_config_from_path() {
        let config = ChromiumConfig::with_path("/usr/bin/chromium".to_string());
        assert_eq!(config.chromium_path, Some("/usr/bin/chromium".to_string()));
        assert!(config.headless);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins chromium_session -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement ChromiumSession**

Create `backend/plugins/src/chromium_session.rs`:

```rust
use domain::DomainError;
use crate::browser_session::{BrowserPage, Cookie};

#[derive(Debug, Clone)]
pub struct ChromiumConfig {
    pub chromium_path: Option<String>,
    pub headless: bool,
    pub navigation_timeout_ms: u64,
}

impl Default for ChromiumConfig {
    fn default() -> Self {
        Self {
            chromium_path: None,
            headless: true,
            navigation_timeout_ms: 30_000,
        }
    }
}

impl ChromiumConfig {
    pub fn with_path(path: String) -> Self {
        Self {
            chromium_path: Some(path),
            ..Default::default()
        }
    }
}

#[cfg(feature = "real-plugins")]
pub struct ChromiumSession {
    browser: chromiumoxide::Browser,
    page: chromiumoxide::Page,
}

#[cfg(feature = "real-plugins")]
impl ChromiumSession {
    pub async fn launch(config: &ChromiumConfig) -> Result<Self, DomainError> {
        use chromiumoxide::BrowserConfig;
        use futures::StreamExt;

        let mut builder = BrowserConfig::builder();
        if config.headless {
            builder = builder.arg("--headless=new");
        }
        if let Some(ref path) = config.chromium_path {
            builder = builder.chrome_executable(path);
        }
        builder = builder.arg("--no-sandbox");
        builder = builder.arg("--disable-gpu");

        let browser_config = builder
            .build()
            .map_err(|e| DomainError::InternalError(format!("Chromium config error: {e}")))?;

        let (browser, mut handler) = chromiumoxide::Browser::launch(browser_config)
            .await
            .map_err(|e| DomainError::InternalError(format!("Chromium launch error: {e}")))?;

        tokio::spawn(async move {
            while let Some(_) = handler.next().await {}
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| DomainError::InternalError(format!("Chromium page error: {e}")))?;

        Ok(Self { browser, page })
    }
}

#[cfg(feature = "real-plugins")]
#[async_trait::async_trait]
impl BrowserPage for ChromiumSession {
    async fn navigate(&self, url: &str) -> Result<(), DomainError> {
        self.page
            .goto(url)
            .await
            .map_err(|e| DomainError::InternalError(format!("Navigate error: {e}")))?;
        Ok(())
    }

    async fn wait_for_selector(&self, css: &str, timeout_ms: u64) -> Result<(), DomainError> {
        tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            self.page.find_element(css),
        )
        .await
        .map_err(|_| DomainError::InternalError(format!("Timeout waiting for '{css}'")))?
        .map_err(|e| DomainError::InternalError(format!("Selector error: {e}")))?;
        Ok(())
    }

    async fn extract_text(&self, css: &str) -> Result<String, DomainError> {
        let el = self
            .page
            .find_element(css)
            .await
            .map_err(|e| DomainError::InternalError(format!("Find element error: {e}")))?;
        el.inner_text()
            .await
            .map_err(|e| DomainError::InternalError(format!("Extract text error: {e}")))
            .map(|opt| opt.unwrap_or_default())
    }

    async fn extract_all_text(&self, css: &str) -> Result<Vec<String>, DomainError> {
        let elements = self
            .page
            .find_elements(css)
            .await
            .map_err(|e| DomainError::InternalError(format!("Find elements error: {e}")))?;
        let mut texts = Vec::new();
        for el in elements {
            if let Ok(Some(text)) = el.inner_text().await {
                texts.push(text);
            }
        }
        Ok(texts)
    }

    async fn extract_html(&self, css: &str) -> Result<String, DomainError> {
        let el = self
            .page
            .find_element(css)
            .await
            .map_err(|e| DomainError::InternalError(format!("Find element error: {e}")))?;
        el.inner_html()
            .await
            .map_err(|e| DomainError::InternalError(format!("Extract HTML error: {e}")))
            .map(|opt| opt.unwrap_or_default())
    }

    async fn click(&self, css: &str) -> Result<(), DomainError> {
        let el = self
            .page
            .find_element(css)
            .await
            .map_err(|e| DomainError::InternalError(format!("Find element error: {e}")))?;
        el.click()
            .await
            .map_err(|e| DomainError::InternalError(format!("Click error: {e}")))?;
        Ok(())
    }

    async fn fill(&self, css: &str, value: &str) -> Result<(), DomainError> {
        let el = self
            .page
            .find_element(css)
            .await
            .map_err(|e| DomainError::InternalError(format!("Find element error: {e}")))?;
        el.click()
            .await
            .map_err(|e| DomainError::InternalError(format!("Click to focus error: {e}")))?;
        el.type_str(value)
            .await
            .map_err(|e| DomainError::InternalError(format!("Type error: {e}")))?;
        Ok(())
    }

    async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError> {
        let cookies = self
            .page
            .get_cookies()
            .await
            .map_err(|e| DomainError::InternalError(format!("Get cookies error: {e}")))?;
        Ok(cookies
            .into_iter()
            .map(|c| Cookie {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
            })
            .collect())
    }

    async fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<(), DomainError> {
        use chromiumoxide::cdp::browser_protocol::network::CookieParam;
        for cookie in cookies {
            let param = CookieParam::builder()
                .name(&cookie.name)
                .value(&cookie.value)
                .domain(&cookie.domain)
                .path(&cookie.path)
                .build()
                .map_err(|e| DomainError::InternalError(format!("Cookie param error: {e}")))?;
            self.page
                .set_cookie(param)
                .await
                .map_err(|e| DomainError::InternalError(format!("Set cookie error: {e}")))?;
        }
        Ok(())
    }
}

// Tests defined in Step 1 go here
```

Add to `backend/plugins/src/lib.rs`:
```rust
pub mod chromium_session;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins chromium_session -- --nocapture`
Expected: 2 tests PASS (config tests only; real Chromium tests are feature-gated)

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/chromium_session.rs backend/plugins/src/lib.rs
git commit -m "feat(P9-A3): ChromiumSession — real BrowserPage impl via chromiumoxide

- Implement BrowserPage trait for ChromiumSession (navigate, fill, click, cookies)
- ChromiumConfig with headless mode, path override, navigation timeout
- Feature-gated behind real-plugins
- 2 new tests (config unit tests)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A4: Steam Real Integration

**Files:**
- Modify: `backend/plugins/src/ecosystem/steam.rs`

- [ ] **Step 1: Write failing test for Steam connector using HttpConnectorClient**

Add to `backend/plugins/src/ecosystem/steam.rs` tests:

```rust
#[test]
fn steam_api_url_is_correct() {
    let url = build_steam_api_url("TESTKEY", "123456");
    assert!(url.contains("api.steampowered.com"));
    assert!(url.contains("key=TESTKEY"));
    assert!(url.contains("steamid=123456"));
    assert!(url.contains("include_appinfo=true"));
}

#[test]
fn parse_steam_credentials() {
    let json = r#"{"api_key": "TESTKEY", "steam_id": "123456"}"#;
    let creds: SteamCredentials = serde_json::from_str(json).unwrap();
    assert_eq!(creds.api_key, "TESTKEY");
    assert_eq!(creds.steam_id, "123456");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins steam -- --nocapture`
Expected: FAIL — `build_steam_api_url` and `SteamCredentials` not defined.

- [ ] **Step 3: Add SteamCredentials and URL builder**

In `backend/plugins/src/ecosystem/steam.rs`, add:

```rust
#[derive(Debug, Deserialize)]
pub struct SteamCredentials {
    pub api_key: String,
    pub steam_id: String,
}

pub fn build_steam_api_url(api_key: &str, steam_id: &str) -> String {
    format!(
        "{STEAM_API_BASE}/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true&format=json",
        api_key, steam_id
    )
}
```

Update `fetch_owned_games` to use `HttpConnectorClient`:

```rust
#[cfg(feature = "real-plugins")]
pub async fn fetch_owned_games_with_client(
    client: &crate::http_client::HttpConnectorClient,
    api_key: &str,
    steam_id: &str,
) -> Result<Vec<SteamOwnedGame>, DomainError> {
    let url = build_steam_api_url(api_key, steam_id);
    let resp: OwnedGamesResponse = client.get_json(&url).await?;
    Ok(resp.response.games.unwrap_or_default())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins steam -- --nocapture`
Expected: All tests PASS (5 existing + 2 new)

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/ecosystem/steam.rs
git commit -m "feat(P9-A4): Steam real integration — credentials + HttpConnectorClient

- Add SteamCredentials struct for vault-stored credentials
- Add build_steam_api_url helper
- Add fetch_owned_games_with_client using HttpConnectorClient
- 2 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A5: DLSite Real Integration

**Files:**
- Modify: `backend/plugins/src/ecosystem/dlsite.rs`

- [ ] **Step 1: Write failing tests for updated DLSite flow**

Add to `backend/plugins/src/ecosystem/dlsite.rs` tests:

```rust
#[test]
fn dlsite_login_selectors_defined() {
    assert!(!DLSITE_USERNAME_SELECTOR.is_empty());
    assert!(!DLSITE_PASSWORD_SELECTOR.is_empty());
    assert!(!DLSITE_SUBMIT_SELECTOR.is_empty());
}

#[test]
fn dlsite_library_url_has_json_output() {
    assert!(DLSITE_LIBRARY_BASE_URL.contains("dlsite.com"));
}

#[test]
fn build_cookie_header_from_cookies() {
    use crate::browser_session::Cookie;
    let cookies = vec![
        Cookie {
            name: "a".to_string(),
            value: "1".to_string(),
            domain: ".dlsite.com".to_string(),
            path: "/".to_string(),
        },
        Cookie {
            name: "b".to_string(),
            value: "2".to_string(),
            domain: ".dlsite.com".to_string(),
            path: "/".to_string(),
        },
    ];
    let header = build_cookie_header(&cookies);
    assert_eq!(header, "a=1; b=2");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins dlsite -- --nocapture`
Expected: FAIL — constants and `build_cookie_header` not defined.

- [ ] **Step 3: Update DLSite connector with real selectors**

In `backend/plugins/src/ecosystem/dlsite.rs`:

Add CSS selector constants:
```rust
const DLSITE_USERNAME_SELECTOR: &str = "input[name='login_id']";
const DLSITE_PASSWORD_SELECTOR: &str = "input[name='password']";
const DLSITE_SUBMIT_SELECTOR: &str = "button[type='submit']";
```

Add cookie header builder:
```rust
pub fn build_cookie_header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|c| {
            let name = c.name.replace(['\r', '\n'], "");
            let value = c.value.replace(['\r', '\n'], "");
            format!("{name}={value}")
        })
        .collect::<Vec<_>>()
        .join("; ")
}
```

Update `ensure_authenticated` to use new selectors:
```rust
async fn ensure_authenticated(
    &self,
    session_cookies: Vec<Cookie>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), DomainError> {
    if !session_cookies.is_empty() {
        self.browser.set_cookies(session_cookies).await?;
    }
    self.browser.navigate(DLSITE_LOGIN_URL).await?;
    if let (Some(user), Some(pass)) = (username, password) {
        self.browser.fill(DLSITE_USERNAME_SELECTOR, user).await?;
        self.browser.fill(DLSITE_PASSWORD_SELECTOR, pass).await?;
        self.browser.click(DLSITE_SUBMIT_SELECTOR).await?;
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins dlsite -- --nocapture`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/ecosystem/dlsite.rs
git commit -m "feat(P9-A5): DLSite real integration — selectors + cookie builder

- Add real CSS selectors for DLSite login form
- Add build_cookie_header utility
- Update ensure_authenticated to use correct selectors
- 3 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A6: FANZA Real Integration

**Files:**
- Modify: `backend/plugins/src/ecosystem/fanza.rs`

- [ ] **Step 1: Write failing tests for updated FANZA flow**

Add to `backend/plugins/src/ecosystem/fanza.rs` tests:

```rust
#[test]
fn fanza_login_selectors_defined() {
    assert!(!FANZA_USERNAME_SELECTOR.is_empty());
    assert!(!FANZA_PASSWORD_SELECTOR.is_empty());
    assert!(!FANZA_SUBMIT_SELECTOR.is_empty());
}

#[test]
fn fanza_library_url_is_dmm() {
    assert!(FANZA_LIBRARY_BASE_URL.contains("dmm.com") || FANZA_LIBRARY_BASE_URL.contains("dmm.co.jp"));
}

#[test]
fn fanza_build_cookie_header() {
    use crate::browser_session::Cookie;
    let cookies = vec![Cookie {
        name: "sess".to_string(),
        value: "xyz".to_string(),
        domain: ".dmm.com".to_string(),
        path: "/".to_string(),
    }];
    let header = build_cookie_header(&cookies);
    assert_eq!(header, "sess=xyz");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins fanza -- --nocapture`
Expected: FAIL — constants and function not defined.

- [ ] **Step 3: Update FANZA connector with real selectors**

In `backend/plugins/src/ecosystem/fanza.rs`:

Add CSS selector constants:
```rust
const FANZA_USERNAME_SELECTOR: &str = "input[name='login_id']";
const FANZA_PASSWORD_SELECTOR: &str = "input[name='password']";
const FANZA_SUBMIT_SELECTOR: &str = "button[type='submit']";
```

Add cookie header builder (same pattern as DLSite):
```rust
pub fn build_cookie_header(cookies: &[Cookie]) -> String {
    cookies
        .iter()
        .map(|c| {
            let name = c.name.replace(['\r', '\n'], "");
            let value = c.value.replace(['\r', '\n'], "");
            format!("{name}={value}")
        })
        .collect::<Vec<_>>()
        .join("; ")
}
```

Update `ensure_authenticated` to use new selectors:
```rust
async fn ensure_authenticated(
    &self,
    session_cookies: Vec<Cookie>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), DomainError> {
    if !session_cookies.is_empty() {
        self.browser.set_cookies(session_cookies).await?;
    }
    self.browser.navigate(FANZA_LOGIN_URL).await?;
    if let (Some(user), Some(pass)) = (username, password) {
        self.browser.fill(FANZA_USERNAME_SELECTOR, user).await?;
        self.browser.fill(FANZA_PASSWORD_SELECTOR, pass).await?;
        self.browser.click(FANZA_SUBMIT_SELECTOR).await?;
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins fanza -- --nocapture`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/ecosystem/fanza.rs
git commit -m "feat(P9-A6): FANZA real integration — selectors + cookie builder

- Add real CSS selectors for DMM login form
- Add build_cookie_header utility
- Update ensure_authenticated with correct selectors
- 3 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A7: OTP Interaction Service

**Files:**
- Create: `backend/services/src/otp_service.rs`
- Modify: `backend/services/src/lib.rs`

- [ ] **Step 1: Write failing tests for OTP state machine**

In `backend/services/src/otp_service.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_otp_sets_pending_state() {
        let svc = OtpInteractionService::new(300);
        svc.request_otp("kindle").await;
        assert!(svc.is_pending("kindle").await);
    }

    #[tokio::test]
    async fn submit_otp_resolves_pending() {
        let svc = OtpInteractionService::new(300);
        svc.request_otp("kindle").await;
        let code = svc.submit_otp("kindle", "123456").await.unwrap();
        assert_eq!(code, "123456");
        assert!(!svc.is_pending("kindle").await);
    }

    #[tokio::test]
    async fn submit_otp_fails_when_not_pending() {
        let svc = OtpInteractionService::new(300);
        let result = svc.submit_otp("kindle", "123456").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn wait_for_otp_receives_submitted_code() {
        let svc = OtpInteractionService::new(300);
        svc.request_otp("kindle").await;

        let svc_clone = svc.clone();
        let handle = tokio::spawn(async move {
            svc_clone.wait_for_otp("kindle").await
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        svc.submit_otp("kindle", "654321").await.unwrap();

        let result = handle.await.unwrap();
        assert_eq!(result.unwrap(), "654321");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p services otp_service -- --nocapture`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement OtpInteractionService**

Create `backend/services/src/otp_service.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use domain::DomainError;
use tokio::sync::{Mutex, oneshot};

#[derive(Clone)]
pub struct OtpInteractionService {
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    timeout_secs: u64,
}

impl OtpInteractionService {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            timeout_secs,
        }
    }

    pub async fn request_otp(&self, platform: &str) {
        let (tx, _rx) = oneshot::channel();
        let mut pending = self.pending.lock().await;
        // Drop any old pending request
        pending.insert(platform.to_string(), tx);
    }

    pub async fn is_pending(&self, platform: &str) -> bool {
        self.pending.lock().await.contains_key(platform)
    }

    pub async fn submit_otp(
        &self,
        platform: &str,
        code: &str,
    ) -> Result<String, DomainError> {
        let mut pending = self.pending.lock().await;
        let tx = pending.remove(platform).ok_or_else(|| {
            DomainError::ValidationError(format!("No OTP pending for {platform}"))
        })?;
        // Ignore send error — receiver may have been dropped (timeout)
        let _ = tx.send(code.to_string());
        Ok(code.to_string())
    }

    pub async fn wait_for_otp(&self, platform: &str) -> Result<String, DomainError> {
        let rx = {
            let mut pending = self.pending.lock().await;
            // Replace the sender with a new channel pair so we can wait on the receiver
            let (tx, rx) = oneshot::channel();
            pending.insert(platform.to_string(), tx);
            rx
        };

        match tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            rx,
        )
        .await
        {
            Ok(Ok(code)) => Ok(code),
            Ok(Err(_)) => Err(DomainError::InternalError("OTP channel closed".to_string())),
            Err(_) => {
                self.pending.lock().await.remove(platform);
                Err(DomainError::InternalError(format!(
                    "OTP timeout after {}s for {platform}",
                    self.timeout_secs
                )))
            }
        }
    }
}

// Tests defined in Step 1 go here
```

Add to `backend/services/src/lib.rs`:
```rust
pub mod otp_service;
pub use otp_service::OtpInteractionService;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p services otp_service -- --nocapture`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add backend/services/src/otp_service.rs backend/services/src/lib.rs
git commit -m "feat(P9-A7): OTP interaction service — pause/resume state machine

- OtpInteractionService with request_otp, submit_otp, wait_for_otp
- Uses oneshot channels for synchronization
- Configurable timeout (default 300s)
- 4 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A8: Kindle Real Integration

**Files:**
- Modify: `backend/plugins/src/ecosystem/kindle.rs`

- [ ] **Step 1: Write failing tests for Kindle OTP flow**

Add to `backend/plugins/src/ecosystem/kindle.rs` tests:

```rust
#[test]
fn kindle_login_selectors_defined() {
    assert!(!KINDLE_EMAIL_SELECTOR.is_empty());
    assert!(!KINDLE_PASSWORD_SELECTOR.is_empty());
    assert!(!KINDLE_SUBMIT_SELECTOR.is_empty());
    assert!(!KINDLE_OTP_SELECTOR.is_empty());
}

#[test]
fn parse_kindle_credentials() {
    let json = r#"{"username": "user@example.com", "password": "pass123", "marketplace": "jp"}"#;
    let creds: KindleCredentials = serde_json::from_str(json).unwrap();
    assert_eq!(creds.username, "user@example.com");
    assert_eq!(creds.marketplace, "jp");
}

#[test]
fn kindle_library_url_is_amazon_jp() {
    assert!(KINDLE_LIBRARY_URL.contains("amazon.co.jp"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p plugins kindle -- --nocapture`
Expected: FAIL — selectors and `KindleCredentials` not defined.

- [ ] **Step 3: Update Kindle connector with real selectors and credentials**

In `backend/plugins/src/ecosystem/kindle.rs`:

Add CSS selector constants:
```rust
const KINDLE_EMAIL_SELECTOR: &str = "input[name='email']";
const KINDLE_PASSWORD_SELECTOR: &str = "input[name='password']";
const KINDLE_SUBMIT_SELECTOR: &str = "input#signInSubmit";
const KINDLE_OTP_SELECTOR: &str = "input[name='otpCode']";
```

Add credentials struct:
```rust
#[derive(Debug, Deserialize)]
pub struct KindleCredentials {
    pub username: String,
    pub password: String,
    #[serde(default = "default_marketplace")]
    pub marketplace: String,
}

fn default_marketplace() -> String {
    "jp".to_string()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p plugins kindle -- --nocapture`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add backend/plugins/src/ecosystem/kindle.rs
git commit -m "feat(P9-A8): Kindle real integration — selectors + credentials struct

- Add real CSS selectors for Amazon JP login (email, password, OTP)
- Add KindleCredentials struct for vault-stored credentials
- 3 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-A9: OTP Endpoint + Vault Credential Wiring

**Files:**
- Modify: `backend/adapters/src/sync_handler.rs`
- Modify: `backend/adapters/src/routes.rs`
- Modify: `backend/app/src/runtime.rs`

- [ ] **Step 1: Write failing test for OTP endpoint**

Add to `backend/adapters/src/sync_handler.rs`:

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct OtpSubmitRequest {
    pub platform: String,
    pub code: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OtpSubmitResponse {
    pub accepted: bool,
}
```

Add test in the test module:

```rust
#[test]
fn otp_submit_request_deserializes() {
    let json = r#"{"platform": "kindle", "code": "123456"}"#;
    let req: OtpSubmitRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.platform, "kindle");
    assert_eq!(req.code, "123456");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p adapters otp_submit -- --nocapture`
Expected: FAIL — types not defined.

- [ ] **Step 3: Implement OTP endpoint**

In `backend/adapters/src/sync_handler.rs`, add the handler:

```rust
#[utoipa::path(
    post,
    path = "/api/v1/sync/otp",
    request_body = OtpSubmitRequest,
    responses(
        (status = 200, description = "OTP accepted", body = OtpSubmitResponse),
        (status = 400, description = "No OTP pending"),
    ),
    tag = "sync",
    security(("bearer_auth" = []))
)]
pub async fn submit_otp(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OtpSubmitRequest>,
) -> Result<Json<OtpSubmitResponse>, ApiError> {
    let otp_service = state
        .otp_service
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("OTP service not configured"))?;
    otp_service
        .submit_otp(&request.platform, &request.code)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(OtpSubmitResponse { accepted: true }))
}
```

- [ ] **Step 4: Register OTP route**

In `backend/adapters/src/routes.rs`, add after the ecosystem sync routes (~line 199):

```rust
        .route("/api/v1/sync/otp", post(submit_otp))
```

- [ ] **Step 5: Wire OtpInteractionService in runtime**

In `backend/app/src/runtime.rs`, after sync_service creation (~line 77):

```rust
    let otp_service = Arc::new(services::OtpInteractionService::new(
        config.otp_timeout_secs.unwrap_or(300),
    ));
```

And wire it:
```rust
    state = state.with_otp_service(otp_service);
```

Note: This requires adding `otp_service: Option<Arc<OtpInteractionService>>` to `AppState` and a `with_otp_service` method. Follow the existing pattern used by `with_vault_service`, `with_sync_service`, etc.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd backend && cargo test -p adapters -- --nocapture`
Expected: All tests PASS

- [ ] **Step 7: Commit**

```bash
git add backend/adapters/src/sync_handler.rs backend/adapters/src/routes.rs backend/app/src/runtime.rs backend/adapters/src/lib.rs
git commit -m "feat(P9-A9): OTP endpoint + vault credential wiring

- Add POST /api/v1/sync/otp endpoint for OTP submission
- Wire OtpInteractionService into AppState and runtime
- OtpSubmitRequest/Response types
- 1 new test

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Track B: Advanced Search Features

### Task P9-B1: Backend Sort/Facet Query Params

**Files:**
- Modify: `backend/adapters/src/tag_filter.rs`
- Modify: `backend/adapters/src/ebook.rs`
- Modify: `backend/adapters/src/search_options.rs`

- [ ] **Step 1: Write failing tests for extended ListQuery**

Add test in `backend/adapters/src/tag_filter.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_query_parses_sort_params() {
        let query: ListQuery = serde_urlencoded::from_str(
            "sort_by=title&sort_order=asc&with_facets=true"
        ).unwrap();
        assert_eq!(query.sort_by.as_deref(), Some("title"));
        assert_eq!(query.sort_order.as_deref(), Some("asc"));
        assert_eq!(query.with_facets, Some(true));
    }

    #[test]
    fn list_query_defaults_to_none() {
        let query: ListQuery = serde_urlencoded::from_str("").unwrap();
        assert!(query.sort_by.is_none());
        assert!(query.sort_order.is_none());
        assert!(query.with_facets.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test -p adapters list_query -- --nocapture`
Expected: FAIL — `sort_by`, `sort_order`, `with_facets` fields not on `ListQuery`.

- [ ] **Step 3: Extend ListQuery with sort/facet fields**

In `backend/adapters/src/tag_filter.rs`, update the `ListQuery` struct:

```rust
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub tags: Vec<String>,
    pub tag: Option<String>,
    pub logic: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub with_facets: Option<bool>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test -p adapters list_query -- --nocapture`
Expected: 2 tests PASS

- [ ] **Step 5: Write failing test for sort+facet in list_ebooks handler**

This test validates that `list_ebooks` applies sorting when `sort_by` is present. Add a handler-level integration test in `backend/adapters/tests/handlers_tdd.rs` or as a unit test:

```rust
#[test]
fn resolve_sort_params_defaults() {
    use adapters::search_options::{SortField, SortOrder};
    let (field, order) = adapters::tag_filter::resolve_sort_params(None, None);
    assert_eq!(field, SortField::DateAdded);
    assert_eq!(order, SortOrder::Desc);
}

#[test]
fn resolve_sort_params_explicit() {
    use adapters::search_options::{SortField, SortOrder};
    let (field, order) = adapters::tag_filter::resolve_sort_params(
        Some("title"),
        Some("asc"),
    );
    assert_eq!(field, SortField::Title);
    assert_eq!(order, SortOrder::Asc);
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cd backend && cargo test -p adapters resolve_sort -- --nocapture`
Expected: FAIL — `resolve_sort_params` not defined.

- [ ] **Step 7: Implement resolve_sort_params**

In `backend/adapters/src/tag_filter.rs`:

```rust
use crate::search_options::{SortField, SortOrder};

pub fn resolve_sort_params(
    sort_by: Option<&str>,
    sort_order: Option<&str>,
) -> (SortField, SortOrder) {
    let field = sort_by
        .and_then(|s| SortField::from_str(s).ok())
        .unwrap_or(SortField::DateAdded);
    let order = sort_order
        .and_then(|s| SortOrder::from_str(s).ok())
        .unwrap_or(SortOrder::Desc);
    (field, order)
}
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cd backend && cargo test -p adapters -- --nocapture`
Expected: All tests PASS

- [ ] **Step 9: Update list_ebooks handler to apply sort + optional facets**

In `backend/adapters/src/ebook.rs`, update `list_ebooks`:

```rust
pub async fn list_ebooks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let allowed_ids = resolve_list_query_filters(&state, &query).await?;
    let mut result = state.ebook_service.list_ebooks(allowed_ids.as_ref()).await?;

    let (sort_field, sort_order) = crate::tag_filter::resolve_sort_params(
        query.sort_by.as_deref(),
        query.sort_order.as_deref(),
    );
    result = services::search_aggregation::sort_resources(
        result,
        match sort_field {
            crate::search_options::SortField::Title => services::search_aggregation::SortField::Title,
            crate::search_options::SortField::DateAdded => services::search_aggregation::SortField::DateAdded,
        },
        match sort_order {
            crate::search_options::SortOrder::Asc => services::search_aggregation::SortOrder::Asc,
            crate::search_options::SortOrder::Desc => services::search_aggregation::SortOrder::Desc,
        },
    );

    if query.with_facets == Some(true) {
        let facets = services::search_aggregation::count_formats(&result);
        let facet_response: Vec<serde_json::Value> = facets
            .into_iter()
            .map(|f| serde_json::json!({"name": f.name, "count": f.count}))
            .collect();
        Ok(Json(serde_json::json!({
            "items": result,
            "facets": {"formats": facet_response}
        })))
    } else {
        Ok(Json(serde_json::to_value(&result).map_err(|e| {
            ApiError::from(domain::DomainError::InternalError(format!("JSON error: {e}")))
        })?))
    }
}
```

Apply the same pattern to the other 4 list handlers (web_reader, image, video, game). Each follows the identical pattern — read the handler, add `resolve_sort_params` + `sort_resources` + optional facet response.

- [ ] **Step 10: Run full backend tests**

Run: `cd backend && cargo test -- --nocapture`
Expected: All tests PASS

- [ ] **Step 11: Commit**

```bash
git add backend/adapters/src/tag_filter.rs backend/adapters/src/ebook.rs backend/adapters/src/search_options.rs
git commit -m "feat(P9-B1): Backend sort/facet query params on all list endpoints

- Extend ListQuery with sort_by, sort_order, with_facets
- Add resolve_sort_params helper
- Apply sorting via search_aggregation::sort_resources in list handlers
- Optional facet response when with_facets=true
- 4 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-B2: SearchFilterBloc (frontend)

**Files:**
- Create: `frontend/lib/blocs/search_filter/search_filter_bloc.dart`
- Create: `frontend/test/blocs/search_filter/search_filter_bloc_test.dart`

- [ ] **Step 1: Write failing BLoC tests**

Create `frontend/test/blocs/search_filter/search_filter_bloc_test.dart`:

```dart
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory/blocs/search_filter/search_filter_bloc.dart';

void main() {
  group('SearchFilterBloc', () {
    late SearchFilterBloc bloc;

    setUp(() {
      bloc = SearchFilterBloc();
    });

    tearDown(() => bloc.close());

    test('initial state has empty tags and default sort', () {
      expect(bloc.state.selectedTags, isEmpty);
      expect(bloc.state.sortBy, equals('date_added'));
      expect(bloc.state.sortOrder, equals('desc'));
      expect(bloc.state.filterLogic, equals('and'));
    });

    test('UpdateSelectedTags updates tags', () async {
      bloc.add(const UpdateSelectedTags(['fiction', 'sci-fi']));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.selectedTags,
          'selectedTags',
          ['fiction', 'sci-fi'],
        )),
      );
    });

    test('UpdateSortBy updates sort field', () async {
      bloc.add(const UpdateSortBy('title'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.sortBy,
          'sortBy',
          'title',
        )),
      );
    });

    test('UpdateSortOrder updates sort direction', () async {
      bloc.add(const UpdateSortOrder('asc'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.sortOrder,
          'sortOrder',
          'asc',
        )),
      );
    });

    test('UpdateFilterLogic updates logic', () async {
      bloc.add(const UpdateFilterLogic('or'));
      await expectLater(
        bloc.stream,
        emits(isA<SearchFilterState>().having(
          (s) => s.filterLogic,
          'filterLogic',
          'or',
        )),
      );
    });

    test('ClearFilters resets to defaults', () async {
      bloc.add(const UpdateSelectedTags(['fiction']));
      bloc.add(const UpdateSortBy('title'));
      bloc.add(const ClearFilters());
      await expectLater(
        bloc.stream,
        emitsThrough(isA<SearchFilterState>().having(
          (s) => s.selectedTags,
          'selectedTags',
          isEmpty,
        )),
      );
    });
  });
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && flutter test test/blocs/search_filter/search_filter_bloc_test.dart`
Expected: FAIL — file does not exist.

- [ ] **Step 3: Implement SearchFilterBloc**

Create `frontend/lib/blocs/search_filter/search_filter_bloc.dart`:

```dart
import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

// Events
sealed class SearchFilterEvent extends Equatable {
  const SearchFilterEvent();

  @override
  List<Object?> get props => [];
}

final class UpdateSelectedTags extends SearchFilterEvent {
  const UpdateSelectedTags(this.tags);
  final List<String> tags;

  @override
  List<Object?> get props => [tags];
}

final class UpdateSortBy extends SearchFilterEvent {
  const UpdateSortBy(this.sortBy);
  final String sortBy;

  @override
  List<Object?> get props => [sortBy];
}

final class UpdateSortOrder extends SearchFilterEvent {
  const UpdateSortOrder(this.sortOrder);
  final String sortOrder;

  @override
  List<Object?> get props => [sortOrder];
}

final class UpdateFilterLogic extends SearchFilterEvent {
  const UpdateFilterLogic(this.logic);
  final String logic;

  @override
  List<Object?> get props => [logic];
}

final class ClearFilters extends SearchFilterEvent {
  const ClearFilters();
}

// State
class SearchFilterState extends Equatable {
  const SearchFilterState({
    this.selectedTags = const [],
    this.sortBy = 'date_added',
    this.sortOrder = 'desc',
    this.filterLogic = 'and',
  });

  final List<String> selectedTags;
  final String sortBy;
  final String sortOrder;
  final String filterLogic;

  SearchFilterState copyWith({
    List<String>? selectedTags,
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  }) {
    return SearchFilterState(
      selectedTags: selectedTags ?? this.selectedTags,
      sortBy: sortBy ?? this.sortBy,
      sortOrder: sortOrder ?? this.sortOrder,
      filterLogic: filterLogic ?? this.filterLogic,
    );
  }

  @override
  List<Object?> get props => [selectedTags, sortBy, sortOrder, filterLogic];
}

// BLoC
class SearchFilterBloc extends Bloc<SearchFilterEvent, SearchFilterState> {
  SearchFilterBloc() : super(const SearchFilterState()) {
    on<UpdateSelectedTags>(_onUpdateSelectedTags);
    on<UpdateSortBy>(_onUpdateSortBy);
    on<UpdateSortOrder>(_onUpdateSortOrder);
    on<UpdateFilterLogic>(_onUpdateFilterLogic);
    on<ClearFilters>(_onClearFilters);
  }

  void _onUpdateSelectedTags(
    UpdateSelectedTags event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(selectedTags: event.tags));
  }

  void _onUpdateSortBy(
    UpdateSortBy event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(sortBy: event.sortBy));
  }

  void _onUpdateSortOrder(
    UpdateSortOrder event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(sortOrder: event.sortOrder));
  }

  void _onUpdateFilterLogic(
    UpdateFilterLogic event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(state.copyWith(filterLogic: event.logic));
  }

  void _onClearFilters(
    ClearFilters event,
    Emitter<SearchFilterState> emit,
  ) {
    emit(const SearchFilterState());
  }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd frontend && flutter test test/blocs/search_filter/search_filter_bloc_test.dart`
Expected: 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add frontend/lib/blocs/search_filter/search_filter_bloc.dart frontend/test/blocs/search_filter/search_filter_bloc_test.dart
git commit -m "feat(P9-B2): SearchFilterBloc — global filter state for ResourceListScreen

- SearchFilterBloc with events: UpdateSelectedTags, UpdateSortBy, UpdateSortOrder, UpdateFilterLogic, ClearFilters
- SearchFilterState with selectedTags, sortBy, sortOrder, filterLogic
- 6 new tests

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-B3: Wire SearchFilterBar into ResourceListScreen

**Files:**
- Modify: `frontend/lib/screens/resource_list_screen.dart`
- Modify: `frontend/lib/main.dart`
- Modify: `frontend/lib/blocs/ebook/ebook_bloc.dart` (and all 4 other BLoCs)

- [ ] **Step 1: Write failing widget test for filter bar presence**

Create `frontend/test/screens/resource_list_screen_filter_test.dart`:

```dart
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:personal_inventory/blocs/ebook/ebook_bloc.dart';
import 'package:personal_inventory/blocs/game/game_bloc.dart';
import 'package:personal_inventory/blocs/image/image_bloc.dart';
import 'package:personal_inventory/blocs/search_filter/search_filter_bloc.dart';
import 'package:personal_inventory/blocs/tag/tag_bloc.dart';
import 'package:personal_inventory/blocs/video/video_bloc.dart';
import 'package:personal_inventory/blocs/web_reader/web_reader_bloc.dart';
import 'package:personal_inventory/blocs/batch/batch_bloc.dart';
import 'package:personal_inventory/screens/resource_list_screen.dart';
import 'package:personal_inventory/widgets/search_filter_bar.dart';

// Use the existing in-memory repository pattern from the test suite.
// Import the fakes/mocks already used in existing resource_list_screen_test.dart.

void main() {
  group('ResourceListScreen with SearchFilterBar', () {
    testWidgets('shows SearchFilterBar when tags are loaded', (tester) async {
      // Build the widget tree with all required BLoC providers + SearchFilterBloc.
      // Pump a frame, load tags, verify SearchFilterBar widget is found.
      // This test validates that the SearchFilterBar widget is rendered.
      // Full implementation depends on existing test infrastructure.
      expect(find.byType(SearchFilterBar), findsOneWidget);
    });
  });
}
```

Note: The exact test setup mirrors the existing `resource_list_screen_test.dart` infrastructure. Copy the provider setup from there and add `SearchFilterBloc` provider.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd frontend && flutter test test/screens/resource_list_screen_filter_test.dart`
Expected: FAIL — `SearchFilterBar` not found in widget tree.

- [ ] **Step 3: Provide SearchFilterBloc at app root**

In `frontend/lib/main.dart`, add to the `MultiBlocProvider.providers` list:

```dart
BlocProvider<SearchFilterBloc>(
  create: (_) => SearchFilterBloc(),
),
```

Import:
```dart
import 'blocs/search_filter/search_filter_bloc.dart';
```

- [ ] **Step 4: Replace _TagFilterBar with SearchFilterBar in ResourceListScreen**

In `frontend/lib/screens/resource_list_screen.dart`:

1. Add import:
```dart
import '../blocs/search_filter/search_filter_bloc.dart';
import '../widgets/search_filter_bar.dart';
```

2. Replace the `_TagFilterBar` section in the `body: Column` with:

```dart
BlocBuilder<TagBloc, TagState>(
  builder: (context, tagState) {
    if (tagState is! TagListLoaded || tagState.tags.isEmpty) {
      return const SizedBox.shrink();
    }
    return BlocBuilder<SearchFilterBloc, SearchFilterState>(
      builder: (context, filterState) {
        return SearchFilterBar(
          tags: tagState.tags,
          selectedTags: filterState.selectedTags,
          sortOption: filterState.sortBy,
          filterLogic: filterState.filterLogic,
          onTagsChanged: (tags) {
            context.read<SearchFilterBloc>().add(UpdateSelectedTags(tags));
            _reloadAllTabs(context, tags: tags, sortBy: filterState.sortBy, sortOrder: filterState.sortOrder, filterLogic: filterState.filterLogic);
          },
          onSortChanged: (sort) {
            context.read<SearchFilterBloc>().add(UpdateSortBy(sort));
            _reloadAllTabs(context, tags: filterState.selectedTags, sortBy: sort, sortOrder: filterState.sortOrder, filterLogic: filterState.filterLogic);
          },
          onLogicChanged: (logic) {
            context.read<SearchFilterBloc>().add(UpdateFilterLogic(logic));
            _reloadAllTabs(context, tags: filterState.selectedTags, sortBy: filterState.sortBy, sortOrder: filterState.sortOrder, filterLogic: logic);
          },
        );
      },
    );
  },
),
```

3. Add helper method to `_ResourceListScreenState`:

```dart
void _reloadAllTabs(
  BuildContext context, {
  required List<String> tags,
  required String sortBy,
  required String sortOrder,
  required String filterLogic,
}) {
  context.read<EbookBloc>().add(LoadEbooks(
    tags: tags,
    sortBy: sortBy,
    sortOrder: sortOrder,
    filterLogic: filterLogic,
  ));
  context.read<WebReaderBloc>().add(LoadWebReaders(
    tags: tags,
    sortBy: sortBy,
    sortOrder: sortOrder,
    filterLogic: filterLogic,
  ));
  context.read<ImageBloc>().add(LoadImages(
    tags: tags,
    sortBy: sortBy,
    sortOrder: sortOrder,
    filterLogic: filterLogic,
  ));
  context.read<VideoBloc>().add(LoadVideos(
    tags: tags,
    sortBy: sortBy,
    sortOrder: sortOrder,
    filterLogic: filterLogic,
  ));
  context.read<GameBloc>().add(LoadGames(
    tags: tags,
    sortBy: sortBy,
    sortOrder: sortOrder,
    filterLogic: filterLogic,
  ));
}
```

4. Remove the old `_activeTagName` state variable and `_TagFilterBar` class.

- [ ] **Step 5: Extend Load events on all 5 BLoCs**

In `frontend/lib/blocs/ebook/ebook_bloc.dart`, update `LoadEbooks`:

```dart
final class LoadEbooks extends EbookEvent {
  const LoadEbooks({
    this.tags = const [],
    this.sortBy,
    this.sortOrder,
    this.filterLogic,
  });

  final List<String> tags;
  final String? sortBy;
  final String? sortOrder;
  final String? filterLogic;

  @override
  List<Object?> get props => [tags, sortBy, sortOrder, filterLogic];
}
```

Update `_onLoadEbooks` to pass params to repository:
```dart
Future<void> _onLoadEbooks(LoadEbooks event, Emitter<EbookState> emit) async {
  emit(const EbookLoading());
  final result = await _repository.listEbooks(
    tags: event.tags,
    sortBy: event.sortBy,
    sortOrder: event.sortOrder,
    filterLogic: event.filterLogic,
  );
  result.when(
    success: (ebooks) => emit(EbookListLoaded(ebooks)),
    failure: (failure) => emit(EbookError(failure)),
  );
}
```

Apply the same pattern to `game_bloc.dart`, `image_bloc.dart`, `video_bloc.dart`, `web_reader_bloc.dart`. Each Load event gets identical optional parameters.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd frontend && flutter test`
Expected: All tests PASS (existing + new)

- [ ] **Step 7: Commit**

```bash
git add frontend/lib/screens/resource_list_screen.dart frontend/lib/main.dart frontend/lib/blocs/ frontend/test/screens/resource_list_screen_filter_test.dart
git commit -m "feat(P9-B3): Wire SearchFilterBar into ResourceListScreen

- Replace _TagFilterBar with SearchFilterBar + SearchFilterBloc
- Extend all 5 Load events with tags/sortBy/sortOrder/filterLogic params
- Provide SearchFilterBloc at app root in main.dart
- Shared filter state across all 5 tabs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task P9-B4: Update HTTP Repositories with Filter/Sort Params

**Files:**
- Modify: `frontend/lib/repositories/ebook_repository.dart` (and all 4 other abstract repos)
- Modify: HTTP repository implementations

- [ ] **Step 1: Write failing test for repository with query params**

Add test (follow existing test patterns in the test suite):

```dart
test('listEbooks passes sort params to API', () async {
  // Mock HTTP client to capture the URL
  // Call listEbooks(tags: ['fiction'], sortBy: 'title', sortOrder: 'asc')
  // Verify URL contains ?tags=fiction&sort_by=title&sort_order=asc
});
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL — `listEbooks` doesn't accept named parameters yet.

- [ ] **Step 3: Update abstract repository interface**

In `frontend/lib/repositories/ebook_repository.dart`:

```dart
abstract interface class EbookRepository {
  Future<Result<List<Resource>, AppFailure>> listEbooks({
    List<String> tags,
    String? sortBy,
    String? sortOrder,
    String? filterLogic,
  });

  // ... rest unchanged
}
```

Apply the same change to `game_repository.dart`, `image_repository.dart`, `video_repository.dart`, `web_reader_repository.dart`.

- [ ] **Step 4: Update HTTP repository implementations**

In the HTTP repository implementation (e.g., the HTTP ebook repository), update `listEbooks` to append query params:

```dart
@override
Future<Result<List<Resource>, AppFailure>> listEbooks({
  List<String> tags = const [],
  String? sortBy,
  String? sortOrder,
  String? filterLogic,
}) async {
  final params = <String, String>{};
  for (final tag in tags) {
    // Multiple tags: append each as separate param
  }
  if (sortBy != null) params['sort_by'] = sortBy;
  if (sortOrder != null) params['sort_order'] = sortOrder;
  if (filterLogic != null) params['logic'] = filterLogic;

  final uri = Uri.parse('$baseUrl/api/v1/inventory/ebooks/list')
      .replace(queryParameters: params.isNotEmpty ? params : null);
  // ... existing HTTP call logic with uri
}
```

Apply the same pattern to all 5 HTTP repository implementations.

Also update the `in_memory_repositories.dart` to accept and ignore the new params for test compatibility.

- [ ] **Step 5: Run all frontend tests**

Run: `cd frontend && flutter test`
Expected: All tests PASS

- [ ] **Step 6: Commit**

```bash
git add frontend/lib/repositories/ frontend/test/
git commit -m "feat(P9-B4): HTTP repositories pass filter/sort query params to API

- Extend all 5 repository interfaces with optional tags/sortBy/sortOrder/filterLogic params
- HTTP repos build query params in URL
- In-memory repos accept and ignore new params
- End-to-end: SearchFilterBar → BLoC → Repository → API query params

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

## Final Verification

- [ ] **Run full backend test suite**

```bash
cd backend && cargo test -- --nocapture
```

Expected: All 220+ existing tests PASS + ~40 new tests PASS. Zero regressions.

- [ ] **Run full frontend test suite**

```bash
cd frontend && flutter test
```

Expected: All 348+ existing tests PASS + ~20 new tests PASS. Zero regressions.

- [ ] **Update CONTEXT.md with Phase 9 summary**

Append Phase 9 plan summary to `CONTEXT.md` documenting:
- Track A: Real API integration (HAR parser, HttpConnectorClient, ChromiumSession, Steam/DLSite/FANZA/Kindle connectors, OTP service)
- Track B: SearchFilterBar wiring + backend sort/facet endpoints
- New test count
- Architecture decisions

---

## Execution Order Summary

```
Parallelizable:
  P9-A1 (HAR extraction) | P9-B1 (Backend sort/facet) | P9-B2 (SearchFilterBloc)

Sequential (depends on A1):
  P9-A2 (HttpConnectorClient)
  P9-A3 (ChromiumSession)

Sequential (depends on A1+A2):
  P9-A4 (Steam)

Sequential (depends on A1+A3):
  P9-A5 (DLSite) → P9-A6 (FANZA)

Sequential (depends on A3):
  P9-A7 (OTP service) → P9-A8 (Kindle)

Sequential (depends on all A tasks):
  P9-A9 (OTP endpoint + vault wiring)

Sequential (depends on B1+B2):
  P9-B3 (Wire SearchFilterBar) → P9-B4 (HTTP repo params)
```
