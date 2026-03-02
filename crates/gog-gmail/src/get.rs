// get.rs - Fetch a single Gmail message by ID.

use crate::{types::Message, GmailError};
use reqwest::Client;

const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// Format in which to retrieve the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageFormat {
    /// Full message including headers and body (default).
    Full,
    /// Only headers (no body).
    Metadata,
    /// Minimal data (IDs and labels only).
    Minimal,
    /// Raw RFC 2822 message in base64url.
    Raw,
}

impl MessageFormat {
    fn as_str(self) -> &'static str {
        match self {
            MessageFormat::Full => "full",
            MessageFormat::Metadata => "metadata",
            MessageFormat::Minimal => "minimal",
            MessageFormat::Raw => "raw",
        }
    }
}

impl std::fmt::Display for MessageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Fetch a single Gmail message by ID.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}?format={format}`
pub async fn get_message(
    client: &Client,
    access_token: &str,
    message_id: &str,
    format: MessageFormat,
) -> Result<Message, GmailError> {
    let url = format!("{GMAIL_BASE}/messages/{message_id}");

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("format", format.as_str())])
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let msg: Message = resp.json().await?;
    Ok(msg)
}

/// Fetch an attachment body by attachment ID.
///
/// Uses: `GET https://gmail.googleapis.com/gmail/v1/users/me/messages/{messageId}/attachments/{id}`
pub async fn get_attachment(
    client: &Client,
    access_token: &str,
    message_id: &str,
    attachment_id: &str,
) -> Result<crate::types::MessagePartBody, GmailError> {
    let url = format!("{GMAIL_BASE}/messages/{message_id}/attachments/{attachment_id}");

    let resp = client.get(&url).bearer_auth(access_token).send().await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let body: crate::types::MessagePartBody = resp.json().await?;
    Ok(body)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_format_display() {
        assert_eq!(MessageFormat::Full.to_string(), "full");
        assert_eq!(MessageFormat::Metadata.to_string(), "metadata");
        assert_eq!(MessageFormat::Minimal.to_string(), "minimal");
        assert_eq!(MessageFormat::Raw.to_string(), "raw");
    }

    #[test]
    fn test_message_format_as_str() {
        assert_eq!(MessageFormat::Full.as_str(), "full");
        assert_eq!(MessageFormat::Metadata.as_str(), "metadata");
        assert_eq!(MessageFormat::Minimal.as_str(), "minimal");
        assert_eq!(MessageFormat::Raw.as_str(), "raw");
    }

    #[test]
    fn test_message_format_equality() {
        assert_eq!(MessageFormat::Full, MessageFormat::Full);
        assert_ne!(MessageFormat::Full, MessageFormat::Raw);
    }
}
