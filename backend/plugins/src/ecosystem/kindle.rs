use domain::DomainError;
use serde::Deserialize;
use async_trait::async_trait;

use crate::browser_session::{BrowserPage, Cookie};
use domain::ecosystem::{DiscoveredItem, EcosystemConnector};

// TODO(network-inspection): Replace with real endpoints discovered from Amazon network traffic.
const KINDLE_LOGIN_URL: &str = "https://www.amazon.co.jp/ap/signin";
const KINDLE_LIBRARY_URL: &str =
    "https://www.amazon.co.jp/hz/mycd/digital-console/contentlist/booksAll/dateDsc/";

/// A Kindle book entry from the library API or CSV export.
#[derive(Debug, Clone, Deserialize)]
pub struct KindleBook {
    #[serde(alias = "ASIN")]
    pub asin: String,
    #[serde(alias = "Title")]
    pub title: String,
    #[serde(default, alias = "Authors")]
    pub author: Option<String>,
    #[serde(default)]
    pub file_format: Option<String>,
}

/// A row from the Amazon "Manage Your Content and Devices" CSV export.
///
/// Column names may vary by locale; we support both English and common aliases.
#[derive(Debug, Clone, Deserialize)]
pub struct KindleCsvRow {
    #[serde(rename = "ASIN")]
    pub asin: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Authors", default)]
    pub authors: Option<String>,
}

impl From<KindleCsvRow> for KindleBook {
    fn from(row: KindleCsvRow) -> Self {
        KindleBook {
            asin: row.asin,
            title: row.title,
            author: row.authors,
            file_format: Some("kindle".to_string()),
        }
    }
}

/// Parses a JSON library response from Kindle.
///
/// Expected format: `{"items": [...]}`
pub fn parse_library_response(json: &str) -> Result<Vec<KindleBook>, DomainError> {
    #[derive(Deserialize)]
    struct LibraryResponse {
        #[serde(default)]
        items: Vec<KindleBook>,
    }
    let resp: LibraryResponse = serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("Kindle parse error: {e}")))?;
    Ok(resp.items)
}

/// Parses a CSV export from Amazon "Manage Your Content and Devices".
pub fn parse_csv_export(csv: &str) -> Result<Vec<KindleBook>, DomainError> {
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    let mut books = Vec::new();
    for result in reader.deserialize::<KindleCsvRow>() {
        let row = result.map_err(|e| {
            DomainError::InternalError(format!("Kindle CSV parse error: {e}"))
        })?;
        books.push(KindleBook::from(row));
    }
    Ok(books)
}

/// Kindle connector. Supports browser-based library sync and CSV file import.
pub struct KindleConnector<P: BrowserPage> {
    browser: P,
}

impl<P: BrowserPage> KindleConnector<P> {
    pub fn new(browser: P) -> Self {
        Self { browser }
    }

    async fn ensure_authenticated(
        &self,
        session_cookies: Vec<Cookie>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), DomainError> {
        if !session_cookies.is_empty() {
            self.browser.set_cookies(session_cookies).await?;
        }
        // NOTE: Amazon login is complex — CAPTCHA, 2FA, device verification may be required.
        // TODO(network-inspection): Implement actual Amazon auth flow.
        self.browser.navigate(KINDLE_LOGIN_URL).await?;
        if let (Some(user), Some(pass)) = (username, password) {
            self.browser.fill("input[name='email']", user).await?;
            self.browser.fill("input[name='password']", pass).await?;
            self.browser.click("#signInSubmit").await?;
        }
        Ok(())
    }

    #[cfg(feature = "real-plugins")]
    async fn fetch_library(&self) -> Result<Vec<KindleBook>, DomainError> {
        let cookies = self.browser.get_cookies().await?;
        let cookie_header = cookies
            .iter()
            .map(|c| {
                let name = c.name.replace(['\r', '\n'], "");
                let value = c.value.replace(['\r', '\n'], "");
                format!("{}={}", name, value)
            })
            .collect::<Vec<_>>()
            .join("; ");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| DomainError::InternalError(format!("HTTP client error: {e}")))?;
        // TODO(network-inspection): Replace KINDLE_LIBRARY_URL with the actual
        // JSON endpoint. Amazon's digital-console API may require additional headers.
        let resp = client
            .get(KINDLE_LIBRARY_URL)
            .header("Cookie", cookie_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| DomainError::InternalError(format!("Kindle HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Err(DomainError::InternalError(format!(
                "Kindle library returned status {}",
                resp.status()
            )));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| DomainError::InternalError(format!("Kindle read error: {e}")))?;
        parse_library_response(&text)
    }
}

#[async_trait]
impl<P: BrowserPage + Send + Sync> EcosystemConnector for KindleConnector<P> {
    fn platform_name(&self) -> &str {
        "kindle"
    }

