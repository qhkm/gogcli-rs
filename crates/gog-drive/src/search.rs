// gog-drive search module
// Search for files in Google Drive using the files.list API with a query string.
// Ported from internal/drive/search.go

use reqwest::Client;

use crate::list::{list_all_files, ListOptions};
use crate::types::{DriveError, DriveFile};

// ---------------------------------------------------------------------------
// SearchOptions
// ---------------------------------------------------------------------------

/// Options for searching Drive files.
#[derive(Default)]
pub struct SearchOptions {
    /// Full-text search query, e.g. `"name contains 'report'"`.
    pub query: String,
    /// Restrict search to a specific folder. None searches all of My Drive.
    pub parent_id: Option<String>,
    /// Include files in the trash.
    pub include_trashed: bool,
    /// Maximum results per page.
    pub page_size: Option<u32>,
    /// Order by clause, e.g. `"modifiedTime desc"`.
    pub order_by: Option<String>,
}

// ---------------------------------------------------------------------------
// search_files
// ---------------------------------------------------------------------------

/// Search for files in Google Drive matching the given query.
///
/// Automatically pages through all results.
pub async fn search_files(
    client: &Client,
    access_token: &str,
    opts: &SearchOptions,
) -> Result<Vec<DriveFile>, DriveError> {
    let list_opts = ListOptions {
        parent_id: opts.parent_id.clone(),
        query: if opts.query.is_empty() {
            None
        } else {
            Some(opts.query.clone())
        },
        page_size: opts.page_size,
        page_token: None,
        include_trashed: opts.include_trashed,
        order_by: opts.order_by.clone(),
    };

    list_all_files(client, access_token, &list_opts).await
}

// ---------------------------------------------------------------------------
// search_by_name
// ---------------------------------------------------------------------------

/// Search for files whose name contains the given substring (case-insensitive).
pub async fn search_by_name(
    client: &Client,
    access_token: &str,
    name: &str,
    parent_id: Option<String>,
) -> Result<Vec<DriveFile>, DriveError> {
    // Drive API query syntax: name contains 'text'
    let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");
    let query = format!("name contains '{}'", escaped);

    let opts = SearchOptions {
        query,
        parent_id,
        ..Default::default()
    };

    search_files(client, access_token, &opts).await
}

// ---------------------------------------------------------------------------
// search_by_mime_type
// ---------------------------------------------------------------------------

/// Search for files with a specific MIME type.
pub async fn search_by_mime_type(
    client: &Client,
    access_token: &str,
    mime_type: &str,
    parent_id: Option<String>,
) -> Result<Vec<DriveFile>, DriveError> {
    let escaped = mime_type.replace('\\', "\\\\").replace('\'', "\\'");
    let query = format!("mimeType = '{}'", escaped);

    let opts = SearchOptions {
        query,
        parent_id,
        ..Default::default()
    };

    search_files(client, access_token, &opts).await
}
