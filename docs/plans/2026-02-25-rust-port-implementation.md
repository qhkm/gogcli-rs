# gogcli-rs: Rust Port Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port gogcli (Go) to Rust as a Cargo workspace, maintaining identical CLI interface for 7 Google services (Gmail, Calendar, Drive, Contacts, Chat, Keep, Forms).

**Architecture:** Cargo workspace with 12 crates. Foundation crates (gog-core, gog-secrets, gog-auth, gog-api) built first, then service crates (gog-gmail, etc.), then CLI binary (gog-cli). Each crate is independently testable.

**Tech Stack:** Rust 2021 edition, clap v4 (derive), tokio, google-apis-rs (google-gmail1, google-calendar3, etc.), yup-oauth2, keyring-rs, serde/serde_json, chrono, thiserror/anyhow.

---

## Phase 1: Foundation (Workspace + Core)

### Task 1: Initialize Cargo workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `.gitignore`
- Create: `crates/gog-core/Cargo.toml`
- Create: `crates/gog-core/src/lib.rs`

**Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/gog-cli",
    "crates/gog-core",
    "crates/gog-auth",
    "crates/gog-secrets",
    "crates/gog-api",
    "crates/gog-gmail",
    "crates/gog-calendar",
    "crates/gog-drive",
    "crates/gog-contacts",
    "crates/gog-chat",
    "crates/gog-keep",
    "crates/gog-forms",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/steipete/gogcli-rs"

[workspace.dependencies]
# Shared deps - all crates reference these
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
```

**Step 2: Create .gitignore**

```
/target
Cargo.lock
*.swp
.env
```

**Step 3: Create gog-core crate**

`crates/gog-core/Cargo.toml`:
```toml
[package]
name = "gog-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
chrono.workspace = true
dirs = "6"
json5 = "0.4"
```

`crates/gog-core/src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod output;
pub mod timeparse;
```

**Step 4: Verify workspace compiles**

Run: `cargo check`
Expected: success (may warn about empty modules)

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: initialize Cargo workspace with gog-core crate"
```

---

### Task 2: Implement gog-core config module

**Files:**
- Create: `crates/gog-core/src/config.rs`

Port `internal/config/config.go` + `internal/config/paths.go`. Config reads `~/.config/gogcli/config.json` (JSON5), shared with Go version.

**Step 1: Write failing test**

