pub mod get;
pub mod labels;
pub mod mime;
pub mod search;
pub mod send;
pub mod thread;
pub mod types;

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
