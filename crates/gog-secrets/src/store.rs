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
// FileStore — stores tokens as JSON files in the keyring directory.
// Mirrors the Go version's file backend (github.com/99designs/keyring).
// ---------------------------------------------------------------------------

pub struct FileStore {
    dir: std::path::PathBuf,
}

impl FileStore {
    /// Create a FileStore using the standard keyring directory
    /// (`<config_dir>/keyring/`).
    pub fn new() -> Result<Self, SecretsError> {
        let config_dir = gog_core::config::config_dir()
            .map_err(|e| SecretsError::Keyring(format!("config dir: {e}")))?;
        let dir = config_dir.join("keyring");
        std::fs::create_dir_all(&dir)
            .map_err(|e| SecretsError::Keyring(format!("create keyring dir: {e}")))?;
        Ok(FileStore { dir })
    }

    /// Create a FileStore with a custom directory (for testing).
    pub fn with_dir(dir: std::path::PathBuf) -> Result<Self, SecretsError> {
        std::fs::create_dir_all(&dir)
            .map_err(|e| SecretsError::Keyring(format!("create keyring dir: {e}")))?;
        Ok(FileStore { dir })
    }

    /// Encode a key into a safe filename (replace : and @ with _).
    fn key_to_filename(key: &str) -> String {
        key.replace(':', "_").replace('@', "_")
    }

    fn file_path(&self, key: &str) -> std::path::PathBuf {
        self.dir.join(format!("{}.json", Self::key_to_filename(key)))
    }

    fn read_value(&self, key: &str) -> Result<String, SecretsError> {
        let path = self.file_path(key);
        std::fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SecretsError::Keyring(format!("No matching entry found in secure storage"))
            } else {
                SecretsError::Keyring(format!("read {}: {e}", path.display()))
            }
        })
    }

    fn write_value(&self, key: &str, value: &str) -> Result<(), SecretsError> {
        let path = self.file_path(key);
        std::fs::write(&path, value)
            .map_err(|e| SecretsError::Keyring(format!("write {}: {e}", path.display())))?;
        // Restrict permissions (best effort on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn delete_value(&self, key: &str) -> Result<(), SecretsError> {
        let path = self.file_path(key);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(SecretsError::Keyring(format!("delete {}: {e}", path.display()))),
        }
    }

    /// List all .json files in the keyring directory and extract the original keys.
    fn list_files(&self) -> Result<Vec<String>, SecretsError> {
        let mut keys = Vec::new();
        let entries = std::fs::read_dir(&self.dir)
            .map_err(|e| SecretsError::Keyring(format!("read keyring dir: {e}")))?;
        for entry in entries {
            let entry = entry.map_err(|e| SecretsError::Keyring(e.to_string()))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(stem) = name.strip_suffix(".json") {
                // The filename is the key with : and @ replaced by _
                // We can't perfectly reverse this, but for token keys we can
                // read the file and extract client/email from the stored data.
                keys.push(stem.to_string());
            }
        }
        Ok(keys)
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new().expect("failed to create FileStore")
    }
}

impl Store for FileStore {
    fn keys(&self) -> Result<Vec<String>, SecretsError> {
        self.list_files()
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

        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| SecretsError::Serialize(e.to_string()))?;

