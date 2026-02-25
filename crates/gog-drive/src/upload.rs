// gog-drive upload module
// Upload files to Google Drive using multipart upload.
// Ported from internal/drive/upload.go

use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::json;

use crate::types::{DriveError, DriveFile};

const UPLOAD_ENDPOINT: &str = "https://www.googleapis.com/upload/drive/v3/files";
const FILES_ENDPOINT: &str = "https://www.googleapis.com/drive/v3/files";

// ---------------------------------------------------------------------------
// UploadOptions
// ---------------------------------------------------------------------------

/// Options for uploading a file to Google Drive.
pub struct UploadOptions {
    /// Display name of the file in Drive.
    pub name: String,
    /// MIME type of the file content being uploaded.
    pub mime_type: String,
    /// Parent folder IDs. If empty, the file is placed in My Drive root.
    pub parents: Vec<String>,
}

// ---------------------------------------------------------------------------
// upload_file
// ---------------------------------------------------------------------------

/// Upload a file to Google Drive using multipart upload.
///
/// This posts to `POST /upload/drive/v3/files?uploadType=multipart`.
/// Returns the created DriveFile metadata.
pub async fn upload_file(
    client: &Client,
    access_token: &str,
    opts: &UploadOptions,
    content: Vec<u8>,
) -> Result<DriveFile, DriveError> {
    // Build the metadata JSON part
    let metadata = if opts.parents.is_empty() {
        json!({
            "name": opts.name,
        })
    } else {
        json!({
            "name": opts.name,
            "parents": opts.parents,
        })
    };
    let metadata_str = serde_json::to_string(&metadata)?;

    // Multipart: part 1 = metadata (application/json), part 2 = file content
    let metadata_part = Part::text(metadata_str).mime_str("application/json").map_err(|e| {
        DriveError::Api {
            status: 0,
            message: format!("invalid metadata MIME: {e}"),
        }
    })?;

    let content_part = Part::bytes(content)
        .file_name(opts.name.clone())
        .mime_str(&opts.mime_type)
        .map_err(|e| DriveError::Api {
            status: 0,
            message: format!("invalid content MIME: {e}"),
        })?;

    let form = Form::new()
        .part("metadata", metadata_part)
        .part("file", content_part);

    let resp = client
        .post(UPLOAD_ENDPOINT)
        .bearer_auth(access_token)
        .query(&[
            ("uploadType", "multipart"),
            ("fields", "id,name,mimeType,size,createdTime,modifiedTime,parents,shared,webViewLink,owners,trashed"),
        ])
        .multipart(form)
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
// create_folder
// ---------------------------------------------------------------------------

/// Create a new folder in Google Drive.
///
/// Returns the created DriveFile metadata (with `mime_type` =
/// `application/vnd.google-apps.folder`).
pub async fn create_folder(
    client: &Client,
    access_token: &str,
    name: &str,
    parent_id: Option<&str>,
) -> Result<DriveFile, DriveError> {
    let metadata = match parent_id {
        Some(pid) => json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": [pid],
        }),
        None => json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder",
        }),
    };

    let resp = client
        .post(FILES_ENDPOINT)
        .bearer_auth(access_token)
        .query(&[("fields", "id,name,mimeType,createdTime,parents,trashed")])
        .json(&metadata)
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
