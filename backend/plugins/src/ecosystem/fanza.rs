use domain::{DomainError, ResourceType};
use serde::Deserialize;
use async_trait::async_trait;

use crate::browser_session::{BrowserPage, Cookie};
use domain::ecosystem::{DiscoveredItem, EcosystemConnector};

// TODO(network-inspection): Replace these with real URLs discovered from network inspection.
const FANZA_LOGIN_URL: &str = "https://accounts.dmm.com/service/login/password";
const FANZA_LIBRARY_BASE_URL: &str = "https://www.dmm.com/digital/mypage";

// CSS selectors for FANZA login form
const FANZA_USERNAME_SELECTOR: &str = "input[name='login_id']";
const FANZA_PASSWORD_SELECTOR: &str = "input[name='password']";
const FANZA_SUBMIT_SELECTOR: &str = "button[type='submit']";

/// A product entry from FANZA purchase history.
#[derive(Debug, Clone, Deserialize)]
pub struct FanzaProduct {
    pub content_id: String,
    pub title: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub maker_name: Option<String>,
}

/// Detects the resource type from FANZA's `content_type` field.
///
/// FANZA content type mapping:
/// - "game" → Game
/// - "doujin" → depends on sub-type, default Game
/// - "anime", "video" → Video
/// - "manga", "comic", "cg" → Image
/// - Everything else → Game
pub fn detect_resource_type(content_type: &str) -> ResourceType {
    match content_type.to_lowercase().as_str() {
        "anime" | "video" | "mov" => ResourceType::Video,
        "manga" | "comic" | "cg" | "illust" => ResourceType::Image,
        _ => ResourceType::Game,
    }
}

/// Parses a JSON array of FanzaProduct objects from FANZA's purchase history response.
pub fn parse_library_response(json: &str) -> Result<Vec<FanzaProduct>, DomainError> {
    serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("FANZA parse error: {e}")))
}

/// Builds a cookie header string from a collection of cookies.
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

/// FANZA connector. Authenticates via DMM account browser session, then fetches purchase history.
pub struct FanzaConnector<P: BrowserPage> {
    browser: P,
}

impl<P: BrowserPage> FanzaConnector<P> {
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
        // TODO(network-inspection): Navigate to auth-check page and detect login status.
        self.browser.navigate(FANZA_LOGIN_URL).await?;
        if let (Some(user), Some(pass)) = (username, password) {
            self.browser.fill(FANZA_USERNAME_SELECTOR, user).await?;
            self.browser.fill(FANZA_PASSWORD_SELECTOR, pass).await?;
            self.browser.click(FANZA_SUBMIT_SELECTOR).await?;
        }
        Ok(())
    }

    #[cfg(feature = "real-plugins")]
    async fn fetch_library(&self) -> Result<Vec<FanzaProduct>, DomainError> {
        let cookies = self.browser.get_cookies().await?;
        let cookie_header = build_cookie_header(&cookies);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| DomainError::InternalError(format!("HTTP client error: {e}")))?;
        // TODO(network-inspection): Replace FANZA_LIBRARY_BASE_URL with the actual
        // paginated JSON endpoint discovered via network inspection.
        let resp = client
            .get(&format!("{FANZA_LIBRARY_BASE_URL}?output=json"))
            .header("Cookie", cookie_header)
            .send()
            .await
            .map_err(|e| DomainError::InternalError(format!("FANZA HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Err(DomainError::InternalError(format!(
                "FANZA library returned status {}",
                resp.status()
            )));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| DomainError::InternalError(format!("FANZA read error: {e}")))?;
        parse_library_response(&text)
    }
}

