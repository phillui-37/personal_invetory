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
            let resp = self.client.get(url).send().await
                .map_err(|e| DomainError::InternalError(format!("HTTP request error: {e}")))?;
            let status = resp.status().as_u16();
            match classify_status(status) {
                ErrorClass::Success => {
                    return resp.json::<T>().await
                        .map_err(|e| DomainError::InternalError(format!("JSON parse error: {e}")));
                }
                ErrorClass::Permanent => {
                    return Err(DomainError::InternalError(format!("HTTP {status}: permanent error")));
                }
                ErrorClass::Transient => {
                    if attempt == self.config.max_retries {
                        return Err(DomainError::InternalError(format!("HTTP {status}: max retries exceeded")));
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
        let resp = self.client.get(url)
            .header("Cookie", cookie_header)
            .send().await
            .map_err(|e| DomainError::InternalError(format!("HTTP request error: {e}")))?;
        let status = resp.status().as_u16();
        let text = resp.text().await
            .map_err(|e| DomainError::InternalError(format!("HTTP read error: {e}")))?;
        Ok((status, text))
    }
}

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