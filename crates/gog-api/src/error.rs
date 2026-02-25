// gog-api error module
// API-specific error types ported from internal/googleapi/errors.go

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("auth required for {service} {email}")]
    AuthRequired {
        service: String,
        email: String,
        client: String,
    },

    #[error("rate limit exceeded after {retries} retries")]
    RateLimitExhausted { retries: u32 },

    #[error("circuit breaker open - too many recent failures")]
    CircuitBreakerOpen,

    #[error("Google API error ({status}): {message}")]
    GoogleApi { status: u16, message: String },

    #[error(transparent)]
    Auth(#[from] gog_auth::AuthError),

    #[error(transparent)]
    Secrets(#[from] gog_secrets::SecretsError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Config(#[from] gog_core::config::ConfigError),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_required_display() {
        let err = ApiError::AuthRequired {
            service: "gmail".to_string(),
            email: "user@example.com".to_string(),
            client: "default".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("auth required for"), "got: {msg}");
        assert!(msg.contains("gmail"), "got: {msg}");
        assert!(msg.contains("user@example.com"), "got: {msg}");
    }

    #[test]
    fn test_rate_limit_display() {
        let err = ApiError::RateLimitExhausted { retries: 5 };
        let msg = err.to_string();
        assert!(msg.contains("rate limit exceeded"), "got: {msg}");
        assert!(msg.contains("5"), "got: {msg}");
    }

    #[test]
    fn test_circuit_breaker_display() {
        let err = ApiError::CircuitBreakerOpen;
        let msg = err.to_string();
        assert!(msg.contains("circuit breaker"), "got: {msg}");
    }

    #[test]
    fn test_google_api_display() {
        let err = ApiError::GoogleApi {
            status: 403,
            message: "Forbidden".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("403"), "got: {msg}");
        assert!(msg.contains("Forbidden"), "got: {msg}");
    }
}
