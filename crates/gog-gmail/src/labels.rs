// labels.rs - Gmail label operations.

use crate::{types::Label, GmailError};
use reqwest::Client;
use serde_json::json;

const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// List all labels for the authenticated user.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/labels`
pub async fn list_labels(client: &Client, access_token: &str) -> Result<Vec<Label>, GmailError> {
    let url = format!("{GMAIL_BASE}/labels");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    #[derive(serde::Deserialize)]
    struct LabelsResponse {
        #[serde(default)]
        labels: Vec<Label>,
    }

    let body: LabelsResponse = resp.json().await?;
    Ok(body.labels)
}

/// Get a specific label by ID.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/labels/{id}`
pub async fn get_label(
    client: &Client,
    access_token: &str,
    label_id: &str,
) -> Result<Label, GmailError> {
    let url = format!("{GMAIL_BASE}/labels/{label_id}");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let label: Label = resp.json().await?;
    Ok(label)
}

/// Create a new label.
///
/// Uses: `POST https://gmail.googleapis.com/gmail/v1/users/me/labels`
pub async fn create_label(
    client: &Client,
    access_token: &str,
    name: &str,
) -> Result<Label, GmailError> {
    let url = format!("{GMAIL_BASE}/labels");

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({ "name": name }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let label: Label = resp.json().await?;
    Ok(label)
}

/// Delete a label by ID.
///
/// Uses: `DELETE https://gmail.googleapis.com/gmail/v1/users/me/labels/{id}`
pub async fn delete_label(
    client: &Client,
    access_token: &str,
    label_id: &str,
) -> Result<(), GmailError> {
    let url = format!("{GMAIL_BASE}/labels/{label_id}");

    let resp = client.delete(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    Ok(())
}

/// Modify labels on a message (add and/or remove).
///
/// Uses: `POST https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}/modify`
pub async fn modify_message_labels(
    client: &Client,
    access_token: &str,
    message_id: &str,
    add_label_ids: &[String],
    remove_label_ids: &[String],
) -> Result<crate::types::Message, GmailError> {
    let url = format!("{GMAIL_BASE}/messages/{message_id}/modify");

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&json!({
            "addLabelIds": add_label_ids,
            "removeLabelIds": remove_label_ids,
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let msg: crate::types::Message = resp.json().await?;
    Ok(msg)
}
