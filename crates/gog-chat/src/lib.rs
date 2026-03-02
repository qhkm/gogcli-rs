// gog-chat: Google Chat API v1 client
//
// Provides types and functions for interacting with the Google Chat REST API:
//   - spaces:   list/get Chat spaces
//   - messages: list/send messages within a space
//   - members:  list members of a space

pub mod error;
pub mod members;
pub mod messages;
pub mod spaces;
pub mod types;

pub use error::ChatError;
pub use types::*;