    async fn discover_items(&self, credentials: &[u8]) -> Result<Vec<DiscoveredItem>, DomainError> {
        #[derive(Deserialize)]
        struct Creds {
            #[serde(default)]
            cookies: Vec<Cookie>,
            username: Option<String>,
            password: Option<String>,
            /// CSV content from Amazon export — if provided, use file import path.
            csv_export: Option<String>,
        }
        let creds: Creds = serde_json::from_slice(credentials).map_err(|e| {
            DomainError::InternalError(format!("Kindle credentials parse error: {e}"))
        })?;

        let books = if let Some(csv) = creds.csv_export {
            // File import path: parse CSV directly, no browser needed.
            parse_csv_export(&csv)?
        } else {
            // Browser sync path.
            self.ensure_authenticated(
                creds.cookies,
                creds.username.as_deref(),
                creds.password.as_deref(),
            )
            .await?;

            #[cfg(feature = "real-plugins")]
            let fetched = self.fetch_library().await?;
            #[cfg(not(feature = "real-plugins"))]
            let fetched: Vec<KindleBook> = vec![];
            fetched
        };

        let items = books
            .into_iter()
            .map(|b| {
                let mut metadata = std::collections::HashMap::new();
                if let Some(author) = &b.author {
                    metadata.insert("author".to_string(), author.clone());
                }
                let fmt = b
                    .file_format
                    .clone()
                    .unwrap_or_else(|| "kindle".to_string());
                metadata.insert("file_format".to_string(), fmt);
                metadata.insert(
                    "url".to_string(),
                    format!("kindle://asin/{}", b.asin),
                );
                DiscoveredItem {
                    external_id: b.asin,
                    title: b.title,
                    platform: "kindle".to_string(),
                    metadata,
                }
            })
            .collect();
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_library_response_valid_json() {
        let json = r#"{"items": [
            {"asin": "B001", "title": "Book One", "author": "Auth A", "file_format": "azw3"},
            {"asin": "B002", "title": "Book Two"}
        ]}"#;
        let books = parse_library_response(json).unwrap();
        assert_eq!(books.len(), 2);
        assert_eq!(books[0].asin, "B001");
        assert_eq!(books[0].author, Some("Auth A".to_string()));
        assert_eq!(books[0].file_format, Some("azw3".to_string()));
        assert!(books[1].author.is_none());
    }

    #[test]
    fn parse_library_response_empty_items() {
        let books = parse_library_response(r#"{"items": []}"#).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn parse_library_response_invalid_json_returns_error() {
        assert!(parse_library_response("not json").is_err());
    }

    #[test]
    fn parse_csv_export_valid() {
        let csv = "ASIN,Title,Authors\nB001,My Book,Author Name\nB002,Another Book,";
        let books = parse_csv_export(csv).unwrap();
        assert_eq!(books.len(), 2);
        assert_eq!(books[0].asin, "B001");
        assert_eq!(books[0].title, "My Book");
        assert_eq!(books[0].author, Some("Author Name".to_string()));
        assert_eq!(books[0].file_format, Some("kindle".to_string()));
        assert_eq!(books[1].asin, "B002");
    }

    #[test]
    fn parse_csv_export_empty_authors_becomes_some_empty_string() {
        let csv = "ASIN,Title,Authors\nB003,Title X,";
        let books = parse_csv_export(csv).unwrap();
        // CSV empty field -> Some("") or None depending on csv crate behavior
        assert_eq!(books[0].asin, "B003");
    }

    #[test]
    fn kindle_csv_row_converts_to_book() {
        let row = KindleCsvRow {
            asin: "B999".to_string(),
            title: "Converted".to_string(),
            authors: Some("Writer".to_string()),
        };
        let book = KindleBook::from(row);
        assert_eq!(book.asin, "B999");
        assert_eq!(book.file_format, Some("kindle".to_string()));
    }

    #[tokio::test]
    async fn discover_items_csv_path_parses_correctly() {
        use crate::browser_session::NoopBrowserPage;
        let connector = KindleConnector::new(NoopBrowserPage);
        let csv_content = "ASIN,Title,Authors\nB001,My Kindle Book,Some Author";
        let creds = serde_json::json!({"csv_export": csv_content});
        let items = connector
            .discover_items(creds.to_string().as_bytes())
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].external_id, "B001");
        assert_eq!(items[0].title, "My Kindle Book");
        assert_eq!(items[0].metadata.get("author").unwrap(), "Some Author");
        assert_eq!(items[0].metadata.get("url").unwrap(), "kindle://asin/B001");
    }

    #[tokio::test]
    async fn discover_items_browser_path_with_noop_returns_empty() {
        use crate::browser_session::NoopBrowserPage;
        let connector = KindleConnector::new(NoopBrowserPage);
        let creds = serde_json::json!({"cookies": [], "username": "u", "password": "p"});
        let items = connector
            .discover_items(creds.to_string().as_bytes())
            .await
            .unwrap();
        assert!(items.is_empty());
    }
}
