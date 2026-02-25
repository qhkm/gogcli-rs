// list.rs - List Google Keep notes using the Keep API v1.

use reqwest::Client;

use crate::{KeepError, types::NoteList};

const KEEP_BASE: &str = "https://keep.googleapis.com/v1";

/// Parameters for listing Keep notes.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    /// Maximum number of notes to return per page (API default applies if `None`).
    pub page_size: Option<u32>,
    /// Page token from a previous list response.
    pub page_token: Option<String>,
    /// Filter expression (e.g. `"trashed = false"`).
    pub filter: Option<String>,
}

/// List notes from Google Keep.
///
/// Uses: `GET https://keep.googleapis.com/v1/notes`
pub async fn list_notes(
    client: &Client,
    access_token: &str,
    params: &ListParams,
) -> Result<NoteList, KeepError> {
    let url = format!("{KEEP_BASE}/notes");

    let mut req = client.get(&url).bearer_auth(access_token);

    if let Some(size) = params.page_size {
        req = req.query(&[("pageSize", size.to_string())]);
    }
    if let Some(token) = &params.page_token {
        req = req.query(&[("pageToken", token.as_str())]);
    }
    if let Some(filter) = &params.filter {
        req = req.query(&[("filter", filter.as_str())]);
    }

    let resp = req.send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(KeepError::Api { status, message });
    }

    let list: NoteList = resp.json().await?;
    Ok(list)
}
