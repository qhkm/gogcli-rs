// Gmail API v1 data types
// Matches Google Gmail API v1 JSON response shapes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    #[serde(default)]
    pub label_ids: Vec<String>,
    #[serde(default)]
    pub snippet: String,
    pub payload: Option<MessagePart>,
    pub internal_date: Option<String>,
    pub size_estimate: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePart {
    pub part_id: Option<String>,
    pub mime_type: Option<String>,
    pub filename: Option<String>,
    pub headers: Option<Vec<Header>>,
    pub body: Option<MessagePartBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePartBody {
    pub size: Option<i64>,
    pub data: Option<String>, // base64url encoded
    #[serde(rename = "attachmentId")]
    pub attachment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
    pub message_list_visibility: Option<String>,
    pub label_list_visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: String,
    pub snippet: Option<String>,
    pub history_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageList {
    #[serde(default)]
    pub messages: Vec<MessageRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRef {
    pub id: String,
    pub thread_id: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_deserialize() {
        let json = r#"{
            "id": "abc123",
            "threadId": "thread456",
            "labelIds": ["INBOX", "UNREAD"],
            "snippet": "Hello world",
            "internalDate": "1700000000000",
            "sizeEstimate": 1234,
            "payload": {
                "mimeType": "text/plain",
                "headers": [
                    {"name": "From", "value": "sender@example.com"},
                    {"name": "Subject", "value": "Test subject"}
                ],
                "body": {
                    "size": 11,
                    "data": "SGVsbG8gd29ybGQ="
                },
                "parts": []
            }
        }"#;

        let msg: Message = serde_json::from_str(json).expect("deserialize message");
        assert_eq!(msg.id, "abc123");
        assert_eq!(msg.thread_id, "thread456");
        assert_eq!(msg.label_ids, vec!["INBOX", "UNREAD"]);
        assert_eq!(msg.snippet, "Hello world");
        assert_eq!(msg.internal_date.as_deref(), Some("1700000000000"));
        assert_eq!(msg.size_estimate, Some(1234));

        let payload = msg.payload.expect("payload present");
        assert_eq!(payload.mime_type.as_deref(), Some("text/plain"));

        let headers = payload.headers.expect("headers present");
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].name, "From");
        assert_eq!(headers[0].value, "sender@example.com");

        let body = payload.body.expect("body present");
        assert_eq!(body.size, Some(11));
        assert_eq!(body.data.as_deref(), Some("SGVsbG8gd29ybGQ="));
    }

    #[test]
    fn test_message_deserialize_minimal() {
        // Message with only required fields - label_ids and snippet should default
        let json = r#"{"id": "x1", "threadId": "t1"}"#;
        let msg: Message = serde_json::from_str(json).expect("deserialize minimal message");
        assert_eq!(msg.id, "x1");
        assert_eq!(msg.thread_id, "t1");
        assert!(msg.label_ids.is_empty());
        assert!(msg.snippet.is_empty());
        assert!(msg.payload.is_none());
    }

    #[test]
    fn test_label_deserialize() {
        let json = r#"{
            "id": "Label_123",
            "name": "MyLabel",
            "type": "user",
            "messageListVisibility": "show",
            "labelListVisibility": "labelShow"
        }"#;

        let label: Label = serde_json::from_str(json).expect("deserialize label");
        assert_eq!(label.id, "Label_123");
        assert_eq!(label.name, "MyLabel");
        assert_eq!(label.label_type.as_deref(), Some("user"));
        assert_eq!(label.message_list_visibility.as_deref(), Some("show"));
        assert_eq!(label.label_list_visibility.as_deref(), Some("labelShow"));
    }

    #[test]
    fn test_label_deserialize_minimal() {
        let json = r#"{"id": "INBOX", "name": "INBOX"}"#;
        let label: Label = serde_json::from_str(json).expect("deserialize minimal label");
        assert_eq!(label.id, "INBOX");
        assert_eq!(label.name, "INBOX");
        assert!(label.label_type.is_none());
    }

    #[test]
    fn test_message_list_deserialize() {
        let json = r#"{
            "messages": [
                {"id": "msg1", "threadId": "thread1"},
                {"id": "msg2", "threadId": "thread2"}
            ],
            "nextPageToken": "token_abc",
            "resultSizeEstimate": 42
        }"#;

        let list: MessageList = serde_json::from_str(json).expect("deserialize message list");
        assert_eq!(list.messages.len(), 2);
        assert_eq!(list.messages[0].id, "msg1");
        assert_eq!(list.messages[0].thread_id, "thread1");
        assert_eq!(list.messages[1].id, "msg2");
        assert_eq!(list.next_page_token.as_deref(), Some("token_abc"));
        assert_eq!(list.result_size_estimate, Some(42));
    }

    #[test]
    fn test_message_list_empty() {
        // Empty result - messages should default to empty vec
        let json = r#"{"resultSizeEstimate": 0}"#;
        let list: MessageList = serde_json::from_str(json).expect("deserialize empty list");
        assert!(list.messages.is_empty());
        assert!(list.next_page_token.is_none());
        assert_eq!(list.result_size_estimate, Some(0));
    }

    #[test]
    fn test_thread_deserialize() {
        let json = r#"{
            "id": "thread_xyz",
            "snippet": "Thread snippet",
            "historyId": "99",
            "messages": [
                {"id": "m1", "threadId": "thread_xyz", "snippet": "msg1"}
            ]
        }"#;

        let thread: Thread = serde_json::from_str(json).expect("deserialize thread");
        assert_eq!(thread.id, "thread_xyz");
        assert_eq!(thread.snippet.as_deref(), Some("Thread snippet"));
        assert_eq!(thread.history_id.as_deref(), Some("99"));
        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].id, "m1");
    }
}
