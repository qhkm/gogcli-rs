// thread.rs - Gmail thread operations.

use crate::{types::Thread, GmailError};
use reqwest::Client;

const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// Fetch a complete thread (with all messages) by thread ID.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/threads/{id}`
pub async fn get_thread(
    client: &Client,
    access_token: &str,
    thread_id: &str,
) -> Result<Thread, GmailError> {
    let url = format!("{GMAIL_BASE}/threads/{thread_id}");

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("format", "full")])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let thread: Thread = resp.json().await?;
    Ok(thread)
}

/// List threads matching a search query.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/threads?q={query}`
pub async fn list_threads(
    client: &Client,
    access_token: &str,
    query: &str,
    max_results: Option<u32>,
    page_token: Option<&str>,
) -> Result<ThreadList, GmailError> {
    let url = format!("{GMAIL_BASE}/threads");

    let mut req = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("q", query)]);

    if let Some(max) = max_results {
        req = req.query(&[("maxResults", max.to_string().as_str())]);
    }
    if let Some(token) = page_token {
        req = req.query(&[("pageToken", token)]);
    }

    let resp = req.send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let list: ThreadList = resp.json().await?;
    Ok(list)
}

/// Summary of a thread as returned in list results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadRef {
    pub id: String,
    pub snippet: Option<String>,
    pub history_id: Option<String>,
}

/// Response from the threads list endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadList {
    #[serde(default)]
    pub threads: Vec<ThreadRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
}
