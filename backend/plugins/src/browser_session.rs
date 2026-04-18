use domain::DomainError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[async_trait::async_trait]
pub trait BrowserPage: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<(), DomainError>;
    async fn wait_for_selector(&self, css: &str, timeout_ms: u64) -> Result<(), DomainError>;
    async fn extract_text(&self, css: &str) -> Result<String, DomainError>;
    async fn extract_all_text(&self, css: &str) -> Result<Vec<String>, DomainError>;
    async fn extract_html(&self, css: &str) -> Result<String, DomainError>;
    async fn click(&self, css: &str) -> Result<(), DomainError>;
    async fn fill(&self, css: &str, value: &str) -> Result<(), DomainError>;
    async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError>;
    async fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<(), DomainError>;
}

pub struct NoopBrowserPage;

#[async_trait::async_trait]
impl BrowserPage for NoopBrowserPage {
    async fn navigate(&self, _url: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn wait_for_selector(&self, _css: &str, _timeout_ms: u64) -> Result<(), DomainError> {
        Ok(())
    }
    async fn extract_text(&self, _css: &str) -> Result<String, DomainError> {
        Ok(String::new())
    }
    async fn extract_all_text(&self, _css: &str) -> Result<Vec<String>, DomainError> {
        Ok(vec![])
    }
    async fn extract_html(&self, _css: &str) -> Result<String, DomainError> {
        Ok(String::new())
    }
    async fn click(&self, _css: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn fill(&self, _css: &str, _value: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn get_cookies(&self) -> Result<Vec<Cookie>, DomainError> {
        Ok(vec![])
    }
    async fn set_cookies(&self, _cookies: Vec<Cookie>) -> Result<(), DomainError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_serializes_to_json() {
        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: ".example.com".to_string(),
            path: "/".to_string(),
        };
        let json = serde_json::to_string(&cookie).expect("serialize");
        assert!(json.contains("session"));
        assert!(json.contains("abc123"));
    }

    #[tokio::test]
    async fn noop_browser_page_returns_ok() {
        let page = NoopBrowserPage;
        page.navigate("https://example.com").await.unwrap();
        let text = page.extract_text(".title").await.unwrap();
        assert!(text.is_empty());
    }
}
