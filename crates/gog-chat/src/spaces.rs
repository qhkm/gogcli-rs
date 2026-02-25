// Google Chat API v1 - Spaces
// Ported from: Chat API spaces resource
// Endpoint: GET https://chat.googleapis.com/v1/spaces

use crate::error::ChatError;
use crate::types::{Space, SpaceList};

const SPACES_ENDPOINT: &str = "https://chat.googleapis.com/v1/spaces";

/// List all spaces the caller is a member of.
///
/// Calls `GET /v1/spaces` and returns the deserialized `SpaceList`.
/// Pass `page_token` to retrieve subsequent pages.
pub async fn list_spaces(
    client: &reqwest::Client,
    page_token: Option<&str>,
    page_size: Option<u32>,
) -> Result<SpaceList, ChatError> {
    let mut req = client.get(SPACES_ENDPOINT);

    if let Some(token) = page_token {
        req = req.query(&[("pageToken", token)]);
    }
    if let Some(size) = page_size {
        req = req.query(&[("pageSize", size.to_string().as_str())]);
    }

    let resp = req.send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api { status, message: msg });
    }

    let list: SpaceList = resp.json().await?;
    Ok(list)
}

/// Get a single space by its resource name (e.g. `spaces/AAAABBBBBBB`).
///
/// Calls `GET /v1/{name}`.
pub async fn get_space(
    client: &reqwest::Client,
    space_name: &str,
) -> Result<Space, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}", space_name);
    let resp = client.get(&url).send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api { status, message: msg });
    }

    let space: Space = resp.json().await?;
    Ok(space)
}

/// Collect all spaces across all pages, following `nextPageToken` automatically.
///
/// Returns a flat `Vec<Space>`. Uses the default page size set by the API.
pub async fn list_all_spaces(client: &reqwest::Client) -> Result<Vec<Space>, ChatError> {
    let mut all = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let list = list_spaces(client, page_token.as_deref(), None).await?;
        all.extend(list.spaces);

        match list.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }

    Ok(all)
}
