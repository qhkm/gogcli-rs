// mime.rs - MIME helpers for Gmail message bodies and headers.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::{GmailError, types::MessagePart};

/// Decode base64url-encoded body data from the Gmail API.
///
/// Gmail uses URL-safe base64 (RFC 4648 §5) without padding. This function
/// handles both padded and unpadded variants.
pub fn decode_body(data: &str) -> Result<String, GmailError> {
    // Normalise: replace any URL-safe chars, then strip padding before decoding.
    let normalised = data.replace('-', "+").replace('_', "/");
    // Strip padding so we can use the no-pad engine regardless.
    let stripped = normalised.trim_end_matches('=');

    // Try URL-safe no-pad first, then fall back to standard no-pad.
    let bytes = URL_SAFE_NO_PAD
        .decode(data.trim_end_matches('='))
        .or_else(|_| {
            use base64::engine::general_purpose::STANDARD_NO_PAD;
            STANDARD_NO_PAD.decode(stripped)
        })
        .map_err(|e| GmailError::Decode(e.to_string()))?;

    String::from_utf8(bytes).map_err(|e| GmailError::Decode(e.to_string()))
}

/// Encode a string to base64url (no padding) for use in Gmail API send requests.
pub fn encode_base64url(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

/// Get a specific header value from a message payload.
/// Header matching is case-insensitive.
pub fn get_header(payload: &MessagePart, name: &str) -> Option<String> {
    let headers = payload.headers.as_ref()?;
    let lower_name = name.to_lowercase();
    headers
        .iter()
        .find(|h| h.name.to_lowercase() == lower_name)
        .map(|h| h.value.clone())
}

/// Extract plain text body (`text/plain`) from a message payload.
///
/// Recursively searches multipart payloads for the first `text/plain` part.
pub fn extract_text_body(payload: &MessagePart) -> Option<String> {
    extract_body_by_mime(payload, "text/plain")
}

/// Extract HTML body (`text/html`) from a message payload.
///
/// Recursively searches multipart payloads for the first `text/html` part.
pub fn extract_html_body(payload: &MessagePart) -> Option<String> {
    extract_body_by_mime(payload, "text/html")
}

fn extract_body_by_mime(payload: &MessagePart, target_mime: &str) -> Option<String> {
    let mime = payload.mime_type.as_deref().unwrap_or("");

    // Direct match at this level
    if mime.eq_ignore_ascii_case(target_mime) {
        if let Some(body) = &payload.body {
            if let Some(data) = &body.data {
                if !data.is_empty() {
                    return decode_body(data).ok();
                }
            }
        }
    }

    // Recurse into sub-parts
    for part in &payload.parts {
        if let Some(text) = extract_body_by_mime(part, target_mime) {
            return Some(text);
        }
    }

    None
}

/// Build an RFC 2822 MIME message suitable for the Gmail API send endpoint.
///
/// Returns the raw MIME message as a `String`. The caller should base64url-
/// encode the result before including it in the API request body.
pub fn build_mime_message(
    from: &str,
    to: &[String],
    cc: &[String],
    bcc: &[String],
    subject: &str,
    body: &str,
    html: bool,
) -> String {
    let content_type = if html { "text/html" } else { "text/plain" };
    let mut msg = String::new();

    msg.push_str(&format!("From: {}\r\n", from));

    if !to.is_empty() {
        msg.push_str(&format!("To: {}\r\n", to.join(", ")));
    }
    if !cc.is_empty() {
        msg.push_str(&format!("Cc: {}\r\n", cc.join(", ")));
    }
    if !bcc.is_empty() {
        msg.push_str(&format!("Bcc: {}\r\n", bcc.join(", ")));
    }

    msg.push_str(&format!("Subject: {}\r\n", subject));
    msg.push_str(&format!(
        "Content-Type: {}; charset=\"UTF-8\"\r\n",
        content_type
    ));
    msg.push_str("MIME-Version: 1.0\r\n");
    msg.push_str("\r\n");
    msg.push_str(body);

    msg
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Header, MessagePart, MessagePartBody};

    fn make_part(mime: &str, data: Option<&str>, parts: Vec<MessagePart>) -> MessagePart {
        MessagePart {
            part_id: None,
            mime_type: Some(mime.to_string()),
            filename: None,
            headers: None,
            body: Some(MessagePartBody {
                size: None,
                data: data.map(String::from),
                attachment_id: None,
            }),
            parts,
        }
    }

    fn make_part_with_headers(
        mime: &str,
        data: Option<&str>,
        headers: Vec<Header>,
        parts: Vec<MessagePart>,
    ) -> MessagePart {
        MessagePart {
            part_id: None,
            mime_type: Some(mime.to_string()),
            filename: None,
            headers: Some(headers),
            body: Some(MessagePartBody {
                size: None,
                data: data.map(String::from),
                attachment_id: None,
            }),
            parts,
        }
    }

    // ---- decode_body -------------------------------------------------------

    #[test]
    fn test_decode_body_valid() {
        // "Hello, World!" base64url encoded (no padding)
        let encoded = URL_SAFE_NO_PAD.encode("Hello, World!");
        let decoded = decode_body(&encoded).expect("decode should succeed");
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_decode_body_valid_with_padding() {
        // Gmail sometimes returns padded base64url
        use base64::engine::general_purpose::URL_SAFE;
        let encoded = URL_SAFE.encode("padded test");
        let decoded = decode_body(&encoded).expect("decode padded should succeed");
        assert_eq!(decoded, "padded test");
    }

    #[test]
    fn test_decode_body_invalid() {
        let result = decode_body("!!!not-valid-base64!!!");
        assert!(result.is_err(), "invalid base64 should return error");
        match result.unwrap_err() {
            GmailError::Decode(_) => {}
            other => panic!("expected Decode error, got: {other:?}"),
        }
    }

    #[test]
    fn test_decode_body_utf8() {
        let encoded = URL_SAFE_NO_PAD.encode("日本語テスト");
        let decoded = decode_body(&encoded).expect("utf8 decode should succeed");
        assert_eq!(decoded, "日本語テスト");
    }

    // ---- extract_text_body -------------------------------------------------

    #[test]
    fn test_extract_text_body() {
        let text_data = URL_SAFE_NO_PAD.encode("Plain text content");
        let part = make_part("text/plain", Some(&text_data), vec![]);
        let result = extract_text_body(&part);
        assert_eq!(result.as_deref(), Some("Plain text content"));
    }

    #[test]
    fn test_extract_text_body_from_multipart() {
        let text_data = URL_SAFE_NO_PAD.encode("Plain text part");
        let html_data = URL_SAFE_NO_PAD.encode("<p>HTML part</p>");

        let text_part = make_part("text/plain", Some(&text_data), vec![]);
        let html_part = make_part("text/html", Some(&html_data), vec![]);

        let multipart = MessagePart {
            part_id: None,
            mime_type: Some("multipart/alternative".to_string()),
            filename: None,
            headers: None,
            body: None,
            parts: vec![text_part, html_part],
        };

        let result = extract_text_body(&multipart);
        assert_eq!(result.as_deref(), Some("Plain text part"));
    }

    #[test]
    fn test_extract_text_body_missing() {
        let html_data = URL_SAFE_NO_PAD.encode("<p>Only HTML</p>");
        let part = make_part("text/html", Some(&html_data), vec![]);
        let result = extract_text_body(&part);
        assert!(result.is_none(), "should return None when no text/plain");
    }

    // ---- extract_html_body -------------------------------------------------

    #[test]
    fn test_extract_html_body() {
        let html_data = URL_SAFE_NO_PAD.encode("<h1>Hello</h1>");
        let part = make_part("text/html", Some(&html_data), vec![]);
        let result = extract_html_body(&part);
        assert_eq!(result.as_deref(), Some("<h1>Hello</h1>"));
    }

    #[test]
    fn test_extract_html_body_from_multipart() {
        let text_data = URL_SAFE_NO_PAD.encode("Plain part");
        let html_data = URL_SAFE_NO_PAD.encode("<b>HTML part</b>");

        let text_part = make_part("text/plain", Some(&text_data), vec![]);
        let html_part = make_part("text/html", Some(&html_data), vec![]);

        let multipart = MessagePart {
            part_id: None,
            mime_type: Some("multipart/alternative".to_string()),
            filename: None,
            headers: None,
            body: None,
            parts: vec![text_part, html_part],
        };

        let result = extract_html_body(&multipart);
        assert_eq!(result.as_deref(), Some("<b>HTML part</b>"));
    }

    // ---- get_header --------------------------------------------------------

    #[test]
    fn test_get_header() {
        let part = make_part_with_headers(
            "text/plain",
            None,
            vec![
                Header {
                    name: "Subject".to_string(),
                    value: "My Subject".to_string(),
                },
                Header {
                    name: "From".to_string(),
                    value: "sender@example.com".to_string(),
                },
                Header {
                    name: "To".to_string(),
                    value: "recipient@example.com".to_string(),
                },
            ],
            vec![],
        );

        assert_eq!(get_header(&part, "Subject").as_deref(), Some("My Subject"));
        assert_eq!(
            get_header(&part, "From").as_deref(),
            Some("sender@example.com")
        );
        assert_eq!(
            get_header(&part, "To").as_deref(),
            Some("recipient@example.com")
        );
    }

    #[test]
    fn test_get_header_missing() {
        let part = make_part_with_headers(
            "text/plain",
            None,
            vec![Header {
                name: "Subject".to_string(),
                value: "Test".to_string(),
            }],
            vec![],
        );

        assert!(
            get_header(&part, "X-Custom-Header").is_none(),
            "missing header should return None"
        );
    }

    #[test]
    fn test_get_header_case_insensitive() {
        let part = make_part_with_headers(
            "text/plain",
            None,
            vec![Header {
                name: "Content-Type".to_string(),
                value: "text/plain; charset=UTF-8".to_string(),
            }],
            vec![],
        );

        // All of these should match the same header
        assert!(get_header(&part, "content-type").is_some());
        assert!(get_header(&part, "CONTENT-TYPE").is_some());
        assert!(get_header(&part, "Content-Type").is_some());
        assert_eq!(
            get_header(&part, "content-type").as_deref(),
            Some("text/plain; charset=UTF-8")
        );
    }

    #[test]
    fn test_get_header_no_headers() {
        let part = make_part("text/plain", None, vec![]);
        // part has Some(vec![]) headers from make_part... actually make_part sets headers: None
        // So this tests the None case.
        assert!(get_header(&part, "Subject").is_none());
    }

    // ---- build_mime_message ------------------------------------------------

    #[test]
    fn test_build_mime_simple() {
        let msg = build_mime_message(
            "sender@example.com",
            &["recipient@example.com".to_string()],
            &[],
            &[],
            "Hello there",
            "This is the body.",
            false,
        );

        assert!(msg.contains("From: sender@example.com"), "missing From");
        assert!(msg.contains("To: recipient@example.com"), "missing To");
        assert!(msg.contains("Subject: Hello there"), "missing Subject");
        assert!(
            msg.contains("Content-Type: text/plain"),
            "should be text/plain"
        );
        assert!(msg.contains("MIME-Version: 1.0"), "missing MIME-Version");
        assert!(msg.contains("This is the body."), "missing body");
    }

    #[test]
    fn test_build_mime_html() {
        let msg = build_mime_message(
            "sender@example.com",
            &["to@example.com".to_string()],
            &[],
            &[],
            "HTML Email",
            "<p>Hello HTML</p>",
            true,
        );

        assert!(
            msg.contains("Content-Type: text/html"),
            "should be text/html, got: {msg}"
        );
        assert!(msg.contains("<p>Hello HTML</p>"), "missing html body");
    }

    #[test]
    fn test_build_mime_cc_bcc() {
        let msg = build_mime_message(
            "sender@example.com",
            &["to@example.com".to_string()],
            &[
                "cc1@example.com".to_string(),
                "cc2@example.com".to_string(),
            ],
            &["bcc@example.com".to_string()],
            "Subject",
            "Body",
            false,
        );

        assert!(msg.contains("Cc: cc1@example.com, cc2@example.com"), "missing Cc");
        assert!(msg.contains("Bcc: bcc@example.com"), "missing Bcc");
    }

    #[test]
    fn test_build_mime_empty_cc_bcc() {
        let msg = build_mime_message(
            "a@b.com",
            &["c@d.com".to_string()],
            &[],
            &[],
            "S",
            "B",
            false,
        );
        // When CC/BCC are empty, they should not appear in headers
        assert!(
            !msg.contains("Cc:"),
            "empty Cc should not produce Cc header"
        );
        assert!(
            !msg.contains("Bcc:"),
            "empty Bcc should not produce Bcc header"
        );
    }

    #[test]
    fn test_build_mime_header_separator() {
        // Headers and body must be separated by a blank line (CRLF CRLF)
        let msg = build_mime_message(
            "a@b.com",
            &["c@d.com".to_string()],
            &[],
            &[],
            "Subj",
            "Body text",
            false,
        );
        assert!(
            msg.contains("\r\n\r\n"),
            "MIME message must have CRLF CRLF separator between headers and body"
        );
    }
}
