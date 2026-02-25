// gog-drive list module
// List files in Google Drive via the files.list API.
// Ported from internal/drive/list.go

use reqwest::Client;

use crate::types::{DriveError, DriveFile, FileList};

const FILES_ENDPOINT: &str = "https://www.googleapis.com/drive/v3/files";

/// Default fields to request from the Drive API for each file.
const DEFAULT_FIELDS: &str =
    "files(id,name,mimeType,size,createdTime,modifiedTime,parents,shared,webViewLink,owners,permissions,trashed),nextPageToken";

// ---------------------------------------------------------------------------
// ListOptions
// ---------------------------------------------------------------------------

/// Options for listing Drive files.
pub struct ListOptions {
    /// Parent folder ID. None means "My Drive root" (no q filter on parents).
    pub parent_id: Option<String>,
    /// Additional query string, appended with `and` to any parent filter.
    pub query: Option<String>,
    /// Maximum number of files to return per page (1-1000, Drive default is 100).
    pub page_size: Option<u32>,
    /// Page token from a previous list response.
    pub page_token: Option<String>,
    /// Include files in the trash.
    pub include_trashed: bool,
    /// Order by clause, e.g. `"name"` or `"modifiedTime desc"`.
    pub order_by: Option<String>,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            parent_id: None,
            query: None,
            page_size: None,
            page_token: None,
            include_trashed: false,
            order_by: None,
        }
    }
}

// ---------------------------------------------------------------------------
// list_files
// ---------------------------------------------------------------------------

/// List files in Google Drive.
///
/// Returns a single page of results. Call repeatedly with `next_page_token`
/// to paginate through all results.
pub async fn list_files(
    client: &Client,
    access_token: &str,
    opts: &ListOptions,
) -> Result<FileList, DriveError> {
    let mut q_parts: Vec<String> = Vec::new();

    if let Some(parent) = &opts.parent_id {
        q_parts.push(format!("'{}' in parents", parent));
    }
    if !opts.include_trashed {
        q_parts.push("trashed = false".to_string());
    }
    if let Some(extra) = &opts.query {
        q_parts.push(extra.clone());
    }

    let q = if q_parts.is_empty() {
        None
    } else {
        Some(q_parts.join(" and "))
    };

    let mut params: Vec<(&str, String)> = vec![("fields", DEFAULT_FIELDS.to_string())];

    if let Some(q) = q {
        params.push(("q", q));
    }
    if let Some(size) = opts.page_size {
        params.push(("pageSize", size.to_string()));
    }
    if let Some(token) = &opts.page_token {
        params.push(("pageToken", token.clone()));
    }
    if let Some(order) = &opts.order_by {
        params.push(("orderBy", order.clone()));
    }

    let resp = client
        .get(FILES_ENDPOINT)
        .bearer_auth(access_token)
        .query(&params)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    let list: FileList = resp.json().await?;
    Ok(list)
}

// ---------------------------------------------------------------------------
// list_all_files
// ---------------------------------------------------------------------------

/// List all files matching the given options, automatically following
/// pagination until no more pages remain.
pub async fn list_all_files(
    client: &Client,
    access_token: &str,
    opts: &ListOptions,
) -> Result<Vec<DriveFile>, DriveError> {
    let mut all_files: Vec<DriveFile> = Vec::new();
    let mut page_token: Option<String> = opts.page_token.clone();

    loop {
        let page_opts = ListOptions {
            parent_id: opts.parent_id.clone(),
            query: opts.query.clone(),
            page_size: opts.page_size,
            page_token: page_token.clone(),
            include_trashed: opts.include_trashed,
            order_by: opts.order_by.clone(),
        };

        let page = list_files(client, access_token, &page_opts).await?;
        all_files.extend(page.files);

        match page.next_page_token {
            Some(token) => page_token = Some(token),
            None => break,
        }
    }

    Ok(all_files)
}

// ---------------------------------------------------------------------------
// get_file
// ---------------------------------------------------------------------------

/// Get a single file's metadata by ID.
pub async fn get_file(
    client: &Client,
    access_token: &str,
    file_id: &str,
) -> Result<DriveFile, DriveError> {
    let url = format!("{}/{}", FILES_ENDPOINT, file_id);
    let fields = "id,name,mimeType,size,createdTime,modifiedTime,parents,shared,webViewLink,owners,permissions,trashed";

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("fields", fields)])
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    let file: DriveFile = resp.json().await?;
    Ok(file)
}

// ---------------------------------------------------------------------------
// delete_file
// ---------------------------------------------------------------------------

/// Move a file to trash (soft-delete) by ID.
///
/// This calls the Drive API's delete endpoint which permanently deletes the
/// file. To trash instead, use `update_file` with `trashed: true`.
/// For typical CLI usage this sends a DELETE request.
pub async fn delete_file(
    client: &Client,
    access_token: &str,
    file_id: &str,
) -> Result<(), DriveError> {
    let url = format!("{}/{}", FILES_ENDPOINT, file_id);

    let resp = client
        .delete(&url)
        .bearer_auth(access_token)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let msg = resp.text().await.unwrap_or_default();
        return Err(DriveError::Api {
            status,
            message: msg,
        });
    }

    Ok(())
}
