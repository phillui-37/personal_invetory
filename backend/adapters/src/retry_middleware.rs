/// Retry middleware and error classification for ecosystem connectors.
/// Implements exponential backoff, credential expiration detection, and rate limiting.

use std::time::Duration;
use domain::DomainError;

/// Configuration for retry behavior.
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,    // 1 second
            max_backoff_ms: 8000,        // 8 seconds
        }
    }
}

/// Classification for HTTP errors: transient (retry) or permanent (fail).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Transient errors: 429, 5xx, timeout → should retry with backoff
    Transient,
    /// Permanent errors: 401, 403 → should fail immediately
    Permanent,
}

/// Classifies an HTTP status code as transient or permanent.
///
/// Transient errors (should retry):
/// - HTTP 429 (Rate Limit)
/// - HTTP 5xx (Server Error)
/// - Timeout
///
/// Permanent errors (should not retry):
/// - HTTP 401 (Unauthorized / Credential Expired)
/// - HTTP 403 (Forbidden / Access Denied)
pub fn classify_error(status_code: u16) -> ErrorClass {
    match status_code {
        401 | 403 => ErrorClass::Permanent,
        429 | 500..=599 => ErrorClass::Transient,
        _ => ErrorClass::Permanent,
    }
}

/// Calculates exponential backoff duration for a given attempt.
///
/// Formula: min(initial * 2^attempt, max_backoff)
///
/// Examples:
/// - attempt 0: 1s
/// - attempt 1: 2s
/// - attempt 2: 4s
/// - attempt 3+: capped at 8s
pub fn calculate_backoff(config: &RetryConfig, attempt: u32) -> Duration {
    let ms = std::cmp::min(
        config.initial_backoff_ms * 2u64.pow(attempt),
        config.max_backoff_ms,
    );
    Duration::from_millis(ms)
}

/// Detects credential expiration from an error.
/// Returns true if the error indicates expired credentials (HTTP 401).
pub fn is_credential_expired(status_code: u16) -> bool {
    status_code == 401
}

/// Detects access denied from an error.
/// Returns true if the error indicates access denied (HTTP 403).
pub fn is_access_denied(status_code: u16) -> bool {
    status_code == 403
}

/// Detects rate limiting from an error.
/// Returns true if the error indicates rate limiting (HTTP 429).
pub fn is_rate_limited(status_code: u16) -> bool {
    status_code == 429
}

/// Extracts retry-after delay from HTTP headers if present.
/// Returns the delay in seconds if found, None otherwise.
pub fn extract_retry_after(retry_after_header: Option<&str>) -> Option<u64> {
    retry_after_header.and_then(|header| {
        header.parse::<u64>().ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_error_401() {
        assert_eq!(classify_error(401), ErrorClass::Permanent);
    }

    #[test]
    fn test_classify_error_429() {
        assert_eq!(classify_error(429), ErrorClass::Transient);
    }

    #[test]
    fn test_classify_error_500() {
        assert_eq!(classify_error(500), ErrorClass::Transient);
    }

    #[test]
    fn test_calculate_backoff_exponential() {
        let config = RetryConfig::default();
        assert_eq!(calculate_backoff(&config, 0).as_millis(), 1000);
        assert_eq!(calculate_backoff(&config, 1).as_millis(), 2000);
        assert_eq!(calculate_backoff(&config, 2).as_millis(), 4000);
        assert_eq!(calculate_backoff(&config, 3).as_millis(), 8000);
        assert_eq!(calculate_backoff(&config, 4).as_millis(), 8000);  // capped
    }

    #[test]
    fn test_credential_expiration_detection() {
        assert!(is_credential_expired(401));
        assert!(!is_credential_expired(403));
        assert!(!is_credential_expired(429));
    }

    #[test]
    fn test_access_denied_detection() {
        assert!(is_access_denied(403));
        assert!(!is_access_denied(401));
        assert!(!is_access_denied(429));
    }

    #[test]
    fn test_rate_limit_detection() {
        assert!(is_rate_limited(429));
        assert!(!is_rate_limited(401));
        assert!(!is_rate_limited(403));
    }

    #[test]
    fn test_extract_retry_after() {
        assert_eq!(extract_retry_after(Some("60")), Some(60));
        assert_eq!(extract_retry_after(Some("invalid")), None);
        assert_eq!(extract_retry_after(None), None);
    }
}
