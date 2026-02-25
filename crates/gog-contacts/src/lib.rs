// gog-contacts: Google People API (Contacts) service crate
// Implements search, list, get, create, update, delete for contacts
// and contact group listing.

pub mod types;
pub mod search;
pub mod create;
pub mod update;
pub mod delete;
pub mod groups;

pub use types::*;
