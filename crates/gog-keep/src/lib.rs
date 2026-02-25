pub mod types;
pub mod list;
pub mod get;

pub use types::*;

#[derive(thiserror::Error, Debug)]
pub enum KeepError {
    #[error("Keep API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
