// search.rs - Search Gmail messages using the Gmail API.
// Ported from internal Gmail search functionality.

use crate::{types::MessageList, GmailError};
use reqwest::Client;

const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// Parameters for a Gmail message search.
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    /// Gmail search query (e.g. "from:alice subject:hello").
    pub query: String,
    /// Maximum number of messages to return (Gmail default is 100, max 500).
    pub max_results: Option<u32>,
    /// Page token returned by a previous search to fetch the next page.
    pub page_token: Option<String>,
    /// Only return messages with these label IDs applied.
    pub label_ids: Vec<String>,
}

/// Search Gmail messages using the Gmail API.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/messages?q={query}`
pub async fn search_messages(
    client: &Client,
    access_token: &str,
    params: &SearchParams,
) -> Result<MessageList, GmailError> {
    let url = format!("{GMAIL_BASE}/messages");

    let mut req = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("q", params.query.as_str())]);

    if let Some(max) = params.max_results {
        req = req.query(&[("maxResults", max.to_string().as_str())]);
    }
    if let Some(token) = &params.page_token {
        req = req.query(&[("pageToken", token.as_str())]);
    }
    for label_id in &params.label_ids {
        req = req.query(&[("labelIds", label_id.as_str())]);
    }

    let resp = req.send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let list: MessageList = resp.json().await?;
    Ok(list)
}
