// gog-contacts: Google People API (Contacts) service crate
// Implements search, list, get, create, update, delete for contacts
// and contact group listing.

pub mod create;
pub mod delete;
pub mod groups;
pub mod search;
pub mod types;
pub mod update;

pub use types::*;
