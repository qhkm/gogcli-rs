// gog-drive types module
// Drive API types: DriveFile, FileList, Permission, DriveError.
// Ported from the Go internal/drive package.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Google MIME type constants
// ---------------------------------------------------------------------------

pub const MIME_FOLDER: &str = "application/vnd.google-apps.folder";
pub const MIME_DOCUMENT: &str = "application/vnd.google-apps.document";
pub const MIME_SPREADSHEET: &str = "application/vnd.google-apps.spreadsheet";
pub const MIME_PRESENTATION: &str = "application/vnd.google-apps.presentation";

// ---------------------------------------------------------------------------
// DriveFile
// ---------------------------------------------------------------------------

/// Represents a file or folder in Google Drive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,

    /// File size in bytes. None for Google native docs (Docs, Sheets, Slides).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<DateTime<Utc>>,

    /// Parent folder IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<String>,

    #[serde(default)]
    pub shared: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_view_link: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<Owner>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Permission>,

    #[serde(default)]
    pub trashed: bool,
}

impl DriveFile {
    /// Returns true if this file is a Google Drive folder.
    pub fn is_folder(&self) -> bool {
        self.mime_type == MIME_FOLDER
    }

    /// Returns true if this file is a Google native document (Docs, Sheets, Slides).
    pub fn is_google_doc(&self) -> bool {
        matches!(
            self.mime_type.as_str(),
            MIME_DOCUMENT | MIME_SPREADSHEET | MIME_PRESENTATION
        )
    }

    /// Returns a human-readable file size string.
    ///
    /// - 0 bytes → "0 B"
    /// - < 1 KB  → "{n} B"
    /// - < 1 MB  → "{n.1} KB"
    /// - < 1 GB  → "{n.1} MB"
    /// - else    → "{n.2} GB"
    ///
    /// Returns `"-"` when the size field is absent (e.g., Google native docs).
    pub fn display_size(&self) -> String {
        match &self.size {
            None => "-".to_string(),
            Some(raw) => {
                let bytes: u64 = raw.parse().unwrap_or(0);
                display_size_bytes(bytes)
            }
        }
    }
}

/// Format a byte count as a human-readable string.
pub fn display_size_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes == 0 {
        return "0 B".to_string();
    }
    if bytes < KB {
        return format!("{} B", bytes);
    }
    if bytes < MB {
        let kb = bytes as f64 / KB as f64;
        return format!("{:.1} KB", kb);
    }
    if bytes < GB {
        let mb = bytes as f64 / MB as f64;
        return format!("{:.1} MB", mb);
    }
    let gb = bytes as f64 / GB as f64;
    format!("{:.2} GB", gb)
}

