// gog-api transport module
// Retry transport and circuit breaker.
// Ported from internal/googleapi/transport.go and internal/googleapi/circuitbreaker.go

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::error::ApiError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const MAX_RATE_LIMIT_RETRIES: u32 = 5;
pub const MAX_5XX_RETRIES: u32 = 3;
pub const RATE_LIMIT_BASE_DELAY: Duration = Duration::from_secs(1);
pub const SERVER_ERROR_RETRY_DELAY: Duration = Duration::from_secs(2);
pub const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;
pub const CIRCUIT_BREAKER_RESET_DURATION: Duration = Duration::from_secs(60);

// ---------------------------------------------------------------------------
// CircuitBreaker
// ---------------------------------------------------------------------------

/// Circuit breaker that opens after a threshold of failures and auto-resets
/// after `reset_duration` has elapsed since the last failure.
pub struct CircuitBreaker {
    failures: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    threshold: u32,
    reset_duration: Duration,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            failures: AtomicU32::new(0),
            last_failure: Mutex::new(None),
            threshold: CIRCUIT_BREAKER_THRESHOLD,
            reset_duration: CIRCUIT_BREAKER_RESET_DURATION,
        }
    }

    /// Returns true if the circuit breaker is open (failures >= threshold AND
    /// within the reset window).
    pub fn is_open(&self) -> bool {
        let failures = self.failures.load(Ordering::SeqCst);
        if failures < self.threshold {
            return false;
        }
        // Check if reset duration has elapsed since last failure
        let guard = self.last_failure.lock().unwrap();
        match *guard {
            None => false,
            Some(last) => {
                if last.elapsed() > self.reset_duration {
                    // Auto-reset: failures >= threshold but reset window expired
                    // We report closed and let record_success clean up on next success
                    false
                } else {
                    true
                }
            }
        }
    }

    /// Reset the failure count to 0 (called on a successful request).
    pub fn record_success(&self) {
        self.failures.store(0, Ordering::SeqCst);
        let mut guard = self.last_failure.lock().unwrap();
        *guard = None;
    }

    /// Increment the failure count and record the failure time.
    pub fn record_failure(&self) {
        let count = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        let mut guard = self.last_failure.lock().unwrap();
        *guard = Some(Instant::now());
        if count >= self.threshold {
            warn!("circuit breaker opened after {} failures", count);
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// RetryConfig
// ---------------------------------------------------------------------------

/// Configuration for the retry logic in `execute_with_retry`.
pub struct RetryConfig {
    pub max_retries_429: u32,
    pub max_retries_5xx: u32,
    pub base_delay: Duration,
    pub circuit_breaker: Option<CircuitBreaker>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries_429: MAX_RATE_LIMIT_RETRIES,
            max_retries_5xx: MAX_5XX_RETRIES,
            base_delay: RATE_LIMIT_BASE_DELAY,
            circuit_breaker: Some(CircuitBreaker::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Backoff calculation
// ---------------------------------------------------------------------------

/// Calculate the delay before the next retry attempt.
///
/// - Checks `Retry-After` header (integer seconds, or HTTP date).
/// - Falls back to exponential backoff with jitter: `base * 2^attempt + rand(0..base/2)`.
pub fn calculate_backoff(
    attempt: u32,
    base_delay: Duration,
    retry_after: Option<&str>,
) -> Duration {
    if let Some(header) = retry_after {
        let trimmed = header.trim();
        // Try parsing as integer seconds
        if let Ok(secs) = trimmed.parse::<i64>() {
            if secs <= 0 {
                return Duration::ZERO;
            }
            return Duration::from_secs(secs as u64);
        }
        // Try parsing as HTTP date (RFC 2822 / RFC 7231 style)
        if let Ok(dt) = httpdate::parse_http_date(trimmed) {
            let now = std::time::SystemTime::now();
            match dt.duration_since(now) {
                Ok(d) => return d,
                Err(_) => return Duration::ZERO, // date is in the past
            }
        }
        // Unknown Retry-After format - fall through to exponential backoff
    }

    // Exponential backoff with jitter
    if base_delay.is_zero() {
        return Duration::ZERO;
    }

    let factor = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let base_ms = base_delay.as_millis() as u64;
    let exp_ms = base_ms.saturating_mul(factor);
    if exp_ms == 0 {
        return Duration::ZERO;
    }

    // Jitter: random in [0, base/2)
    let jitter_range_ms = base_ms / 2;
    let jitter_ms = if jitter_range_ms > 0 {
        // Use a simple deterministic-ish approach safe for non-crypto use.
        // We mix the attempt and a nonce from the system clock low bits.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0) as u64;
        (nonce.wrapping_add(attempt as u64 * 6364136223846793005)) % jitter_range_ms
    } else {
        0
    };

    Duration::from_millis(exp_ms + jitter_ms)
}

// ---------------------------------------------------------------------------
// execute_with_retry
// ---------------------------------------------------------------------------

/// Execute an HTTP request with retry logic for 429 and 5xx responses.
///
/// - Checks the circuit breaker before sending.
/// - Clones the request via `try_clone()` so it can be replayed on retry.
/// - On 429: parses `Retry-After` header, exponential backoff, retries up to `max_retries_429`.
/// - On 5xx: records a circuit breaker failure, fixed delay, retries up to `max_retries_5xx`.
/// - On success (< 400): records circuit breaker success and returns.
/// - On other 4xx: returns the response as-is.
pub async fn execute_with_retry(
    client: &reqwest::Client,
    request_builder: reqwest::RequestBuilder,
    config: &RetryConfig,
) -> Result<reqwest::Response, ApiError> {
    // Check circuit breaker before doing anything
    if let Some(cb) = &config.circuit_breaker {
        if cb.is_open() {
            return Err(ApiError::CircuitBreakerOpen);
        }
    }

    // Build the initial request; we need a cloneable `reqwest::Request`.
    let request = request_builder.build()?;

    let mut retries_429: u32 = 0;
    let mut retries_5xx: u32 = 0;

    loop {
        // Clone the request for this attempt (try_clone returns None for streaming bodies)
        let attempt_req = request.try_clone().ok_or_else(|| ApiError::GoogleApi {
            status: 0,
            message: "request body is not cloneable (streaming body)".to_string(),
        })?;

        let resp = client.execute(attempt_req).await?;
        let status = resp.status().as_u16();

        // Success
        if status < 400 {
            if let Some(cb) = &config.circuit_breaker {
                cb.record_success();
            }
            return Ok(resp);
        }

        // Rate limit (429)
        if status == 429 {
            if retries_429 >= config.max_retries_429 {
                return Err(ApiError::RateLimitExhausted {
                    retries: retries_429,
                });
            }

            let retry_after_header = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let delay = calculate_backoff(
                retries_429,
                config.base_delay,
                retry_after_header.as_deref(),
            );

            debug!(
                delay_ms = delay.as_millis(),
                attempt = retries_429 + 1,
                max = config.max_retries_429,
                "rate limited, retrying"
            );

            // Consume response body to free the connection
            drop(resp);

            tokio::time::sleep(delay).await;
            retries_429 += 1;
            continue;
        }

        // Server error (5xx)
        if status >= 500 {
            if let Some(cb) = &config.circuit_breaker {
                cb.record_failure();
            }

            if retries_5xx >= config.max_retries_5xx {
                return Ok(resp);
            }

            debug!(status, attempt = retries_5xx + 1, "server error, retrying");

            drop(resp);
            tokio::time::sleep(SERVER_ERROR_RETRY_DELAY).await;
            retries_5xx += 1;
            continue;
        }

        // Other 4xx: return as-is
        return Ok(resp);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // -----------------------------------------------------------------
    // CircuitBreaker tests
    // -----------------------------------------------------------------

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new();
        assert!(!cb.is_open(), "new circuit breaker should be closed");
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new();
        for _ in 0..CIRCUIT_BREAKER_THRESHOLD {
            cb.record_failure();
        }
        assert!(
            cb.is_open(),
            "circuit breaker should be open after {} failures",
            CIRCUIT_BREAKER_THRESHOLD
        );
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new();
        for _ in 0..CIRCUIT_BREAKER_THRESHOLD {
            cb.record_failure();
        }
        assert!(cb.is_open(), "should be open before reset");
        cb.record_success();
        assert!(!cb.is_open(), "should be closed after record_success");
    }

    #[test]
    fn test_circuit_breaker_auto_resets_after_duration() {
        // Use a circuit breaker with a zero reset duration so it's already expired.
        // (Avoids Instant subtraction overflow on Windows where Instant can't
        // go before process start time.)
        let cb = CircuitBreaker {
            failures: AtomicU32::new(CIRCUIT_BREAKER_THRESHOLD),
            last_failure: Mutex::new(Some(Instant::now())),
            threshold: CIRCUIT_BREAKER_THRESHOLD,
            reset_duration: Duration::ZERO,
        };
        // Even though failures >= threshold, the reset duration has passed
        assert!(
            !cb.is_open(),
            "circuit breaker should auto-reset after reset_duration"
        );
    }

    #[test]
    fn test_circuit_breaker_stays_closed_below_threshold() {
        let cb = CircuitBreaker::new();
        for _ in 0..(CIRCUIT_BREAKER_THRESHOLD - 1) {
            cb.record_failure();
        }
        assert!(
            !cb.is_open(),
            "circuit breaker should stay closed below threshold"
        );
    }

    // -----------------------------------------------------------------
    // calculate_backoff tests
    // -----------------------------------------------------------------

    #[test]
    fn test_calculate_backoff_base() {
        let base = Duration::from_millis(100);
        // attempt 0 → 100ms base + jitter (0..50ms) → range [100, 150)
        let d = calculate_backoff(0, base, None);
        assert!(
            d >= base && d < base + base / 2 + Duration::from_millis(1),
            "attempt 0 should be roughly base_delay, got {:?}",
            d
        );
    }

    #[test]
    fn test_calculate_backoff_exponential() {
        let base = Duration::from_millis(100);
        // attempt 2 → 400ms base + jitter
        let d = calculate_backoff(2, base, None);
        let expected_base = base * 4; // 400ms
        assert!(
            d >= expected_base,
            "attempt 2 should be >= 400ms, got {:?}",
            d
        );
    }

    #[test]
    fn test_calculate_backoff_retry_after_seconds() {
        let base = Duration::from_secs(1);
        let d = calculate_backoff(0, base, Some("5"));
        assert_eq!(
            d,
            Duration::from_secs(5),
            "Retry-After: 5 should give 5s delay"
        );
    }

    #[test]
    fn test_calculate_backoff_zero_base() {
        let d = calculate_backoff(0, Duration::ZERO, None);
        assert_eq!(
            d,
            Duration::ZERO,
            "zero base_delay should give zero backoff"
        );
    }

    #[test]
    fn test_calculate_backoff_negative_retry_after() {
        let d = calculate_backoff(0, Duration::from_secs(1), Some("-1"));
        assert_eq!(
            d,
            Duration::ZERO,
            "negative Retry-After should give zero delay"
        );
    }

    // -----------------------------------------------------------------
    // RetryConfig tests
    // -----------------------------------------------------------------

    #[test]
    fn test_retry_config_default() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_retries_429, MAX_RATE_LIMIT_RETRIES);
        assert_eq!(cfg.max_retries_5xx, MAX_5XX_RETRIES);
        assert_eq!(cfg.base_delay, RATE_LIMIT_BASE_DELAY);
        assert!(
            cfg.circuit_breaker.is_some(),
            "default should have circuit breaker"
        );
    }
}
