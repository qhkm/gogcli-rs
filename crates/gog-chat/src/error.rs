// gog-chat error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("Chat API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("missing field: {0}")]
    MissingField(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_error_api_display() {
        let err = ChatError::Api {
            status: 403,
            message: "Forbidden".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("403"), "got: {msg}");
        assert!(msg.contains("Forbidden"), "got: {msg}");
    }

    #[test]
    fn test_chat_error_missing_field_display() {
        let err = ChatError::MissingField("name".to_string());
        let msg = err.to_string();
        assert!(msg.contains("missing field"), "got: {msg}");
        assert!(msg.contains("name"), "got: {msg}");
    }

    #[test]
    fn test_chat_error_json() {
        let parse_err = serde_json::from_str::<serde_json::Value>("{invalid}").unwrap_err();
        let err = ChatError::Json(parse_err);
        let msg = err.to_string();
        // Just ensure it converts without panic and produces a non-empty message.
        assert!(!msg.is_empty(), "Json error display must be non-empty");
    }
}