#[async_trait]
impl<P: BrowserPage + Send + Sync> EcosystemConnector for FanzaConnector<P> {
    fn platform_name(&self) -> &str {
        "fanza"
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
            DomainError::InternalError(format!("FANZA credentials parse error: {e}"))
        })?;
        self.ensure_authenticated(
            creds.cookies,
            creds.username.as_deref(),
            creds.password.as_deref(),
        )
        .await?;

        #[cfg(feature = "real-plugins")]
        let products = self.fetch_library().await?;
        #[cfg(not(feature = "real-plugins"))]
        let products: Vec<FanzaProduct> = vec![];

        let items = products
            .into_iter()
            .map(|p| {
                let resource_type = detect_resource_type(&p.content_type);
                let mut metadata = std::collections::HashMap::new();
                if let Some(maker) = p.maker_name {
                    metadata.insert("maker".to_string(), maker);
                }
                metadata.insert(
                    "url".to_string(),
                    format!(
                        "https://www.dmm.com/digital/video/-/detail/=/cid={}/",
                        p.content_id
                    ),
                );
                metadata.insert("resource_type".to_string(), format!("{resource_type:?}"));
                DiscoveredItem {
                    external_id: p.content_id,
                    title: p.title,
                    platform: "fanza".to_string(),
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
    fn detect_resource_type_game_default() {
        assert_eq!(detect_resource_type("game"), ResourceType::Game);
        assert_eq!(detect_resource_type("doujin"), ResourceType::Game);
        assert_eq!(detect_resource_type(""), ResourceType::Game);
    }

    #[test]
    fn detect_resource_type_video_types() {
        assert_eq!(detect_resource_type("anime"), ResourceType::Video);
        assert_eq!(detect_resource_type("video"), ResourceType::Video);
        assert_eq!(detect_resource_type("MOV"), ResourceType::Video);
    }

    #[test]
    fn detect_resource_type_image_types() {
        assert_eq!(detect_resource_type("manga"), ResourceType::Image);
        assert_eq!(detect_resource_type("comic"), ResourceType::Image);
        assert_eq!(detect_resource_type("cg"), ResourceType::Image);
        assert_eq!(detect_resource_type("illust"), ResourceType::Image);
    }

    #[test]
    fn detect_resource_type_case_insensitive() {
        assert_eq!(detect_resource_type("Anime"), ResourceType::Video);
        assert_eq!(detect_resource_type("MANGA"), ResourceType::Image);
    }

    #[test]
    fn parse_library_response_valid_json() {
        let json = r#"[
            {"content_id": "abc001", "title": "Test Game", "content_type": "game", "maker_name": "Maker A"},
            {"content_id": "vid002", "title": "Test Video", "content_type": "anime", "maker_name": null}
        ]"#;
        let products = parse_library_response(json).unwrap();
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].content_id, "abc001");
        assert_eq!(products[0].content_type, "game");
        assert_eq!(products[0].maker_name, Some("Maker A".to_string()));
        assert_eq!(products[1].content_type, "anime");
        assert!(products[1].maker_name.is_none());
    }

    #[test]
    fn parse_library_response_empty_array() {
        let products = parse_library_response("[]").unwrap();
        assert!(products.is_empty());
    }

    #[test]
    fn parse_library_response_invalid_json_returns_error() {
        let result = parse_library_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn fanza_product_missing_optional_fields() {
        let json = r#"{"content_id": "x001", "title": "Game"}"#;
        let product: FanzaProduct = serde_json::from_str(json).unwrap();
        assert_eq!(product.content_type, "");
        assert!(product.maker_name.is_none());
    }

    #[tokio::test]
    async fn discover_items_with_noop_browser_returns_empty() {
        use crate::browser_session::NoopBrowserPage;
        let connector = FanzaConnector::new(NoopBrowserPage);
        let creds = serde_json::json!({"cookies": [], "username": "u", "password": "p"});
        let items = connector
            .discover_items(creds.to_string().as_bytes())
            .await
            .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn real_fixture_deserializes() {
        let json = include_str!("../../tests/fixtures/real/fanza_library.json");
        let products: Vec<FanzaProduct> = serde_json::from_str(json).expect("deserialize fixture");
        assert_eq!(products.len(), 3);
        assert_eq!(products[0].content_id, "d_123456");
        assert_eq!(products[0].title, "FANZA Game Alpha");
        assert_eq!(products[0].content_type, "game");
        assert_eq!(products[0].maker_name, Some("Maker X".to_string()));
        assert_eq!(products[1].content_type, "comic");
        assert_eq!(products[2].content_type, "video");
    }

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
}
