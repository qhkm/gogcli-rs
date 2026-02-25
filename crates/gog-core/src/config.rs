// gog-core config module
// Ported from internal/config/{config.go,paths.go,clients.go,credentials.go}

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const APP_NAME: &str = "gogcli";
pub const DEFAULT_CLIENT_NAME: &str = "default";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot resolve config directory: {0}")]
    ConfigDir(String),

    #[error("read config: {0}")]
    ReadConfig(std::io::Error),

    #[error("parse config {path}: {message}")]
    ParseConfig { path: PathBuf, message: String },

    #[error("write config: {0}")]
    WriteConfig(String),

    #[error("invalid client name: {0}")]
    InvalidClientName(String),

    #[error("oauth credentials missing: {path}")]
    CredentialsMissing { path: PathBuf },
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::ReadConfig(e)
    }
}

// ---------------------------------------------------------------------------
// ConfigFile struct
// ---------------------------------------------------------------------------

fn is_none<T>(v: &Option<T>) -> bool {
    v.is_none()
}

fn is_empty_map(m: &HashMap<String, String>) -> bool {
    m.is_empty()
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(skip_serializing_if = "is_none")]
    pub keyring_backend: Option<String>,

    #[serde(skip_serializing_if = "is_none")]
    pub default_timezone: Option<String>,

    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub account_aliases: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub account_clients: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "is_empty_map")]
    pub client_domains: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// ClientCredentials struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}

// Google OAuth JSON structure: { "installed": { ... } } or { "web": { ... } }
#[derive(Debug, Deserialize)]
struct GoogleOAuthInner {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct GoogleCredentialsFile {
    installed: Option<GoogleOAuthInner>,
    web: Option<GoogleOAuthInner>,
}

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Returns `$XDG_CONFIG_HOME/gogcli` or `~/.config/gogcli`.
pub fn config_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs::config_dir()
        .ok_or_else(|| ConfigError::ConfigDir("cannot determine user config directory".into()))?;
    Ok(base.join(APP_NAME))
}

/// Returns `config_dir()/config.json`.
pub fn config_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("config.json"))
}

/// Creates the config directory if it doesn't exist, returns the path.
pub fn ensure_dir() -> Result<PathBuf, ConfigError> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| ConfigError::WriteConfig(format!("ensure config dir: {e}")))?;
    Ok(dir)
}

/// Returns `config_dir()/keyring`.
pub fn keyring_dir() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("keyring"))
}

/// Creates the keyring directory if it doesn't exist, returns the path.
pub fn ensure_keyring_dir() -> Result<PathBuf, ConfigError> {
    let dir = keyring_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| ConfigError::WriteConfig(format!("ensure keyring dir: {e}")))?;
    Ok(dir)
}

// ---------------------------------------------------------------------------
// Client name normalization
// ---------------------------------------------------------------------------

/// Normalizes a client name to lowercase, allowing alphanumeric, `-`, `_`, `.`.
/// Empty input returns an error.
pub fn normalize_client_name(raw: &str) -> Result<String, ConfigError> {
    let name = raw.trim().to_lowercase();
    if name.is_empty() {
        return Err(ConfigError::InvalidClientName("empty".into()));
    }
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            continue;
        }
        return Err(ConfigError::InvalidClientName(format!("{:?}", raw)));
    }
    Ok(name)
}

/// Like `normalize_client_name`, but empty/whitespace returns `"default"`.
pub fn normalize_client_name_or_default(name: &str) -> Result<String, ConfigError> {
    if name.trim().is_empty() {
        return Ok(DEFAULT_CLIENT_NAME.to_string());
    }
    normalize_client_name(name)
}

// ---------------------------------------------------------------------------
// Credentials path
// ---------------------------------------------------------------------------

/// Returns the path to the credentials file for the given client name.
/// `"default"` → `credentials.json`, others → `credentials-{name}.json`.
pub fn client_credentials_path_for(client: &str) -> Result<PathBuf, ConfigError> {
    let dir = config_dir()?;
    let normalized = normalize_client_name_or_default(client)?;
    if normalized == DEFAULT_CLIENT_NAME {
        Ok(dir.join("credentials.json"))
    } else {
        Ok(dir.join(format!("credentials-{normalized}.json")))
    }
}

// ---------------------------------------------------------------------------
// Config read / write
// ---------------------------------------------------------------------------

/// Reads the JSON5 config file. Returns default if the file does not exist.
pub fn read_config() -> Result<ConfigFile, ConfigError> {
    let path = config_path()?;
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ConfigFile::default()),
        Err(e) => return Err(ConfigError::ReadConfig(e)),
    };
    let text = String::from_utf8_lossy(&bytes);
    json5::from_str(&text).map_err(|e| ConfigError::ParseConfig {
        path,
        message: e.to_string(),
    })
}

