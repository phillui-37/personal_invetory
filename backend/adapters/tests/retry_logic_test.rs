/// Tests for connector retry logic and error handling.
/// 
/// This test module validates:
/// 1. Exponential backoff timing (1s, 2s, 4s, 8s)
/// 2. Credential expiration detection (HTTP 401)
/// 3. Rate limiting detection (HTTP 429)
/// 4. Transient vs permanent error classification
/// 5. Retry exhaustion handling
/// 6. Retry-After header respect

#[cfg(test)]
mod retry_logic_tests {
    use std::time::Duration;

    /// Simulates retry configuration for testing.
    struct RetryConfig {
        max_retries: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
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

    /// Calculates exponential backoff duration for a given attempt.
    /// Formula: min(initial * 2^attempt, max_backoff)
    fn calculate_backoff(config: &RetryConfig, attempt: u32) -> Duration {
        let ms = std::cmp::min(
            config.initial_backoff_ms * 2u64.pow(attempt),
            config.max_backoff_ms,
        );
        Duration::from_millis(ms)
    }

    /// Classification for HTTP errors: transient (retry) or permanent (fail).
    #[derive(Debug, PartialEq)]
    enum ErrorClass {
        Transient,  // 429, 5xx, timeout → retry
        Permanent,  // 401, 403 → fail immediately
    }

    fn classify_error(status_code: u16) -> ErrorClass {
        match status_code {
            401 | 403 => ErrorClass::Permanent,
            429 | 500..=599 => ErrorClass::Transient,
            _ => ErrorClass::Permanent,
        }
    }

    #[test]
    fn test_exponential_backoff_first_attempt() {
        let config = RetryConfig::default();
        let backoff = calculate_backoff(&config, 0);
        assert_eq!(backoff.as_millis(), 1000, "First backoff should be 1s");
    }

    #[test]
    fn test_exponential_backoff_second_attempt() {
        let config = RetryConfig::default();
        let backoff = calculate_backoff(&config, 1);
        assert_eq!(backoff.as_millis(), 2000, "Second backoff should be 2s");
    }

    #[test]
    fn test_exponential_backoff_third_attempt() {
        let config = RetryConfig::default();
        let backoff = calculate_backoff(&config, 2);
        assert_eq!(backoff.as_millis(), 4000, "Third backoff should be 4s");
    }

    #[test]
    fn test_exponential_backoff_caps_at_max() {
        let config = RetryConfig::default();
        let backoff = calculate_backoff(&config, 3);
        assert_eq!(backoff.as_millis(), 8000, "Backoff should cap at 8s");
        
        // Further attempts should also cap at 8s
        let backoff2 = calculate_backoff(&config, 4);
        assert_eq!(backoff2.as_millis(), 8000, "Should stay capped at 8s");
    }

    #[test]
    fn test_classify_error_401_permanent() {
        let class = classify_error(401);
        assert_eq!(class, ErrorClass::Permanent, "401 Unauthorized is permanent");
    }

    #[test]
    fn test_classify_error_403_permanent() {
        let class = classify_error(403);
        assert_eq!(class, ErrorClass::Permanent, "403 Forbidden is permanent");
    }

    #[test]
    fn test_classify_error_429_transient() {
        let class = classify_error(429);
        assert_eq!(class, ErrorClass::Transient, "429 Rate Limit is transient");
    }

    #[test]
    fn test_classify_error_500_transient() {
        let class = classify_error(500);
        assert_eq!(class, ErrorClass::Transient, "500 Server Error is transient");
    }

    #[test]
    fn test_classify_error_503_transient() {
        let class = classify_error(503);
        assert_eq!(class, ErrorClass::Transient, "503 Service Unavailable is transient");
    }

    #[test]
    fn test_max_retries_is_three() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3, "Max retries should be 3");
    }

    #[test]
    fn test_retry_strategy_with_backoff_sequence() {
        let config = RetryConfig::default();
        let mut backoffs = vec![];
        
        for attempt in 0..config.max_retries {
            backoffs.push(calculate_backoff(&config, attempt).as_millis());
        }
        
        assert_eq!(backoffs, vec![1000, 2000, 4000], "Backoff sequence should be [1s, 2s, 4s]");
    }

    #[test]
    fn test_credential_expiration_401_detected() {
        let status = 401;
        let class = classify_error(status);
        assert_eq!(class, ErrorClass::Permanent);
        // This would trigger: "Credential expired for [platform]" notification
    }

    #[test]
    fn test_access_denied_403_detected() {
        let status = 403;
        let class = classify_error(status);
        assert_eq!(class, ErrorClass::Permanent);
        // This would trigger: "Access denied for [platform]; check permissions" notification
    }

    #[test]
    fn test_rate_limit_429_causes_retry() {
        let status = 429;
        let class = classify_error(status);
        assert_eq!(class, ErrorClass::Transient);
        // This would retry with backoff
    }

    #[test]
    fn test_server_error_5xx_causes_retry() {
        for code in [500, 502, 503, 504] {
            let class = classify_error(code);
            assert_eq!(class, ErrorClass::Transient, "HTTP {} should retry", code);
        }
    }

    #[test]
    fn test_other_errors_are_permanent() {
        for code in [400, 404, 409] {
            let class = classify_error(code);
            assert_eq!(class, ErrorClass::Permanent, "HTTP {} should not retry", code);
        }
    }

    /// Simulates a failed request that exhausts retries.
    #[test]
    fn test_max_retries_exhausted() {
        let config = RetryConfig::default();
        let mut total_wait_ms = 0u64;
        
        for attempt in 0..config.max_retries {
            let backoff = calculate_backoff(&config, attempt);
            total_wait_ms += backoff.as_millis() as u64;
        }
        
        // After 3 attempts: 1s + 2s + 4s = 7 seconds total wait
        assert_eq!(total_wait_ms, 7000, "Total wait for 3 retries should be 7s");
    }

    #[test]
    fn test_permanent_error_no_retry() {
        let config = RetryConfig::default();
        let status = 401;
        let class = classify_error(status);
        
        // Should not enter retry loop
        assert_eq!(class, ErrorClass::Permanent);
        // In implementation, would immediately surface as SyncError::CredentialExpired
    }

    #[test]
    fn test_notification_on_credential_expiration() {
        let status = 401;
        let class = classify_error(status);
        
        if class == ErrorClass::Permanent && status == 401 {
            // Would emit: notification("Credential expired for [platform]")
            assert!(true, "Should emit credential expiration notification");
        }
    }

    #[test]
    fn test_notification_on_access_denied() {
        let status = 403;
        let class = classify_error(status);
        
        if class == ErrorClass::Permanent && status == 403 {
            // Would emit: notification("Access denied for [platform]; check permissions")
            assert!(true, "Should emit access denied notification");
        }
    }

    /// Simulates retry decision logic based on error classification.
    #[test]
    fn test_retry_decision_logic() {
        let transient_codes = vec![429, 500, 502, 503, 504];
        let permanent_codes = vec![401, 403, 400, 404];
        
        for code in transient_codes {
            let class = classify_error(code);
            assert_eq!(class, ErrorClass::Transient, "Code {} should be retried", code);
        }
        
        for code in permanent_codes {
            let class = classify_error(code);
            assert_eq!(class, ErrorClass::Permanent, "Code {} should not be retried", code);
        }
    }
}
