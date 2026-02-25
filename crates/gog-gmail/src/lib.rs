pub mod types;
pub mod search;
pub mod get;
pub mod send;
pub mod labels;
pub mod thread;
pub mod mime;

pub use types::*;

#[derive(thiserror::Error, Debug)]
pub enum GmailError {
    #[error("Gmail API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("decode error: {0}")]
    Decode(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
