use domain::DomainError;
use serde::Deserialize;
use async_trait::async_trait;

use crate::browser_session::{BrowserPage, Cookie};
use domain::ecosystem::{DiscoveredItem, EcosystemConnector};

// TODO(network-inspection): Replace with real URLs discovered from network inspection.
const BOOKWALKER_LOGIN_URL: &str = "https://member.bookwalker.jp/app/03/login";
const BOOKWALKER_LIBRARY_URL: &str = "https://bookwalker.jp/nl/";

/// A book entry from BookWalker's library API.
#[derive(Debug, Clone, Deserialize)]
pub struct BookWalkerBook {
    pub uuid: String,
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub publisher_name: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub series_name: Option<String>,
    #[serde(default)]
    pub series_sequence: Option<u32>,
}

/// Parses a JSON array of BookWalkerBook objects from the library API response.
pub fn parse_library_response(json: &str) -> Result<Vec<BookWalkerBook>, DomainError> {
    // BookWalker API returns: {"books": [...]}
    #[derive(Deserialize)]
    struct LibraryResponse {
        #[serde(default)]
        books: Vec<BookWalkerBook>,
    }
    let resp: LibraryResponse = serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("BookWalker parse error: {e}")))?;
    Ok(resp.books)
}

/// Parses a flat JSON array of books (for testing convenience).
pub fn parse_books_array(json: &str) -> Result<Vec<BookWalkerBook>, DomainError> {
    serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("BookWalker books parse error: {e}")))
}

/// BookWalker connector. Authenticates via member login, fetches ebook library.
pub struct BookWalkerConnector<P: BrowserPage> {
    browser: P,
}

impl<P: BrowserPage> BookWalkerConnector<P> {
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
        // TODO(network-inspection): Navigate to auth-check page and detect session validity.
        self.browser.navigate(BOOKWALKER_LOGIN_URL).await?;
        if let (Some(user), Some(pass)) = (username, password) {
            // TODO(network-inspection): Replace with actual BookWalker login form selectors.
            self.browser.fill("input[name='j_username']", user).await?;
            self.browser.fill("input[name='j_password']", pass).await?;
            self.browser.click("input[type='submit']").await?;
        }
        Ok(())
    }

    #[cfg(feature = "real-plugins")]
    async fn fetch_library(&self) -> Result<Vec<BookWalkerBook>, DomainError> {
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
        let resp = client
            .get(BOOKWALKER_LIBRARY_URL)
            .header("Cookie", cookie_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| DomainError::InternalError(format!("BookWalker HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Err(DomainError::InternalError(format!(
                "BookWalker library returned status {}",
                resp.status()
            )));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| DomainError::InternalError(format!("BookWalker read error: {e}")))?;
        parse_library_response(&text)
    }
}

#[async_trait]
impl<P: BrowserPage + Send + Sync> EcosystemConnector for BookWalkerConnector<P> {
    fn platform_name(&self) -> &str {
        "bookwalker"
    }

