# Changelog

## 0.1.0 (2026-02-26)

Initial release — full Rust port of gogcli (Go).

### Features

- **12-crate workspace** — modular architecture with clear dependency boundaries
- **7 Google services** — Gmail, Calendar, Drive, Contacts, Chat, Keep, Forms
- **OAuth 2.0 browser flow** — local callback server on random port, CSRF state validation, auto browser open
- **File-based token store** — stores tokens as JSON files in config directory (matches Go version's `keyring_backend: "file"`)
- **Keychain support** — macOS Keychain and Linux Secret Service backends
- **Output modes** — JSON (`--json`), TSV (`--plain`), field selection (`--select`), envelope stripping (`--results-only`)
- **Retry with circuit breaker** — exponential backoff, configurable thresholds
- **Shell completions** — bash, zsh, fish, powershell via `gog completion`
- **Multi-account** — per-client credential isolation, account aliases

### Commands

| Command | Subcommands |
|---------|-------------|
| `gog auth` | `add`, `remove`, `status`, `list` |
| `gog config` | `show`, `get`, `set`, `reset` |
| `gog gmail` | `search`, `get`, `send`, `labels`, `thread` |
| `gog calendar` | `list`, `create`, `delete`, `freebusy`, `calendars` |
| `gog drive` | `ls`, `search`, `upload`, `download`, `get` |
| `gog contacts` | `search`, `create`, `update`, `delete`, `groups` |
| `gog chat` | `spaces`, `messages`, `members` |
| `gog keep` | `list`, `get` |
| `gog forms` | `get`, `responses` |

### Tests

- 280 unit/integration tests across all 12 crates
- End-to-end tested: Gmail, Calendar, Drive working against live Google APIs
