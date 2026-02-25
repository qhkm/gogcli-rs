// gog-drive download module
// Download file content from Google Drive.
// Ported from internal/drive/download.go

use reqwest::Client;

use crate::types::DriveError;

const FILES_ENDPOINT: &str = "https://www.googleapis.com/drive/v3/files";

// ---------------------------------------------------------------------------
// download_file
// ---------------------------------------------------------------------------

/// Download the raw bytes of a file from Google Drive.
///
/// This calls `GET /drive/v3/files/{fileId}?alt=media`.
///
/// Note: This does not work for Google native docs (Docs, Sheets, Slides).
/// Use the Drive export endpoint for those instead.
pub async fn download_file(
    client: &Client,
    access_token: &str,
    file_id: &str,
) -> Result<Vec<u8>, DriveError> {
    let url = format!("{}/{}", FILES_ENDPOINT, file_id);

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("alt", "media")])
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

    let bytes = resp.bytes().await?;
    Ok(bytes.to_vec())
}

// ---------------------------------------------------------------------------
// export_file
// ---------------------------------------------------------------------------

/// Export a Google native document (Docs, Sheets, Slides) to the specified MIME type.
///
/// Common export MIME types:
/// - Google Docs → `"application/pdf"` or `"text/plain"` or `"application/vnd.openxmlformats-officedocument.wordprocessingml.document"`
/// - Google Sheets → `"text/csv"` or `"application/pdf"`
/// - Google Slides → `"application/pdf"`
pub async fn export_file(
    client: &Client,
    access_token: &str,
    file_id: &str,
    export_mime_type: &str,
) -> Result<Vec<u8>, DriveError> {
    let url = format!("{}/{}/export", FILES_ENDPOINT, file_id);

    let resp = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&[("mimeType", export_mime_type)])
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

    let bytes = resp.bytes().await?;
    Ok(bytes.to_vec())
}