        let key = token_key(&normalized_client, &email);
        self.write_value(&key, &json)
    }

    fn get_token(&self, client: &str, email: &str) -> Result<Token, SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }

        let normalized_client = normalize_client(client)?;
        let key = token_key(&normalized_client, &email);
        let json = self.read_value(&key)?;

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
        self.delete_value(&key)
    }

    fn list_tokens(&self) -> Result<Vec<Token>, SecretsError> {
        let mut tokens = Vec::new();
        let entries = std::fs::read_dir(&self.dir)
            .map_err(|e| SecretsError::Keyring(format!("read keyring dir: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| SecretsError::Keyring(e.to_string()))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only process token_*.json files (skip default_account, etc.)
            if !name_str.starts_with("token_") || !name_str.ends_with(".json") {
                continue;
            }

            let json = match std::fs::read_to_string(entry.path()) {
                Ok(j) => j,
                Err(_) => continue,
            };

            let stored: StoredToken = match serde_json::from_str(&json) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Reconstruct client and email from the filename.
            // Filename: token_{client}_{email}.json
            // We need to parse this carefully since email contains _ (was @).
            let stem = name_str.strip_suffix(".json").unwrap_or(&name_str);
            let rest = match stem.strip_prefix("token_") {
                Some(r) => r,
                None => continue,
            };

            // Split on first _ to get client, rest is email-ish
            // But email had @ replaced with _, so we read all files and
            // use a heuristic: find the first _ after which the rest looks like an email.
            // Simpler approach: try the key against the store.
            // Actually, the safest approach: store client+email in the JSON itself.
            // For now, use a simple split: first segment is client, rest is email.
            if let Some(idx) = rest.find('_') {
                let client = &rest[..idx];
                let email_part = &rest[idx + 1..];
                // The email had @ replaced with _, so the first _ in email_part
                // that's followed by something with a dot is likely the @ replacement.
                // Actually, we know the format: "default_dr.noranizaahmad_gmail.com"
                // So client=default, email=dr.noranizaahmad_gmail.com
                // We need to find where the @ was. Look for common TLD patterns.
                let email = reconstruct_email(email_part);
                tokens.push(Token {
                    client: client.to_string(),
                    email,
                    services: stored.services,
                    scopes: stored.scopes,
                    created_at: stored.created_at,
                    refresh_token: stored.refresh_token,
                });
            }
        }

        Ok(tokens)
    }

    fn get_default_account(&self, client: &str) -> Result<Option<String>, SecretsError> {
        let normalized_client = normalize_client(client)?;
        let key = format!("default_account:{}", normalized_client);

        match self.read_value(&key) {
            Ok(email) => return Ok(Some(email.trim().to_string())),
            Err(_) => {}
        }

        // Fall back to the global default_account key
        match self.read_value("default_account") {
            Ok(email) => Ok(Some(email.trim().to_string())),
            Err(_) => Ok(None),
        }
    }

    fn set_default_account(&self, client: &str, email: &str) -> Result<(), SecretsError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(SecretsError::MissingEmail);
        }

        let normalized_client = normalize_client(client)?;
        let key = format!("default_account:{}", normalized_client);

        self.write_value(&key, &email)?;
        self.write_value("default_account", &email)?;

        Ok(())
    }
}

/// Reconstruct an email from a filename-safe string.
/// The @ was replaced with _, so "dr.noranizaahmad_gmail.com" → "dr.noranizaahmad@gmail.com".
/// Heuristic: find the last _ before a segment that looks like a domain (contains a dot).
fn reconstruct_email(s: &str) -> String {
    // Find all _ positions and try replacing the last ones with @
    let underscores: Vec<usize> = s.match_indices('_').map(|(i, _)| i).collect();
    for &pos in underscores.iter().rev() {
        let after = &s[pos + 1..];
        if after.contains('.') && !after.starts_with('.') && !after.ends_with('.') {
            let mut result = String::with_capacity(s.len());
            result.push_str(&s[..pos]);
            result.push('@');
            result.push_str(after);
            return result;
        }
    }
    // Fallback: return as-is
    s.to_string()
}

// ---------------------------------------------------------------------------
// open_store — factory that reads config and returns the right store.
// ---------------------------------------------------------------------------

/// Open the default store based on config.json's `keyring_backend` setting.
/// Falls back to `FileStore` if not specified or on any keyring error.
pub fn open_store() -> Result<Box<dyn Store>, SecretsError> {
    let cfg = gog_core::config::read_config().unwrap_or_default();
    let backend = cfg.keyring_backend.as_deref().unwrap_or("file");

    match backend {
        "file" | "auto" | "" => Ok(Box::new(FileStore::new()?)),
        "keychain" | "keyring" | "secret-service" => Ok(Box::new(KeyringStore::new())),
        other => Err(SecretsError::Keyring(format!("unknown keyring_backend: {other}"))),
    }
}

// ---------------------------------------------------------------------------
// KeyringStore — uses the OS keychain (macOS Keychain, Secret Service, etc.)
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
