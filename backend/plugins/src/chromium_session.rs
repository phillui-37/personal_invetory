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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chromium_config_defaults() {
        let config = ChromiumConfig::default();
        assert_eq!(config.navigation_timeout_ms, 30_000);
        assert!(config.headless);
        assert!(config.chromium_path.is_none());
    }

    #[test]
    fn chromium_config_from_path() {
        let config = ChromiumConfig::with_path("/usr/bin/chromium".to_string());
        assert_eq!(config.chromium_path, Some("/usr/bin/chromium".to_string()));
        assert!(config.headless);
    }
}