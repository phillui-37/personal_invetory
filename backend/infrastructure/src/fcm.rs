use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use domain::{DomainError, PushNotifier};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const JWT_EXPIRY_SECS: u64 = 3600;

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    project_id: String,
    client_email: String,
    private_key: String,
}

#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    exp: u64,
    iat: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FcmError {
    #[error("invalid service account JSON: {0}")]
    InvalidConfig(String),
    #[error("JWT signing failed: {0}")]
    JwtError(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
}

impl From<FcmError> for DomainError {
    fn from(e: FcmError) -> Self {
        DomainError::InternalError(e.to_string())
    }
}

#[derive(Debug)]
pub struct FcmClient {
    service_account: ServiceAccount,
    http: Client,
    /// Overrideable token endpoint for testing.
    token_url: String,
    /// Overrideable FCM base URL for testing.
    fcm_base_url: String,
}

impl FcmClient {
    pub fn new(service_account_json: &str) -> Result<Self, FcmError> {
        let service_account: ServiceAccount = serde_json::from_str(service_account_json)
            .map_err(|e| FcmError::InvalidConfig(e.to_string()))?;

        Ok(Self {
            service_account,
            http: Client::new(),
            token_url: GOOGLE_TOKEN_URL.to_string(),
            fcm_base_url: "https://fcm.googleapis.com".to_string(),
        })
    }

    /// For testing — override both endpoints.
    pub fn with_urls(mut self, token_url: &str, fcm_base_url: &str) -> Self {
        self.token_url = token_url.to_string();
        self.fcm_base_url = fcm_base_url.to_string();
        self
    }

    async fn get_access_token(&self) -> Result<String, FcmError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let claims = JwtClaims {
            iss: self.service_account.client_email.clone(),
            scope: FCM_SCOPE.to_string(),
            aud: self.token_url.clone(),
            iat: now,
            exp: now + JWT_EXPIRY_SECS,
        };

        let header = Header::new(Algorithm::RS256);
        let key = EncodingKey::from_rsa_pem(self.service_account.private_key.as_bytes())
            .map_err(|e| FcmError::JwtError(e.to_string()))?;
        let jwt = jsonwebtoken::encode(&header, &claims, &key)
            .map_err(|e| FcmError::JwtError(e.to_string()))?;

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ];

        let resp = self
            .http
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| FcmError::HttpError(e.to_string()))?;

        let token: TokenResponse = resp
            .json()
            .await
            .map_err(|e| FcmError::HttpError(e.to_string()))?;

        Ok(token.access_token)
    }

    pub async fn send_notification(
        &self,
        resource_id: Uuid,
        title: &str,
        body: &str,
    ) -> Result<(), FcmError> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v1/projects/{}/messages:send",
            self.fcm_base_url, self.service_account.project_id
        );

        let payload = serde_json::json!({
            "message": {
                "topic": format!("resource_{resource_id}"),
                "notification": {
                    "title": title,
                    "body": body
                },
                "data": {
                    "resource_id": resource_id.to_string()
                }
            }
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| FcmError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(FcmError::HttpError(format!("{status}: {text}")));
        }

        Ok(())
    }
}

#[async_trait]
impl PushNotifier for FcmClient {
    async fn send(
        &self,
        resource_id: Uuid,
        title: &str,
        body: &str,
    ) -> Result<(), DomainError> {
        self.send_notification(resource_id, title, body)
            .await
            .map_err(DomainError::from)
    }
}

/// Wrap FcmClient in Arc for easy DI.
pub struct ArcFcmClient(pub Arc<FcmClient>);

#[async_trait]
impl PushNotifier for ArcFcmClient {
    async fn send(
        &self,
        resource_id: Uuid,
        title: &str,
        body: &str,
    ) -> Result<(), DomainError> {
        self.0.send(resource_id, title, body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Minimal self-signed RSA key (2048 bit) for testing JWT creation.
    const TEST_PRIVATE_KEY: &str = include_str!("../tests/fixtures/test_rsa_private.pem");

    fn test_service_account_json(token_url: &str) -> String {
        serde_json::json!({
            "project_id": "test-project",
            "client_email": "test@test-project.iam.gserviceaccount.com",
            "private_key": TEST_PRIVATE_KEY,
            "token_uri": token_url
        })
        .to_string()
    }

    #[test]
    fn fcm_client_new_fails_gracefully_on_invalid_json() {
        let result = FcmClient::new("not valid json");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FcmError::InvalidConfig(_)));
    }

    #[test]
    fn fcm_client_new_fails_gracefully_on_missing_fields() {
        let result = FcmClient::new(r#"{"project_id": "p"}"#);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fcm_send_builds_correct_request_shape() {
        let server = MockServer::start().await;

        // Token endpoint mock
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"access_token": "fake-token", "token_type": "Bearer", "expires_in": 3600})),
            )
            .mount(&server)
            .await;

        // FCM send endpoint mock
        Mock::given(method("POST"))
            .and(path("/v1/projects/test-project/messages:send"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"name": "projects/test-project/messages/123"})),
            )
            .mount(&server)
            .await;

        let sa_json = test_service_account_json(&format!("{}/token", server.uri()));
        let client = FcmClient::new(&sa_json)
            .expect("build client")
            .with_urls(&format!("{}/token", server.uri()), &server.uri());

        let resource_id = Uuid::new_v4();
        let result = client
            .send_notification(resource_id, "New Chapter", "Chapter 42 is out!")
            .await;

        assert!(result.is_ok(), "expected ok, got: {result:?}");
    }

    #[tokio::test]
    async fn fcm_send_returns_error_on_non_success_status() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"access_token": "fake-token", "token_type": "Bearer", "expires_in": 3600})),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/projects/test-project/messages:send"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sa_json = test_service_account_json(&format!("{}/token", server.uri()));
        let client = FcmClient::new(&sa_json)
            .expect("build client")
            .with_urls(&format!("{}/token", server.uri()), &server.uri());

        let result = client
            .send_notification(Uuid::new_v4(), "title", "body")
            .await;

        assert!(matches!(result, Err(FcmError::HttpError(_))));
    }
}
