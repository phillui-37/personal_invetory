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
        let _ = tx.send(code.to_string());
        Ok(code.to_string())
    }

    pub async fn wait_for_otp(&self, platform: &str) -> Result<String, DomainError> {
        let rx = {
            let mut pending = self.pending.lock().await;
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