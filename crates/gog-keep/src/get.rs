// get.rs - Fetch a single Google Keep note using the Keep API v1.

use reqwest::Client;

use crate::{KeepError, types::Note};

const KEEP_BASE: &str = "https://keep.googleapis.com/v1";

/// Get a single note by its resource name or ID.
///
/// `note_id` may be either the bare ID (e.g. `"abc123"`) or the full
/// resource name (e.g. `"notes/abc123"`).
///
/// Uses: `GET https://keep.googleapis.com/v1/notes/{noteId}`
pub async fn get_note(
    client: &Client,
    access_token: &str,
    note_id: &str,
) -> Result<Note, KeepError> {
    // Accept either a bare ID or a full resource name.
    let resource = if note_id.starts_with("notes/") {
        note_id.to_string()
    } else {
        format!("notes/{note_id}")
    };

    let url = format!("{KEEP_BASE}/{resource}");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(KeepError::Api { status, message });
    }

    let note: Note = resp.json().await?;
    Ok(note)
}