/// Atomically writes the config file (write tmp, rename).
pub fn write_config(cfg: &ConfigFile) -> Result<(), ConfigError> {
    ensure_dir()?;
    let path = config_path()?;
    let mut json = serde_json::to_string_pretty(cfg)
        .map_err(|e| ConfigError::WriteConfig(format!("encode config: {e}")))?;
    json.push('\n');

    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json.as_bytes())
        .map_err(|e| ConfigError::WriteConfig(format!("write config tmp: {e}")))?;
    std::fs::rename(&tmp, &path)
        .map_err(|e| ConfigError::WriteConfig(format!("commit config: {e}")))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Credentials read
// ---------------------------------------------------------------------------

/// Reads Google OAuth credentials for the given client.
/// The file may contain a nested `installed` or `web` object (standard Google format),
/// or a flat `{ client_id, client_secret }`.
pub fn read_client_credentials_for(client: &str) -> Result<ClientCredentials, ConfigError> {
    let path = client_credentials_path_for(client)?;
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ConfigError::CredentialsMissing { path });
        }
        Err(e) => return Err(ConfigError::ReadConfig(e)),
    };

    // Try nested Google format first.
    if let Ok(gcf) = serde_json::from_slice::<GoogleCredentialsFile>(&bytes) {
        let inner = gcf.installed.or(gcf.web);
        if let Some(inner) = inner {
            if !inner.client_id.is_empty() && !inner.client_secret.is_empty() {
                return Ok(ClientCredentials {
                    client_id: inner.client_id,
                    client_secret: inner.client_secret,
                });
            }
        }
    }

    // Fall back to flat format.
    let creds: ClientCredentials = serde_json::from_slice(&bytes)
        .map_err(|e| ConfigError::ParseConfig { path: path.clone(), message: e.to_string() })?;
    if creds.client_id.is_empty() || creds.client_secret.is_empty() {
        return Err(ConfigError::ParseConfig {
            path,
            message: "missing client_id or client_secret".into(),
        });
    }
    Ok(creds)
}

// ---------------------------------------------------------------------------
// Path expansion
// ---------------------------------------------------------------------------

