# gogcli-rs: Rust Port Design

**Date:** 2026-02-25
**Status:** Approved

## Goals

- Port gogcli (Go) to Rust for performance, ecosystem benefits, and distribution
- Maintain identical CLI command structure (drop-in replacement)
- Support 7 services: Gmail, Calendar, Drive, Contacts, Chat, Keep, Forms
- Share config directory with Go version (no re-authentication needed)

## Crate Architecture

Cargo workspace with per-service crates:

```
~/rust/gogcli-rs/
├── Cargo.toml                    # workspace definition
├── crates/
│   ├── gog-cli/                  # Binary crate - CLI entry point (clap)
│   ├── gog-core/                 # Shared types, config, errors, timeparse
│   ├── gog-auth/                 # OAuth 2.0 flow, token management, scopes
│   ├── gog-secrets/              # Keyring abstraction (keyring-rs)
│   ├── gog-api/                  # Google API transport, retry, circuit breaker
│   ├── gog-gmail/                # Gmail service logic
│   ├── gog-calendar/             # Calendar service logic
│   ├── gog-drive/                # Drive service logic
│   ├── gog-contacts/             # Contacts service logic
│   ├── gog-chat/                 # Chat service logic
│   ├── gog-keep/                 # Keep service logic
│   └── gog-forms/                # Forms service logic
```

**Dependency flow:** `gog-cli` → `gog-{service}` → `gog-api` → `gog-auth` → `gog-secrets` → `gog-core`

## Key Dependencies

| Rust Crate | Replaces (Go) | Purpose |
|---|---|---|
| `clap` v4 (derive) | `kong` | CLI parsing |
| `google-gmail1`, `google-calendar3`, etc. | `google.golang.org/api` | Google API clients |
| `yup-oauth2` | `golang.org/x/oauth2` | OAuth2 for Google APIs |
| `keyring` | `99designs/keyring` | Cross-platform secret storage |
| `tokio` | goroutines | Async runtime |
| `reqwest` | `net/http` | HTTP client |
| `serde` + `serde_json` | `encoding/json` | Serialization |
| `json5` | `json5` Go lib | JSON5 config parsing |
| `thiserror` + `anyhow` | `errors` | Error handling |
| `chrono` + `chrono-tz` | `time` | Date/time + timezone |
| `crossterm` + `colored` | `termenv` | Terminal colors + TTY |
| `dirs` | custom paths | XDG config directories |
| `open` | `open_browser.go` | Open URL in browser |

## CLI Compatibility

Identical command structure and global flags:

```bash
gog gmail search "from:alice"
gog calendar list --from today --to "next week"
gog --account user@gmail.com --json gmail search "query"
```

Global flags: `--account/-a`, `--client`, `--json/-j`, `--plain/-p`, `--color`, `--dry-run/-n`, `--force/-y`, `--no-input`, `--verbose/-v`, `--results-only`, `--select`

## Auth & Secrets

- Reuse `yup-oauth2` for token management (google-apis-rs dependency)
- `keyring` crate for cross-platform keyring access
- Read same config directory (`~/.config/gogcli/`) - backward compatible
- Same token key format: `token:<client>:<email>`
- Service account support via `yup-oauth2::ServiceAccountAuthenticator`

## Error Handling

Custom error types via `thiserror`:
- `AuthRequired`, `NotFound`, `PermissionDenied`, `RateLimited`
- Transparent wrapping of API and IO errors
- Exit codes match Go version for script compatibility

## Testing

- Unit: `#[cfg(test)]` modules per crate
- Integration: `wiremock` for mock HTTP responses
- Port Go test fixtures/golden files
- CI: `cargo test` + `cargo clippy` + `cargo fmt --check`

## Services Scope

| Service | Go Source | Priority |
|---------|-----------|----------|
| Gmail | `gmail*.go` (~7K lines) | P0 |
| Calendar | `calendar*.go` (~11K lines) | P0 |
| Drive | `drive*.go` | P0 |
| Contacts | `contacts*.go` | P1 |
| Chat | `chat*.go` | P1 |
| Keep | `keep.go` | P2 |
| Forms | `forms*.go` | P2 |

## Complexity Notes

- **Calendar recurrence**: Complex RRULE parsing + conflict detection
- **Gmail MIME**: Quoted-printable, RFC 2047 headers, ISO-2022-JP
- **OAuth flow**: Desktop + manual/headless modes
- **Timezone handling**: Calendar events across zones, date-only vs datetime
