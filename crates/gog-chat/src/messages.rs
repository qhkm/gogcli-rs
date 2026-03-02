// Google Chat API v1 - Messages
// Endpoints:
//   GET  https://chat.googleapis.com/v1/{parent}/messages
//   POST https://chat.googleapis.com/v1/{parent}/messages

use serde_json::Value;

use crate::error::ChatError;
use crate::types::{ChatMessage, MessageList};

/// List messages in a space.
///
/// `space_name` should be the resource name of the space,
/// e.g. `spaces/AAAABBBBBBB`.
///
/// Calls `GET /v1/{space_name}/messages`.
pub async fn list_messages(
    client: &reqwest::Client,
    space_name: &str,
    page_token: Option<&str>,
    page_size: Option<u32>,
    filter: Option<&str>,
) -> Result<MessageList, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}/messages", space_name);
    let mut req = client.get(&url);

    if let Some(token) = page_token {
        req = req.query(&[("pageToken", token)]);
    }
    if let Some(size) = page_size {
        req = req.query(&[("pageSize", size.to_string().as_str())]);
    }
    if let Some(f) = filter {
        req = req.query(&[("filter", f)]);
    }

    let resp = req.send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api {
            status,
            message: msg,
        });
    }

    let list: MessageList = resp.json().await?;
    Ok(list)
}

/// Get a single message by its resource name,
/// e.g. `spaces/AAA/messages/BBB`.
///
/// Calls `GET /v1/{message_name}`.
pub async fn get_message(
    client: &reqwest::Client,
    message_name: &str,
) -> Result<ChatMessage, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}", message_name);
    let resp = client.get(&url).send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api {
            status,
            message: msg,
        });
    }

    let message: ChatMessage = resp.json().await?;
    Ok(message)
}

/// Send a plain-text message to a space.
///
/// `space_name` should be the resource name of the space,
/// e.g. `spaces/AAAABBBBBBB`.
///
/// Calls `POST /v1/{space_name}/messages`.
///
/// Returns the created `ChatMessage` as returned by the API.
pub async fn send_message(
    client: &reqwest::Client,
    space_name: &str,
    text: &str,
) -> Result<ChatMessage, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}/messages", space_name);
    let body = serde_json::json!({ "text": text });

    let resp = client.post(&url).json(&body).send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api {
            status,
            message: msg,
        });
    }

    let message: ChatMessage = resp.json().await?;
    Ok(message)
}

/// Send a message with an arbitrary JSON body to a space.
///
/// Use this when you need full control over the request body, e.g. to attach
/// cards, specify a thread, or set a message ID.
///
/// Calls `POST /v1/{space_name}/messages`.
pub async fn send_message_raw(
    client: &reqwest::Client,
    space_name: &str,
    body: Value,
) -> Result<ChatMessage, ChatError> {
    let url = format!("https://chat.googleapis.com/v1/{}/messages", space_name);

    let resp = client.post(&url).json(&body).send().await?;
    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(ChatError::Api {
            status,
            message: msg,
        });
    }

    let message: ChatMessage = resp.json().await?;
    Ok(message)
}

/// Collect all messages in a space across all pages.
///
/// Follows `nextPageToken` automatically and returns a flat `Vec<ChatMessage>`.
pub async fn list_all_messages(
    client: &reqwest::Client,
    space_name: &str,
    filter: Option<&str>,
) -> Result<Vec<ChatMessage>, ChatError> {
    let mut all = Vec::new();
    let mut page_token: Option<String> = None;

    loop {
        let list = list_messages(client, space_name, page_token.as_deref(), None, filter).await?;
        all.extend(list.messages);

        match list.next_page_token {
            Some(token) if !token.is_empty() => page_token = Some(token),
            _ => break,
        }
    }

    Ok(all)
}