/// Expands `~` at the start of a path to the user's home directory.
/// An empty string returns an empty `PathBuf`.
pub fn expand_path(path: &str) -> Result<PathBuf, ConfigError> {
    if path.is_empty() {
        return Ok(PathBuf::new());
    }
    if path == "~" {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::ConfigDir("cannot determine home directory".into()))?;
        return Ok(home);
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::ConfigDir("cannot determine home directory".into()))?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(path))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    // -- normalize_client_name_or_default -----------------------------------

    #[test]
    fn test_normalize_client_default() {
        assert_eq!(normalize_client_name_or_default("").unwrap(), "default");
        assert_eq!(normalize_client_name_or_default("   ").unwrap(), "default");
        assert_eq!(normalize_client_name_or_default("\t").unwrap(), "default");
    }

    #[test]
    fn test_normalize_client_valid() {
        assert_eq!(normalize_client_name_or_default("work").unwrap(), "work");
        assert_eq!(normalize_client_name_or_default("My-Client").unwrap(), "my-client");
        assert_eq!(normalize_client_name_or_default("FOO_BAR.baz").unwrap(), "foo_bar.baz");
    }

    #[test]
    fn test_normalize_client_invalid() {
        assert!(normalize_client_name_or_default("bad name!").is_err());
        assert!(normalize_client_name_or_default("hello world").is_err());
        assert!(normalize_client_name_or_default("a@b").is_err());
    }

    // -- expand_path --------------------------------------------------------

    #[test]
    fn test_expand_path_tilde() {
        let result = expand_path("~/foo/bar").unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(result, home.join("foo/bar"));
    }

    #[test]
    fn test_expand_path_absolute() {
        let result = expand_path("/usr/bin").unwrap();
        assert_eq!(result, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_expand_path_empty() {
        let result = expand_path("").unwrap();
        assert_eq!(result, PathBuf::new());
    }

    #[test]
    fn test_expand_path_tilde_only() {
        let result = expand_path("~").unwrap();
        let home = dirs::home_dir().unwrap();
        assert_eq!(result, home);
    }

    // -- ConfigFile ---------------------------------------------------------

    #[test]
    fn test_config_file_default() {
        let cfg = ConfigFile::default();
        assert!(cfg.keyring_backend.is_none());
        assert!(cfg.default_timezone.is_none());
        assert!(cfg.account_aliases.is_empty());
        assert!(cfg.account_clients.is_empty());
        assert!(cfg.client_domains.is_empty());
    }

    #[test]
    fn test_config_file_roundtrip() {
        let mut cfg = ConfigFile::default();
        cfg.keyring_backend = Some("secret-service".to_string());
        cfg.default_timezone = Some("America/New_York".to_string());
        cfg.account_aliases
            .insert("me@example.com".to_string(), "personal".to_string());
        cfg.account_clients
            .insert("work@corp.com".to_string(), "work".to_string());
        cfg.client_domains
            .insert("corp.com".to_string(), "work".to_string());

        let json = serde_json::to_string_pretty(&cfg).unwrap();
        let decoded: ConfigFile = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, decoded);
    }

    #[test]
    fn test_config_file_skip_empty_on_serialize() {
        let cfg = ConfigFile::default();
        let json = serde_json::to_string(&cfg).unwrap();
        // None fields and empty maps should not appear in the output.
        assert!(!json.contains("keyring_backend"));
        assert!(!json.contains("default_timezone"));
        assert!(!json.contains("account_aliases"));
        assert!(!json.contains("account_clients"));
        assert!(!json.contains("client_domains"));
    }

    // -- config_dir ---------------------------------------------------------

    #[test]
    fn test_config_dir_returns_path() {
        let dir = config_dir().unwrap();
        assert!(dir.ends_with("gogcli"));
    }

    // -- client_credentials_path_for ----------------------------------------

    #[test]
    fn test_credentials_path_default() {
        let path = client_credentials_path_for("").unwrap();
        assert!(path.ends_with("credentials.json"));
        assert!(!path.to_string_lossy().contains("credentials-"));
    }

    #[test]
    fn test_credentials_path_named() {
        let path = client_credentials_path_for("work").unwrap();
        assert!(path.ends_with("credentials-work.json"));
    }

    #[test]
    fn test_credentials_path_default_explicit() {
        let path = client_credentials_path_for("default").unwrap();
        assert!(path.ends_with("credentials.json"));
    }

    #[test]
    fn test_credentials_path_invalid_name() {
        assert!(client_credentials_path_for("bad name!").is_err());
    }

    // -- read_config (file-system, uses temp dir via env override) ----------

    #[test]
    fn test_read_config_missing_returns_default() {
        // With a non-existent XDG_CONFIG_HOME the dirs crate still resolves
        // a path; if the file simply doesn't exist, read_config returns Default.
        // We only verify the behaviour when we can control the env.
        // For a portable check: call read_config and either get Ok(default)
        // or get Ok(_) — we should not get a parse error.
        let result = read_config();
        assert!(result.is_ok(), "read_config returned error: {result:?}");
    }

    // -- write_config + read_config round-trip using a temp dir ---------------

    /// Reads a ConfigFile from an arbitrary path using the same JSON5 logic as
    /// `read_config`, but against a caller-supplied path. Used for isolated
    /// temp-dir tests on macOS where `dirs::config_dir()` ignores env vars.
    fn read_config_from(path: &std::path::Path) -> Result<ConfigFile, ConfigError> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ConfigFile::default())
            }
            Err(e) => return Err(ConfigError::ReadConfig(e)),
        };
        let text = String::from_utf8_lossy(&bytes);
        json5::from_str(&text).map_err(|e| ConfigError::ParseConfig {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    /// Writes a ConfigFile atomically to an arbitrary path.
    fn write_config_to(cfg: &ConfigFile, path: &std::path::Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::WriteConfig(format!("ensure dir: {e}")))?;
        }
        let mut json = serde_json::to_string_pretty(cfg)
            .map_err(|e| ConfigError::WriteConfig(format!("encode: {e}")))?;
        json.push('\n');
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json.as_bytes())
            .map_err(|e| ConfigError::WriteConfig(format!("write tmp: {e}")))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| ConfigError::WriteConfig(format!("rename: {e}")))?;
        Ok(())
    }

    #[test]
    fn test_write_read_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("gogcli").join("config.json");

        let mut cfg = ConfigFile::default();
        cfg.keyring_backend = Some("file".to_string());
        cfg.account_aliases
            .insert("me@example.com".to_string(), "me".to_string());

        write_config_to(&cfg, &config_path).expect("write_config_to failed");
        assert!(config_path.exists(), "config.json not created");

        let read_back = read_config_from(&config_path).expect("read_config_from failed");
        assert_eq!(read_back.keyring_backend, cfg.keyring_backend);
        assert_eq!(read_back.account_aliases, cfg.account_aliases);
    }

    #[test]
    fn test_read_config_json5_comments() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.json");
        // JSON5 allows single-line and block comments.
        std::fs::write(
            &config_path,
            r#"
// this is a comment
{
  keyring_backend: "secret-service", // inline comment
  default_timezone: "UTC",
}
"#,
        )
        .unwrap();
        let cfg = read_config_from(&config_path).expect("json5 parse failed");
        assert_eq!(cfg.keyring_backend.as_deref(), Some("secret-service"));
        assert_eq!(cfg.default_timezone.as_deref(), Some("UTC"));
    }
}
