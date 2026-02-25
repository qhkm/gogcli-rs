// gog-secrets store module
// Ported from internal/secrets/store.go

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use gog_core::config::normalize_client_name_or_default;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("missing email")]
    MissingEmail,

    #[error("missing refresh token")]
    MissingRefreshToken,

    #[error("missing secret key")]
    MissingKey,

    #[error("invalid client: {0}")]
    InvalidClient(String),

    #[error("keyring error: {0}")]
    Keyring(String),

    #[error("serialize error: {0}")]
    Serialize(String),

    #[error("deserialize error: {0}")]
    Deserialize(String),
}

// ---------------------------------------------------------------------------
// Token types
// ---------------------------------------------------------------------------

/// Public token struct. `refresh_token` is NOT serialized to JSON (kept separate
/// in the keyring payload via `StoredToken`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub client: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    /// NOT serialized – stored separately in keyring JSON via `StoredToken`.
    #[serde(skip)]
    pub refresh_token: String,
}

/// Internal struct serialized into the keyring JSON payload.
#[derive(Debug, Serialize, Deserialize)]
struct StoredToken {
    pub refresh_token: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Store trait
// ---------------------------------------------------------------------------

pub trait Store {
    fn keys(&self) -> Result<Vec<String>, SecretsError>;
    fn set_token(&self, client: &str, email: &str, token: &Token) -> Result<(), SecretsError>;
    fn get_token(&self, client: &str, email: &str) -> Result<Token, SecretsError>;
    fn delete_token(&self, client: &str, email: &str) -> Result<(), SecretsError>;
    fn list_tokens(&self) -> Result<Vec<Token>, SecretsError>;
    fn get_default_account(&self, client: &str) -> Result<Option<String>, SecretsError>;
    fn set_default_account(&self, client: &str, email: &str) -> Result<(), SecretsError>;
}

// ---------------------------------------------------------------------------
// Key format helpers
// ---------------------------------------------------------------------------

/// Format: "token:{client}:{email}"
pub fn token_key(client: &str, email: &str) -> String {
    format!("token:{}:{}", client, email)
}

/// Legacy format: "token:{email}" (for migration from single-client setups)
pub fn legacy_token_key(email: &str) -> String {
    format!("token:{}", email)
}

/// Parse a key like "token:default:user@gmail.com" into (client, email).
///
/// Rules:
/// - Must start with "token:"
/// - If rest has no colon (legacy): client = "default", email = rest
/// - If rest has one colon: client = parts[0], email = parts[1]
/// - Empty client or email → None
pub fn parse_token_key(key: &str) -> Option<(String, String)> {
    let rest = key.strip_prefix("token:")?;

    if rest.trim().is_empty() {
        return None;
    }

    if !rest.contains(':') {
        // Legacy format: "token:{email}"
        let email = rest.to_string();
        if email.trim().is_empty() {
            return None;
        }
        return Some(("default".to_string(), email));
    }

    // Standard format: "token:{client}:{email}"
    let mut parts = rest.splitn(2, ':');
    let client = parts.next()?.to_string();
    let email = parts.next()?.to_string();

    if client.trim().is_empty() || email.trim().is_empty() {
        return None;
    }

    Some((client, email))
}

// ---------------------------------------------------------------------------
// Normalization helpers
// ---------------------------------------------------------------------------

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn normalize_client(raw: &str) -> Result<String, SecretsError> {
    normalize_client_name_or_default(raw)
        .map_err(|e| SecretsError::InvalidClient(e.to_string()))
}

// ---------------------------------------------------------------------------
// KeyringStore
// ---------------------------------------------------------------------------

pub struct KeyringStore {
    service_name: String,
}

impl KeyringStore {
    pub fn new() -> Self {
        KeyringStore {
            service_name: "gogcli".to_string(),
        }
    }

    fn entry(&self, key: &str) -> Result<keyring::Entry, SecretsError> {
        keyring::Entry::new(&self.service_name, key)
            .map_err(|e| SecretsError::Keyring(e.to_string()))
    }
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for KeyringStore {
    fn keys(&self) -> Result<Vec<String>, SecretsError> {
        // The keyring crate v3 doesn't support listing keys.
        // Return an empty vec for now; proper listing will be implemented later.
        Ok(vec![])
    }

    fn set_token(&self, client: &str, email: &str, token: &Token) -> Result<(), SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }
        if token.refresh_token.is_empty() {
            return Err(SecretsError::MissingRefreshToken);
        }

        let normalized_client = normalize_client(client)?;

        let created_at = if token.created_at == DateTime::<Utc>::default() {
            Utc::now()
        } else {
            token.created_at
        };

        let stored = StoredToken {
            refresh_token: token.refresh_token.clone(),
            services: token.services.clone(),
            scopes: token.scopes.clone(),
            created_at,
        };

        let json = serde_json::to_string(&stored)
            .map_err(|e| SecretsError::Serialize(e.to_string()))?;

        let key = token_key(&normalized_client, &email);
        self.entry(&key)?
            .set_password(&json)
            .map_err(|e| SecretsError::Keyring(e.to_string()))?;

        Ok(())
    }