/// Return a friendly label for Google MIME types.
/// Unknown MIME types are returned as-is.
pub fn mime_type_label(mime: &str) -> &str {
    match mime {
        MIME_FOLDER => "Folder",
        MIME_DOCUMENT => "Google Docs",
        MIME_SPREADSHEET => "Google Sheets",
        MIME_PRESENTATION => "Google Slides",
        "application/pdf" => "PDF",
        "image/jpeg" => "JPEG",
        "image/png" => "PNG",
        "text/plain" => "Plain Text",
        "application/zip" => "ZIP",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Owner
// ---------------------------------------------------------------------------

/// A file owner (embedded in DriveFile).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
}

// ---------------------------------------------------------------------------
// Permission
// ---------------------------------------------------------------------------

/// A sharing permission on a Drive file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    pub id: String,
    pub role: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

// ---------------------------------------------------------------------------
// FileList
// ---------------------------------------------------------------------------

/// A paginated list of Drive files, as returned by the `files.list` API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileList {
    #[serde(default)]
    pub files: Vec<DriveFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// DriveError
// ---------------------------------------------------------------------------

#[derive(Error, Debug)]
pub enum DriveError {
    #[error("Drive API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("missing field: {0}")]
    MissingField(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // DriveFile deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_drive_file_deserialize() {
        let json = r#"{
            "id": "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms",
            "name": "My Document",
            "mimeType": "application/vnd.google-apps.document",
            "size": null,
            "createdTime": "2024-01-15T10:30:00Z",
            "modifiedTime": "2024-02-20T14:45:00Z",
            "parents": ["0AMt2lbOlyFrKUk9PVA"],
            "shared": true,
            "webViewLink": "https://docs.google.com/document/d/1BxiMVs0/edit",
            "owners": [
                {"displayName": "Alice Smith", "emailAddress": "alice@example.com"}
            ],
            "trashed": false
        }"#;

        let file: DriveFile = serde_json::from_str(json).expect("deserialize DriveFile");
        assert_eq!(file.id, "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgVE2upms");
        assert_eq!(file.name, "My Document");
        assert_eq!(file.mime_type, MIME_DOCUMENT);
        assert!(file.shared);
        assert!(!file.trashed);
        assert_eq!(file.parents.len(), 1);
        assert_eq!(file.owners.len(), 1);
        assert_eq!(file.owners[0].display_name, "Alice Smith");
        assert_eq!(
            file.owners[0].email_address.as_deref(),
            Some("alice@example.com")
        );
        assert_eq!(
            file.web_view_link.as_deref(),
            Some("https://docs.google.com/document/d/1BxiMVs0/edit")
        );
    }

    // -----------------------------------------------------------------------
    // FileList deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_file_list_deserialize() {
        let json = r#"{
            "files": [
                {
                    "id": "file1",
                    "name": "Folder A",
                    "mimeType": "application/vnd.google-apps.folder"
                },
                {
                    "id": "file2",
                    "name": "Report.pdf",
                    "mimeType": "application/pdf",
                    "size": "204800"
                }
            ],
            "nextPageToken": "page-token-abc"
        }"#;

        let list: FileList = serde_json::from_str(json).expect("deserialize FileList");
        assert_eq!(list.files.len(), 2);
        assert_eq!(list.next_page_token.as_deref(), Some("page-token-abc"));
        assert_eq!(list.files[0].name, "Folder A");
        assert_eq!(list.files[1].name, "Report.pdf");
    }

    #[test]
    fn test_file_list_deserialize_no_paging() {
        let json = r#"{"files": []}"#;
        let list: FileList = serde_json::from_str(json).expect("deserialize empty FileList");
        assert!(list.files.is_empty());
        assert!(list.next_page_token.is_none());
    }

    // -----------------------------------------------------------------------
    // is_folder
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_folder() {
        let mut file = minimal_file("folder-id", "My Folder", MIME_FOLDER);
        assert!(file.is_folder(), "expected folder MIME type to be detected");

        file.mime_type = "application/pdf".to_string();
        assert!(!file.is_folder(), "PDF should not be a folder");
    }

    // -----------------------------------------------------------------------
    // is_google_doc
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_google_doc() {
        let doc = minimal_file("d1", "Doc", MIME_DOCUMENT);
        assert!(doc.is_google_doc(), "Docs MIME should be google doc");

        let sheet = minimal_file("s1", "Sheet", MIME_SPREADSHEET);
        assert!(sheet.is_google_doc(), "Sheets MIME should be google doc");

        let slide = minimal_file("p1", "Slides", MIME_PRESENTATION);
        assert!(
            slide.is_google_doc(),
            "Presentation MIME should be google doc"
        );

        let folder = minimal_file("f1", "Folder", MIME_FOLDER);
        assert!(!folder.is_google_doc(), "Folder should not be google doc");

        let pdf = minimal_file("pdf1", "Report", "application/pdf");
        assert!(!pdf.is_google_doc(), "PDF should not be google doc");
    }

    // -----------------------------------------------------------------------
    // display_size
    // -----------------------------------------------------------------------

    #[test]
    fn test_display_size_zero() {
        let file = minimal_file_with_size("f", "f", "application/pdf", "0");
        assert_eq!(file.display_size(), "0 B");
    }

    #[test]
    fn test_display_size() {
        // Bytes
        let f = minimal_file_with_size("f", "f", "application/pdf", "512");
        assert_eq!(f.display_size(), "512 B");

        // Kilobytes
        let f = minimal_file_with_size("f", "f", "application/pdf", "2048");
        assert_eq!(f.display_size(), "2.0 KB");

        // Megabytes
        let f = minimal_file_with_size("f", "f", "application/pdf", "5242880"); // 5 MB
        assert_eq!(f.display_size(), "5.0 MB");

        // Gigabytes
        let f = minimal_file_with_size("f", "f", "application/pdf", "1073741824"); // 1 GB
        assert_eq!(f.display_size(), "1.00 GB");
    }

    #[test]
    fn test_display_size_no_size_field() {
        // Google native docs have no size
        let file = minimal_file("d1", "Doc", MIME_DOCUMENT);
        assert_eq!(file.display_size(), "-");
    }

    // -----------------------------------------------------------------------
    // mime_type_label
    // -----------------------------------------------------------------------

    #[test]
    fn test_mime_type_label() {
        assert_eq!(mime_type_label(MIME_DOCUMENT), "Google Docs");
        assert_eq!(mime_type_label(MIME_SPREADSHEET), "Google Sheets");
        assert_eq!(mime_type_label(MIME_PRESENTATION), "Google Slides");
        assert_eq!(mime_type_label(MIME_FOLDER), "Folder");
        assert_eq!(mime_type_label("application/pdf"), "PDF");
    }

    #[test]
    fn test_mime_type_label_unknown() {
        let unknown = "application/x-custom-format";
        assert_eq!(mime_type_label(unknown), unknown);
    }

    // -----------------------------------------------------------------------
    // Permission deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_permission_deserialize() {
        let json = r#"{
            "id": "perm-001",
            "role": "reader",
            "type": "user",
            "emailAddress": "bob@example.com",
            "displayName": "Bob Jones"
        }"#;

        let perm: Permission = serde_json::from_str(json).expect("deserialize Permission");
        assert_eq!(perm.id, "perm-001");
        assert_eq!(perm.role, "reader");
        assert_eq!(perm.type_, "user");
        assert_eq!(perm.email_address.as_deref(), Some("bob@example.com"));
        assert_eq!(perm.display_name.as_deref(), Some("Bob Jones"));
    }

    #[test]
    fn test_permission_deserialize_anyone() {
        let json = r#"{
            "id": "perm-002",
            "role": "commenter",
            "type": "anyone"
        }"#;

        let perm: Permission = serde_json::from_str(json).expect("deserialize anyone Permission");
        assert_eq!(perm.type_, "anyone");
        assert!(perm.email_address.is_none());
        assert!(perm.display_name.is_none());
    }

    // -----------------------------------------------------------------------
    // trashed field
    // -----------------------------------------------------------------------

    #[test]
    fn test_file_is_trashed() {
        let json = r#"{
            "id": "trashed-file-id",
            "name": "Old File",
            "mimeType": "application/pdf",
            "trashed": true
        }"#;

        let file: DriveFile = serde_json::from_str(json).expect("deserialize trashed DriveFile");
        assert!(file.trashed, "expected trashed to be true");
    }

    #[test]
    fn test_file_not_trashed_by_default() {
        let json = r#"{
            "id": "active-file",
            "name": "Active File",
            "mimeType": "application/pdf"
        }"#;

        let file: DriveFile = serde_json::from_str(json).expect("deserialize DriveFile");
        assert!(!file.trashed, "trashed should default to false");
    }

    // -----------------------------------------------------------------------
    // display_size_bytes helper
    // -----------------------------------------------------------------------

    #[test]
    fn test_display_size_bytes_boundaries() {
        assert_eq!(display_size_bytes(0), "0 B");
        assert_eq!(display_size_bytes(1), "1 B");
        assert_eq!(display_size_bytes(1023), "1023 B");
        assert_eq!(display_size_bytes(1024), "1.0 KB");
        assert_eq!(display_size_bytes(1024 * 1024 - 1), "1024.0 KB");
        assert_eq!(display_size_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(display_size_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn minimal_file(id: &str, name: &str, mime: &str) -> DriveFile {
        DriveFile {
            id: id.to_string(),
            name: name.to_string(),
            mime_type: mime.to_string(),
            size: None,
            created_time: None,
            modified_time: None,
            parents: vec![],
            shared: false,
            web_view_link: None,
            owners: vec![],
            permissions: vec![],
            trashed: false,
        }
    }

    fn minimal_file_with_size(id: &str, name: &str, mime: &str, size: &str) -> DriveFile {
        let mut f = minimal_file(id, name, mime);
        f.size = Some(size.to_string());
        f
    }
}