Add to `crates/gog-core/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const APP_NAME: &str = "gogcli";
pub const DEFAULT_CLIENT_NAME: &str = "default";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("resolve config dir: {0}")]
    ConfigDir(String),
    #[error("read config: {0}")]
    ReadConfig(#[from] std::io::Error),
    #[error("parse config {path}: {source}")]
    ParseConfig { path: PathBuf, source: String },
    #[error("write config: {0}")]
    WriteConfig(String),
    #[error("invalid client name: {0}")]
    InvalidClientName(String),
    #[error("credentials missing (expected at {path})")]
    CredentialsMissing { path: PathBuf },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyring_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_timezone: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub account_aliases: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub account_clients: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub client_domains: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Returns the config directory: $XDG_CONFIG_HOME/gogcli or ~/.config/gogcli
pub fn config_dir() -> Result<PathBuf, ConfigError> {
    dirs::config_dir()
        .map(|d| d.join(APP_NAME))
        .ok_or_else(|| ConfigError::ConfigDir("cannot resolve user config directory".into()))
}

/// Returns path to config.json
pub fn config_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("config.json"))
}

/// Ensures config directory exists, returns its path
pub fn ensure_dir() -> Result<PathBuf, ConfigError> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).map_err(ConfigError::ReadConfig)?;
    Ok(dir)
}

/// Returns keyring directory path
pub fn keyring_dir() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("keyring"))
}

pub fn ensure_keyring_dir() -> Result<PathBuf, ConfigError> {
    let dir = keyring_dir()?;
    std::fs::create_dir_all(&dir).map_err(ConfigError::ReadConfig)?;
    Ok(dir)
}

/// Returns credentials file path for the given client
pub fn client_credentials_path_for(client: &str) -> Result<PathBuf, ConfigError> {
    let dir = config_dir()?;
    let normalized = normalize_client_name_or_default(client)?;
    if normalized == DEFAULT_CLIENT_NAME {
        Ok(dir.join("credentials.json"))
    } else {
        Ok(dir.join(format!("credentials-{normalized}.json")))
    }
}

pub fn normalize_client_name_or_default(name: &str) -> Result<String, ConfigError> {
    let name = name.trim().to_lowercase();
    if name.is_empty() {
        return Ok(DEFAULT_CLIENT_NAME.to_string());
    }
    // Validate: alphanumeric + hyphens only
    if name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        Ok(name)
    } else {
        Err(ConfigError::InvalidClientName(name))
    }
}

/// Read config file (JSON5). Returns default if file doesn't exist.
pub fn read_config() -> Result<ConfigFile, ConfigError> {
    let path = config_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            json5::from_str(&contents).map_err(|e| ConfigError::ParseConfig {
                path,
                source: e.to_string(),
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ConfigFile::default()),
        Err(e) => Err(ConfigError::ReadConfig(e)),
    }
}

/// Write config file atomically (write tmp, rename)
pub fn write_config(cfg: &ConfigFile) -> Result<(), ConfigError> {
    ensure_dir()?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| ConfigError::WriteConfig(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, format!("{json}\n")).map_err(ConfigError::ReadConfig)?;
    std::fs::rename(&tmp, &path).map_err(ConfigError::ReadConfig)?;
    Ok(())
}

/// Read OAuth client credentials for a named client
pub fn read_client_credentials_for(client: &str) -> Result<ClientCredentials, ConfigError> {
    let path = client_credentials_path_for(client)?;
    let contents = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            return ConfigError::CredentialsMissing { path: path.clone() };
        }
        ConfigError::ReadConfig(e)
    })?;

    // Google credentials JSON has nested "installed" or "web" object
    #[derive(Deserialize)]
    struct GoogleCreds {
        client_id: String,
        client_secret: String,
    }
    #[derive(Deserialize)]
    struct GoogleCredsWrapper {
        installed: Option<GoogleCreds>,
        web: Option<GoogleCreds>,
    }

    // Try wrapper format first, then flat format
    if let Ok(wrapper) = serde_json::from_str::<GoogleCredsWrapper>(&contents) {
        if let Some(creds) = wrapper.installed.or(wrapper.web) {
            return Ok(ClientCredentials {
                client_id: creds.client_id,
                client_secret: creds.client_secret,
            });
        }
    }

    serde_json::from_str::<ClientCredentials>(&contents)
        .map_err(|e| ConfigError::ParseConfig { path, source: e.to_string() })
}

/// Expand ~ at beginning of path to home directory
pub fn expand_path(path: &str) -> Result<PathBuf, ConfigError> {
    if path.is_empty() {
        return Ok(PathBuf::new());
    }
    if path == "~" {
        return dirs::home_dir()
            .ok_or_else(|| ConfigError::ConfigDir("cannot resolve home directory".into()));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::ConfigDir("cannot resolve home directory".into()))?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_client_default() {
        assert_eq!(normalize_client_name_or_default("").unwrap(), "default");
        assert_eq!(normalize_client_name_or_default("  ").unwrap(), "default");
    }

    #[test]
    fn test_normalize_client_valid() {
        assert_eq!(normalize_client_name_or_default("work").unwrap(), "work");
        assert_eq!(normalize_client_name_or_default("My-Client").unwrap(), "my-client");
    }

    #[test]
    fn test_normalize_client_invalid() {
        assert!(normalize_client_name_or_default("bad name!").is_err());
    }

    #[test]
    fn test_expand_path_tilde() {
        let expanded = expand_path("~/foo/bar").unwrap();
        assert!(expanded.ends_with("foo/bar"));
        assert!(!expanded.to_string_lossy().contains('~'));
    }

    #[test]
    fn test_expand_path_absolute() {
        let expanded = expand_path("/usr/bin").unwrap();
        assert_eq!(expanded, PathBuf::from("/usr/bin"));
    }

    #[test]
    fn test_config_file_default() {
        let cfg = ConfigFile::default();
        assert!(cfg.keyring_backend.is_none());
        assert!(cfg.account_aliases.is_empty());
    }

    #[test]
    fn test_config_file_roundtrip() {
        let mut cfg = ConfigFile::default();
        cfg.keyring_backend = Some("file".to_string());
        cfg.account_aliases.insert("work".to_string(), "alice@company.com".to_string());
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: ConfigFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.keyring_backend.as_deref(), Some("file"));
        assert_eq!(parsed.account_aliases.get("work").unwrap(), "alice@company.com");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p gog-core`
Expected: all tests PASS

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(core): implement config module with JSON5 support"
```

---

### Task 3: Implement gog-core error types

**Files:**
- Create: `crates/gog-core/src/error.rs`

Port `internal/googleapi/errors.go` + `internal/errfmt/errfmt.go`.

**Step 1: Write error types with tests**

```rust
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Exit codes matching the Go version for script compatibility
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

#[derive(Error, Debug)]
pub enum GogError {
    #[error("auth required for {service} {email}")]
    AuthRequired {
        service: String,
        email: String,
        client: Option<String>,
        #[source]
        cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("rate limit exceeded after {retries} retries")]
    RateLimited {
        retry_after: Option<Duration>,
        retries: u32,
    },

