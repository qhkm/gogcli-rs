// send.rs - Send email via the Gmail API.

use crate::mime::build_mime_message;
use crate::{types::Message, GmailError};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use reqwest::Client;
use serde_json::json;

const GMAIL_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

/// Parameters for sending an email via Gmail.
#[derive(Debug, Clone, Default)]
pub struct SendParams {
    /// Primary recipients.
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    pub bcc: Vec<String>,
    /// Email subject.
    pub subject: String,
    /// Email body (plain text or HTML, depending on `html` flag).
    pub body: String,
    /// If `true`, the body is treated as HTML (`text/html`);
    /// otherwise plain text (`text/plain`).
    pub html: bool,
    /// Optional Reply-To header value.
    pub reply_to: Option<String>,
    /// Thread ID to reply into (sets `threadId` in the API request).
    pub thread_id: Option<String>,
}

/// Send an email via the Gmail API.
///
/// Builds an RFC 2822 MIME message, base64url-encodes it, and POSTs to
/// `https://gmail.googleapis.com/gmail/v1/users/me/messages/send`.
pub async fn send_message(
    client: &Client,
    access_token: &str,
    from: &str,
    params: &SendParams,
) -> Result<Message, GmailError> {
    let url = format!("{GMAIL_BASE}/messages/send");

    let mut mime = build_mime_message(
        from,
        &params.to,
        &params.cc,
        &params.bcc,
        &params.subject,
        &params.body,
        params.html,
    );

    // Optionally add Reply-To header (insert before the blank line separator).
    if let Some(reply_to) = &params.reply_to {
        // Insert before the CRLF CRLF that separates headers from body.
        if let Some(sep_pos) = mime.find("\r\n\r\n") {
            let reply_header = format!("Reply-To: {}\r\n", reply_to);
            mime.insert_str(sep_pos, &reply_header);
        }
    }

    let raw = URL_SAFE_NO_PAD.encode(mime.as_bytes());

    let mut body = json!({ "raw": raw });
    if let Some(thread_id) = &params.thread_id {
        body["threadId"] = json!(thread_id);
    }

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let message = resp.text().await.unwrap_or_default();
        return Err(GmailError::Api { status, message });
    }

    let sent: Message = resp.json().await?;
    Ok(sent)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_params_default() {
        let params = SendParams::default();
        assert!(params.to.is_empty(), "default to should be empty");
        assert!(params.cc.is_empty(), "default cc should be empty");
        assert!(params.bcc.is_empty(), "default bcc should be empty");
        assert!(params.subject.is_empty());
        assert!(params.body.is_empty());
        assert!(!params.html, "html should default to false");
        assert!(params.reply_to.is_none());
        assert!(params.thread_id.is_none());
    }

    #[test]
    fn test_send_params_fields() {
        let params = SendParams {
            to: vec!["a@b.com".to_string()],
            cc: vec!["c@d.com".to_string()],
            bcc: vec!["e@f.com".to_string()],
            subject: "Hello".to_string(),
            body: "Body".to_string(),
            html: true,
            reply_to: Some("r@r.com".to_string()),
            thread_id: Some("tid123".to_string()),
        };

        assert_eq!(params.to, vec!["a@b.com"]);
        assert_eq!(params.cc, vec!["c@d.com"]);
        assert_eq!(params.bcc, vec!["e@f.com"]);
        assert_eq!(params.subject, "Hello");
        assert_eq!(params.body, "Body");
        assert!(params.html);
        assert_eq!(params.reply_to.as_deref(), Some("r@r.com"));
        assert_eq!(params.thread_id.as_deref(), Some("tid123"));
    }

    #[test]
    fn test_send_builds_valid_raw_payload() {
        // Verify that the MIME we build can be base64url encoded/decoded back.
        let mime = build_mime_message(
            "from@example.com",
            &["to@example.com".to_string()],
            &[],
            &[],
            "Test Subject",
            "Test body text.",
            false,
        );

        let raw = URL_SAFE_NO_PAD.encode(mime.as_bytes());
        // Round-trip: decode should give back the original MIME
        let decoded_bytes = URL_SAFE_NO_PAD.decode(&raw).expect("decode raw");
        let decoded_str = String::from_utf8(decoded_bytes).expect("utf8");
        assert_eq!(decoded_str, mime);
    }
}
