// gog-core error module
// Ported from internal/googleapi/errors.go and internal/errfmt/errfmt.go

use std::time::Duration;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Exit code constants (matching Go version for script compatibility)
// ---------------------------------------------------------------------------

pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const USAGE_ERROR: i32 = 2;
    pub const AUTH_REQUIRED: i32 = 3;
    pub const NOT_FOUND: i32 = 4;
    pub const PERMISSION_DENIED: i32 = 5;
    pub const RATE_LIMITED: i32 = 6;
    pub const QUOTA_EXCEEDED: i32 = 7;
    pub const CIRCUIT_BREAKER: i32 = 8;
}

// ---------------------------------------------------------------------------
// GogError enum
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum GogError {
    /// Authentication required for a service/account.
    #[error("auth required for {service} {email}")]
    AuthRequired {
        service: String,
        email: String,
        client: Option<String>,
    },

    /// Rate limit exceeded.
    #[error("rate limit exceeded after {retries} retries")]
    RateLimited {
        retry_after: Option<Duration>,
        retries: u32,
    },

    /// Circuit breaker tripped.
    #[error("circuit breaker is open, too many recent failures - try again later")]
    CircuitBreakerOpen,

    /// API quota exceeded.
    #[error("API quota exceeded")]
    QuotaExceeded { resource: Option<String> },

    /// Resource not found.
    #[error("{resource} not found")]
    NotFound {
        resource: String,
        id: Option<String>,
    },

    /// Permission denied.
    #[error("permission denied")]
    PermissionDenied {
        resource: Option<String>,
        action: Option<String>,
    },

    /// Google API error (HTTP status code + message).
    #[error("Google API error ({code}): {message}")]
    GoogleApi {
        code: u16,
        message: String,
        reason: Option<String>,
    },

    /// User-facing message with optional cause.
    /// NOTE: String is not an Error so thiserror won't auto-treat it as source.
    #[error("{0}")]
    UserFacing(String),

    /// CLI usage error.
    #[error("{0}")]
    Usage(String),

    /// Config error.
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Catch-all.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// ---------------------------------------------------------------------------
// Methods
// ---------------------------------------------------------------------------

impl GogError {
    /// Map each variant to the appropriate process exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            GogError::AuthRequired { .. } => exit_codes::AUTH_REQUIRED,
            GogError::RateLimited { .. } => exit_codes::RATE_LIMITED,
            GogError::CircuitBreakerOpen => exit_codes::CIRCUIT_BREAKER,
            GogError::QuotaExceeded { .. } => exit_codes::QUOTA_EXCEEDED,
            GogError::NotFound { .. } => exit_codes::NOT_FOUND,
            GogError::PermissionDenied { .. } => exit_codes::PERMISSION_DENIED,
            GogError::GoogleApi { .. } => exit_codes::GENERAL_ERROR,
            GogError::UserFacing(_) => exit_codes::GENERAL_ERROR,
            GogError::Usage(_) => exit_codes::USAGE_ERROR,
            GogError::Config(_) => exit_codes::GENERAL_ERROR,
            GogError::Io(_) => exit_codes::GENERAL_ERROR,
            GogError::Other(_) => exit_codes::GENERAL_ERROR,
        }
    }

    /// Return a user-friendly message, matching Go's errfmt.Format behaviour.
    pub fn format_for_user(&self) -> String {
        match self {
            GogError::AuthRequired { service, email, .. } => {
                format!(
                    "No auth for {service} {email}.\n\nOAuth (browser flow):\n  gog auth add {email} --services {service}\n\nWorkspace service account (domain-wide delegation):\n  gog auth service-account set {email} --key <service-account.json>"
                )
            }

            GogError::Config(crate::config::ConfigError::CredentialsMissing { path }) => {
                format!(
                    "OAuth client credentials missing (OAuth client ID JSON).\nDownload from: https://console.cloud.google.com/apis/credentials (Create Credentials → OAuth client ID → Desktop app → Download JSON)\nThen run: gog auth credentials <credentials.json> (expected at {})",
                    path.display()
                )
            }

            GogError::Usage(msg) => {
                format!("{msg}\nRun with --help to see usage")
            }

            GogError::NotFound { resource, id } => match id {
                Some(id) => format!("{resource} not found: {id}"),
                None => format!("{resource} not found"),
            },

            GogError::GoogleApi { code, message, reason } => match reason {
                Some(r) => format!("Google API error ({code} {r}): {message}"),
                None => format!("Google API error ({code}): {message}"),
            },

            _ => self.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display overrides for variants with conditional formatting
// ---------------------------------------------------------------------------

// The #[error] attribute on NotFound only shows the resource.
// format_for_user handles id; for Display we also want to show id when present.
// We override via a custom Display using the existing thiserror-generated one
// but we need to give callers std::fmt::Display that matches Go's Error() output.
//
// Instead, we keep the #[error] simple and add a dedicated impl fmt::Display
// by relying on thiserror's generated one - it already handles the basic case.
// We override only where the Go version had conditional logic via format_for_user.

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // -----------------------------------------------------------------------
    // Exit code tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_auth_required_exit_code() {
        let err = GogError::AuthRequired {
            service: "gmail".to_string(),
            email: "user@example.com".to_string(),
            client: None,
        };
        assert_eq!(err.exit_code(), exit_codes::AUTH_REQUIRED);
    }

    #[test]
    fn test_not_found_exit_code() {
        let err = GogError::NotFound {
            resource: "message".to_string(),
            id: Some("abc123".to_string()),
        };
        assert_eq!(err.exit_code(), exit_codes::NOT_FOUND);
    }

    #[test]
    fn test_rate_limited_exit_code() {
        let err = GogError::RateLimited {
            retry_after: None,
            retries: 3,
        };
        assert_eq!(err.exit_code(), exit_codes::RATE_LIMITED);
    }

    #[test]
    fn test_general_error_exit_code() {
        let err = GogError::Io(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        assert_eq!(err.exit_code(), exit_codes::GENERAL_ERROR);
    }

    #[test]
    fn test_usage_exit_code() {
        let err = GogError::Usage("unknown flag --foo".to_string());
        assert_eq!(err.exit_code(), exit_codes::USAGE_ERROR);
    }

    #[test]
    fn test_circuit_breaker_exit_code() {
        assert_eq!(GogError::CircuitBreakerOpen.exit_code(), exit_codes::CIRCUIT_BREAKER);
    }

    #[test]
    fn test_quota_exceeded_exit_code() {
        let err = GogError::QuotaExceeded { resource: None };
        assert_eq!(err.exit_code(), exit_codes::QUOTA_EXCEEDED);
    }

    #[test]
    fn test_permission_denied_exit_code() {
        let err = GogError::PermissionDenied { resource: None, action: None };
        assert_eq!(err.exit_code(), exit_codes::PERMISSION_DENIED);
    }

    // -----------------------------------------------------------------------
    // Display / format_for_user tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_not_found_display() {
        let err = GogError::NotFound {
            resource: "message".to_string(),
            id: Some("abc123".to_string()),
        };
        // format_for_user shows the id
        assert_eq!(err.format_for_user(), "message not found: abc123");
    }

    #[test]
    fn test_not_found_display_no_id() {
        let err = GogError::NotFound {
            resource: "message".to_string(),
            id: None,
        };
        assert_eq!(err.format_for_user(), "message not found");
    }

    #[test]
    fn test_auth_required_format_for_user() {
        let err = GogError::AuthRequired {
            service: "gmail".to_string(),
            email: "user@example.com".to_string(),
            client: None,
        };
        let msg = err.format_for_user();
        assert!(msg.contains("gog auth add"), "expected 'gog auth add' in: {msg}");
        assert!(msg.contains("gmail"), "expected service name in: {msg}");
        assert!(msg.contains("user@example.com"), "expected email in: {msg}");
    }

    #[test]
    fn test_usage_format_for_user() {
        let err = GogError::Usage("unknown flag --foo".to_string());
        let msg = err.format_for_user();
        assert!(msg.contains("--help"), "expected '--help' in: {msg}");
        assert!(msg.contains("unknown flag --foo"), "expected original message in: {msg}");
    }

    #[test]
    fn test_rate_limited_display() {
        let err = GogError::RateLimited {
            retry_after: Some(Duration::from_secs(30)),
            retries: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("5"), "expected retry count in: {msg}");
    }

    #[test]
    fn test_rate_limited_display_no_retry_after() {
        let err = GogError::RateLimited {
            retry_after: None,
            retries: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("3"), "expected retry count in: {msg}");
    }

    #[test]
    fn test_google_api_format_for_user_with_reason() {
        let err = GogError::GoogleApi {
            code: 403,
            message: "Forbidden".to_string(),
            reason: Some("rateLimitExceeded".to_string()),
        };
        let msg = err.format_for_user();
        assert!(msg.contains("403"), "expected code in: {msg}");
        assert!(msg.contains("rateLimitExceeded"), "expected reason in: {msg}");
        assert!(msg.contains("Forbidden"), "expected message in: {msg}");
    }

    #[test]
    fn test_google_api_format_for_user_no_reason() {
        let err = GogError::GoogleApi {
            code: 500,
            message: "Internal Server Error".to_string(),
            reason: None,
        };
        let msg = err.format_for_user();
        assert_eq!(msg, "Google API error (500): Internal Server Error");
    }

    #[test]
    fn test_circuit_breaker_display() {
        let msg = GogError::CircuitBreakerOpen.to_string();
        assert!(msg.contains("circuit breaker"), "expected 'circuit breaker' in: {msg}");
    }

    #[test]
    fn test_user_facing_display() {
        let err = GogError::UserFacing("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
        assert_eq!(err.format_for_user(), "something went wrong");
    }

    #[test]
    fn test_config_credentials_missing_format_for_user() {
        use crate::config::ConfigError;
        let err = GogError::Config(ConfigError::CredentialsMissing {
            path: "/home/user/.config/gogcli/credentials.json".into(),
        });
        let msg = err.format_for_user();
        assert!(msg.contains("OAuth client credentials missing"), "got: {msg}");
        assert!(msg.contains("console.cloud.google.com"), "got: {msg}");
    }

    #[test]
    fn test_exit_code_constants() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::GENERAL_ERROR, 1);
        assert_eq!(exit_codes::USAGE_ERROR, 2);
        assert_eq!(exit_codes::AUTH_REQUIRED, 3);
        assert_eq!(exit_codes::NOT_FOUND, 4);
        assert_eq!(exit_codes::PERMISSION_DENIED, 5);
        assert_eq!(exit_codes::RATE_LIMITED, 6);
        assert_eq!(exit_codes::QUOTA_EXCEEDED, 7);
        assert_eq!(exit_codes::CIRCUIT_BREAKER, 8);
    }
}
