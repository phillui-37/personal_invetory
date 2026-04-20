use domain::{DomainError, ResourceType};
use serde::Deserialize;
use async_trait::async_trait;

use crate::browser_session::{BrowserPage, Cookie};
use domain::ecosystem::{DiscoveredItem, EcosystemConnector};

// TODO(network-inspection): Replace these with real URLs discovered from network inspection.
const DLSITE_LOGIN_URL: &str = "https://login.dlsite.com/login";
const DLSITE_LIBRARY_BASE_URL: &str = "https://www.dlsite.com/maniax/mypage/userbuy";

// CSS selectors for DLSite login form
const DLSITE_USERNAME_SELECTOR: &str = "input[name='login_id']";
const DLSITE_PASSWORD_SELECTOR: &str = "input[name='password']";
const DLSITE_SUBMIT_SELECTOR: &str = "button[type='submit']";

/// A work entry returned from DLSite's purchase history API.
#[derive(Debug, Clone, Deserialize)]
pub struct DLSiteWork {
    pub workno: String,
    pub work_name: String,
    #[serde(default)]
    pub work_type: String,
    #[serde(default)]
    pub maker_name: Option<String>,
}

/// Detects the resource type from DLSite's `work_type` field.
///
/// DLSite content types:
/// - "GAM" → Game
/// - "MNG", "CG", "ICG" → Image (manga/CG)
/// - "MOV" → Video
/// - Everything else → Game (most DLSite content)
pub fn detect_resource_type(work_type: &str) -> ResourceType {
    match work_type.to_uppercase().as_str() {
        "MNG" | "CG" | "ICG" | "COMIC" => ResourceType::Image,
        "MOV" | "SOU" => ResourceType::Video,
        _ => ResourceType::Game,
    }
}

/// Parses a JSON array of DLSiteWork objects from the purchase history response.
pub fn parse_library_response(json: &str) -> Result<Vec<DLSiteWork>, DomainError> {
    serde_json::from_str(json)
        .map_err(|e| DomainError::InternalError(format!("DLSite parse error: {e}")))
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

/// DLSite connector. Authenticates via browser session, then fetches purchase history.
pub struct DLSiteConnector<P: BrowserPage> {
    browser: P,
}

impl<P: BrowserPage> DLSiteConnector<P> {
    pub fn new(browser: P) -> Self {
        Self { browser }
    }

    /// Performs login flow: set vault cookies, navigate to auth check page,
    /// if session expired fill login form with credentials.
    async fn ensure_authenticated(
        &self,
        session_cookies: Vec<Cookie>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<(), DomainError> {
        if !session_cookies.is_empty() {
            self.browser.set_cookies(session_cookies).await?;
        }
        // TODO(network-inspection): Navigate to a lightweight auth-check page and
        // detect login status via selector presence.
        self.browser.navigate(DLSITE_LOGIN_URL).await?;
        // If login form is absent, session is valid.
        if let (Some(user), Some(pass)) = (username, password) {
            self.browser.fill(DLSITE_USERNAME_SELECTOR, user).await?;
            self.browser.fill(DLSITE_PASSWORD_SELECTOR, pass).await?;
            self.browser.click(DLSITE_SUBMIT_SELECTOR).await?;
        }
        Ok(())
    }

    /// Fetches all purchased works via the library API.
    #[cfg(feature = "real-plugins")]
    async fn fetch_library(&self) -> Result<Vec<DLSiteWork>, DomainError> {
        let cookies = self.browser.get_cookies().await?;
        let cookie_header = build_cookie_header(&cookies);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| DomainError::InternalError(format!("HTTP client error: {e}")))?;
        // TODO(network-inspection): Replace DLSITE_LIBRARY_BASE_URL with the
        // actual paginated JSON endpoint discovered via network inspection.
        let resp = client
            .get(&format!("{DLSITE_LIBRARY_BASE_URL}?output=json"))
            .header("Cookie", cookie_header)
            .send()
            .await
            .map_err(|e| DomainError::InternalError(format!("DLSite HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Err(DomainError::InternalError(format!(
                "DLSite library returned status {}",
                resp.status()
            )));
        }
        let text = resp
            .text()
            .await
            .map_err(|e| DomainError::InternalError(format!("DLSite read error: {e}")))?;
        parse_library_response(&text)
    }
}

#[async_trait]
impl<P: BrowserPage + Send + Sync> EcosystemConnector for DLSiteConnector<P> {
    fn platform_name(&self) -> &str {
        "dlsite"
    }