    fn get_token(&self, client: &str, email: &str) -> Result<Token, SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }

        let normalized_client = normalize_client(client)?;
        let key = token_key(&normalized_client, &email);

        let json = self
            .entry(&key)?
            .get_password()
            .map_err(|e| SecretsError::Keyring(e.to_string()))?;

        let stored: StoredToken = serde_json::from_str(&json)
            .map_err(|e| SecretsError::Deserialize(e.to_string()))?;

        Ok(Token {
            client: normalized_client,
            email,
            services: stored.services,
            scopes: stored.scopes,
            created_at: stored.created_at,
            refresh_token: stored.refresh_token,
        })
    }

    fn delete_token(&self, client: &str, email: &str) -> Result<(), SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }

        let normalized_client = normalize_client(client)?;
        let key = token_key(&normalized_client, &email);

        match self.entry(&key)?.delete_credential() {
            Ok(()) => {}
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(SecretsError::Keyring(e.to_string())),
        }

        Ok(())
    }

    fn list_tokens(&self) -> Result<Vec<Token>, SecretsError> {
        // The keyring crate v3 doesn't support listing keys.
        // Return an empty vec for now.
        Ok(vec![])
    }

    fn get_default_account(&self, client: &str) -> Result<Option<String>, SecretsError> {
        let normalized_client = normalize_client(client)?;
        let key = format!("default_account:{}", normalized_client);

        match self.entry(&key)?.get_password() {
            Ok(email) => return Ok(Some(email)),
            Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(SecretsError::Keyring(e.to_string())),
        }

        // Fall back to the global default_account key
        match self.entry("default_account")?.get_password() {
            Ok(email) => Ok(Some(email)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecretsError::Keyring(e.to_string())),
        }
    }

    fn set_default_account(&self, client: &str, email: &str) -> Result<(), SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }

        let normalized_client = normalize_client(client)?;
        let key = format!("default_account:{}", normalized_client);

        self.entry(&key)?
            .set_password(&email)
            .map_err(|e| SecretsError::Keyring(e.to_string()))?;

        self.entry("default_account")?
            .set_password(&email)
            .map_err(|e| SecretsError::Keyring(e.to_string()))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // --- token_key ---

    #[test]
    fn test_token_key_format() {
        assert_eq!(token_key("default", "a@b.com"), "token:default:a@b.com");
    }

    // --- legacy_token_key ---

    #[test]
    fn test_legacy_token_key_format() {
        assert_eq!(legacy_token_key("a@b.com"), "token:a@b.com");
    }

    // --- parse_token_key ---

    #[test]
    fn test_parse_token_key_standard() {
        let result = parse_token_key("token:default:a@b.com");
        assert_eq!(result, Some(("default".to_string(), "a@b.com".to_string())));
    }

    #[test]
    fn test_parse_token_key_legacy() {
        // "token:a@b.com" → client = "default", email = "a@b.com"
        let result = parse_token_key("token:a@b.com");
        assert_eq!(result, Some(("default".to_string(), "a@b.com".to_string())));
    }

    #[test]
    fn test_parse_token_key_named_client() {
        let result = parse_token_key("token:work:a@b.com");
        assert_eq!(result, Some(("work".to_string(), "a@b.com".to_string())));
    }

    #[test]
    fn test_parse_token_key_invalid() {
        assert_eq!(parse_token_key("not-a-token-key"), None);
    }

    #[test]
    fn test_parse_token_key_empty_parts() {
        // "token:" → rest is empty → None
        assert_eq!(parse_token_key("token:"), None);
        // "token::" → client is empty → None
        assert_eq!(parse_token_key("token::"), None);
    }

    // --- normalize_email ---

    #[test]
    fn test_normalize_email() {
        assert_eq!(normalize_email(" Alice@Example.COM "), "alice@example.com");
    }

    // --- StoredToken roundtrip ---

    #[test]
    fn test_stored_token_roundtrip() {
        let original = StoredToken {
            refresh_token: "my_refresh_token".to_string(),
            services: vec!["gmail".to_string(), "calendar".to_string()],
            scopes: vec!["https://mail.google.com/".to_string()],
            created_at: Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
        };

        let json = serde_json::to_string(&original).expect("serialize failed");
        let decoded: StoredToken = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(decoded.refresh_token, original.refresh_token);
        assert_eq!(decoded.services, original.services);
        assert_eq!(decoded.scopes, original.scopes);
        assert_eq!(decoded.created_at, original.created_at);
    }

    // --- validation ---

    #[test]
    fn test_token_key_with_empty_email_fails() {
        let store = KeyringStore::new();
        let token = Token {
            client: "default".to_string(),
            email: "".to_string(),
            services: vec![],
            scopes: vec![],
            created_at: Utc::now(),
            refresh_token: "some_token".to_string(),
        };
        let result = store.set_token("default", "", &token);
        assert!(
            matches!(result, Err(SecretsError::MissingEmail)),
            "expected MissingEmail, got {:?}",
            result
        );
    }

    // --- keyring integration (requires real keyring) ---

    #[test]
    #[ignore]
    fn test_keyring_store_roundtrip() {
        let store = KeyringStore::new();

        let token = Token {
            client: "default".to_string(),
            email: "test@example.com".to_string(),
            services: vec!["gmail".to_string()],
            scopes: vec!["https://mail.google.com/".to_string()],
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            refresh_token: "test_refresh_token_12345".to_string(),
        };

        store
            .set_token("default", "test@example.com", &token)
            .expect("set_token failed");

        let retrieved = store
            .get_token("default", "test@example.com")
            .expect("get_token failed");

        assert_eq!(retrieved.email, "test@example.com");
        assert_eq!(retrieved.refresh_token, "test_refresh_token_12345");
        assert_eq!(retrieved.services, vec!["gmail"]);
        assert_eq!(
            retrieved.scopes,
            vec!["https://mail.google.com/".to_string()]
        );

        store
            .delete_token("default", "test@example.com")
            .expect("delete_token failed");
    }
}