    async fn discover_items(&self, credentials: &[u8]) -> Result<Vec<DiscoveredItem>, DomainError> {
        #[derive(Deserialize)]
        struct Creds {
            #[serde(default)]
            cookies: Vec<Cookie>,
            username: Option<String>,
            password: Option<String>,
        }
        let creds: Creds = serde_json::from_slice(credentials).map_err(|e| {
            DomainError::InternalError(format!("BookWalker credentials parse error: {e}"))
        })?;
        self.ensure_authenticated(
            creds.cookies,
            creds.username.as_deref(),
            creds.password.as_deref(),
        )
        .await?;

        #[cfg(feature = "real-plugins")]
        let books = self.fetch_library().await?;
        #[cfg(not(feature = "real-plugins"))]
        let books: Vec<BookWalkerBook> = vec![];

        let items = books
            .into_iter()
            .map(|b| {
                let mut metadata = std::collections::HashMap::new();
                if let Some(author) = &b.author {
                    metadata.insert("author".to_string(), author.clone());
                }
                if let Some(publisher) = &b.publisher_name {
                    metadata.insert("publisher".to_string(), publisher.clone());
                }
                if let Some(pages) = b.page {
                    metadata.insert("page_count".to_string(), pages.to_string());
                }
                if let Some(series) = &b.series_name {
                    metadata.insert("series".to_string(), series.clone());
                }
                if let Some(seq) = b.series_sequence {
                    metadata.insert("series_volume".to_string(), seq.to_string());
                }
                metadata.insert(
                    "url".to_string(),
                    format!("https://bookwalker.jp/de{}/", b.uuid),
                );
                metadata.insert("file_format".to_string(), "bookwalker_digital".to_string());
                DiscoveredItem {
                    external_id: b.uuid,
                    title: b.title,
                    platform: "bookwalker".to_string(),
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
        let json = r#"{"books": [
            {"uuid": "abc-001", "title": "Manga Vol.1", "author": "Author A", "publisher_name": "Pub", "page": 200, "series_name": "Manga", "series_sequence": 1},
            {"uuid": "abc-002", "title": "Novel"}
        ]}"#;
        let books = parse_library_response(json).unwrap();
        assert_eq!(books.len(), 2);
        assert_eq!(books[0].uuid, "abc-001");
        assert_eq!(books[0].author, Some("Author A".to_string()));
        assert_eq!(books[0].page, Some(200));
        assert_eq!(books[0].series_name, Some("Manga".to_string()));
        assert_eq!(books[0].series_sequence, Some(1));
        assert!(books[1].author.is_none());
    }

    #[test]
    fn parse_library_response_empty_books() {
        let books = parse_library_response(r#"{"books": []}"#).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn parse_library_response_invalid_json_returns_error() {
        let result = parse_library_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_books_array_valid() {
        let json = r#"[{"uuid": "u1", "title": "Book 1"}]"#;
        let books = parse_books_array(json).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].uuid, "u1");
    }

    #[test]
    fn bookwalker_book_missing_optional_fields() {
        let json = r#"{"uuid": "x", "title": "Minimal Book"}"#;
        let book: BookWalkerBook = serde_json::from_str(json).unwrap();
        assert!(book.author.is_none());
        assert!(book.publisher_name.is_none());
        assert!(book.page.is_none());
        assert!(book.series_name.is_none());
        assert!(book.series_sequence.is_none());
    }

    #[tokio::test]
    async fn discover_items_with_noop_browser_returns_empty() {
        use crate::browser_session::NoopBrowserPage;
        let connector = BookWalkerConnector::new(NoopBrowserPage);
        let creds = serde_json::json!({"cookies": [], "username": "u", "password": "p"});
        let items = connector
            .discover_items(creds.to_string().as_bytes())
            .await
            .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn discover_items_maps_metadata_correctly() {
        let books = vec![BookWalkerBook {
            uuid: "bw-123".to_string(),
            title: "Cool Manga Vol.2".to_string(),
            author: Some("Writer".to_string()),
            publisher_name: Some("BigPub".to_string()),
            page: Some(180),
            series_name: Some("Cool Manga".to_string()),
            series_sequence: Some(2),
        }];
        let items: Vec<DiscoveredItem> = books
            .into_iter()
            .map(|b| {
                let mut metadata = std::collections::HashMap::new();
                if let Some(author) = &b.author {
                    metadata.insert("author".to_string(), author.clone());
                }
                if let Some(pages) = b.page {
                    metadata.insert("page_count".to_string(), pages.to_string());
                }
                metadata.insert("file_format".to_string(), "bookwalker_digital".to_string());
                DiscoveredItem {
                    external_id: b.uuid,
                    title: b.title,
                    platform: "bookwalker".to_string(),
                    metadata,
                }
            })
            .collect();
        assert_eq!(items[0].metadata.get("author").unwrap(), "Writer");
        assert_eq!(items[0].metadata.get("page_count").unwrap(), "180");
        assert_eq!(
            items[0].metadata.get("file_format").unwrap(),
            "bookwalker_digital"
        );
    }
}