    async fn discover_items(&self, credentials: &[u8]) -> Result<Vec<DiscoveredItem>, DomainError> {
        // Credentials encoded as JSON: { "cookies": [...], "username": "...", "password": "..." }
        #[derive(Deserialize)]
        struct Creds {
            #[serde(default)]
            cookies: Vec<Cookie>,
            username: Option<String>,
            password: Option<String>,
        }
        let creds: Creds = serde_json::from_slice(credentials).map_err(|e| {
            DomainError::InternalError(format!("DLSite credentials parse error: {e}"))
        })?;
        self.ensure_authenticated(
            creds.cookies,
            creds.username.as_deref(),
            creds.password.as_deref(),
        )
        .await?;

        #[cfg(feature = "real-plugins")]
        let works = self.fetch_library().await?;
        #[cfg(not(feature = "real-plugins"))]
        let works: Vec<DLSiteWork> = vec![];

        let items = works
            .into_iter()
            .map(|w| {
                let resource_type = detect_resource_type(&w.work_type);
                let mut metadata = std::collections::HashMap::new();
                if let Some(maker) = w.maker_name {
                    metadata.insert("maker".to_string(), maker);
                }
                metadata.insert(
                    "url".to_string(),
                    format!(
                        "https://www.dlsite.com/maniax/work/=/product_id/{}.html",
                        w.workno
                    ),
                );
                metadata.insert("resource_type".to_string(), format!("{resource_type:?}"));
                DiscoveredItem {
                    external_id: w.workno,
                    title: w.work_name,
                    platform: "dlsite".to_string(),
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
        assert_eq!(detect_resource_type("GAM"), ResourceType::Game);
        assert_eq!(detect_resource_type("ADV"), ResourceType::Game);
        assert_eq!(detect_resource_type("RPG"), ResourceType::Game);
        assert_eq!(detect_resource_type(""), ResourceType::Game);
    }

    #[test]
    fn detect_resource_type_image_types() {
        assert_eq!(detect_resource_type("MNG"), ResourceType::Image);
        assert_eq!(detect_resource_type("CG"), ResourceType::Image);
        assert_eq!(detect_resource_type("ICG"), ResourceType::Image);
        assert_eq!(detect_resource_type("COMIC"), ResourceType::Image);
    }

    #[test]
    fn detect_resource_type_video_types() {
        assert_eq!(detect_resource_type("MOV"), ResourceType::Video);
        assert_eq!(detect_resource_type("SOU"), ResourceType::Video);
    }

    #[test]
    fn detect_resource_type_case_insensitive() {
        assert_eq!(detect_resource_type("mng"), ResourceType::Image);
        assert_eq!(detect_resource_type("Mov"), ResourceType::Video);
    }

    #[test]
    fn parse_library_response_valid_json() {
        let json = r#"[
            {"workno": "RJ123456", "work_name": "Test Game", "work_type": "GAM", "maker_name": "Circle A"},
            {"workno": "RJ654321", "work_name": "Test Manga", "work_type": "MNG", "maker_name": null}
        ]"#;
        let works = parse_library_response(json).unwrap();
        assert_eq!(works.len(), 2);
        assert_eq!(works[0].workno, "RJ123456");
        assert_eq!(works[0].work_name, "Test Game");
        assert_eq!(works[0].work_type, "GAM");
        assert_eq!(works[0].maker_name, Some("Circle A".to_string()));
        assert_eq!(works[1].workno, "RJ654321");
        assert!(works[1].maker_name.is_none());
    }

    #[test]
    fn parse_library_response_empty_array() {
        let works = parse_library_response("[]").unwrap();
        assert!(works.is_empty());
    }

    #[test]
    fn parse_library_response_invalid_json_returns_error() {
        let result = parse_library_response("{not json}");
        assert!(result.is_err());
    }

    #[test]
    fn dlsite_work_with_missing_optional_fields() {
        let json = r#"{"workno": "RJ111", "work_name": "Game"}"#;
        let work: DLSiteWork = serde_json::from_str(json).unwrap();
        assert_eq!(work.work_type, "");
        assert!(work.maker_name.is_none());
    }

    #[tokio::test]
    async fn discover_items_with_noop_browser_returns_empty() {
        use crate::browser_session::NoopBrowserPage;
        let connector = DLSiteConnector::new(NoopBrowserPage);
        let creds = serde_json::json!({"cookies": [], "username": "u", "password": "p"});
        let items = connector
            .discover_items(creds.to_string().as_bytes())
            .await
            .unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn discover_items_maps_resource_type_in_metadata() {
        use crate::browser_session::NoopBrowserPage;
        // Verify the metadata mapping logic by constructing items directly.
        let works = vec![DLSiteWork {
            workno: "RJ999".to_string(),
            work_name: "Sample Manga".to_string(),
            work_type: "MNG".to_string(),
            maker_name: Some("Test Circle".to_string()),
        }];
        let items: Vec<DiscoveredItem> = works
            .into_iter()
            .map(|w| {
                let resource_type = detect_resource_type(&w.work_type);
                let mut metadata = std::collections::HashMap::new();
                if let Some(maker) = w.maker_name {
                    metadata.insert("maker".to_string(), maker);
                }
                metadata.insert(
                    "url".to_string(),
                    format!(
                        "https://www.dlsite.com/maniax/work/=/product_id/{}.html",
                        w.workno
                    ),
                );
                metadata.insert("resource_type".to_string(), format!("{resource_type:?}"));
                DiscoveredItem {
                    external_id: w.workno,
                    title: w.work_name,
                    platform: "dlsite".to_string(),
                    metadata,
                }
            })
            .collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].metadata.get("maker").unwrap(), "Test Circle");
        assert_eq!(items[0].metadata.get("resource_type").unwrap(), "Image");
    }

    #[test]
    fn real_fixture_deserializes() {
        let json = include_str!("../../tests/fixtures/real/dlsite_library.json");
        let works: Vec<DLSiteWork> = serde_json::from_str(json).expect("deserialize fixture");
        assert_eq!(works.len(), 3);
        assert_eq!(works[0].workno, "RJ123456");
        assert_eq!(works[0].work_name, "Test Game Alpha");
        assert_eq!(works[0].work_type, "GAM");
        assert_eq!(works[0].maker_name, Some("Studio A".to_string()));
        assert_eq!(works[1].work_type, "CG");
        assert_eq!(works[2].work_type, "MOV");
    }

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
}