    #[error("circuit breaker is open, too many recent failures - try again later")]
    CircuitBreakerOpen,

    #[error("API quota exceeded{}", resource.as_ref().map(|r| format!(" for {r}")).unwrap_or_default())]
    QuotaExceeded { resource: Option<String> },

    #[error("{resource} not found{}", id.as_ref().map(|i| format!(": {i}")).unwrap_or_default())]
    NotFound {
        resource: String,
        id: Option<String>,
    },

    #[error("permission denied{}", action.as_ref().map(|a| format!(": cannot {a} {}", resource.as_deref().unwrap_or(""))).unwrap_or_default())]
    PermissionDenied {
        resource: Option<String>,
        action: Option<String>,
    },

    #[error("Google API error ({code}): {message}")]
    GoogleApi {
        code: u16,
        message: String,
        reason: Option<String>,
    },

    #[error("{0}")]
    UserFacing(String, #[source] Option<Box<dyn std::error::Error + Send + Sync>>),

    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl GogError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::AuthRequired { .. } => exit_codes::AUTH_REQUIRED,
            Self::RateLimited { .. } => exit_codes::RATE_LIMITED,
            Self::CircuitBreakerOpen => exit_codes::CIRCUIT_BREAKER,
            Self::QuotaExceeded { .. } => exit_codes::QUOTA_EXCEEDED,
            Self::NotFound { .. } => exit_codes::NOT_FOUND,
            Self::PermissionDenied { .. } => exit_codes::PERMISSION_DENIED,
            Self::Usage(_) => exit_codes::USAGE_ERROR,
            _ => exit_codes::GENERAL_ERROR,
        }
    }

    /// Format error for user display (matching Go errfmt.Format)
    pub fn format_for_user(&self) -> String {
        match self {
            Self::AuthRequired { service, email, .. } => {
                format!(
                    "No auth for {service} {email}.\n\n\
                     OAuth (browser flow):\n  gog auth add {email} --services {service}\n\n\
                     Workspace service account (domain-wide delegation):\n  \
                     gog auth service-account set {email} --key <service-account.json>"
                )
            }
            Self::Config(crate::config::ConfigError::CredentialsMissing { path }) => {
                format!(
                    "OAuth client credentials missing (OAuth client ID JSON).\n\
                     Download from: https://console.cloud.google.com/apis/credentials\n\
                     Then run: gog auth credentials <credentials.json> (expected at {})",
                    path.display()
                )
            }
            Self::Usage(msg) => format!("{msg}\nRun with --help to see usage"),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_required_exit_code() {
        let err = GogError::AuthRequired {
            service: "gmail".into(),
            email: "test@example.com".into(),
            client: None,
            cause: None,
        };
        assert_eq!(err.exit_code(), exit_codes::AUTH_REQUIRED);
    }

    #[test]
    fn test_not_found_display() {
        let err = GogError::NotFound {
            resource: "message".into(),
            id: Some("abc123".into()),
        };
        assert_eq!(err.to_string(), "message not found: abc123");
    }

    #[test]
    fn test_auth_required_user_format() {
        let err = GogError::AuthRequired {
            service: "gmail".into(),
            email: "alice@example.com".into(),
            client: None,
            cause: None,
        };
        let msg = err.format_for_user();
        assert!(msg.contains("gog auth add alice@example.com --services gmail"));
    }

    #[test]
    fn test_rate_limited_display() {
        let err = GogError::RateLimited {
            retry_after: Some(Duration::from_secs(5)),
            retries: 3,
        };
        assert!(err.to_string().contains("3 retries"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p gog-core`
Expected: all tests PASS

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(core): implement error types with exit codes and user formatting"
```

---

### Task 4: Implement gog-core output module

**Files:**
- Create: `crates/gog-core/src/output.rs`

Port `internal/outfmt/outfmt.go`. Handles JSON/TSV/colored text modes, --results-only, --select.

**Step 1: Write output module with tests**

```rust
use serde::Serialize;
use serde_json::Value;
use std::io::Write;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    #[default]
    Text,
    Json,
    Plain, // TSV
}

#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub results_only: bool,
    pub select_fields: Vec<String>,
}

impl OutputConfig {
    pub fn from_flags(json: bool, plain: bool) -> Result<Self, String> {
        if json && plain {
            return Err("invalid output mode (cannot combine --json and --plain)".into());
        }
        let mode = if json {
            OutputMode::Json
        } else if plain {
            OutputMode::Plain
        } else {
            OutputMode::Text
        };
        Ok(Self { mode, ..Default::default() })
    }

    pub fn is_json(&self) -> bool { self.mode == OutputMode::Json }
    pub fn is_plain(&self) -> bool { self.mode == OutputMode::Plain }
}

/// Write JSON output with optional transforms (--results-only, --select)
pub fn write_json<W: Write>(
    w: &mut W,
    value: &impl Serialize,
    config: &OutputConfig,
) -> Result<(), crate::error::GogError> {
    let mut val = serde_json::to_value(value)
        .map_err(|e| crate::error::GogError::Other(e.into()))?;

    if config.results_only {
        val = unwrap_primary(val);
    }

    if !config.select_fields.is_empty() {
        val = select_fields(val, &config.select_fields);
    }

    let output = serde_json::to_string_pretty(&val)
        .map_err(|e| crate::error::GogError::Other(e.into()))?;
    writeln!(w, "{output}").map_err(crate::error::GogError::Io)
}

fn unwrap_primary(v: Value) -> Value {
    let Value::Object(ref map) = v else { return v };

    // Check explicit "results" key
    if let Some(results) = map.get("results") {
        return results.clone();
    }

    // Known envelope keys to skip
    let meta_keys: &[&str] = &[
        "nextPageToken", "next_cursor", "has_more", "count",
        "query", "dry_run", "dryRun", "op", "action", "note",
    ];

    let candidates: Vec<&String> = map.keys()
        .filter(|k| !meta_keys.contains(&k.as_str()))
        .collect();

    // Single non-meta key → unwrap it
    if candidates.len() == 1 {
        return map[candidates[0]].clone();
    }

    // Prefer array candidates
    for k in &candidates {
        if map[*k].is_array() {
            return map[*k].clone();
        }
    }

    v
}

fn select_fields(v: Value, fields: &[String]) -> Value {
    match v {
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(|item| select_from_item(item, fields)).collect())
        }
        other => select_from_item(other, fields),
    }
}

fn select_from_item(v: Value, fields: &[String]) -> Value {
    let Value::Object(map) = v else { return v };
    let mut out = serde_json::Map::new();
    for field in fields {
        if let Some(val) = get_at_path(&Value::Object(map.clone()), field) {
            out.insert(field.clone(), val);
        }
    }
    Value::Object(out)
}

fn get_at_path(v: &Value, path: &str) -> Option<Value> {
    let segments: Vec<&str> = path.split('.').collect();
    let mut current = v.clone();
    for seg in segments {
        let seg = seg.trim();
        if seg.is_empty() { return None; }
        match current {
            Value::Object(ref map) => {
                current = map.get(seg)?.clone();
            }
            Value::Array(ref arr) => {
                let i: usize = seg.parse().ok()?;
                current = arr.get(i)?.clone();
            }
            _ => return None,
        }
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_output_config_from_flags() {
        assert!(OutputConfig::from_flags(true, true).is_err());
        assert_eq!(OutputConfig::from_flags(true, false).unwrap().mode, OutputMode::Json);
        assert_eq!(OutputConfig::from_flags(false, true).unwrap().mode, OutputMode::Plain);
        assert_eq!(OutputConfig::from_flags(false, false).unwrap().mode, OutputMode::Text);
    }

    #[test]
    fn test_unwrap_primary_results_key() {
        let v = json!({"results": [1, 2, 3], "nextPageToken": "abc"});
        assert_eq!(unwrap_primary(v), json!([1, 2, 3]));
    }

    #[test]
    fn test_unwrap_primary_single_candidate() {
        let v = json!({"messages": [{"id": "1"}], "nextPageToken": "t"});
        assert_eq!(unwrap_primary(v), json!([{"id": "1"}]));
    }

    #[test]
    fn test_select_fields_flat() {
        let v = json!({"id": "1", "name": "Alice", "email": "a@b.com"});
        let result = select_fields(v, &["id".into(), "name".into()]);
        assert_eq!(result, json!({"id": "1", "name": "Alice"}));
    }

    #[test]
    fn test_select_fields_array() {
        let v = json!([{"id": "1", "name": "A"}, {"id": "2", "name": "B"}]);
        let result = select_fields(v, &["id".into()]);
        assert_eq!(result, json!([{"id": "1"}, {"id": "2"}]));
    }

    #[test]
    fn test_get_at_path_nested() {
        let v = json!({"user": {"name": "Alice"}});
        assert_eq!(get_at_path(&v, "user.name"), Some(json!("Alice")));
    }

    #[test]
    fn test_write_json_output() {
        let mut buf = Vec::new();
        let config = OutputConfig { mode: OutputMode::Json, ..Default::default() };
        let data = json!({"hello": "world"});
        write_json(&mut buf, &data, &config).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("\"hello\": \"world\""));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p gog-core`
Expected: all tests PASS

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(core): implement output formatting with JSON transforms"
```

---

### Task 5: Implement gog-core timeparse module

**Files:**
- Create: `crates/gog-core/src/timeparse.rs`

Port `internal/timeparse/parse.go`. Parses relative dates ("today", "tomorrow", "2 days ago"), absolute dates, ISO 8601.

**Step 1: Write timeparse with tests**

```rust
use chrono::{DateTime, Local, NaiveDate, NaiveTime, Duration, Utc, TimeZone};

#[derive(Debug, Clone)]
pub enum ParsedTime {
    DateTime(DateTime<Utc>),
    DateOnly(NaiveDate),
}

/// Parse a flexible date/time string into a UTC DateTime.
/// Supports: "now", "today", "tomorrow", "yesterday", "N days ago",
/// "next week", ISO 8601, and common date formats.
pub fn parse_time(input: &str) -> Result<ParsedTime, String> {
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        return Err("empty time string".into());
    }

    let now = Local::now();
    let today = now.date_naive();

    match input.as_str() {
        "now" => return Ok(ParsedTime::DateTime(Utc::now())),
        "today" => return Ok(ParsedTime::DateOnly(today)),
        "tomorrow" => return Ok(ParsedTime::DateOnly(today + Duration::days(1))),
        "yesterday" => return Ok(ParsedTime::DateOnly(today - Duration::days(1))),
        "next week" => return Ok(ParsedTime::DateOnly(today + Duration::weeks(1))),
        "last week" => return Ok(ParsedTime::DateOnly(today - Duration::weeks(1))),
        _ => {}
    }

    // "N days ago", "N hours ago", etc.
    if let Some(parsed) = parse_relative(&input, now) {
        return Ok(parsed);
    }

    // ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(&input) {
        return Ok(ParsedTime::DateTime(dt.with_timezone(&Utc)));
    }
    // Try with original casing too
    if let Ok(dt) = DateTime::parse_from_rfc3339(input.trim()) {
        return Ok(ParsedTime::DateTime(dt.with_timezone(&Utc)));
    }

    // Date-only: YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        return Ok(ParsedTime::DateOnly(d));
    }

    // MM/DD/YYYY
    if let Ok(d) = NaiveDate::parse_from_str(&input, "%m/%d/%Y") {
        return Ok(ParsedTime::DateOnly(d));
    }

    Err(format!("cannot parse time: {input}"))
}

fn parse_relative(input: &str, now: DateTime<Local>) -> Option<ParsedTime> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() == 3 && parts[2] == "ago" {
        let n: i64 = parts[0].parse().ok()?;
        let duration = match parts[1].trim_end_matches('s') {
            "day" => Duration::days(n),
            "hour" => Duration::hours(n),
            "minute" | "min" => Duration::minutes(n),
            "week" => Duration::weeks(n),
            "month" => Duration::days(n * 30), // approximate
            _ => return None,
        };
        return Some(ParsedTime::DateTime((now - duration).with_timezone(&Utc)));
    }

    // "in N days/hours"
    if parts.len() == 3 && parts[0] == "in" {
        let n: i64 = parts[1].parse().ok()?;
        let duration = match parts[2].trim_end_matches('s') {
            "day" => Duration::days(n),
            "hour" => Duration::hours(n),
            "minute" | "min" => Duration::minutes(n),
            "week" => Duration::weeks(n),
            _ => return None,
        };
        return Some(ParsedTime::DateTime((now + duration).with_timezone(&Utc)));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_today() {
        let result = parse_time("today").unwrap();
        assert!(matches!(result, ParsedTime::DateOnly(_)));
    }

    #[test]
    fn test_parse_tomorrow() {
        let result = parse_time("tomorrow").unwrap();
        if let ParsedTime::DateOnly(d) = result {
            assert_eq!(d, Local::now().date_naive() + Duration::days(1));
        } else {
            panic!("expected DateOnly");
        }
    }

    #[test]
    fn test_parse_days_ago() {
        let result = parse_time("3 days ago").unwrap();
        assert!(matches!(result, ParsedTime::DateTime(_)));
    }

    #[test]
    fn test_parse_in_2_hours() {
        let result = parse_time("in 2 hours").unwrap();
        assert!(matches!(result, ParsedTime::DateTime(_)));
    }

    #[test]
    fn test_parse_iso8601() {
        let result = parse_time("2026-01-15").unwrap();
        if let ParsedTime::DateOnly(d) = result {
            assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        } else {
            panic!("expected DateOnly");
        }
    }

    #[test]
    fn test_parse_empty_fails() {
        assert!(parse_time("").is_err());
    }

    #[test]
    fn test_parse_garbage_fails() {
        assert!(parse_time("not-a-date").is_err());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p gog-core`
Expected: all tests PASS

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(core): implement flexible date/time parsing"
```

---

## Phase 2: Auth & Secrets

### Task 6: Implement gog-secrets crate

**Files:**
- Create: `crates/gog-secrets/Cargo.toml`
- Create: `crates/gog-secrets/src/lib.rs`
- Create: `crates/gog-secrets/src/store.rs`

Port `internal/secrets/store.go`. Uses `keyring` crate for cross-platform secret storage.

**Step 1: Create crate**

`crates/gog-secrets/Cargo.toml`:
```toml
[package]
name = "gog-secrets"
version.workspace = true
edition.workspace = true

[dependencies]
gog-core = { path = "../gog-core" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
chrono.workspace = true
keyring = "3"
```

**Step 2: Write store module with Token type and Store trait**

Port the `Store` trait, `Token` struct, `KeyringStore`, key format (`token:<client>:<email>`), and `ParseTokenKey`. Include tests for key parsing, token roundtrip serialization, and normalize helpers.

The store module should mirror Go's interface: `Keys()`, `SetToken()`, `GetToken()`, `DeleteToken()`, `ListTokens()`, `GetDefaultAccount()`, `SetDefaultAccount()`.

**Step 3: Run tests**

Run: `cargo test -p gog-secrets`
Expected: all tests PASS (keyring tests may need `#[ignore]` for CI since they need a real keyring)

**Step 4: Commit**

```bash
git add -A
git commit -m "feat(secrets): implement keyring-backed token storage"
```

---

### Task 7: Implement gog-auth crate

**Files:**
- Create: `crates/gog-auth/Cargo.toml`
- Create: `crates/gog-auth/src/lib.rs`
- Create: `crates/gog-auth/src/scopes.rs`
- Create: `crates/gog-auth/src/oauth.rs`
- Create: `crates/gog-auth/src/server.rs`
- Create: `crates/gog-auth/src/token.rs`

Port `internal/googleauth/`. Defines Service enum, scope mappings, OAuth 2.0 flow (desktop + manual).

**Step 1: Create crate with Service enum and scopes**

`crates/gog-auth/Cargo.toml`:
```toml
[package]
name = "gog-auth"
version.workspace = true
edition.workspace = true

[dependencies]
gog-core = { path = "../gog-core" }
gog-secrets = { path = "../gog-secrets" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
reqwest = { version = "0.12", features = ["json"] }
open = "5"
url = "2"
```

**Step 2: Implement scopes.rs** - Port the full `Service` enum (7 services for this port: Gmail, Calendar, Drive, Contacts, Chat, Keep, Forms), all scope mappings, `ScopeOptions` (readonly, drive scope mode), and `scopes_for_services()`.

**Step 3: Implement oauth.rs** - Port the desktop OAuth flow: build auth URL, start local HTTP callback server, exchange code for token, store in keyring. Also port the manual flow (for headless).

**Step 4: Implement token.rs** - Token validation, email extraction from token.

**Step 5: Run tests**

Run: `cargo test -p gog-auth`
Expected: scope/service parsing tests PASS, OAuth flow tests may need mocking

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(auth): implement OAuth 2.0 flow and scope management"
```

---

## Phase 3: API Transport

### Task 8: Implement gog-api crate

**Files:**
- Create: `crates/gog-api/Cargo.toml`
- Create: `crates/gog-api/src/lib.rs`
- Create: `crates/gog-api/src/client.rs`
- Create: `crates/gog-api/src/transport.rs`
- Create: `crates/gog-api/src/error.rs`
- Create: `crates/gog-api/src/service_account.rs`

Port `internal/googleapi/`. Provides authenticated HTTP client, retry transport with circuit breaker, service factories for Google API clients.

**Step 1: Create crate**

`crates/gog-api/Cargo.toml`:
```toml
[package]
name = "gog-api"
version.workspace = true
edition.workspace = true

[dependencies]
gog-core = { path = "../gog-core" }
gog-auth = { path = "../gog-auth" }
gog-secrets = { path = "../gog-secrets" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
reqwest = { version = "0.12", features = ["json"] }
# Google API clients (add per service as needed)
google-gmail1 = "*"
google-calendar3 = "*"
google-drive3 = "*"
google-people1 = "*"
google-chat1 = "*"
google-keep1 = "*"
google-forms1 = "*"
yup-oauth2 = "11"
hyper = "1"
hyper-rustls = "0.27"
hyper-util = "0.1"
```

**Step 2: Implement transport.rs** - Port `RetryTransport` with exponential backoff, circuit breaker, 429/5xx retry logic. Use reqwest middleware or a wrapper.

**Step 3: Implement client.rs** - Build authenticated hyper client using `yup-oauth2` token source. Port `optionsForAccount()` pattern - resolve credentials, get refresh token from keyring, create authenticated HTTP client.

**Step 4: Implement service_account.rs** - Port service account impersonation for Keep (domain-wide delegation).

**Step 5: Run tests**

Run: `cargo test -p gog-api`
Expected: transport retry tests PASS

**Step 6: Commit**

```bash
git add -A
git commit -m "feat(api): implement authenticated transport with retry and circuit breaker"
```

---

## Phase 4: Service Crates (one per service)

Each service crate follows the same pattern. I'll detail Gmail as the template, then list the others.

### Task 9: Implement gog-gmail crate (template for all services)

**Files:**
- Create: `crates/gog-gmail/Cargo.toml`
- Create: `crates/gog-gmail/src/lib.rs`
- Create: `crates/gog-gmail/src/search.rs`
- Create: `crates/gog-gmail/src/get.rs`
- Create: `crates/gog-gmail/src/send.rs`
- Create: `crates/gog-gmail/src/labels.rs`
- Create: `crates/gog-gmail/src/thread.rs`
- Create: `crates/gog-gmail/src/mime.rs`

Port core Gmail operations from `internal/cmd/gmail*.go`.

**Step 1: Create crate skeleton**

**Step 2: Implement search** - Port `GmailSearchCmd.Run()`: call Gmail API messages.list with query, format results.

**Step 3: Implement get** - Port `GmailGetCmd.Run()`: fetch message by ID, decode MIME body.

**Step 4: Implement mime.rs** - Port `gmail_mime.go`: MIME parsing, quoted-printable decoding, RFC 2047 header decoding.

**Step 5: Implement send** - Port `GmailSendCmd.Run()`: compose MIME message, handle attachments, reply/forward.

**Step 6: Implement labels** - Port label CRUD operations.

**Step 7: Implement thread** - Port thread view with message grouping.

**Step 8: Run tests, commit**

```bash
cargo test -p gog-gmail
git add -A && git commit -m "feat(gmail): implement search, get, send, labels, thread"
```

---

### Task 10: Implement gog-calendar crate

**Files:** `crates/gog-calendar/src/{lib,list,create,edit,delete,freebusy,colors}.rs`

Port `internal/cmd/calendar*.go`. Key features: event CRUD, recurrence rules, free/busy, conflict detection, timezone handling, all-day events.

---

### Task 11: Implement gog-drive crate

**Files:** `crates/gog-drive/src/{lib,list,search,upload,download,permissions,share}.rs`

Port `internal/cmd/drive*.go`. Key features: file listing, search, upload/download, permissions, shared drives.

---

### Task 12: Implement gog-contacts crate

**Files:** `crates/gog-contacts/src/{lib,search,create,update,delete,groups}.rs`

Port `internal/cmd/contacts*.go`. Uses People API. Key features: contact CRUD, custom fields, birthday, directory.

---

### Task 13: Implement gog-chat crate

**Files:** `crates/gog-chat/src/{lib,spaces,messages,members}.rs`

Port `internal/cmd/chat*.go`. Key features: spaces, messages, threads, DMs.

---

### Task 14: Implement gog-keep crate

**Files:** `crates/gog-keep/src/{lib,list,get}.rs`

Port `internal/cmd/keep.go`. Read-only, requires service account.

---

### Task 15: Implement gog-forms crate

**Files:** `crates/gog-forms/src/{lib,get,list,responses}.rs`

Port `internal/cmd/forms*.go`. Key features: form get, list responses.

---

## Phase 5: CLI Binary

### Task 16: Implement gog-cli binary crate

**Files:**
- Create: `crates/gog-cli/Cargo.toml`
- Create: `crates/gog-cli/src/main.rs`
- Create: `crates/gog-cli/src/commands/mod.rs`
- Create: `crates/gog-cli/src/commands/auth.rs`
- Create: `crates/gog-cli/src/commands/gmail.rs`
- Create: `crates/gog-cli/src/commands/calendar.rs`
- Create: `crates/gog-cli/src/commands/drive.rs`
- Create: `crates/gog-cli/src/commands/contacts.rs`
- Create: `crates/gog-cli/src/commands/chat.rs`
- Create: `crates/gog-cli/src/commands/keep.rs`
- Create: `crates/gog-cli/src/commands/forms.rs`
- Create: `crates/gog-cli/src/output.rs`
- Create: `crates/gog-cli/src/error.rs`

**Step 1: Create crate with clap CLI structure**

`crates/gog-cli/Cargo.toml`:
```toml
[package]
name = "gog-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "gog"
path = "src/main.rs"

[dependencies]
gog-core = { path = "../gog-core" }
gog-auth = { path = "../gog-auth" }
gog-secrets = { path = "../gog-secrets" }
gog-api = { path = "../gog-api" }
gog-gmail = { path = "../gog-gmail" }
gog-calendar = { path = "../gog-calendar" }
gog-drive = { path = "../gog-drive" }
gog-contacts = { path = "../gog-contacts" }
gog-chat = { path = "../gog-chat" }
gog-keep = { path = "../gog-keep" }
gog-forms = { path = "../gog-forms" }
clap = { version = "4", features = ["derive", "env"] }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
crossterm = "0.28"
colored = "2"
```

**Step 2: Implement main.rs with clap CLI structure**

```rust
use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod error;
mod output;

#[derive(Parser)]
#[command(name = "gog", version, about = "Google Workspace CLI")]
struct Cli {
    /// Account email for API commands
    #[arg(short, long, env = "GOG_ACCOUNT")]
    account: Option<String>,

    /// OAuth client name
    #[arg(long, env = "GOG_CLIENT", default_value = "default")]
    client: String,

    /// Output JSON
    #[arg(short, long, env = "GOG_JSON")]
    json: bool,

    /// Output stable TSV
    #[arg(short, long, env = "GOG_PLAIN")]
    plain: bool,

    /// Drop envelope in JSON mode
    #[arg(long)]
    results_only: bool,

    /// Select fields in JSON mode
    #[arg(long)]
    select: Option<String>,

    /// Preview mode (no mutations)
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Skip confirmations
    #[arg(short = 'y', long)]
    force: bool,

    /// Never prompt
    #[arg(long)]
    no_input: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Auth and credentials
    Auth(commands::auth::AuthCmd),
    /// Gmail
    #[command(alias = "mail", alias = "email")]
    Gmail(commands::gmail::GmailCmd),
    /// Google Calendar
    #[command(alias = "cal")]
    Calendar(commands::calendar::CalendarCmd),
    /// Google Drive
    #[command(alias = "drv")]
    Drive(commands::drive::DriveCmd),
    /// Google Contacts
    #[command(alias = "contact")]
    Contacts(commands::contacts::ContactsCmd),
    /// Google Chat
    Chat(commands::chat::ChatCmd),
    /// Google Keep
    Keep(commands::keep::KeepCmd),
    /// Google Forms
    #[command(alias = "form")]
    Forms(commands::forms::FormsCmd),
    /// Print version
    Version,
    // Desire path aliases
    /// Send email (alias for gmail send)
    Send(commands::gmail::GmailSendCmd),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let exit_code = match run(cli).await {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e.format_for_user());
            e.exit_code()
        }
    };
    process::exit(exit_code);
}

async fn run(cli: Cli) -> Result<(), gog_core::error::GogError> {
    let output_config = gog_core::output::OutputConfig::from_flags(cli.json, cli.plain)
        .map_err(|e| gog_core::error::GogError::Usage(e))?;

    // Dispatch to subcommand
    match cli.command {
        Commands::Version => {
            if output_config.is_json() {
                let version = env!("CARGO_PKG_VERSION");
                println!("{}", serde_json::json!({"version": version}));
            } else {
                println!("gog {}", env!("CARGO_PKG_VERSION"));
            }
            Ok(())
        }
        // Each command delegates to its service crate
        _ => todo!("command dispatch"),
    }
}
```

**Step 3: Wire up each command module** - Each `commands/*.rs` file defines clap subcommands that delegate to the corresponding service crate.

**Step 4: Run basic smoke test**

Run: `cargo run -p gog-cli -- version`
Expected: prints version

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(cli): implement main binary with clap CLI structure"
```

---

## Phase 6: Integration & Polish

### Task 17: Add CI and linting

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `rustfmt.toml`
- Create: `clippy.toml`

Set up GitHub Actions: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`.

---

### Task 18: Add shell completions

**Files:**
- Modify: `crates/gog-cli/src/commands/mod.rs`

Use `clap_complete` to generate bash/zsh/fish completions matching Go version's `gog completion` command.

---

### Task 19: Cross-platform build and release

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `Cargo.toml` profile settings

Set up cross-compilation for macOS (arm64+x86_64), Linux (x86_64+arm64), Windows. Binary signing.

---

## Implementation Order Summary

| Phase | Tasks | Crates | Est. Files |
|-------|-------|--------|-----------|
| 1. Foundation | 1-5 | gog-core | ~8 files |
| 2. Auth | 6-7 | gog-secrets, gog-auth | ~8 files |
| 3. Transport | 8 | gog-api | ~6 files |
| 4. Services | 9-15 | gog-gmail through gog-forms | ~35 files |
| 5. CLI | 16 | gog-cli | ~12 files |
| 6. Polish | 17-19 | CI/release | ~4 files |

**Total: 19 tasks, ~73 files, 12 crates**

Each task produces a working, tested, committable unit. Services can be implemented in parallel once Phase 3 is complete.
