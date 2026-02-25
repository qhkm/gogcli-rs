// Google Chat API v1 - Members (Memberships)
// Endpoint: GET https://chat.googleapis.com/v1/{parent}/members

use crate::error::ChatError;
use crate::types::{Member, MemberList};

/// List members of a space.
///
/// `space_name` should be the resource name of the space,
/// e.g. `spaces/AAAABBBBBBB`.
///
/// Calls `GET /v1/{space_name}/members`.
pub async fn list_members(
    client: &reqwest::Client,
    space_name: &str,
    page_token: Option<&str>,
    page_size: Option<u32>,
) -> Result<MemberList, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}/members", space_name);
    let mut req = client.get(&url);

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

    let list: MemberList = resp.json().await?;
    Ok(list)
}

/// Get a single membership by its resource name,
/// e.g. `spaces/AAA/members/BBB`.
///
/// Calls `GET /v1/{member_name}`.
pub async fn get_member(
    client: &reqwest::Client,
    member_name: &str,
) -> Result<Member, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}", member_name);
    let resp = client.get(&url).send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api { status, message: msg });
    }

    let member: Member = resp.json().await?;
    Ok(member)
}

/// Collect all members of a space across all pages.
///
/// Follows `nextPageToken` automatically and returns a flat `Vec<Member>`.
pub async fn list_all_members(
    client: &reqwest::Client,
    space_name: &str,
) -> Result<Vec<Member>, ChatError> {
    let mut all = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let list = list_members(client, space_name, page_token.as_deref(), None).await?;
        all.extend(list.memberships);

        match list.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }

    Ok(all)
}
