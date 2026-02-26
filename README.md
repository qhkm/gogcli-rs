# gogcli-rs

A fast, ergonomic Google Workspace CLI written in Rust. Port of [gogcli](https://github.com/qhkm/gogcli) (Go).

## Features

- **Gmail** — search, read, send, labels, threads
- **Calendar** — list events, create, delete, freebusy, list calendars
- **Drive** — list, search, upload, download, file info
- **Contacts** — search, create, update, delete, contact groups
- **Chat** — spaces, messages, members
- **Keep** — list notes, get note
- **Forms** — get form, list/get responses

All commands support `--json`, `--plain` (TSV), `--results-only`, and `--select` output modes.

## Installation

```bash
# Build from source
cargo build --release
cp target/release/gog ~/.local/bin/
```

## Setup

### 1. Create OAuth Client

1. Go to [Google Cloud Console > Credentials](https://console.cloud.google.com/auth/clients)
2. Click **"Create Client"**
3. Application type: **"Desktop app"** (required for loopback redirect)
4. Download the JSON file

### 2. Store Credentials

```bash
# Copy the downloaded JSON to the config directory
# macOS: ~/Library/Application Support/gogcli/
# Linux: ~/.config/gogcli/
cp client_secret_*.json "$(gog config dir)/credentials.json"
```

The file should have the `installed` key (Google's standard format):
```json
{
  "installed": {
    "client_id": "...",
    "client_secret": "..."
  }
}
```

### 3. Enable APIs

Enable these APIs in your Google Cloud project:
- Gmail API
- Google Calendar API
- Google Drive API
- People API (for contacts)
- Google Chat API
- Google Keep API
- Google Forms API

### 4. Authorize

```bash
gog auth add user@gmail.com
```

This opens your browser for OAuth consent. A local server on a random port captures the callback automatically.

## Usage

```bash
# Gmail
gog gmail search "from:boss subject:urgent" --max-results 10
gog gmail get <message-id>
gog gmail send --to user@example.com --subject "Hello" --body "Hi there"
gog gmail labels

# Calendar
gog calendar list --max-results 10
gog calendar create --summary "Meeting" --start "2024-03-01T10:00:00" --end "2024-03-01T11:00:00"
gog calendar calendars

# Drive
gog drive ls --max-results 20
gog drive search "quarterly report"
gog drive upload ./file.pdf
gog drive download <file-id> -o ./output.pdf

# Contacts
gog contacts search "John"
gog contacts create --name "Jane Doe" --email "jane@example.com"

# Chat
gog chat spaces
gog chat messages <space-name>

# Keep
gog keep list
gog keep get <note-id>

# Forms
gog forms get <form-id>
gog forms responses <form-id>
```

### Output Modes

```bash
gog gmail search "test" --json                    # JSON output
gog gmail search "test" --plain                   # TSV output (for scripts)
gog gmail search "test" --json --results-only     # Strip envelope, just results
gog gmail search "test" --json --select "id,threadId"  # Select specific fields
```

### Multiple Accounts

```bash
gog auth add work@company.com
gog auth add personal@gmail.com

gog --account work@company.com gmail search "project"
gog --account personal@gmail.com drive ls
```

## Architecture

Cargo workspace with 12 crates:

```
gogcli-rs/
├── crates/
│   ├── gog-cli          # Binary — clap v4 CLI
│   ├── gog-core         # Config, output, error types, time parsing
│   ├── gog-auth         # OAuth 2.0 flow, scopes, local callback server
│   ├── gog-secrets      # Token storage (file-based + keychain backends)
│   ├── gog-api          # Authenticated HTTP client, retry, circuit breaker
│   ├── gog-gmail        # Gmail API types and operations
│   ├── gog-calendar     # Calendar API types and operations
│   ├── gog-drive        # Drive API types and operations
│   ├── gog-contacts     # People API types and operations
│   ├── gog-chat         # Chat API types and operations
│   ├── gog-keep         # Keep API types and operations
│   └── gog-forms        # Forms API types and operations
```

Dependency flow: `gog-cli → gog-{service} → gog-api → gog-auth → gog-secrets → gog-core`

## Configuration

Config file: `~/Library/Application Support/gogcli/config.json` (macOS) or `~/.config/gogcli/config.json` (Linux)

```json5
{
  keyring_backend: "file",        // "file" (default), "keychain", "secret-service"
  default_timezone: "Asia/Kuala_Lumpur",
  account_aliases: {
    "me@example.com": "me"
  }
}
```

## Token Storage

Tokens are stored based on the `keyring_backend` config:

- **`file`** (default) — JSON files in `<config_dir>/keyring/`, one per account
- **`keychain`** — macOS Keychain
- **`secret-service`** — Linux Secret Service (GNOME Keyring, KDE Wallet)

## License

Apache-2.0
